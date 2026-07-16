// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Duration;

use futures_util::StreamExt;
use http::StatusCode;
use metrics::{counter, gauge, histogram};
use sqlx::PgPool;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc2822;
use tokio::io::AsyncReadExt;
use tokio_util::io::StreamReader;
use tracing::{info, info_span, warn};

use wiyci_common::db;
use wiyci_common::models::fetch_tasks::FetchTask;
use wiyci_common::models::logs::NewLog;
use wiyci_common::models::statistics::StatisticsDelta;

use crate::HttpClient;
use crate::storage::LogStorage;
use crate::workers::util::PollingWorkerRunner;

const MAX_ATTEMPTS: u32 = 5;
const MAX_CONTENT_SIZE: u64 = 10 * 1024 * 1024;

fn calc_retry_interval(num_attempts: u32) -> Option<Duration> {
    if num_attempts < MAX_ATTEMPTS {
        Some(Duration::from_days(num_attempts as u64 + 1))
    } else {
        None
    }
}

pub struct FetchWorker {
    pool: PgPool,
    client: HttpClient,
    storage: LogStorage,
}

#[derive(Debug)]
enum FetchReject {
    RequestFailed(reqwest_middleware::Error),
    StoreFailed(std::io::Error),
    BadHttpCode(StatusCode),
    BadContentType(String),
    ZeroSize,
}

impl std::fmt::Display for FetchReject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RequestFailed(error) if error.is_timeout() => write!(f, "timeout"),
            Self::RequestFailed(error) => write!(f, "{error}"),
            Self::StoreFailed(error) => write!(f, "{error}"),
            Self::BadHttpCode(code) => write!(f, "bad HTTP code {}", code.as_u16()),
            Self::BadContentType(content_type) => write!(f, "bad MIME type {content_type}"),
            Self::ZeroSize => write!(f, "zero size response"),
        }
    }
}

enum FetchStatus {
    Success(NewLog),
    Reject(FetchReject),
}

impl FetchWorker {
    pub fn new(pool: PgPool, client: HttpClient, storage: LogStorage) -> Self {
        Self {
            pool,
            client,
            storage,
        }
    }

    async fn fetch_and_store_log(&self, fetch_task: &FetchTask) -> anyhow::Result<FetchStatus> {
        let response = match self.client.get(&fetch_task.url).send().await {
            Ok(response) => response,
            Err(error) => return Ok(FetchStatus::Reject(FetchReject::RequestFailed(error))),
        };

        if response.status() != StatusCode::OK {
            return Ok(FetchStatus::Reject(FetchReject::BadHttpCode(
                response.status(),
            )));
        }

        let get_header = |name| {
            response
                .headers()
                .get(name)
                .and_then(|header_value| header_value.to_str().ok())
        };

        if let Some(content_type) = get_header("content-type")
            && !content_type.starts_with("text/plain")
        {
            return Ok(FetchStatus::Reject(FetchReject::BadContentType(
                content_type.to_string(),
            )));
        }

        let etag = get_header("etag").map(|s| s.to_owned());
        let last_modified =
            get_header("last-modified").and_then(|s| OffsetDateTime::parse(s, &Rfc2822).ok());

        let stream = response.bytes_stream();
        let reader = StreamReader::new(stream.map(|r| r.map_err(std::io::Error::other)));
        let mut reader = reader.take(MAX_CONTENT_SIZE);

        let mut file = tokio::fs::File::from_std(self.storage.create(fetch_task.id as u64)?);

        let size = match tokio::io::copy(&mut reader, &mut file).await {
            Err(error) => return Ok(FetchStatus::Reject(FetchReject::StoreFailed(error))),
            Ok(0) => return Ok(FetchStatus::Reject(FetchReject::ZeroSize)),
            Ok(size) => size,
        };

        let mut is_truncated = false;
        if size == MAX_CONTENT_SIZE {
            let mut reader = reader.into_inner();
            let mut probe = [0u8; 1];
            if reader.read(&mut probe).await? > 0 {
                is_truncated = true;
                warn!("log is truncated at {MAX_CONTENT_SIZE}");
            }
        }

        file.sync_all().await?;

        Ok(FetchStatus::Success(NewLog {
            id: fetch_task.id,
            fetch_task_id: fetch_task.id,
            size,
            last_modified,
            etag,
            is_truncated,
        }))
    }

    async fn fetch_log(&self, task: &FetchTask) -> anyhow::Result<()> {
        if let Some(next_fetch_attempt_at) = task.next_fetch_attempt_at {
            histogram!("wiyci_daemon_fetch_overdue_age_seconds").record(
                (OffsetDateTime::now_utc() - next_fetch_attempt_at)
                    .as_seconds_f64()
                    .max(0.0),
            );
        }

        match self.fetch_and_store_log(task).await? {
            FetchStatus::Success(new_log) => {
                db::logs::create(&self.pool, &new_log).await?;
                db::fetch_tasks::resolve(&self.pool, task.id, new_log.id).await?;
                #[expect(clippy::needless_update)]
                let statistics = db::statistics::apply_delta(
                    &self.pool,
                    &StatisticsDelta {
                        stored_logs_size: new_log.size as i64,
                        ..Default::default()
                    },
                )
                .await?;
                counter!("wiyci_daemon_fetch_logs_total", "status" => "success").increment(1);
                counter!("wiyci_daemon_fetch_bytes_total").increment(new_log.size);
                histogram!("wiyci_daemon_fetch_log_size_bytes").record(new_log.size as f64);
                gauge!("wiyci_daemon_statistics_stored_logs_bytes")
                    .set(statistics.stored_logs_size as f64);
                info!("log fetched");
            }
            FetchStatus::Reject(reject) => {
                db::fetch_tasks::register_failure(
                    &self.pool,
                    task.id,
                    &format!("{reject}"),
                    calc_retry_interval(task.num_attempts),
                )
                .await?;
                counter!("wiyci_daemon_fetch_logs_total", "status" => "reject").increment(1);
                warn!(%reject, "log fetch failed");
            }
        }

        Ok(())
    }

    #[cfg_attr(not(coverage), tracing::instrument(name = "FetchWorker", skip_all))]
    pub async fn run(&self) -> anyhow::Result<()> {
        PollingWorkerRunner::new(
            "Fetch",
            async || Ok(db::fetch_tasks::get_next_for_fetch(&self.pool).await?),
            async |task| self.fetch_log(task).await,
        )
        .with_span(|task| info_span!("task", url = task.url))
        .run()
        .await
    }
}
