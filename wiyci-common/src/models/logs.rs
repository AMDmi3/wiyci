// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;

use time::OffsetDateTime;

use crate::models::snippets::SnippetKind;

pub struct NewLog {
    pub id: i32,
    pub fetch_task_id: i32,

    pub size: u64,
    pub last_modified: Option<OffsetDateTime>,
    pub etag: Option<String>,
    pub is_truncated: bool,
}

pub struct ParsedLog {
    pub parser_version: u32,
    pub parsed_num_lines: u32,
    pub parsed_snippet_counts: HashMap<SnippetKind, u64>,
}

pub struct Log {
    pub id: i32,
    pub fetch_task_id: Option<i32>,
    pub created_at: OffsetDateTime,

    pub url: String,
    pub project_name: String,
    pub version: String,
    pub variant: String,
    pub source_pkgname: Option<String>,
    pub binary_pkgname: Option<String>,

    pub size: u64,
    pub last_modified: Option<OffsetDateTime>,
    pub etag: Option<String>,
    pub is_truncated: bool,

    pub parsed_at: Option<OffsetDateTime>,
    pub parser_version: Option<u32>,
    pub parsed_num_lines: Option<u32>,
    pub parsed_snippet_counts: Option<HashMap<SnippetKind, u64>>,
}
