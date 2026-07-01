// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use indoc::indoc;
use sqlx::PgPool;

use crate::models::logs::NewLog;

pub async fn create(pool: &PgPool, log: &NewLog) -> sqlx::Result<()> {
    sqlx::query(indoc! {"
        INSERT INTO logs(id, fetch_task_id, url, project_name, variant, version, size, last_modified, etag)
             SELECT $1, $2, url, project_name, variant, version, $3, $4, $5
               FROM fetch_tasks
              WHERE id = $2
    "})
    .bind(log.id)
    .bind(log.fetch_task_id)
    .bind(log.size as i32)
    .bind(log.last_modified)
    .bind(&log.etag)
    .execute(pool)
    .await?;
    Ok(())
}
