// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::sync::LazyLock;

use regex::Regex;

use crate::matching::captures::SimplifiedCaptures;
use crate::matching::common::{SnippetMatchResult, SnippetMatcher};
use crate::snippets::{Snippet, TestOutcome, TestResult};

static PATTERN: &str = r"^\[ +(OK|SKIPPED|FAILED) +\] ([^ ]+) \([0-9]+ ms\)$";
static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(PATTERN).unwrap());

#[derive(Default)]
pub struct GtestTestResultMatcher;

impl GtestTestResultMatcher {
    pub const TASTE_PATTERN: &str = PATTERN;
}

impl SnippetMatcher for GtestTestResultMatcher {
    fn match_line(&mut self, line: &str) -> SnippetMatchResult {
        let Some(m) = REGEX.captures(line) else {
            return SnippetMatchResult::NoMatch;
        };

        let outcome = match m.get_str(1) {
            "OK" => TestOutcome::Passed,
            "SKIPPED" => TestOutcome::Skipped,
            "FAILED" => TestOutcome::Failed,
            _ => {
                unreachable!("nonempty string ensured by the regex");
            }
        };

        Snippet::TestResult(TestResult {
            lines: vec![line.to_string()],
            name: m.get_any(2),
            outcome,
        })
        .into()
    }
}
