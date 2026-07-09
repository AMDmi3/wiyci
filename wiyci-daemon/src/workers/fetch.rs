// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::{Duration, Instant};

use futures_util::StreamExt;
use http::StatusCode;
use metrics::{counter, histogram};
use sqlx::PgPool;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc2822;
use tokio_util::io::StreamReader;
use tracing::{debug, error, info, warn};

use wiyci_common::db;
use wiyci_common::models::fetch_tasks::FetchTask;
use wiyci_common::models::logs::NewLog;

use crate::HttpClient;
use crate::storage::LogStorage;

const RETRY_INTERVAL: Duration = Duration::from_mins(1);
const ITERATION_INTERVAL: Duration = Duration::from_secs(5);

const MAX_ATTEMPTS: u32 = 5;

fn calc_retry_interval(num_attempts: u32) -> Option<Duration> {
    if num_attempts < MAX_ATTEMPTS {
        Some(Duration::from_days(num_attempts as u64 + 1))
    } else {
        None
    }
}

pub struct FetchLogsWorker {
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

enum FetchStatus<T> {
    Success(T),
    Reject(FetchReject),
}

impl FetchLogsWorker {
    pub fn new(pool: PgPool, client: HttpClient, storage: LogStorage) -> Self {
        Self {
            pool,
            client,
            storage,
        }
    }

    async fn fetch_and_store_log(
        &self,
        fetch_task: &FetchTask,
    ) -> anyhow::Result<FetchStatus<NewLog>> {
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
        let mut reader = StreamReader::new(stream.map(|r| r.map_err(std::io::Error::other)));

        let mut file = tokio::fs::File::from_std(self.storage.create(fetch_task.id as u64)?);

        let size = match tokio::io::copy(&mut reader, &mut file).await {
            Err(error) => return Ok(FetchStatus::Reject(FetchReject::StoreFailed(error))),
            Ok(0) => return Ok(FetchStatus::Reject(FetchReject::ZeroSize)),
            Ok(size) => size,
        };

        file.sync_all().await?;

        Ok(FetchStatus::Success(NewLog {
            id: fetch_task.id,
            fetch_task_id: fetch_task.id,
            size,
            last_modified,
            etag,
        }))
    }

    async fn fetch_log_inner(&self, task: &FetchTask) -> anyhow::Result<FetchStatus<()>> {
        match self.fetch_and_store_log(task).await? {
            FetchStatus::Success(new_log) => {
                db::logs::create(&self.pool, &new_log).await?;
                db::fetch_tasks::resolve(&self.pool, task.id, new_log.id).await?;
                Ok(FetchStatus::Success(()))
            }
            FetchStatus::Reject(reject) => {
                db::fetch_tasks::register_failure(
                    &self.pool,
                    task.id,
                    &format!("{reject}"),
                    calc_retry_interval(task.num_attempts),
                )
                .await?;
                Ok(FetchStatus::Reject(reject))
            }
        }
    }

    #[cfg_attr(
        not(coverage),
        tracing::instrument(name = "fetch_log", skip_all, fields(url = task.url))
    )]
    async fn fetch_log(&self, task: &FetchTask) -> anyhow::Result<()> {
        let start = Instant::now();

        let overdue_seconds = task
            .next_fetch_attempt_at
            .map(|next_fetch_attempt_at| {
                (OffsetDateTime::now_utc() - next_fetch_attempt_at)
                    .as_seconds_f64()
                    .max(0.0)
            })
            .unwrap_or_default();

        info!(overdue_seconds, "fetching log");
        histogram!("wiyci_daemon_log_fetch_overdue_age_seconds").record(overdue_seconds);

        let res = self.fetch_log_inner(task).await;

        let duration_seconds = Instant::now()
            .saturating_duration_since(start)
            .as_secs_f64();

        histogram!("wiyci_daemon_log_fetch_duration_seconds").record(duration_seconds);

        match &res {
            Ok(FetchStatus::Reject(reject)) => {
                counter!("wiyci_daemon_log_fetches_total", "status" => "reject").increment(1);
                warn!(%reject, "log fetch failed");
            }
            Ok(_) => {
                counter!("wiyci_daemon_log_fetches_total", "status" => "success").increment(1);
                info!(duration_seconds, "log fetched");
            }
            Err(_) => {
                counter!("wiyci_daemon_log_fetches_total", "status" => "failed").increment(1);
            }
        }
        res.map(|_| ())
    }

    async fn process_next_task(&self) -> anyhow::Result<bool> {
        let Some(task) = db::fetch_tasks::get_next_for_fetch(&self.pool).await? else {
            return Ok(false);
        };

        self.fetch_log(&task).await?;
        Ok(true)
    }

    #[cfg_attr(not(coverage), tracing::instrument(name = "FetchLogsWorker", skip_all))]
    pub async fn run(&self) -> anyhow::Result<()> {
        loop {
            match self.process_next_task().await {
                Ok(true) => {}
                Ok(false) => {
                    debug!("waiting for tasks");
                    tokio::time::sleep(ITERATION_INTERVAL).await;
                }
                Err(error) => {
                    error!(%error, "failure in worker iteration");
                    tokio::time::sleep(RETRY_INTERVAL).await;
                }
            }
        }
    }
}
