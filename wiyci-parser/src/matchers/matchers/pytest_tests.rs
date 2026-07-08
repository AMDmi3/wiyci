// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::LazyLock;

use regex::Regex;

use crate::matchers::common::{SnippetMatchResult, SnippetMatcher};
use crate::matching::SimplifiedCaptures;
use crate::snippets::{TestOutcome, TestResult};

static PATTERN: &str = r"^([^:]+::[^ ]+) FAILED +\[[0-9 ]{3}%\]$";
static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(PATTERN).unwrap());

#[derive(Default)]
pub struct PytestTestResultMatcher;

impl PytestTestResultMatcher {
    pub const TASTE_PATTERN: &str = PATTERN;
}

impl SnippetMatcher for PytestTestResultMatcher {
    fn match_line(&mut self, line: &str) -> SnippetMatchResult {
        let Some(m) = REGEX.captures(line) else {
            return SnippetMatchResult::NoMatch;
        };

        TestResult {
            lines: vec![line.to_string()],
            name: m.get_any(1),
            outcome: TestOutcome::Failed,
        }
        .into()
    }
}
