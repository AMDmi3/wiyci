// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::LazyLock;

use regex::Regex;

use crate::matchers::common::{SnippetMatchResult, SnippetMatcher};
use crate::matching::SimplifiedCaptures;
use crate::snippets::CompilerWarning;

// XXX: is position (second number after path) mandatory?
// Note: the regex ensures line number fits into any 32 bit integer
static PATTERN: &str =
    r"^(.*\.(?:c|cc|cxx|cpp|cc|h|hh|hpp|hxx)):([0-9]{1,9}):[0-9]+: (warning: .* \[(-W.*)\])$";
static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(PATTERN).unwrap());

struct Details {
    path: String,
    line_number: u32,
    category: String,
    message: String,
}

#[derive(Default)]
pub struct CompilerWarningMatcher;

impl CompilerWarningMatcher {
    pub const TASTE_PATTERN: &str = PATTERN;
}

impl SnippetMatcher for CompilerWarningMatcher {
    fn match_line(&mut self, line: &str) -> SnippetMatchResult {
        let Some(m) = REGEX.captures(line) else {
            return SnippetMatchResult::NoMatch;
        };

        let details = Details {
            path: m.get_any(1),
            line_number: m.get_any(2),
            category: m.get_any(4),
            message: m.get_any(3),
        };

        CompilerWarning {
            lines: vec![line.to_string()],
            path: details.path,
            line_number: details.line_number,
            category: details.category,
            message: details.message,
        }
        .into()
    }
}
