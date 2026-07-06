// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(PartialEq, Eq, Hash)]
pub struct NewFetchTask {
    pub url: String,
    pub version: String,
    pub variant: String,
}

#[derive(FromRow)]
pub struct FetchTask {
    pub id: i32,
    pub created_at: OffsetDateTime,

    pub url: String,
    pub project_name: String,
    pub version: String,
    pub variant: String,

    #[sqlx(try_from = "i32")]
    pub num_attempts: u32,
    pub next_fetch_attempt_at: Option<OffsetDateTime>,
    pub last_fetch_attempted_at: Option<OffsetDateTime>,
    pub last_error: Option<String>,
    pub log_id: Option<i32>,
}
