// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

mod snippets;

use std::io::BufReader;

use metrics::counter;
use sqlx::PgPool;
use tracing::{info, info_span};

use wiyci_common::db;
use wiyci_common::models::logs::{Log, ParsedLog};
use wiyci_parser::LogParser;

use crate::storage::LogStorage;
use crate::workers::util::PollingWorkerRunner;

// Bump this on each pasing logic change to reparse stored logs
const VERSION: u32 = LogParser::VERSION + 1;

const MAX_PARSED_LINE_LENGTH: Option<usize> = Some(10240);
const MAX_PARSED_SNIPPETS_PER_KIND: Option<u64> = Some(1000);

pub struct ParseWorker {
    pool: PgPool,
    storage: LogStorage,
}

impl ParseWorker {
    pub fn new(pool: PgPool, storage: LogStorage) -> Self {
        Self { pool, storage }
    }

    async fn parse_log(&self, log: &Log) -> anyhow::Result<()> {
        let (status, snippets, counts) = {
            let id = log.id;
            let storage = self.storage.clone();
            let parser = LogParser::default().with_max_line_length(MAX_PARSED_LINE_LENGTH);
            tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
                let mut processor = snippets::SnippetProcessor::default()
                    .with_max_snippets_per_kind(MAX_PARSED_SNIPPETS_PER_KIND);
                let status =
                    parser.parse(BufReader::new(storage.open(id as u64)?), &mut processor)?;
                Ok((status, processor.snippets, processor.counts))
            })
            .await??
        };

        for (kind, count) in &counts {
            let kind: &'static str = kind.into();
            counter!("wiyci_daemon_parse_used_snippets_total", "kind" => kind).increment(*count);
        }

        let parsed = ParsedLog {
            parser_version: VERSION,
            parsed_num_lines: status.num_parsed_lines as u32,
            parsed_snippet_counts: counts,
        };

        counter!("wiyci_daemon_parse_lines_total").increment(status.num_parsed_lines);
        counter!("wiyci_daemon_parse_parses_total", "type" => if log.parser_version.is_none() { "first" } else { "reparse" }).increment(1);

        let mut tx = self.pool.begin().await?;
        db::logs::apply_parsed(&mut tx, log.id, &parsed).await?;
        db::snippets::replace_for_log(&mut tx, log.id, &snippets).await?;
        db::projects::update_snippet_counts(&mut tx, &log.project_name).await?;
        tx.commit().await?;

        info!(
            lines = status.num_parsed_lines,
            snippets_used = snippets.len(),
            "log parsed"
        );

        Ok(())
    }

    #[cfg_attr(not(coverage), tracing::instrument(name = "Parse", skip_all))]
    pub async fn run(&self) -> anyhow::Result<()> {
        PollingWorkerRunner::new(
            "Parse",
            async || Ok(db::logs::get_next_for_parsing(&self.pool, VERSION).await?),
            async |log| self.parse_log(log).await,
        )
        .with_span(|log| info_span!("log", id = log.id))
        .run()
        .await
    }
}
