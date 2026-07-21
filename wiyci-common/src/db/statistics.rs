// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::HashMap;

use indoc::indoc;
use sqlx::{FromRow, Postgres, types::Json};

use crate::db::common::parse_snippet_counts;
use crate::models::statistics::{SnippetCountStatistics, Statistics, StatisticsDelta};

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

#[derive(FromRow)]
pub struct DbSnippetCountStatistics {
    pub num_projects: i64,
    pub num_snippets: Option<Json<HashMap<String, i64>>>,
    pub num_projects_by_snippet: Option<Json<HashMap<String, i64>>>,
}

impl From<DbSnippetCountStatistics> for SnippetCountStatistics {
    fn from(db: DbSnippetCountStatistics) -> Self {
        Self {
            num_projects: db.num_projects as u64,
            num_snippets: db
                .num_snippets
                .map(parse_snippet_counts)
                .unwrap_or_default(),
            num_projects_by_snippet: db
                .num_projects_by_snippet
                .map(parse_snippet_counts)
                .unwrap_or_default(),
        }
    }
}

pub async fn get_snippet_counts(
    conn: impl sqlx::Acquire<'_, Database = Postgres>,
) -> sqlx::Result<SnippetCountStatistics> {
    let mut tx = conn.begin().await?;

    let statistics: DbSnippetCountStatistics = sqlx::query_as(indoc! {"
        WITH
            expanded AS (
                SELECT name,
                       key::TEXT,
                       value::INTEGER
                  FROM projects
                     , jsonb_each(snippet_counts) AS _(key, value)
            )
          , aggregated AS (
                SELECT key
                     , SUM(value) AS value_sum
                     , COUNT(value) AS value_count
                  FROM expanded
                 GROUP BY key
            )
        SELECT
            COUNT(*) AS num_projects
          , (
                SELECT jsonb_object_agg(key, value_sum)
                  FROM aggregated
            ) AS num_snippets
          , (
                SELECT jsonb_object_agg(key, value_count)
                FROM aggregated
            ) AS num_projects_by_snippet
        FROM projects
    "})
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(statistics.into())
}
