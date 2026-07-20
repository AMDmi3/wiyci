// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use indoc::indoc;
use sqlx::Postgres;

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
