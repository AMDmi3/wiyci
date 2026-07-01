// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Duration;

use indoc::indoc;
use sqlx::PgPool;

use crate::models::fetch_tasks::{FetchTask, NewFetchTask};

pub async fn register_failure(
    pool: &PgPool,
    id: i32,
    error: &str,
    retry_after: Option<Duration>,
) -> sqlx::Result<()> {
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
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn resolve(pool: &PgPool, id: i32, log_id: i32) -> sqlx::Result<()> {
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
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_next_for_fetch(pool: &PgPool) -> sqlx::Result<Option<FetchTask>> {
    let task = sqlx::query_as(indoc! {"
          SELECT *
            FROM fetch_tasks
           WHERE next_fetch_attempt_at < now()
        ORDER BY next_fetch_attempt_at, id
           LIMIT 1
    "})
    .fetch_optional(pool)
    .await?;
    Ok(task)
}

pub async fn update_tasks_for_project(
    pool: &PgPool,
    project_name: &str,
    tasks: &[NewFetchTask],
) -> sqlx::Result<()> {
    let mut tx = pool.begin().await?;

    for task in tasks {
        sqlx::query(indoc! {"
            INSERT INTO fetch_tasks(project_name, url, variant, version)
                 VALUES ($1, $2, $3, $4)
            ON CONFLICT (project_name, url)
              DO UPDATE
                    SET variant = EXCLUDED.variant
                      , version = EXCLUDED.version
                  WHERE fetch_tasks.variant IS DISTINCT FROM EXCLUDED.variant
                     OR fetch_tasks.version IS DISTINCT FROM EXCLUDED.version
        "})
        .bind(project_name)
        .bind(&task.url)
        .bind(&task.variant)
        .bind(&task.version)
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
