// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(FromRow)]
pub struct BulkUpdateStatus {
    pub last_project_name: Option<String>,
    pub next_update_at: OffsetDateTime,
}
