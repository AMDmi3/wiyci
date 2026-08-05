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
#[template(path = "root.html")]
struct TemplateParams<'a> {
    my_route: &'a MyRoute,
    recent_problematic_versions: &'a [Version],
}

#[cfg_attr(not(coverage), tracing::instrument(skip_all))]
pub async fn root(my_route: MyRoute, State(state): State<Arc<AppState>>) -> HandlerResult {
    let recent_problematic_versions = db::versions::list_recent_problematic(
        &state.pool,
        crate::constants::RECENT_VERSIONS_PER_PAGE,
    )
    .await?;

    Ok(Html(
        TemplateParams {
            my_route: &my_route,
            recent_problematic_versions: &recent_problematic_versions,
        }
        .render()?,
    )
    .into_response())
}
