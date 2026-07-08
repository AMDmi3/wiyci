// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::LazyLock;

use regex::RegexSet;

use crate::matching::common::SnippetMatcher;
use crate::matching::matchers::*;

macro_rules! snippet_matcher_factory {
    ($($matcher:ident),+ $(,)?) => {
        static REGEX_SET: LazyLock<RegexSet> = LazyLock::new(|| RegexSet::new([
            $(
                $matcher::TASTE_PATTERN,
            )+
        ]).unwrap());

        pub fn try_spawn_matchers(line: &str) -> impl Iterator<Item = Box<dyn SnippetMatcher>> {
            let matches = REGEX_SET.matches(line);
            let mut res: Vec<Box<dyn SnippetMatcher>> = vec![];
            let mut _index = 0;
            $(
                if matches.matched(_index) {
                    res.push(Box::new($matcher::default()));
                }
                _index += 1;
            )+
            res.into_iter()
        }
    }
}

snippet_matcher_factory! {
    CtestTestResultMatcher,
    GtestTestResultMatcher,
    PytestTestResultMatcher,
    CompilerWarningMatcher,
}
