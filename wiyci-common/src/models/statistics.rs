// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::FromRow;

#[derive(FromRow)]
pub struct Statistics {
    #[sqlx(try_from = "i64")]
    pub stored_logs_size: u64,
}

#[derive(Default)]
pub struct StatisticsDelta {
    pub stored_logs_size: i64,
}
