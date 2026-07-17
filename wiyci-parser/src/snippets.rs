// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

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

#[derive(Debug, PartialEq, Eq, Hash, Clone, IntoStaticStr)]
#[non_exhaustive]
pub enum Snippet {
    CompilerWarning(CompilerWarning),
    TestResult(TestResult),
}

macro_rules! try_from_snippet {
    ($type:ty $(, $variant:ident)+ $(,)?) => {
        impl TryFrom<Snippet> for $type {
            type Error = Snippet;

            fn try_from(snippet: Snippet) -> Result<Self, Self::Error> {
                match snippet {
                    $(Snippet::$variant(snippet) => Ok(snippet),)+
                    other => Err(other),
                }
            }
        }
    }
}

try_from_snippet!(CompilerWarning, CompilerWarning);
try_from_snippet!(TestResult, TestResult);
