// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::sync::Arc;

use askama::Template;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use itertools::Itertools as _;
use time::OffsetDateTime;

use wiyci_common::db;
use wiyci_common::models::logs::Log;
use wiyci_common::models::projects::Project;
use wiyci_common::models::snippets::SnippetKind;

use crate::result::HandlerResult;
use crate::routes::MyRoute;
use crate::state::AppState;

enum Verdict {
    NotParsed,
    Success,
    Warning,
    Failure,
}

#[derive(Default)]
struct Counts {
    lines: u64,
    warnings: u64,
    passed_tests: u64,
    failed_tests: u64,
    skipped_tests: u64,
}

impl Counts {
    fn has_tests(&self) -> bool {
        self.passed_tests > 0 || self.failed_tests > 0 || self.skipped_tests > 0
    }
}

struct LogSummary {
    id: i32,
    created_at: OffsetDateTime,
    url: String,
    version: String,
    variant: String,
    source_pkgname: Option<String>,
    binary_pkgname: Option<String>,
    size: u64,
    last_modified: Option<OffsetDateTime>,
    counts: Counts,
    verdict: Verdict,
}

impl From<Log> for LogSummary {
    fn from(log: Log) -> Self {
        let mut counts = Counts {
            lines: log.parsed_num_lines.unwrap_or_default() as u64,
            ..Default::default()
        };

        if let Some(snippet_counts) = log.parsed_snippet_counts {
            counts.warnings = snippet_counts
                .get(&SnippetKind::CompilerWarning)
                .copied()
                .unwrap_or_default();
            counts.failed_tests = snippet_counts
                .get(&SnippetKind::FailedTest)
                .copied()
                .unwrap_or_default();
        };

        let mut verdict = Verdict::NotParsed;
        if log.parsed_at.is_some() {
            verdict = Verdict::Success;
        }
        if counts.warnings > 0 {
            verdict = Verdict::Warning;
        }
        if counts.failed_tests > 0 {
            verdict = Verdict::Failure;
        }

        Self {
            id: log.id,
            created_at: log.created_at,
            url: log.url,
            version: log.version,
            variant: log.variant,
            source_pkgname: log.source_pkgname,
            binary_pkgname: log.binary_pkgname,
            size: log.size,
            last_modified: log.last_modified,
            counts,
            verdict,
        }
    }
}

impl LogSummary {
    pub fn freshness(&self) -> OffsetDateTime {
        self.last_modified.unwrap_or(self.created_at)
    }
}

#[derive(Template)]
#[template(path = "project.html")]
struct TemplateParams<'a> {
    my_route: &'a MyRoute,
    project: &'a Project,
    sections: &'a Vec<Section>,
}

struct Section {
    version: String,
    logs: Vec<LogSummary>,
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
        let log: LogSummary = log.into();
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
                a.freshness()
                    .cmp(&b.freshness())
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
