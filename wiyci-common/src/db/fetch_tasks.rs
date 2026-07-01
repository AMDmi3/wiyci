// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use indoc::indoc;
use sqlx::PgPool;

use crate::models::fetch_tasks::NewFetchTask;

pub async fn update_tasks_for_project(
    pool: &PgPool,
    project_name: &str,
    tasks: &[NewFetchTask],
) -> sqlx::Result<()> {
    let mut tx = pool.begin().await?;

    let mut actual_ids: Vec<i32> = Default::default();

    for task in tasks {
        let id = sqlx::query_scalar(indoc! {"
            INSERT INTO fetch_tasks(project_name, url, variant, version)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT(url) DO UPDATE
            SET project_name = $1,
                variant = $3,
                version = $4,
                last_requested_at = now()
            RETURNING id
        "})
        .bind(project_name)
        .bind(&task.url)
        .bind(&task.variant)
        .bind(&task.version)
        .fetch_one(&mut *tx)
        .await?;

        actual_ids.push(id);
    }

    sqlx::query(indoc! {"
        DELETE FROM fetch_tasks
         WHERE project_name = $1
           AND id != ALL($2)
    "})
    .bind(project_name)
    .bind(&actual_ids)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}
