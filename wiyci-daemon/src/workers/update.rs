// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

mod tasks;

use std::time::{Duration, Instant};

use metrics::{counter, histogram};
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::{debug, error, info, warn};

use wiyci_common::api;
use wiyci_common::db;
use wiyci_common::models::projects::Project;

use crate::HttpClient;

const RETRY_INTERVAL: Duration = Duration::from_mins(1);
const ITERATION_INTERVAL: Duration = Duration::from_secs(5);

const ACTIVE_PROJECT_UPDATE_PERIOD: Duration = Duration::from_days(1);
const INACTIVE_PROJECT_UPDATE_PERIOD: Duration = Duration::from_days(7);

pub struct UpdateProjectsWorker {
    pool: PgPool,
    client: HttpClient,
}

struct Status {
    num_tasks: usize,
}

impl UpdateProjectsWorker {
    pub fn new(pool: PgPool, client: HttpClient) -> Self {
        Self { pool, client }
    }

    async fn update_project_inner(&self, project: &Project) -> anyhow::Result<Status> {
        let repology_packages =
            api::repology::fetch_project_packages(&self.client, &project.name).await?;

        let tasks = match tasks::generate_tasks(&repology_packages) {
            Ok(tasks) => tasks,
            Err(error) => {
                warn!(%error, "failed to generate tasks");
                Default::default()
            }
        };

        let update_period = if tasks.is_empty() {
            INACTIVE_PROJECT_UPDATE_PERIOD
        } else {
            ACTIVE_PROJECT_UPDATE_PERIOD
        };

        db::fetch_tasks::update_tasks_for_project(&self.pool, &project.name, &tasks).await?;

        db::projects::register_update(&self.pool, &project.name, tasks.len() as u32, update_period)
            .await?;

        Ok(Status {
            num_tasks: tasks.len(),
        })
    }

    #[cfg_attr(
        not(coverage),
        tracing::instrument(name = "update_project", skip_all, fields(project_name = project.name))
    )]
    async fn update_project(&self, project: &Project) -> anyhow::Result<()> {
        let start = Instant::now();

        let overdue_seconds = (OffsetDateTime::now_utc() - project.next_update_at)
            .as_seconds_f64()
            .max(0.0);

        info!(overdue_seconds, "updating project");
        histogram!("wiyci_daemon_project_update_overdue_age_seconds").record(overdue_seconds);

        let res = self.update_project_inner(project).await;

        let duration_seconds = Instant::now()
            .saturating_duration_since(start)
            .as_secs_f64();

        histogram!("wiyci_daemon_project_update_duration_seconds").record(duration_seconds);

        match &res {
            Ok(status) => {
                counter!("wiyci_daemon_project_updates_total", "status" => "success", "type" => if status.num_tasks == 0 { "inactive" } else { "active" }).increment(1);
                info!(
                    duration_seconds,
                    num_tasks = status.num_tasks,
                    "project updated"
                );
            }
            Err(_) => {
                counter!("wiyci_daemon_project_updates_total", "status" => "failed").increment(1);
            }
        }
        res.map(|_| ())
    }

    async fn process_next_task(&self) -> anyhow::Result<bool> {
        let Some(project) = db::projects::get_next_for_update(&self.pool).await? else {
            return Ok(false);
        };

        self.update_project(&project).await?;
        Ok(true)
    }

    #[cfg_attr(
        not(coverage),
        tracing::instrument(name = "UpdateProjectsWorker", skip_all)
    )]
    pub async fn run(&self) -> anyhow::Result<()> {
        loop {
            match self.process_next_task().await {
                Ok(true) => {}
                Ok(false) => {
                    debug!("waiting for tasks");
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
