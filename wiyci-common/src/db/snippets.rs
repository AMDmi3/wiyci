// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use indoc::indoc;
use sqlx::{FromRow, PgPool};

use crate::models::snippets::{NewSnippet, Snippet};

pub async fn replace_for_log(
    pool: &PgPool,
    log_id: i32,
    snippets: &[NewSnippet],
) -> sqlx::Result<()> {
    let mut tx = pool.begin().await?;

    sqlx::query(indoc! {"
        DELETE FROM snippets
         WHERE log_id = $1
    "})
    .bind(log_id)
    .execute(&mut *tx)
    .await?;

    for snippet in snippets {
        sqlx::query(indoc! {"
            INSERT INTO snippets(log_id, kind, text)
                 VALUES ($1, $2, $3)
        "})
        .bind(log_id)
        .bind(snippet.kind)
        .bind(&snippet.text)
        .execute(&mut *tx)
        .await?;
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
            text: db.text.split('\n').map(|s| s.to_string()).collect(),
        })
    }
}

pub async fn list_for_log(pool: &PgPool, log_id: i32) -> sqlx::Result<Vec<Snippet>> {
    let snippets: Vec<DbSnippet> = sqlx::query_as(indoc! {"
        SELECT *
          FROM snippets
         WHERE log_id = $1
    "})
    .bind(log_id)
    .fetch_all(pool)
    .await?;
    Ok(snippets
        .into_iter()
        .filter_map(|snippet| snippet.try_into().ok())
        .collect())
}
