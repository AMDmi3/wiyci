// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::time::Duration;

use indoc::indoc;
use sqlx::Postgres;

use crate::models::fetch_tasks::{FetchTask, NewFetchTask};

pub async fn register_failure(
    conn: impl sqlx::Acquire<'_, Database = Postgres>,
    id: i32,
    error: &str,
    retry_after: Option<Duration>,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    sqlx::query(indoc! {"
        UPDATE fetch_tasks
           SET num_attempts = num_attempts + 1
             , last_fetch_attempted_at = now()
             , next_fetch_attempt_at = now() + $3
             , last_error = $2
         WHERE id = $1
    "})
    .bind(id)
    .bind(error)
    .bind(retry_after)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn resolve(
    conn: impl sqlx::Acquire<'_, Database = Postgres>,
    id: i32,
    log_id: i32,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    sqlx::query(indoc! {"
        UPDATE fetch_tasks
           SET num_attempts = num_attempts + 1
             , last_fetch_attempted_at = now()
             , next_fetch_attempt_at = NULL
             , log_id = $2
         WHERE id = $1
    "})
    .bind(id)
    .bind(log_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn get_next_for_fetch(
    conn: impl sqlx::Acquire<'_, Database = Postgres>,
) -> sqlx::Result<Option<FetchTask>> {
    let mut tx = conn.begin().await?;

    let task = sqlx::query_as(indoc! {"
          SELECT *
            FROM fetch_tasks
           WHERE next_fetch_attempt_at < now()
        ORDER BY next_fetch_attempt_at, id
           LIMIT 1
    "})
    .fetch_optional(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(task)
}

pub async fn update_tasks_for_project(
    conn: impl sqlx::Acquire<'_, Database = Postgres>,
    project_name: &str,
    tasks: &[NewFetchTask],
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    for task in tasks {
        sqlx::query(indoc! {"
            INSERT INTO fetch_tasks(project_name, url, version, variant, source_pkgname, binary_pkgname)
                 VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (project_name, url)
              DO UPDATE
                    SET version = EXCLUDED.version
                      , variant = EXCLUDED.variant
                      , source_pkgname = EXCLUDED.source_pkgname
                      , binary_pkgname = EXCLUDED.binary_pkgname
                  WHERE fetch_tasks.version IS DISTINCT FROM EXCLUDED.version
                     OR fetch_tasks.variant IS DISTINCT FROM EXCLUDED.variant
                     OR fetch_tasks.source_pkgname IS DISTINCT FROM EXCLUDED.source_pkgname
                     OR fetch_tasks.binary_pkgname IS DISTINCT FROM EXCLUDED.binary_pkgname
        "})
        .bind(project_name)
        .bind(&task.url)
        .bind(&task.version)
        .bind(&task.variant)
        .bind(&task.source_pkgname)
        .bind(&task.binary_pkgname)
        .execute(&mut *tx)
        .await?;
    }

    let actual_urls: Vec<&str> = tasks.iter().map(|task| task.url.as_ref()).collect();

    sqlx::query(indoc! {"
        DELETE FROM fetch_tasks
         WHERE project_name = $1
           AND url != ALL($2)
    "})
    .bind(project_name)
    .bind(&actual_urls)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}
