// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use indoc::indoc;
use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;

use crate::models::logs::{Log, NewLog, ParsedLog};

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

#[derive(FromRow)]
pub struct DbLog {
    pub id: i32,
    pub fetch_task_id: i32,
    pub created_at: OffsetDateTime,

    pub url: String,
    pub project_name: String,
    pub variant: Option<String>,
    pub version: Option<String>,

    pub size: i32,
    pub last_modified: Option<OffsetDateTime>,
    pub etag: Option<String>,

    pub parsed_at: Option<OffsetDateTime>,
    pub parser_version: Option<i32>,
    pub parsed_num_lines: Option<i32>,
}

pub async fn get_next_for_parsing(
    pool: &PgPool,
    current_parser_version: u32,
) -> sqlx::Result<Option<Log>> {
    let log: Option<DbLog> = sqlx::query_as(indoc! {r#"
          SELECT *
            FROM logs
           WHERE parser_version IS NULL OR parser_version < $1
        ORDER BY parser_version, id
           LIMIT 1
    "#})
    .bind(current_parser_version as i32)
    .fetch_optional(pool)
    .await?;
    Ok(log.map(|log| Log {
        id: log.id,
        fetch_task_id: log.fetch_task_id,
        created_at: log.created_at,

        url: log.url,
        project_name: log.project_name,
        variant: log.variant,
        version: log.version,

        size: log.size as u64,
        last_modified: log.last_modified,
        etag: log.etag,

        parsed_at: log.parsed_at,
        parser_version: log.parser_version.map(|v| v as u32),
        parsed_num_lines: log.parsed_num_lines.map(|v| v as u32),
    }))
}

pub async fn apply_parsed(pool: &PgPool, id: i32, parsed_log: &ParsedLog) -> sqlx::Result<()> {
    sqlx::query(indoc! {"
        UPDATE logs
           SET parsed_at = now()
             , parser_version = $2
             , parsed_num_lines = $3
         WHERE id = $1
    "})
    .bind(id)
    .bind(parsed_log.parser_version as i32)
    .bind(parsed_log.parsed_num_lines as i32)
    .execute(pool)
    .await?;
    Ok(())
}
