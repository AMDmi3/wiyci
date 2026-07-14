// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use axum_test::TestServer;
use sqlx::PgPool;

use wiyci_web::create_app;

#[sqlx::test(migrator = "wiyci_common::MIGRATOR")]
async fn test_root(pool: PgPool) {
    let server = TestServer::new(create_app(pool).await.unwrap());
    let response = server.get("/").await;
    response.assert_status_ok();
    response.assert_header("content-type", "text/html; charset=utf-8");
    // croaks on <div> inside <dl>, though it's valid HTML5
    // https://html.spec.whatwg.org/multipage/grouping-content.html#the-dl-element
    //assert!(
    //    !tidier::Doc::new(response.text(), false)
    //        .unwrap()
    //        .has_issues()
    //);
}
