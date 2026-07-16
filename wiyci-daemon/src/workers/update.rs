// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

mod tasks;

use std::time::Duration;

use metrics::{counter, histogram};
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::{info, info_span};

use wiyci_common::api;
use wiyci_common::db;
use wiyci_common::models::projects::Project;

use crate::HttpClient;
use crate::workers::util::PollingWorkerRunner;

const ACTIVE_PROJECT_UPDATE_PERIOD: Duration = Duration::from_days(1);
const INACTIVE_PROJECT_UPDATE_PERIOD: Duration = Duration::from_days(7);

pub struct UpdateWorker {
    pool: PgPool,
    client: HttpClient,
}

impl UpdateWorker {
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

        let tasks = tasks::generate_tasks(&repology_packages);

        let update_period = if tasks.is_empty() {
            INACTIVE_PROJECT_UPDATE_PERIOD
        } else {
            ACTIVE_PROJECT_UPDATE_PERIOD
        };

        db::fetch_tasks::update_tasks_for_project(&self.pool, &project.name, &tasks).await?;

        db::projects::register_update(&self.pool, &project.name, tasks.len() as u32, update_period)
            .await?;

        counter!("wiyci_daemon_update_projects_total", "type" => if tasks.is_empty() { "inactive" } else { "active" }).increment(1);
        counter!("wiyci_daemon_update_tasks_total").increment(tasks.len() as u64);
        info!(
            project_name = project.name,
            num_tasks = tasks.len(),
            "project updated"
        );

        Ok(())
    }

    #[cfg_attr(not(coverage), tracing::instrument(name = "Update", skip_all))]
    pub async fn run(&self) -> anyhow::Result<()> {
        PollingWorkerRunner::new(
            "Update",
            async || Ok(db::projects::get_next_for_update(&self.pool).await?),
            async |project| self.update_project(project).await,
        )
        .with_span(|project| info_span!("project", name = project.name))
        .run()
        .await
    }
}
