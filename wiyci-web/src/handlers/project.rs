// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use askama::Template;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};

use wiyci_common::db;
use wiyci_common::models::projects::Project;

use crate::result::HandlerResult;
use crate::routes::MyRoute;
use crate::state::AppState;

#[derive(Template)]
#[template(path = "project.html")]
struct TemplateParams<'a> {
    my_route: &'a MyRoute,
    project: &'a Project,
}

#[cfg_attr(not(coverage), tracing::instrument(skip_all))]
pub async fn project(
    my_route: MyRoute,
    Path(name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> HandlerResult {
    let Some(project) = db::projects::get_by_name(&state.pool, &name).await? else {
        return Ok((StatusCode::NOT_FOUND, "Project not found").into_response());
    };

    Ok(Html(
        TemplateParams {
            my_route: &my_route,
            project: &project,
        }
        .render()?,
    )
    .into_response())
}
