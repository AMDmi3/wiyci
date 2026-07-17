// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::snippets::Snippet;

pub enum SnippetMatchResult {
    NoMatch,
    Continued,
    Complete(Snippet),
}

impl<T> From<T> for SnippetMatchResult
where
    T: Into<Snippet>,
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
