// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(PartialEq, Eq, Hash)]
pub struct NewFetchTask {
    pub url: String,
    pub variant: Option<String>,
    pub version: Option<String>,
}

#[derive(FromRow)]
pub struct FetchTask {
    pub id: i32,
    pub created_at: OffsetDateTime,
    pub last_requested_at: Option<OffsetDateTime>,
    pub project_name: String,
    pub url: String,
    pub variant: Option<String>,
    pub version: Option<String>,
}
