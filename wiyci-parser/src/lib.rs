// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![cfg_attr(test, feature(coverage_attribute))]

#[macro_use]
mod typed_storage;

mod lines;
mod matching;
pub mod snippets;

use std::collections::HashMap;
use std::io::BufRead;

use bitflags::bitflags;

use crate::lines::SafeLines;
use crate::matching::common::SnippetMatchResult;
use crate::matching::factory::try_spawn_matchers;
use crate::snippets::{AnySnippet, Snippet, SnippetKind, SnippetStorage};

bitflags! {
    #[derive(Debug, Default, PartialEq, Eq)]
    pub struct Flags: u32 {
        const HAD_TRUNCATED_LINES = 1 << 0;
        const HAD_INVALID_UTF8 = 1 << 1;
        const TOO_MANY_LINES = 1 << 2;
        const TOO_MANY_SNIPPETS = 1 << 3;
    }
}

#[derive(Debug, Default)]
pub struct LogParseReport {
    pub parsed_lines: u64,
    pub snippets: SnippetStorage,
    pub flags: Flags,
}

#[derive(Default)]
pub struct LogParser {
    max_line_length: Option<usize>,
    max_lines: Option<u64>,
    max_total_snippets: Option<usize>,
    max_snippets_per_kind: Option<usize>,
}

type SnippetCounts = HashMap<(SnippetKind, u64), usize>;

impl LogParser {
    // Bump this on each change of parser output, so the daemon could reparse stored logs
    pub const VERSION: u32 = 5;

    pub fn with_max_line_length(mut self, max_line_length: Option<usize>) -> Self {
        self.max_line_length = max_line_length;
        self
    }

    pub fn with_max_lines(mut self, max_lines: Option<u64>) -> Self {
        self.max_lines = max_lines;
        self
    }

    pub fn with_max_total_snippets(mut self, max_total_snippets: Option<usize>) -> Self {
        self.max_total_snippets = max_total_snippets;
        self
    }

    pub fn with_max_snippets_per_kind(mut self, max_snippets_per_kind: Option<usize>) -> Self {
        self.max_snippets_per_kind = max_snippets_per_kind;
        self
    }

    fn try_push_snippet(
        &self,
        snippet: AnySnippet,
        report: &mut LogParseReport,
        snippet_counts: &mut SnippetCounts,
    ) {
        if self
            .max_total_snippets
            .is_some_and(|limit| report.snippets.len() >= limit)
        {
            report.flags |= Flags::TOO_MANY_SNIPPETS;
            return;
        }

        if let Some(limiting_discriminant) = snippet.limiting_discriminant() {
            let counter = snippet_counts
                .entry((snippet.kind(), limiting_discriminant))
                .or_default();
            if self
                .max_snippets_per_kind
                .is_some_and(|limit| *counter >= limit)
            {
                report.flags |= Flags::TOO_MANY_SNIPPETS;
            } else {
                report.snippets.push(snippet);
                *counter += 1;
            }
        } else {
            report.snippets.push(snippet);
        }
    }

    pub fn parse(&self, reader: impl BufRead) -> std::io::Result<LogParseReport> {
        let lines = SafeLines::new(reader).with_max_line_length(self.max_line_length);
        let mut res = LogParseReport::default();
        let mut current_matchers = vec![];
        let mut snippet_counts: SnippetCounts = Default::default();

        for line in lines {
            let line = line?;
            if line.was_truncated {
                res.flags |= Flags::HAD_TRUNCATED_LINES;
            }
            if line.had_invalid_utf8 {
                res.flags |= Flags::HAD_INVALID_UTF8;
            }

            if self.max_lines.is_some_and(|n| res.parsed_lines >= n) {
                res.flags |= Flags::TOO_MANY_LINES;
                break;
            }

            let line = strip_ansi_escapes::strip_str(line.string);

            current_matchers.extend(try_spawn_matchers(&line));

            current_matchers.retain_mut(|matcher| match matcher.match_line(&line) {
                SnippetMatchResult::NoMatch => false,
                SnippetMatchResult::Complete(snippet) => {
                    self.try_push_snippet(snippet, &mut res, &mut snippet_counts);
                    false
                }
                SnippetMatchResult::Continued => true,
            });

            res.parsed_lines += 1;
        }

        // flush remaining matchers
        for mut matcher in current_matchers {
            if let SnippetMatchResult::Complete(snippet) = matcher.flush() {
                self.try_push_snippet(snippet, &mut res, &mut snippet_counts);
            }
        }

        Ok(res)
    }
}
