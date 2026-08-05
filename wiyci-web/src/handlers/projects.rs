// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::sync::Arc;

use askama::Template;
use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse};
use serde::Deserialize;

use wiyci_common::db;
use wiyci_common::models::projects::Project;
use wiyci_common::models::snippets::SnippetKind;

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
    search_term: &'a str,
    is_too_many_results: bool,
}

#[cfg_attr(not(coverage), tracing::instrument(skip_all))]
pub async fn projects(
    my_route: MyRoute,
    Query(query): Query<QueryParams>,
    State(state): State<Arc<AppState>>,
) -> HandlerResult {
    let mut projects = if let Some(search) = Some(&query.search).filter(|s| !s.is_empty()) {
        db::projects::list_by_search(&state.pool, search, crate::constants::PROJECTS_PER_PAGE + 1)
            .await?
    } else {
        db::projects::list_by_range(
            &state.pool,
            None,
            None,
            crate::constants::PROJECTS_PER_PAGE + 1,
        )
        .await?
    };

    let is_too_many_results = if projects.len() > crate::constants::PROJECTS_PER_PAGE as usize {
        projects.pop();
        true
    } else {
        false
    };

    Ok(Html(
        TemplateParams {
            my_route: &my_route,
            projects: &projects,
            search_term: &query.search,
            is_too_many_results,
        }
        .render()?,
    )
    .into_response())
}
