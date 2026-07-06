// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use time::OffsetDateTime;

pub struct NewLog {
    pub id: i32,
    pub fetch_task_id: i32,

    pub size: u64,
    pub last_modified: Option<OffsetDateTime>,
    pub etag: Option<String>,
}

pub struct ParsedLog {
    pub parser_version: u32,
    pub parsed_num_lines: u32,
}

pub struct Log {
    pub id: i32,
    pub fetch_task_id: i32,
    pub created_at: OffsetDateTime,

    pub url: String,
    pub project_name: String,
    pub version: String,
    pub variant: String,

    pub size: u64,
    pub last_modified: Option<OffsetDateTime>,
    pub etag: Option<String>,

    pub parsed_at: Option<OffsetDateTime>,
    pub parser_version: Option<u32>,
    pub parsed_num_lines: Option<u32>,
}
