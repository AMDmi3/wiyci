// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::time::Duration;

use indoc::indoc;
use sqlx::{FromRow, Postgres, types::Json};
use time::OffsetDateTime;

use crate::models::projects::Project;

pub async fn create(
    conn: impl sqlx::Acquire<'_, Database = Postgres>,
    name: &str,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    sqlx::query(indoc! {"
        INSERT INTO projects(name)
             VALUES ($1)
        ON CONFLICT (name)
         DO NOTHING
    "})
    .bind(name)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn register_update(
    conn: impl sqlx::Acquire<'_, Database = Postgres>,
    name: &str,
    num_tasks: u32,
    update_period: Duration,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    sqlx::query(indoc! {"
        UPDATE projects
           SET last_updated_at = now()
             , num_tasks = $2
             , next_update_at = now() + $3
         WHERE name = $1
    "})
    .bind(name)
    .bind(num_tasks as i32)
    .bind(update_period)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

#[derive(FromRow)]
pub struct DbProject {
    pub name: String,
    pub created_at: OffsetDateTime,
    pub num_tasks: i32,
    pub next_update_at: OffsetDateTime,
    pub last_updated_at: Option<OffsetDateTime>,
    pub snippet_counts: Option<Json<HashMap<String, u64>>>,
}

impl From<DbProject> for Project {
    fn from(db: DbProject) -> Self {
        Self {
            name: db.name,
            created_at: db.created_at,
            num_tasks: db.num_tasks as u32,
            next_update_at: db.next_update_at,
            last_updated_at: db.last_updated_at,
            snippet_counts: db
                .snippet_counts
                .map(|json| {
                    json.into_inner()
                        .into_iter()
                        .filter_map(|(k, v)| k.parse().ok().map(|k| (k, v)))
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
}

pub async fn get_by_name(
    conn: impl sqlx::Acquire<'_, Database = Postgres>,
    name: &str,
) -> sqlx::Result<Option<Project>> {
    let mut tx = conn.begin().await?;

    let project: Option<DbProject> = sqlx::query_as(indoc! {"
        SELECT *
          FROM projects
         WHERE name = $1
    "})
    .bind(name)
    .fetch_optional(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(project.map(|project| project.into()))
}

pub async fn get_next_for_update(
    conn: impl sqlx::Acquire<'_, Database = Postgres>,
) -> sqlx::Result<Option<Project>> {
    let mut tx = conn.begin().await?;

    let project: Option<DbProject> = sqlx::query_as(indoc! {"
          SELECT *
            FROM projects
           WHERE next_update_at < now()
        ORDER BY next_update_at, name
           LIMIT 1
    "})
    .fetch_optional(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(project.map(|project| project.into()))
}

pub async fn list_latest(
    conn: impl sqlx::Acquire<'_, Database = Postgres>,
    count: u64,
) -> sqlx::Result<Vec<Project>> {
    let mut tx = conn.begin().await?;

    let projects: Vec<DbProject> = sqlx::query_as(indoc! {"
          SELECT *
            FROM projects
        ORDER BY created_at DESC
           LIMIT $1
    "})
    .bind(count as i64)
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(projects.into_iter().map(|project| project.into()).collect())
}

pub async fn list_by_range(
    conn: impl sqlx::Acquire<'_, Database = Postgres>,
    from: Option<&str>,
    to: Option<&str>,
    count: u64,
) -> sqlx::Result<Vec<Project>> {
    let mut tx = conn.begin().await?;

    let projects: Vec<DbProject> = sqlx::query_as(match (from, to) {
        (Some(_), Some(_)) => indoc! {"
              SELECT *
                FROM projects
               WHERE name >= $1
                 AND name <= $2
            ORDER BY name
               LIMIT $3
        "},
        (Some(_), None) => indoc! {"
              SELECT *
                FROM projects
               WHERE name >= $1
            ORDER BY name
               LIMIT $3
        "},
        (None, Some(_)) => indoc! {"
              SELECT *
                FROM projects
               WHERE name <= $2
            ORDER BY name
               LIMIT $3
        "},
        (None, None) => indoc! {"
              SELECT *
                FROM projects
            ORDER BY name
               LIMIT $3
        "},
    })
    .bind(from)
    .bind(to)
    .bind(count as i64)
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(projects.into_iter().map(|project| project.into()).collect())
}

fn escape_like(input: &str) -> String {
    input
        .chars()
        .flat_map(|ch| {
            let escape = matches!(ch, '\\' | '%' | '_').then(|| '\\');
            escape.into_iter().chain(std::iter::once(ch))
        })
        .collect()
}

pub async fn list_by_search(
    conn: impl sqlx::Acquire<'_, Database = Postgres>,
    search_term: &str,
    count: u64,
) -> sqlx::Result<Vec<Project>> {
    let mut tx = conn.begin().await?;

    let escaped_search_term = escape_like(search_term);

    let projects: Vec<DbProject> = sqlx::query_as(indoc! {"
        (SELECT *
           FROM projects
          WHERE name = $1)

          UNION ALL
    
        (SELECT *
           FROM projects
          WHERE name LIKE ($2 || '%')
            AND name != $1
          ORDER BY name
          LIMIT $3)

          UNION ALL
    
        (SELECT *
           FROM projects
          WHERE name LIKE ('%' || $2 || '%')
            AND name NOT LIKE ($2 || '%')
          ORDER BY name
          LIMIT $3)

        LIMIT $3
    "})
    .bind(search_term)
    .bind(&escaped_search_term)
    .bind(count as i64)
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(projects.into_iter().map(|project| project.into()).collect())
}

pub async fn update_snippet_counts(
    conn: impl sqlx::Acquire<'_, Database = Postgres>,
    name: &str,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    sqlx::query(indoc! {"
        WITH
            new_counts AS (
                SELECT key
                     , MAX(value::BIGINT) AS value
                  FROM logs, jsonb_each(parsed_snippet_counts) AS counts(key, value)
                 WHERE project_name = $1
                 GROUP BY key
            )
        UPDATE projects
           SET snippet_counts = (
                   SELECT jsonb_object_agg(key, value)
                     FROM new_counts
               )
         WHERE projects.name = $1
    "})
    .bind(name)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_like() {
        assert_eq!(escape_like("abcd"), "abcd");
        assert_eq!(escape_like("%a(b_c)d%"), r#"\%a(b\_c)d\%"#);
        assert_eq!(escape_like(r#"\%\"#), r#"\\\%\\"#);
    }
}
