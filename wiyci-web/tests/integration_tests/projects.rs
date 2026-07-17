// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use axum_test::TestServer;
use sqlx::PgPool;

use wiyci_web::create_app;

#[sqlx::test(migrator = "wiyci_common::MIGRATOR")]
async fn test_projects_empty(pool: PgPool) {
    let server = TestServer::new(create_app(pool).await.unwrap());
    let response = server.get("/projects").await;
    response.assert_status_ok();
    response.assert_header("content-type", "text/html; charset=utf-8");
    assert!(
        !tidier::Doc::new(response.text(), false)
            .unwrap()
            .has_issues()
    );
}

#[sqlx::test(migrator = "wiyci_common::MIGRATOR", fixtures("firefox_project"))]
async fn test_project_nonempty(pool: PgPool) {
    let server = TestServer::new(create_app(pool).await.unwrap());
    let response = server.get("/projects").await;
    response.assert_status_ok();
    response.assert_header("content-type", "text/html; charset=utf-8");
    response.assert_text_contains("firefox");
    assert!(
        !tidier::Doc::new(response.text(), false)
            .unwrap()
            .has_issues()
    );
}

#[sqlx::test(migrator = "wiyci_common::MIGRATOR", fixtures("firefox_project"))]
async fn test_project_search(pool: PgPool) {
    let server = TestServer::new(create_app(pool).await.unwrap());
    let response = server.get("/projects?search=irefo").await;
    response.assert_status_ok();
    response.assert_header("content-type", "text/html; charset=utf-8");
    response.assert_text_contains("firefox");
    assert!(
        !tidier::Doc::new(response.text(), false)
            .unwrap()
            .has_issues()
    );
}
