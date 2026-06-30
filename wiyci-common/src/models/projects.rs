// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(FromRow)]
pub struct Project {
    pub name: String,
    pub created_at: OffsetDateTime,
    #[sqlx(try_from = "i32")]
    pub num_tasks: u32,
    pub next_update_at: OffsetDateTime,
    pub last_updated_at: Option<OffsetDateTime>,
}
