// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use axum_test::TestServer;
use sqlx::PgPool;

use wiyci_web::create_app;

#[sqlx::test(migrator = "wiyci_common::MIGRATOR")]
async fn test_projects_empty(pool: PgPool) {
    let server = TestServer::new(create_app(pool).await.unwrap());
    insta::assert_snapshot!(server.get("/projects").await);
}

#[sqlx::test(migrator = "wiyci_common::MIGRATOR", fixtures("firefox_project"))]
async fn test_projects_nonempty(pool: PgPool) {
    let server = TestServer::new(create_app(pool).await.unwrap());
    insta::assert_snapshot!(server.get("/projects").await);
}

#[sqlx::test(migrator = "wiyci_common::MIGRATOR", fixtures("firefox_project"))]
async fn test_projects_search(pool: PgPool) {
    let server = TestServer::new(create_app(pool).await.unwrap());
    insta::assert_snapshot!(server.get("/projects?search=irefo").await);
}
