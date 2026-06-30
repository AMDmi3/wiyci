// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use askama::Template;
use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse};
use serde::Deserialize;

use wiyci_common::db;
use wiyci_common::models::projects::Project;

use crate::result::HandlerResult;
use crate::routes::MyRoute;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct QueryParams {
    #[serde(default)]
    pub search: String,
}

#[derive(Template)]
#[template(path = "projects.html")]
struct TemplateParams<'a> {
    my_route: &'a MyRoute,
    projects: &'a [Project],
}

#[cfg_attr(not(coverage), tracing::instrument(skip_all))]
pub async fn projects(
    my_route: MyRoute,
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> HandlerResult {
    let projects = if let Some(search) = Some(&query.search).filter(|s| !s.is_empty()) {
        db::projects::list_by_search(&state.pool, search, crate::constants::PROJECTS_PER_PAGE)
            .await?
    } else {
        db::projects::list_by_range(&state.pool, None, None, crate::constants::PROJECTS_PER_PAGE)
            .await?
    };

    Ok(Html(
        TemplateParams {
            my_route: &my_route,
            projects: &projects,
        }
        .render()?,
    )
    .into_response())
}
