// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![cfg_attr(test, feature(coverage_attribute))]

mod lines;
mod matching;
pub mod snippets;

use std::io::BufRead;

use crate::lines::SafeLines;
use crate::matching::common::SnippetMatchResult;
use crate::matching::factory::try_spawn_matchers;
use crate::snippets::Snippet;

#[derive(Debug, Default)]
pub struct ParseStatus {
    pub num_parsed_lines: u64,
    pub num_truncated_lines: u64,
    pub num_invalid_utf8_lines: u64,
    pub is_truncated: bool,
    pub is_interrupted: bool,
}

#[derive(Default)]
pub struct LogParser {
    max_line_length: Option<usize>,
    max_lines: Option<u64>,
}

pub trait SnippetHandler {
    fn handle_snippet(&mut self, snippet: Snippet) -> std::ops::ControlFlow<(), ()>;
}

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

    pub fn parse(
        &self,
        reader: impl BufRead,
        receiver: &mut impl SnippetHandler,
    ) -> std::io::Result<ParseStatus> {
        let lines = SafeLines::new(reader).with_max_line_length(self.max_line_length);
        let mut status = ParseStatus::default();
        let mut current_matchers = vec![];

        for line in lines {
            let line = line?;
            if self
                .max_lines
                .is_some_and(|limit| status.num_parsed_lines >= limit)
            {
                status.is_truncated = true;
                break;
            }
            status.num_parsed_lines += 1;
            status.num_truncated_lines += line.was_truncated as u64;
            status.num_invalid_utf8_lines += line.had_invalid_utf8 as u64;
            let line = strip_ansi_escapes::strip_str(line.string);

            current_matchers.extend(try_spawn_matchers(&line));

            current_matchers.retain_mut(|matcher| {
                !status.is_interrupted
                    && match matcher.match_line(&line) {
                        SnippetMatchResult::NoMatch => false,
                        SnippetMatchResult::Complete(snippet) => {
                            if receiver.handle_snippet(snippet).is_break() {
                                status.is_interrupted = true;
                            }
                            false
                        }
                        SnippetMatchResult::Continued => true,
                    }
            });

            if status.is_interrupted {
                break;
            }
        }

        // flush remaining matchers
        for mut matcher in current_matchers {
            if !status.is_interrupted
                && let SnippetMatchResult::Complete(snippet) = matcher.flush()
                && receiver.handle_snippet(snippet).is_break()
            {
                status.is_interrupted = true;
                break;
            }
        }

        Ok(status)
    }
}
