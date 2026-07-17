// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use sqlx::PgPool;

use wiyci_common::MIGRATOR;

#[sqlx::test]
async fn test_migrations_reversible(pool: PgPool) {
    MIGRATOR.run(&pool).await.unwrap();
    MIGRATOR.undo(&pool, 0).await.unwrap();
    MIGRATOR.run(&pool).await.unwrap();
    MIGRATOR.undo(&pool, 0).await.unwrap();
}
