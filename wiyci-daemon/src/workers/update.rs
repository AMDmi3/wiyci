// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

mod tasks;

use std::time::{Duration, Instant};

use metrics::{counter, histogram};
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::{error, info};

use wiyci_common::api;
use wiyci_common::db;

use crate::HttpClient;

const RETRY_INTERVAL: Duration = Duration::from_mins(1);
const ITERATION_INTERVAL: Duration = Duration::from_secs(5);

const ACTIVE_PROJECT_UPDATE_PERIOD: Duration = Duration::from_days(1);
const INACTIVE_PROJECT_UPDATE_PERIOD: Duration = Duration::from_days(7);

pub struct UpdateProjectsWorker {
    pool: PgPool,
    client: HttpClient,
}

impl UpdateProjectsWorker {
    pub fn new(pool: PgPool, client: HttpClient) -> Self {
        Self { pool, client }
    }

    async fn update_next_project(&self) -> anyhow::Result<bool> {
        let start = Instant::now();

        let Some(project) = db::projects::get_next_for_update(&self.pool).await? else {
            return Ok(false);
        };

        let overdue_seconds = (OffsetDateTime::now_utc() - project.next_update_at)
            .as_seconds_f64()
            .max(0.0);

        info!(project.name, overdue_seconds, "updating project");
        histogram!("wiyci_daemon_project_update_overdue_age_seconds").record(overdue_seconds);

        let repology_packages =
            api::repology::fetch_project_packages(self.client.as_ref(), &project.name).await?;

        info!(
            project.name,
            num_packages = repology_packages.len(),
            "fetched repology packages"
        );

        let tasks = tasks::generate_tasks(&repology_packages);

        let update_period = if tasks.is_empty() {
            INACTIVE_PROJECT_UPDATE_PERIOD
        } else {
            ACTIVE_PROJECT_UPDATE_PERIOD
        };

        db::fetch_tasks::update_tasks_for_project(&self.pool, &project.name, &tasks).await?;

        db::projects::register_update(&self.pool, &project.name, tasks.len() as u32, update_period)
            .await?;

        let check_duration_seconds = Instant::now()
            .saturating_duration_since(start)
            .as_secs_f64();

        info!(
            project.name,
            check_duration_seconds,
            num_tasks = tasks.len(),
            "project updated"
        );
        histogram!("wiyci_daemon_project_update_duration_seconds").record(check_duration_seconds);
        counter!("wiyci_daemon_project_updates_total", "type" => if tasks.is_empty() { "inactive" } else { "active" }).increment(1);

        Ok(true)
    }

    #[cfg_attr(
        not(coverage),
        tracing::instrument(name = "UpdateProjectsWorker", skip_all)
    )]
    pub async fn run(&self) -> anyhow::Result<()> {
        loop {
            match self.update_next_project().await {
                Ok(true) => {}
                Ok(false) => {
                    info!("waiting for projects to update");
                    tokio::time::sleep(ITERATION_INTERVAL).await;
                }
                Err(error) => {
                    error!(%error, "failure in worker iteration");
                    tokio::time::sleep(RETRY_INTERVAL).await;
                }
            }
        }
    }
}
