// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::BufReader;

use metrics::counter;
use sqlx::PgPool;
use tracing::{info, info_span};

use wiyci_common::db;
use wiyci_common::models::logs::{Log, ParsedLog};
use wiyci_common::models::snippets::{NewSnippet, SnippetKind};
use wiyci_parser::snippets::{CompilerWarning, TestOutcome, TestResult};
use wiyci_parser::{LogParseReport, LogParser};

use crate::storage::LogStorage;
use crate::workers::util::PollingWorkerRunner;

// Bump this on each pasing logic change to reparse stored logs
const VERSION: u32 = LogParser::VERSION + 1;

pub struct ParseWorker {
    pool: PgPool,
    storage: LogStorage,
}

impl ParseWorker {
    pub fn new(pool: PgPool, storage: LogStorage) -> Self {
        Self { pool, storage }
    }

    fn pick_snippets(
        &self,
        report: &LogParseReport,
    ) -> (Vec<NewSnippet>, HashMap<SnippetKind, u64>) {
        let snippets: RefCell<Vec<NewSnippet>> = Default::default();
        let counts: RefCell<HashMap<SnippetKind, u64>> = Default::default();

        let accept = |snippet: NewSnippet| {
            *counts.borrow_mut().entry(snippet.kind).or_default() += 1;
            snippets.borrow_mut().push(snippet);
        };

        let count_only = |kind: SnippetKind| {
            *counts.borrow_mut().entry(kind).or_default() += 1;
        };

        for snippet in report.snippets.get::<CompilerWarning>() {
            accept(NewSnippet {
                kind: SnippetKind::CompilerWarning,
                text: snippet.lines.join("\n"),
            });
        }
        for snippet in report.snippets.get::<TestResult>() {
            match snippet.outcome {
                TestOutcome::Failed => {
                    accept(NewSnippet {
                        kind: SnippetKind::FailedTest,
                        text: snippet.lines.join("\n"),
                    });
                }
                TestOutcome::Passed => {
                    count_only(SnippetKind::PassedTest);
                }
                TestOutcome::Skipped => {
                    count_only(SnippetKind::SkippedTest);
                }
            }
        }

        (snippets.into_inner(), counts.into_inner())
    }

    async fn parse_log(&self, log: &Log) -> anyhow::Result<()> {
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

        for (kind, count) in &report.snippets.counts_per_kind() {
            let kind: &'static str = kind.into();
            counter!("wiyci_daemon_parse_parsed_snippets_total", "kind" => kind).increment(*count);
        }

        let (snippets, snippet_counts) = self.pick_snippets(&report);
        for (kind, count) in &snippet_counts {
            let kind: &'static str = kind.into();
            counter!("wiyci_daemon_parse_used_snippets_total", "kind" => kind).increment(*count);
        }

        let parsed = ParsedLog {
            parser_version: VERSION,
            parsed_num_lines: report.parsed_lines as u32,
            parsed_snippet_counts: snippet_counts,
        };

        counter!("wiyci_daemon_parse_lines_total").increment(report.parsed_lines);
        counter!("wiyci_daemon_parse_parses_total", "type" => if log.parser_version.is_none() { "first" } else { "reparse" }).increment(1);

        let mut tx = self.pool.begin().await?;
        db::logs::apply_parsed(&mut tx, log.id, &parsed).await?;
        db::snippets::replace_for_log(&mut tx, log.id, &snippets).await?;
        db::projects::update_snippet_counts(&mut tx, &log.project_name).await?;
        tx.commit().await?;

        info!(
            lines = report.parsed_lines,
            snippets_parsed = report.snippets.len(),
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
