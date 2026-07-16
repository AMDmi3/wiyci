// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;

use strum::IntoStaticStr;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct CompilerWarning {
    pub lines: Vec<String>,
    pub path: String,
    pub line_number: u32,
    pub category: String,
    pub message: String,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum TestOutcome {
    Passed,
    Failed,
    Skipped,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct TestResult {
    pub lines: Vec<String>,
    pub name: String,
    pub outcome: TestOutcome,
}

macro_rules! declare_snippets {
    ($($kind:ident),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, IntoStaticStr)]
        #[non_exhaustive]
        pub enum SnippetKind {
            $($kind,)+
        }

        pub enum AnySnippet {
            $($kind($kind),)+
        }

        $(
            impl From<$kind> for AnySnippet {
                fn from(snippet: $kind) -> Self {
                    Self::$kind(snippet)
                }
            }
        )+

        typed_storage!(
            #[derive(Clone, Debug)]
            #[non_exhaustive]
            pub SnippetStorage<Vec>{$($kind,)+}
        );

        impl SnippetStorage {
            pub fn counts_per_kind(&self) -> HashMap<SnippetKind, u64> {
                [
                    $(
                        (SnippetKind::$kind, self.get::<$kind>().len() as u64),
                    )+
                ].into_iter().collect()
            }

            pub fn is_empty(&self) -> bool {
                $(
                    self.get::<$kind>().is_empty() &&
                )+
                true
            }

            pub fn len(&self) -> usize {
                $(
                    self.get::<$kind>().len() +
                )+
                0
            }

            pub fn push_with_limit(&mut self, snippet: AnySnippet, limit: Option<usize>) -> bool {
                match snippet {
                    $(
                        AnySnippet::$kind(snippet) => {
                            let storage = self.get_mut();
                            if limit.is_none_or(|limit| storage.len() < limit) {
                                storage.push(snippet);
                                true
                            } else {
                                false
                            }
                        },
                    )+
                }
            }
        }
    }
}

declare_snippets! {
    CompilerWarning,
    TestResult,
}
