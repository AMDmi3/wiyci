// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::HashMap;
use std::ops::ControlFlow;

use metrics::counter;

use wiyci_common::models::snippets::{NewSnippet, SnippetKind};
use wiyci_parser::SnippetHandler;
use wiyci_parser::snippets::*;

#[derive(Default)]
pub struct SnippetProcessor {
    max_snippets_per_kind: Option<u64>,

    //num_parsed_snippets: u64,
    pub snippets: Vec<NewSnippet>,
    pub counts: HashMap<SnippetKind, u64>,
}

enum SnippetVerdict {
    Use(NewSnippet),
    Count(SnippetKind),
    Skip,
}

impl From<NewSnippet> for SnippetVerdict {
    fn from(new_snippet: NewSnippet) -> Self {
        Self::Use(new_snippet)
    }
}

impl From<SnippetKind> for SnippetVerdict {
    fn from(kind: SnippetKind) -> Self {
        Self::Count(kind)
    }
}

impl SnippetProcessor {
    pub fn with_max_snippets_per_kind(mut self, max_snippets_per_kind: Option<u64>) -> Self {
        self.max_snippets_per_kind = max_snippets_per_kind;
        self
    }

    fn process_snippet(&self, snippet: Snippet) -> SnippetVerdict {
        match snippet {
            Snippet::CompilerWarning(snippet) => NewSnippet {
                kind: SnippetKind::CompilerWarning,
                text: snippet.lines.join("\n"),
            }
            .into(),
            Snippet::TestResult(snippet) => match snippet.outcome {
                TestOutcome::Failed => NewSnippet {
                    kind: SnippetKind::FailedTest,
                    text: snippet.lines.join("\n"),
                }
                .into(),
                TestOutcome::Skipped => SnippetKind::SkippedTest.into(),
                TestOutcome::Passed => SnippetKind::PassedTest.into(),
            },
            _ => SnippetVerdict::Skip,
        }
    }
}

impl SnippetHandler for SnippetProcessor {
    fn handle_snippet(&mut self, snippet: Snippet) -> ControlFlow<()> {
        let original_kind_name: &'static str = (&snippet).into();

        match self.process_snippet(snippet) {
            SnippetVerdict::Use(new_snippet) => {
                let count = self.counts.entry(new_snippet.kind).or_default();
                if self
                    .max_snippets_per_kind
                    .is_none_or(|limit| *count < limit)
                {
                    *self.counts.entry(new_snippet.kind).or_default() += 1;
                    self.snippets.push(new_snippet);
                    counter!("wiyci_daemon_parse_parsed_snippets_total", "kind" => original_kind_name, "verdict" => "use").increment(1);
                } else {
                    counter!("wiyci_daemon_parse_parsed_snippets_total", "kind" => original_kind_name, "verdict" => "limit").increment(1);
                }
            }
            SnippetVerdict::Count(kind) => {
                *self.counts.entry(kind).or_default() += 1;
                counter!("wiyci_daemon_parse_parsed_snippets_total", "kind" => original_kind_name, "verdict" => "count").increment(1);
            }
            SnippetVerdict::Skip => {
                counter!("wiyci_daemon_parse_parsed_snippets_total", "kind" => original_kind_name, "verdict" => "skip").increment(1);
            }
        }

        ControlFlow::Continue(())
    }
}
