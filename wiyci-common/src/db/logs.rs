// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::HashMap;

use indoc::indoc;
use sqlx::{FromRow, Postgres, types::Json};
use time::OffsetDateTime;

use crate::models::logs::{Log, NewLog, ParsedLog};

pub async fn create(
    conn: impl sqlx::Acquire<'_, Database = Postgres>,
    log: &NewLog,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    sqlx::query(indoc! {"
        INSERT INTO logs(id, fetch_task_id, url, project_name, version, variant, source_pkgname, binary_pkgname, size, last_modified, etag, is_truncated)
             SELECT $1, $2, url, project_name, version, variant, source_pkgname, binary_pkgname, $3, $4, $5, $6
               FROM fetch_tasks
              WHERE id = $2
    "})
    .bind(log.id)
    .bind(log.fetch_task_id)
    .bind(log.size as i32)
    .bind(log.last_modified)
    .bind(&log.etag)
    .bind(log.is_truncated)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

#[derive(FromRow)]
pub struct DbLog {
    pub id: i32,
    pub fetch_task_id: Option<i32>,
    pub created_at: OffsetDateTime,

    pub url: String,
    pub project_name: String,
    pub version: String,
    pub variant: String,
    pub source_pkgname: Option<String>,
    pub binary_pkgname: Option<String>,

    pub size: i32,
    pub last_modified: Option<OffsetDateTime>,
    pub etag: Option<String>,
    pub is_truncated: bool,

    pub parsed_at: Option<OffsetDateTime>,
    pub parser_version: Option<i32>,
    pub parsed_num_lines: Option<i32>,
    pub parsed_snippet_counts: Option<Json<HashMap<String, u64>>>,
}

impl From<DbLog> for Log {
    fn from(db: DbLog) -> Self {
        Self {
            id: db.id,
            fetch_task_id: db.fetch_task_id,
            created_at: db.created_at,

            url: db.url,
            project_name: db.project_name,
            version: db.version,
            variant: db.variant,
            source_pkgname: db.source_pkgname,
            binary_pkgname: db.binary_pkgname,

            size: db.size as u64,
            last_modified: db.last_modified,
            etag: db.etag,
            is_truncated: db.is_truncated,

            parsed_at: db.parsed_at,
            parser_version: db.parser_version.map(|v| v as u32),
            parsed_num_lines: db.parsed_num_lines.map(|v| v as u32),
            parsed_snippet_counts: db.parsed_snippet_counts.map(|json| {
                json.into_inner()
                    .into_iter()
                    .filter_map(|(k, v)| k.parse().ok().map(|k| (k, v)))
                    .collect()
            }),
        }
    }
}

pub async fn get_next_for_parsing(
    conn: impl sqlx::Acquire<'_, Database = Postgres>,
    current_parser_version: u32,
) -> sqlx::Result<Option<Log>> {
    let mut tx = conn.begin().await?;

    let log: Option<DbLog> = sqlx::query_as(indoc! {r#"
          SELECT *
            FROM logs
           WHERE parser_version IS NULL OR parser_version < $1
        ORDER BY parser_version, id
           LIMIT 1
    "#})
    .bind(current_parser_version as i32)
    .fetch_optional(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(log.map(|log| log.into()))
}

pub async fn apply_parsed(
    conn: impl sqlx::Acquire<'_, Database = Postgres>,
    id: i32,
    parsed_log: &ParsedLog,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    sqlx::query(indoc! {"
        UPDATE logs
           SET parsed_at = now()
             , parser_version = $2
             , parsed_num_lines = $3
             , parsed_snippet_counts = $4
         WHERE id = $1
    "})
    .bind(id)
    .bind(parsed_log.parser_version as i32)
    .bind(parsed_log.parsed_num_lines as i32)
    .bind(Json(&parsed_log.parsed_snippet_counts))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn list_for_project(
    conn: impl sqlx::Acquire<'_, Database = Postgres>,
    project_name: &str,
) -> sqlx::Result<Vec<Log>> {
    let mut tx = conn.begin().await?;

    let logs: Vec<DbLog> = sqlx::query_as(indoc! {"
        SELECT *
          FROM logs
         WHERE project_name = $1
           AND parser_version IS NOT NULL
    "})
    .bind(project_name)
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(logs.into_iter().map(|log| log.into()).collect())
}

pub async fn get_by_id(
    conn: impl sqlx::Acquire<'_, Database = Postgres>,
    id: i32,
) -> sqlx::Result<Option<Log>> {
    let mut tx = conn.begin().await?;

    let log: Option<DbLog> = sqlx::query_as(indoc! {"
        SELECT *
          FROM logs
         WHERE id = $1
    "})
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(log.map(|log| log.into()))
}
