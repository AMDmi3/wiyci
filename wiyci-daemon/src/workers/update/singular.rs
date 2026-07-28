// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::time::Duration;

use metrics::{counter, histogram};
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::{info, info_span};

use wiyci_common::api;
use wiyci_common::db;
use wiyci_common::models::projects::Project;

use crate::HttpClient;
use crate::util::duration::DurationExt;
use crate::workers::update::common::get_latest_versions;
use crate::workers::update::tasks::generate_tasks;
use crate::workers::util::PollingWorkerRunner;

const ACTIVE_PROJECT_UPDATE_PERIOD: Duration = Duration::from_days(1);
const INACTIVE_PROJECT_UPDATE_PERIOD: Duration = Duration::from_days(7);
const UPDATE_PERIOD_JITTER: f64 = 0.1;

pub struct SingularUpdateWorker {
    pool: PgPool,
    client: HttpClient,
}

impl SingularUpdateWorker {
    pub fn new(pool: PgPool, client: HttpClient) -> Self {
        Self { pool, client }
    }

    async fn update_project(&self, project: &Project) -> anyhow::Result<()> {
        histogram!("wiyci_daemon_update_overdue_age_seconds").record(
            (OffsetDateTime::now_utc() - project.next_update_at)
                .as_seconds_f64()
                .max(0.0),
        );

        let repology_packages =
            api::repology::fetch_project_packages(&self.client, &project.name).await?;

        let tasks = generate_tasks(&repology_packages);
        let latest_versions = get_latest_versions(&repology_packages);

        let mut tx = self.pool.begin().await?;
        db::projects::create_or_update(
            &mut tx,
            &project.name,
            tasks.len() as u32,
            &latest_versions,
            (if !tasks.is_empty() {
                ACTIVE_PROJECT_UPDATE_PERIOD
            } else {
                INACTIVE_PROJECT_UPDATE_PERIOD
            })
            .with_jitter(UPDATE_PERIOD_JITTER)
            .trimmed_to_micros(),
            false,
        )
        .await?;
        db::fetch_tasks::update_tasks_for_project(&mut tx, &project.name, &tasks).await?;
        // update latest_snippet_counts which depends on latest_versions which may've changed
        db::projects::update_snippet_counts(&mut tx, &project.name).await?;
        tx.commit().await?;

        counter!("wiyci_daemon_update_projects_total", "activeness" => if tasks.is_empty() { "inactive" } else { "active" }, "type" => "singular").increment(1);
        counter!("wiyci_daemon_update_tasks_total").increment(tasks.len() as u64);
        info!(num_tasks = tasks.len(), "project updated");

        Ok(())
    }

    #[cfg_attr(not(coverage), tracing::instrument(name = "SingularUpdate", skip_all))]
    pub async fn run(&self) -> anyhow::Result<()> {
        PollingWorkerRunner::new(
            "SingularUpdate",
            async || Ok(db::projects::get_next_for_update(&self.pool).await?),
            async |project| self.update_project(project).await,
        )
        .with_span(|project| info_span!("project", name = project.name))
        .run()
        .await
    }
}
