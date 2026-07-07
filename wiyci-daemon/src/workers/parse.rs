// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::BufReader;
use std::time::{Duration, Instant};

use metrics::{counter, histogram};
use sqlx::PgPool;
use tracing::{debug, error, info};

use wiyci_common::db;
use wiyci_common::models::logs::{Log, ParsedLog};
use wiyci_parser::{LogParseReport, LogParser};

use crate::storage::LogStorage;

const RETRY_INTERVAL: Duration = Duration::from_mins(1);
const ITERATION_INTERVAL: Duration = Duration::from_secs(5);

pub struct ParseLogsWorker {
    pool: PgPool,
    storage: LogStorage,
}

impl ParseLogsWorker {
    pub fn new(pool: PgPool, storage: LogStorage) -> Self {
        Self { pool, storage }
    }

    async fn parse_log_inner(&self, log: &Log) -> anyhow::Result<()> {
        let report = {
            let id = log.id;
            let storage = self.storage.clone();
            let parser = LogParser::default()
                .with_max_line_length(Some(10240))
                .with_max_snippets_per_kind(Some(1000));
            tokio::task::spawn_blocking(move || -> anyhow::Result<LogParseReport> {
                Ok(parser.parse(BufReader::new(storage.open(id as u64)?))?)
            })
            .await??
        };

        let parsed = ParsedLog {
            parser_version: LogParser::VERSION,
            parsed_num_lines: report.parsed_lines as u32,
            parsed_snippet_counts: report.snippets.counts_per_kind(),
        };

        db::logs::apply_parsed(&self.pool, log.id, &parsed).await?;

        counter!("wiyci_daemon_log_parsed_lines_total").increment(report.parsed_lines);
        counter!("wiyci_daemon_log_parses_total", "type" => if log.parser_version.is_none() { "first" } else { "reparse" }).increment(1);

        Ok(())
    }

    #[cfg_attr(
        not(coverage),
        tracing::instrument(name = "parse_log", skip_all, fields(id = log.id))
    )]
    async fn parse_log(&self, log: &Log) -> anyhow::Result<()> {
        let start = Instant::now();

        info!("parsing log");

        let res = self.parse_log_inner(log).await;

        let duration_seconds = Instant::now()
            .saturating_duration_since(start)
            .as_secs_f64();

        histogram!("wiyci_daemon_log_parse_duration_seconds").record(duration_seconds);

        match &res {
            Ok(_) => {
                counter!("wiyci_daemon_log_parses_total", "status" => "success").increment(1);
                info!(duration_seconds, "log parsed");
            }
            Err(_) => {
                counter!("wiyci_daemon_log_parses_total", "status" => "failed").increment(1);
            }
        }
        res
    }

    async fn process_next_task(&self) -> anyhow::Result<bool> {
        let Some(log) = db::logs::get_next_for_parsing(&self.pool, LogParser::VERSION).await?
        else {
            return Ok(false);
        };

        self.parse_log(&log).await?;
        Ok(true)
    }

    #[cfg_attr(not(coverage), tracing::instrument(name = "ParseLogsWorker", skip_all))]
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
