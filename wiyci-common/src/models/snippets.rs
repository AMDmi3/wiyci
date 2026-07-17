// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use strum::{EnumString, IntoStaticStr};

#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    PartialEq,
    Serialize,
    Deserialize,
    EnumString,
    sqlx::Type,
    IntoStaticStr,
)]
#[sqlx(type_name = "text")]
#[non_exhaustive]
pub enum SnippetKind {
    CompilerWarning,
    FailedTest,
    PassedTest,
    SkippedTest,
}

pub struct NewSnippet {
    pub kind: SnippetKind,
    pub text: String,
}

pub struct Snippet {
    pub id: i32,
    pub log_id: i32,
    pub kind: SnippetKind,
    pub lines: Vec<String>,
}
