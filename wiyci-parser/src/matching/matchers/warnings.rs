// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::mem::take;
use std::sync::LazyLock;

use regex::Regex;

use crate::matching::captures::SimplifiedCaptures;
use crate::matching::common::{SnippetMatchResult, SnippetMatcher};
use crate::snippets::{CompilerWarning, Snippet};

// XXX: is position (second number after path) mandatory?
// Note: the regex ensures line number fits into any 32 bit integer
static WARNING_PATTERN: &str =
    r"^(.*\.(?:c|cc|cxx|cpp|cc|h|hh|hpp|hxx|xs)):([0-9]{1,9}):[0-9]+: (warning: .* \[(-W.*)\])$";
static WARNING_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(WARNING_PATTERN).unwrap());

static QUOTED_CODE_PATTERN: &str = r"^[ ]{1,5}([0-9]{0,4}) \| (.*)$";
static QUOTED_CODE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(QUOTED_CODE_PATTERN).unwrap());

struct Details {
    path: String,
    line_number: u32,
    category: String,
    message: String,
}

//impl From<(Vec<String>, Details)> for CompilerWarning {
//    fn from((lines, details): (Vec<String>, Details)) -> Self {
impl CompilerWarning {
    fn from_state(lines: Vec<String>, details: Details) -> Self {
        Self {
            lines,
            path: details.path,
            line_number: details.line_number,
            category: details.category,
            message: details.message,
        }
    }
}

#[derive(Default)]
pub struct CompilerWarningMatcher {
    lines: Vec<String>,
    details: Option<Details>,
}

impl CompilerWarningMatcher {
    pub const TASTE_PATTERN: &str = WARNING_PATTERN;
}

impl SnippetMatcher for CompilerWarningMatcher {
    fn match_line(&mut self, line: &str) -> SnippetMatchResult {
        // TODO: Could consume `in function XXX` message which may come before the warning
        // TODO: Could consume more stuff, such as `In file included from ...` and `note:` blocks
        if self.details.is_none() {
            // before `warning:` line parsed: we only expect `warning:` line
            let Some(m) = WARNING_REGEX.captures(line) else {
                return SnippetMatchResult::NoMatch;
            };

            self.lines.push(line.to_string());
            self.details = Some(Details {
                path: m.get_any(1),
                line_number: m.get_any(2),
                category: m.get_any(4),
                message: m.get_any(3),
            });
            SnippetMatchResult::Continued
        } else {
            // after `warning:` line parsed: there can be more lines with warning details
            if QUOTED_CODE_REGEX.is_match(line) {
                self.lines.push(line.to_string());
                SnippetMatchResult::Continued
            } else {
                Snippet::CompilerWarning(CompilerWarning::from_state(
                    take(&mut self.lines),
                    self.details
                        .take()
                        .expect("we're in !self.detauls.is_none() branch"),
                ))
                .into()
            }
        }
    }

    fn flush(&mut self) -> SnippetMatchResult {
        if let Some(details) = self.details.take() {
            Snippet::CompilerWarning(CompilerWarning::from_state(take(&mut self.lines), details))
                .into()
        } else {
            SnippetMatchResult::NoMatch
        }
    }
}
