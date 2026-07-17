// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::sync::LazyLock;

use regex::Regex;

use crate::matching::captures::SimplifiedCaptures;
use crate::matching::common::{SnippetMatchResult, SnippetMatcher};
use crate::snippets::{Snippet, TestOutcome, TestResult};

static PATTERN: &str = r"^ *[0-9]+/[0-9]+ Test +#[0-9]+: ([^ ]+) \.+(   Passed|\*\*\*(?:Failed|Exception|Not Run|Skipped))";
static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(PATTERN).unwrap());

#[derive(Default)]
pub struct CtestTestResultMatcher;

impl CtestTestResultMatcher {
    pub const TASTE_PATTERN: &str = PATTERN;
}

impl SnippetMatcher for CtestTestResultMatcher {
    fn match_line(&mut self, line: &str) -> SnippetMatchResult {
        let Some(m) = REGEX.captures(line) else {
            return SnippetMatchResult::NoMatch;
        };

        let outcome = match &m.get_str(2)[3..] {
            "Failed" | "Exception" => TestOutcome::Failed,
            "Passed" => TestOutcome::Passed,
            "Not Run" | "Skipped" => TestOutcome::Skipped,
            _ => {
                unreachable!("nonempty string ensured by the regex");
            }
        };

        Snippet::TestResult(TestResult {
            lines: vec![line.to_string()],
            name: m.get_any(1),
            outcome,
        })
        .into()
    }
}
