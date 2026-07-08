// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![cfg_attr(test, feature(coverage_attribute))]

#[macro_use]
mod typed_storage;

mod lines;
mod matching;
pub mod snippets;

use std::io::BufRead;

use bitflags::bitflags;

use crate::lines::SafeLines;
use crate::matching::common::SnippetMatchResult;
use crate::matching::factory::try_spawn_matchers;
use crate::snippets::SnippetStorage;

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

impl LogParser {
    // Bump this on each change of parser output, so the daemon could reparse stored logs
    pub const VERSION: u32 = 2;

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

    pub fn parse(&self, reader: impl BufRead) -> std::io::Result<LogParseReport> {
        let lines = SafeLines::new(reader).with_max_line_length(self.max_line_length);
        let mut res = LogParseReport::default();
        let mut current_matchers = vec![];

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
                    if self
                        .max_total_snippets
                        .is_some_and(|n| res.snippets.len() >= n)
                    {
                        res.flags |= Flags::TOO_MANY_SNIPPETS;
                    } else if !res
                        .snippets
                        .push_with_limit(snippet, self.max_snippets_per_kind)
                    {
                        res.flags |= Flags::TOO_MANY_SNIPPETS;
                    }
                    false
                }
                SnippetMatchResult::Continued => true,
            });

            res.parsed_lines += 1;
        }

        Ok(res)
    }
}
