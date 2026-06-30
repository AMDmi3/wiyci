// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Duration;

use metrics::histogram;
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::{error, info};

use wiyci_common::db;

const RETRY_INTERVAL: Duration = Duration::from_mins(1);
const ITERATION_INTERVAL: Duration = Duration::from_mins(1);

const ACTIVE_PROJECT_UPDATE_PERIOD: Duration = Duration::from_days(1);
const INACTIVE_PROJECT_UPDATE_PERIOD: Duration = Duration::from_days(7);

pub struct UpdateProjectsWorker {
    pool: PgPool,
}

impl UpdateProjectsWorker {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn update_next_project(&self) -> anyhow::Result<bool> {
        let Some(project) = db::projects::get_next_for_update(&self.pool).await? else {
            return Ok(false);
        };

        let overdue_seconds = (OffsetDateTime::now_utc() - project.next_update_at)
            .as_seconds_f64()
            .max(0.0);

        info!(project.name, overdue_seconds, "updating project");

        histogram!("wiyci_daemon_project_update_overdue_age_seconds").record(overdue_seconds);

        let num_tasks: usize = 0;
        let update_period = if num_tasks == 0 {
            INACTIVE_PROJECT_UPDATE_PERIOD
        } else {
            ACTIVE_PROJECT_UPDATE_PERIOD
        };

        db::projects::register_update(&self.pool, &project.name, 0, update_period).await?;

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
