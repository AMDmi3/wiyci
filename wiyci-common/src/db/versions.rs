// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::HashMap;

use indoc::indoc;
use sqlx::{FromRow, Postgres, types::Json};
use time::OffsetDateTime;

use crate::db::common::convert_snippet_counts;
use crate::models::versions::Version;

pub async fn update_snippet_counts(
    conn: impl sqlx::Acquire<'_, Database = Postgres>,
    project_name: &str,
    version: &str,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    sqlx::query(indoc! {"
        WITH
            all_counts AS (
                (SELECT key
                      , value::BIGINT AS value
                   FROM logs
                      , jsonb_each(parsed_snippet_counts) AS counts(key, value)
                  WHERE project_name = $1
                    AND version = $2)

                  UNION ALL

                (SELECT key
                      , value::BIGINT AS value
                   FROM versions
                      , jsonb_each(max_snippet_counts) AS counts(key, value)
                  WHERE project_name = $1
                    AND version = $2)
            )
          , max_counts AS (
                SELECT key
                     , MAX(value) AS value
                  FROM all_counts
                 GROUP BY key
            )

             INSERT INTO versions(project_name, version, max_snippet_counts)
             SELECT $1
                  , $2
                  , (
                        SELECT jsonb_object_agg(key, value)
                          FROM max_counts
                    )
        ON CONFLICT (project_name, version)
          DO UPDATE
                SET max_snippet_counts = EXCLUDED.max_snippet_counts
                  , last_updated_at = now()
    "})
    .bind(project_name)
    .bind(version)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

#[derive(FromRow)]
pub struct DbVersion {
    id: i32,
    created_at: OffsetDateTime,
    last_updated_at: OffsetDateTime,
    project_name: String,
    version: String,
    max_snippet_counts: Option<Json<HashMap<String, u64>>>,
}

impl From<DbVersion> for Version {
    fn from(db: DbVersion) -> Self {
        Self {
            id: db.id,
            created_at: db.created_at,
            last_updated_at: db.last_updated_at,
            project_name: db.project_name,
            version: db.version,
            max_snippet_counts: convert_snippet_counts(db.max_snippet_counts),
        }
    }
}

pub async fn list_recent_problematic_for_any_projects(
    conn: impl sqlx::Acquire<'_, Database = Postgres>,
    limit: u64,
) -> sqlx::Result<Vec<Version>> {
    let mut tx = conn.begin().await?;

    // NOTE: don't forget to sync condition to versions_recent_problematic_idx index
    let versions: Vec<DbVersion> = sqlx::query_as(indoc! {"
    WITH
        candidates AS (
              SELECT *
                FROM versions
               WHERE (versions.max_snippet_counts->>'CompilerWarning')::integer > 0
                  OR (versions.max_snippet_counts->>'FailedTest')::integer > 0
            ORDER BY created_at DESC
               LIMIT $1 * 2
        )
          SELECT versions.*
            FROM candidates AS versions
                 INNER JOIN projects
                 ON versions.project_name = projects.name
           WHERE version = ANY(latest_versions)
        ORDER BY created_at DESC
           LIMIT $1
    "})
    .bind(limit as i64)
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(versions.into_iter().map(|version| version.into()).collect())
}

pub async fn list_recent_problematic_for_known_projects(
    conn: impl sqlx::Acquire<'_, Database = Postgres>,
    limit: u64,
) -> sqlx::Result<Vec<Version>> {
    let mut tx = conn.begin().await?;

    // NOTE: don't forget to sync condition to versions_recent_problematic_idx index
    let versions: Vec<DbVersion> = sqlx::query_as(indoc! {"
    WITH
        candidates AS (
              SELECT *
                FROM versions
               WHERE (versions.max_snippet_counts->>'CompilerWarning')::integer > 0
                  OR (versions.max_snippet_counts->>'FailedTest')::integer > 0
            ORDER BY created_at DESC
               LIMIT $1 * 2
        )
          SELECT versions.*
            FROM candidates AS versions
                 INNER JOIN projects
                 ON versions.project_name = projects.name
           WHERE version = ANY(latest_versions)
             AND projects.created_at < now() - interval '1 week'
        ORDER BY created_at DESC
           LIMIT $1
    "})
    .bind(limit as i64)
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(versions.into_iter().map(|version| version.into()).collect())
}
