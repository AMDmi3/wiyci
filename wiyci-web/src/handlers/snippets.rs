// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::sync::Arc;

use askama::Template;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};

use wiyci_common::db;
use wiyci_common::models::logs::Log;
use wiyci_common::models::snippets::Snippet;

use crate::result::HandlerResult;
use crate::routes::MyRoute;
use crate::state::AppState;

#[derive(Template)]
#[template(path = "snippets.html")]
struct TemplateParams<'a> {
    my_route: &'a MyRoute,
    log: &'a Log,
    snippets: &'a [Snippet],
}

#[cfg_attr(not(coverage), tracing::instrument(skip_all))]
pub async fn snippets(
    my_route: MyRoute,
    Path(log_id): Path<u32>,
    State(state): State<Arc<AppState>>,
) -> HandlerResult {
    let Some(log) = db::logs::get_by_id(&state.pool, log_id as i32).await? else {
        return Ok((StatusCode::NOT_FOUND, "Log not found").into_response());
    };

    let snippets = db::snippets::list_for_log(&state.pool, log_id as i32).await?;

    Ok(Html(
        TemplateParams {
            my_route: &my_route,
            log: &log,
            snippets: &snippets,
        }
        .render()?,
    )
    .into_response())
}
