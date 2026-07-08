// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::snippets::AnySnippet;

pub enum SnippetMatchResult {
    NoMatch,
    Continued,
    Complete(AnySnippet),
}

impl<T> From<T> for SnippetMatchResult
where
    T: Into<AnySnippet>,
{
    fn from(snippet: T) -> Self {
        Self::Complete(snippet.into())
    }
}

pub trait SnippetMatcher {
    fn match_line(&mut self, line: &str) -> SnippetMatchResult;
    fn flush(&mut self) -> SnippetMatchResult {
        SnippetMatchResult::NoMatch
    }
}
