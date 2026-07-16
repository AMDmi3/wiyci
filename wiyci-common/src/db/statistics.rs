// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use indoc::indoc;
use sqlx::PgPool;

use crate::models::statistics::{Statistics, StatisticsDelta};

pub async fn get(pool: &PgPool) -> sqlx::Result<Statistics> {
    let statistics: Statistics = sqlx::query_as(indoc! {"
        SELECT *
          FROM statistics
         LIMIT 1
    "})
    .fetch_one(pool)
    .await?;
    Ok(statistics)
}

pub async fn apply_delta(pool: &PgPool, delta: &StatisticsDelta) -> sqlx::Result<()> {
    sqlx::query(indoc! {"
        UPDATE statistics
           SET stored_logs_size = GREATEST(stored_logs_size + COALESCE($1, 0), 0)
    "})
    .bind(delta.stored_logs_size)
    .execute(pool)
    .await?;
    Ok(())
}
