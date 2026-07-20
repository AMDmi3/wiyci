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
static WARNING_PATTERN: &str = r"^ *warning: (.*)$";
static WARNING_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(WARNING_PATTERN).unwrap());

static LOCATION_PATTERN: &str = r"^ *--> (src/.+\.rs):([0-9]{1,9}):[0-9]+$";
static LOCATION_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(LOCATION_PATTERN).unwrap());

static DETAILS_PATTERN: &str = r"^(?: *[0-9]* \|.*| *= (note|help): .*)$";
static DETAILS_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(DETAILS_PATTERN).unwrap());

static CATEGORY_MENTION_PATTERN: &str = r"#\[warn\(([^)]+)\)]";
static CATEGORY_MENTION_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(CATEGORY_MENTION_PATTERN).unwrap());

#[derive(Default)]
enum State {
    #[default]
    Start,
    Warning {
        message: String,
    },
    WarningLocation {
        message: String,
        path: String,
        line_number: u32,
    },
}

#[derive(Default)]
pub struct RustCompilerWarningMatcher {
    lines: Vec<String>,
    state: State,
    category: Option<String>,
}

impl RustCompilerWarningMatcher {
    pub const TASTE_PATTERN: &str = WARNING_PATTERN;
}

impl SnippetMatcher for RustCompilerWarningMatcher {
    fn match_line(&mut self, line: &str) -> SnippetMatchResult {
        match &mut self.state {
            State::Start => {
                let Some(m) = WARNING_REGEX.captures(line) else {
                    return SnippetMatchResult::NoMatch;
                };

                self.lines.push(line.to_string());
                self.state = State::Warning {
                    message: m.get_any(1),
                };
                SnippetMatchResult::Continued
            }
            State::Warning { message } => {
                let Some(m) = LOCATION_REGEX.captures(line) else {
                    return SnippetMatchResult::NoMatch;
                };

                self.lines.push(line.to_string());
                self.state = State::WarningLocation {
                    message: take(message),
                    path: m.get_any(1),
                    line_number: m.get_any(2),
                };
                SnippetMatchResult::Continued
            }
            State::WarningLocation {
                message,
                path,
                line_number,
            } => {
                if DETAILS_REGEX.is_match(line) {
                    self.lines.push(line.to_string());
                    if self.category.is_none()
                        && let Some(m) = CATEGORY_MENTION_REGEX.captures(line)
                    {
                        self.category = Some(m.get_any(1));
                    }
                    SnippetMatchResult::Continued
                } else {
                    Snippet::CompilerWarning(CompilerWarning {
                        lines: take(&mut self.lines),
                        path: take(path),
                        line_number: *line_number,
                        category: take(&mut self.category),
                        message: take(message),
                    })
                    .into()
                }
            }
        }
    }

    fn flush(&mut self) -> SnippetMatchResult {
        if let State::WarningLocation {
            message,
            path,
            line_number,
        } = take(&mut self.state)
        {
            Snippet::CompilerWarning(CompilerWarning {
                lines: take(&mut self.lines),
                path,
                line_number,
                category: take(&mut self.category),
                message,
            })
            .into()
        } else {
            SnippetMatchResult::NoMatch
        }
    }
}
