// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::HashMap;

use strum::IntoStaticStr;

pub trait Snippet {
    /// Unique value used for limiting number of same-kind items.
    ///
    /// When number of generated snippets is limited, snippets with
    /// different descriptors are considered different, even if these
    /// are of same type.
    ///
    /// Snippets with no descriptor are not subject to limiting.
    fn limiting_discriminant(&self) -> Option<u64> {
        Some(0)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct CompilerWarning {
    pub lines: Vec<String>,
    pub path: String,
    pub line_number: u32,
    pub category: String,
    pub message: String,
}

impl Snippet for CompilerWarning {}

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

impl Snippet for TestResult {
    fn limiting_discriminant(&self) -> Option<u64> {
        Some(self.outcome as u64)
    }
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

        impl AnySnippet {
            pub fn kind(&self) -> SnippetKind {
                match self {
                    $(Self::$kind(..) => SnippetKind::$kind,)+
                }
            }
        }

        impl Snippet for AnySnippet {
            fn limiting_discriminant(&self) -> Option<u64> {
                match self {
                    $(Self::$kind(snippet) => snippet.limiting_discriminant(),)+
                }
            }
        }

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

            pub fn push(&mut self, snippet: AnySnippet) {
                match snippet {
                    $(AnySnippet::$kind(snippet) => self.get_mut().push(snippet),)+
                }
            }
        }
    }
}

declare_snippets! {
    CompilerWarning,
    TestResult,
}
