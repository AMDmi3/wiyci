// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use indoc::indoc;
use sqlx::{FromRow, Postgres};
use uuid::Uuid;
use xxhash_rust::xxh3::xxh3_128;

use crate::models::snippets::{NewSnippet, Snippet};

pub async fn replace_for_log(
    conn: impl sqlx::Acquire<'_, Database = Postgres>,
    log_id: i32,
    snippets: &[NewSnippet],
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    sqlx::query(indoc! {"
        DELETE FROM snippets
         WHERE log_id = $1
    "})
    .bind(log_id)
    .execute(&mut *tx)
    .await?;

    for snippet in snippets {
        if let Some(warning_type) = &snippet.warning_type
            && let Some(warning_message) = &snippet.warning_message
        {
            sqlx::query(indoc! {"
                WITH
                    text AS (
                        INSERT INTO texts (id, text)
                             VALUES ($3, $4)
                        ON CONFLICT (id)
                         DO NOTHING
                    )
                  , warning_type_inserted AS (
                        INSERT INTO warning_types(text)
                             VALUES ($5)
                        ON CONFLICT (text)
                         DO NOTHING
                          RETURNING id
                    )
                  , warning_type_id AS (
                        SELECT id
                          FROM warning_type_inserted

                         UNION ALL

                        SELECT id
                          FROM warning_types
                         WHERE text = $5

                         LIMIT 1
                    )
                  , warning_message_inserted AS (
                        INSERT INTO warning_messages(text)
                             VALUES ($6)
                        ON CONFLICT (text)
                         DO NOTHING
                          RETURNING id
                    )
                  , warning_message_id AS (
                        SELECT id
                          FROM warning_message_inserted

                         UNION ALL

                        SELECT id
                          FROM warning_messages
                         WHERE text = $6

                         LIMIT 1
                    )
                INSERT INTO snippets(log_id, kind, text_id, warning_type_id, warning_message_id)
                     VALUES ($1, $2, $3, (SELECT id FROM warning_type_id), (SELECT id FROM warning_message_id))
            "})
            .bind(log_id)
            .bind(snippet.kind)
            .bind(Uuid::from_u128(xxh3_128(snippet.text.as_bytes())))
            .bind(&snippet.text)
            .bind(warning_type)
            .bind(warning_message)
            .execute(&mut *tx)
            .await?;
        } else {
            sqlx::query(indoc! {"
                WITH
                    text AS (
                        INSERT INTO texts (id, text)
                             VALUES ($3, $4)
                        ON CONFLICT (id)
                         DO NOTHING
                    )
                INSERT INTO snippets(log_id, kind, text_id)
                     VALUES ($1, $2, $3)
            "})
            .bind(log_id)
            .bind(snippet.kind)
            .bind(Uuid::from_u128(xxh3_128(snippet.text.as_bytes())))
            .bind(&snippet.text)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(())
}

#[derive(FromRow)]
pub struct DbSnippet {
    pub id: i32,
    pub log_id: i32,
    pub kind: String,
    pub text: String,
}

impl TryFrom<DbSnippet> for Snippet {
    type Error = ();

    fn try_from(db: DbSnippet) -> Result<Self, Self::Error> {
        Ok(Self {
            id: db.id,
            log_id: db.log_id,
            kind: db.kind.parse().map_err(|_| ())?,
            lines: db.text.split('\n').map(|s| s.to_string()).collect(),
        })
    }
}

pub async fn list_for_log(
    conn: impl sqlx::Acquire<'_, Database = Postgres>,
    log_id: i32,
) -> sqlx::Result<Vec<Snippet>> {
    let mut tx = conn.begin().await?;

    let snippets: Vec<DbSnippet> = sqlx::query_as(indoc! {"
        SELECT snippets.id AS id
             , log_id
             , kind
             , COALESCE(snippets.text, texts.text) AS text
          FROM snippets
               LEFT JOIN texts
               ON texts.id = snippets.text_id
         WHERE log_id = $1
    "})
    .bind(log_id)
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(snippets
        .into_iter()
        .filter_map(|snippet| snippet.try_into().ok())
        .collect())
}
