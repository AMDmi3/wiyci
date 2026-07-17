// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::sync::LazyLock;

use regex::Regex;

use crate::matching::captures::SimplifiedCaptures;
use crate::matching::common::{SnippetMatchResult, SnippetMatcher};
use crate::snippets::{TestOutcome, TestResult};

static MAIN_PATTERN: &str = r"^([^:]+::[^ ]+) (FAILED|PASSED) +\[[0-9 ]{3}%\]$";
static MAIN_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(MAIN_PATTERN).unwrap());

#[derive(Default)]
pub struct PytestTestResultMatcher;

impl PytestTestResultMatcher {
    pub const TASTE_PATTERN: &str = MAIN_PATTERN;
}

impl SnippetMatcher for PytestTestResultMatcher {
    fn match_line(&mut self, line: &str) -> SnippetMatchResult {
        let Some(m) = MAIN_REGEX.captures(line) else {
            return SnippetMatchResult::NoMatch;
        };

        let outcome = match m.get_str(2) {
            "FAILED" => TestOutcome::Failed,
            "PASSED" => TestOutcome::Passed,
            _ => {
                unreachable!("nonempty string ensured by the regex");
            }
        };

        TestResult {
            lines: vec![line.to_string()],
            name: m.get_any(1),
            outcome,
        }
        .into()
    }
}

static SKIPPED_PATTERN: &str = r"^([^:]+::[^ ]+) SKIPPED +(?:\(.*\) +)?\[[0-9 ]{3}%\]$";
static SKIPPED_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(SKIPPED_PATTERN).unwrap());

#[derive(Default)]
pub struct PytestTestSkippedResultMatcher;

impl PytestTestSkippedResultMatcher {
    pub const TASTE_PATTERN: &str = SKIPPED_PATTERN;
}

impl SnippetMatcher for PytestTestSkippedResultMatcher {
    fn match_line(&mut self, line: &str) -> SnippetMatchResult {
        let Some(m) = SKIPPED_REGEX.captures(line) else {
            return SnippetMatchResult::NoMatch;
        };

        TestResult {
            lines: vec![line.to_string()],
            name: m.get_any(1),
            outcome: TestOutcome::Skipped,
        }
        .into()
    }
}
