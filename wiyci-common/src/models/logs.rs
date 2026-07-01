// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::FromRow;
use time::OffsetDateTime;

pub struct NewLog {
    pub id: i32,
    pub fetch_task_id: i32,

    pub size: u64,
    pub last_modified: Option<OffsetDateTime>,
    pub etag: Option<String>,
}

#[derive(FromRow)]
pub struct Log {
    pub id: i32,
    pub fetch_task_id: i32,
    pub created_at: OffsetDateTime,

    pub url: String,
    pub project_name: String,
    pub variant: Option<String>,
    pub version: Option<String>,

    #[sqlx(try_from = "i32")]
    pub size: u64,
    pub last_modified: Option<OffsetDateTime>,
    pub etag: Option<String>,
}
