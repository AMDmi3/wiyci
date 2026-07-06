// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::sync::Arc;

use askama::Template;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use itertools::Itertools as _;

use wiyci_common::db;
use wiyci_common::models::logs::Log;
use wiyci_common::models::projects::Project;

use crate::result::HandlerResult;
use crate::routes::MyRoute;
use crate::state::AppState;

#[derive(Template)]
#[template(path = "project.html")]
struct TemplateParams<'a> {
    my_route: &'a MyRoute,
    project: &'a Project,
    sections: &'a Vec<Section>,
}

struct Section {
    version: String,
    logs: Vec<Log>,
}

#[cfg_attr(not(coverage), tracing::instrument(skip_all))]
pub async fn project(
    my_route: MyRoute,
    Path(project_name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> HandlerResult {
    let Some(project) = db::projects::get_by_name(&state.pool, &project_name).await? else {
        return Ok((StatusCode::NOT_FOUND, "Project not found").into_response());
    };

    let mut sections: HashMap<String, Section> = Default::default();

    for log in db::logs::list_for_project(&state.pool, &project_name).await? {
        sections
            .entry(log.version.clone())
            .or_insert_with(|| Section {
                version: log.version.clone(),
                logs: Default::default(),
            })
            .logs
            .push(log);
    }

    let sections: Vec<_> = sections
        .into_values()
        .map(|mut section| {
            section.logs.sort_unstable_by(|a, b| {
                a.datetime()
                    .cmp(&b.datetime())
                    .then(a.id.cmp(&b.id)) // just to ensure stable order
                    .reverse()
            });
            section
        })
        .sorted_unstable_by(|a, b| libversion::version_compare2(&a.version, &b.version).reverse())
        .collect();

    Ok(Html(
        TemplateParams {
            my_route: &my_route,
            project: &project,
            sections: &sections,
        }
        .render()?,
    )
    .into_response())
}
