// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::{Duration, Instant};

use anyhow::bail;
use futures_util::StreamExt;
use http::StatusCode;
use metrics::{counter, histogram};
use sqlx::PgPool;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc2822;
use tokio_util::io::StreamReader;
use tracing::{error, info, warn};

use wiyci_common::db;
use wiyci_common::models::fetch_tasks::FetchTask;
use wiyci_common::models::logs::NewLog;

use crate::HttpClient;
use crate::storage::LogStorage;

const RETRY_INTERVAL: Duration = Duration::from_mins(1);
const ITERATION_INTERVAL: Duration = Duration::from_mins(1);

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

impl FetchLogsWorker {
    pub fn new(pool: PgPool, client: HttpClient, storage: LogStorage) -> Self {
        Self {
            pool,
            client,
            storage,
        }
    }

    async fn fetch_and_store_log(&self, fetch_task: &FetchTask) -> anyhow::Result<NewLog> {
        let response = self.client.get(&fetch_task.url).send().await?;
        if response.status() != StatusCode::OK {
            bail!("bad HTTP status {}", response.status().as_u16());
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
            bail!("unexpected content-type \"{}\"", content_type);
        }

        let etag = get_header("etag").map(|s| s.to_owned());
        let last_modified =
            get_header("last-modified").and_then(|s| OffsetDateTime::parse(s, &Rfc2822).ok());

        let stream = response.bytes_stream();
        let mut reader = StreamReader::new(stream.map(|r| r.map_err(std::io::Error::other)));

        let mut file = tokio::fs::File::from_std(self.storage.create(fetch_task.id as u64)?);

        let size = tokio::io::copy(&mut reader, &mut file).await?;
        if size == 0 {
            bail!("empty file received");
        }

        file.sync_all().await?;

        Ok(NewLog {
            id: fetch_task.id,
            fetch_task_id: fetch_task.id,
            size,
            last_modified,
            etag,
        })
    }

    async fn fetch_next_log(&self) -> anyhow::Result<bool> {
        let start = Instant::now();

        let Some(task) = db::fetch_tasks::get_next_for_fetch(&self.pool).await? else {
            return Ok(false);
        };

        let overdue_seconds = task
            .next_fetch_attempt_at
            .map(|next_fetch_attempt_at| {
                (OffsetDateTime::now_utc() - next_fetch_attempt_at)
                    .as_seconds_f64()
                    .max(0.0)
            })
            .unwrap_or_default();

        info!(task.url, overdue_seconds, "fetching log");
        histogram!("wiyci_daemon_log_fetch_overdue_age_seconds").record(overdue_seconds);

        let new_log = match self.fetch_and_store_log(&task).await {
            Err(error) => {
                warn!(%error, "log fetch failed");
                counter!("wiyci_daemon_log_fetches_total", "status" => "failure").increment(1);
                db::fetch_tasks::register_failure(
                    &self.pool,
                    task.id,
                    &format!("{error}"),
                    calc_retry_interval(task.num_attempts),
                )
                .await?;
                return Ok(true);
            }
            Ok(new_log) => new_log,
        };

        db::logs::create(&self.pool, &new_log).await?;
        db::fetch_tasks::resolve(&self.pool, task.id, new_log.id).await?;

        let task_duration_seconds = Instant::now()
            .saturating_duration_since(start)
            .as_secs_f64();

        info!(url = task.url, task_duration_seconds, "task complete");
        histogram!("wiyci_daemon_log_fetch_duration_seconds").record(task_duration_seconds);
        counter!("wiyci_daemon_log_fetches_total", "status" => "success").increment(1);

        Ok(true)
    }

    #[cfg_attr(not(coverage), tracing::instrument(name = "FetchLogsWorker", skip_all))]
    pub async fn run(&self) -> anyhow::Result<()> {
        loop {
            match self.fetch_next_log().await {
                Ok(true) => {}
                Ok(false) => {
                    info!("waiting for tasks");
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
