// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::sync::Arc;

use askama::Template;
use axum::extract::State;
use axum::response::{Html, IntoResponse};

use wiyci_common::db;
use wiyci_common::models::snippets::SnippetKind;
use wiyci_common::models::versions::Version;

use crate::result::HandlerResult;
use crate::routes::MyRoute;
use crate::state::AppState;
use crate::time::FormatElapsed;

#[derive(Template)]
#[template(path = "recent.html")]
struct TemplateParams<'a> {
    my_route: &'a MyRoute,
    recent_problematic_versions: &'a [Version],
}

#[cfg_attr(not(coverage), tracing::instrument(skip_all))]
pub async fn recent(my_route: MyRoute, State(state): State<Arc<AppState>>) -> HandlerResult {
    let mut recent_problematic_versions = db::versions::list_recent_problematic_for_known_projects(
        &state.pool,
        crate::constants::RECENT_VERSIONS_PER_PAGE,
    )
    .await?;

    if recent_problematic_versions.len() < (crate::constants::RECENT_VERSIONS_PER_PAGE / 2) as usize
    {
        recent_problematic_versions = db::versions::list_recent_problematic_for_any_projects(
            &state.pool,
            crate::constants::RECENT_VERSIONS_PER_PAGE,
        )
        .await?;
    }

    Ok(Html(
        TemplateParams {
            my_route: &my_route,
            recent_problematic_versions: &recent_problematic_versions,
        }
        .render()?,
    )
    .into_response())
}
