// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use indoc::indoc;
use sqlx::Postgres;

use crate::models::statistics::{Statistics, StatisticsDelta};

pub async fn get(conn: impl sqlx::Acquire<'_, Database = Postgres>) -> sqlx::Result<Statistics> {
    let mut tx = conn.begin().await?;

    let statistics: Statistics = sqlx::query_as(indoc! {"
        SELECT *
          FROM statistics
         LIMIT 1
    "})
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(statistics)
}

pub async fn apply_delta(
    conn: impl sqlx::Acquire<'_, Database = Postgres>,
    delta: &StatisticsDelta,
) -> sqlx::Result<Statistics> {
    let mut tx = conn.begin().await?;

    let statistics: Statistics = sqlx::query_as(indoc! {"
           UPDATE statistics
              SET stored_logs_size = GREATEST(stored_logs_size + COALESCE($1, 0), 0)
        RETURNING *
    "})
    .bind(delta.stored_logs_size)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(statistics)
}
