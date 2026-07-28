// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::time::Duration;

use indoc::indoc;
use sqlx::Postgres;

use crate::models::bulk_update::BulkUpdateStatus;

pub async fn get_pending_status(
    conn: impl sqlx::Acquire<'_, Database = Postgres>,
) -> sqlx::Result<Option<BulkUpdateStatus>> {
    let mut tx = conn.begin().await?;

    let status: Option<BulkUpdateStatus> = sqlx::query_as(indoc! {"
        SELECT *
          FROM bulk_update_status
         WHERE last_project_name IS NOT NULL
            OR next_update_at <= now()
         LIMIT 1
    "})
    .fetch_optional(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(status)
}

pub async fn finish_batch(
    conn: impl sqlx::Acquire<'_, Database = Postgres>,
    last_project_name: &str,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    sqlx::query(indoc! {"
         UPDATE bulk_update_status
            SET last_project_name = $1
    "})
    .bind(last_project_name)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn finish_update(
    conn: impl sqlx::Acquire<'_, Database = Postgres>,
    period: Duration,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    sqlx::query(indoc! {"
         UPDATE bulk_update_status
            SET last_project_name = NULL
              , next_update_at = GREATEST(next_update_at + $1, now())
    "})
    .bind(period)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}
