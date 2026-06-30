// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use axum_test::TestServer;
use sqlx::PgPool;

use wiyci_web::create_app;

#[sqlx::test(migrator = "wiyci_common::MIGRATOR")]
async fn test_project_not_found(pool: PgPool) {
    let server = TestServer::new(create_app(pool).await.unwrap());
    let response = server.get("/project/nonexistent").await;
    response.assert_status_not_found();
}

#[sqlx::test(migrator = "wiyci_common::MIGRATOR", fixtures("firefox_project"))]
async fn test_project_existing(pool: PgPool) {
    let server = TestServer::new(create_app(pool).await.unwrap());
    let response = server.get("/project/firefox").await;
    response.assert_status_ok();
    response.assert_header("content-type", "text/html; charset=utf-8");
    response.assert_text_contains("firefox");
    assert!(
        !tidier::Doc::new(response.text(), false)
            .unwrap()
            .has_issues()
    );
}
