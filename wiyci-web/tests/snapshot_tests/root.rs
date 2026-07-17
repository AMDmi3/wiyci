// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use axum_test::TestServer;
use sqlx::PgPool;

use wiyci_web::create_app;

#[sqlx::test(migrator = "wiyci_common::MIGRATOR")]
async fn test_root(pool: PgPool) {
    let server = TestServer::new(create_app(pool).await.unwrap());
    assert_snapshot!(server.get("/").await);
}
