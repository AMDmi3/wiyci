// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use askama::Template;
use axum::extract::State;
use axum::response::{Html, IntoResponse};

use crate::result::HandlerResult;
use crate::routes::MyRoute;
use crate::state::AppState;

#[derive(Template)]
#[template(path = "root.html")]
struct TemplateParams<'a> {
    my_route: &'a MyRoute,
}

#[cfg_attr(not(coverage), tracing::instrument(skip_all))]
pub async fn root(
    my_route: MyRoute,
    State(_state /* XXX: will be used */): State<Arc<AppState>>,
) -> HandlerResult {
    Ok(Html(
        TemplateParams {
            my_route: &my_route,
        }
        .render()?,
    )
    .into_response())
}
