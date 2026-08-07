// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::time::Duration;

use anyhow::Context as _;
use metrics::counter;
use sqlx::PgPool;
use tracing::{info, info_span};

use wiyci_common::api;
use wiyci_common::db;
use wiyci_common::models::bulk_update::BulkUpdateStatus;

use crate::HttpClient;
use crate::workers::update::common::get_latest_versions;
use crate::workers::update::tasks::generate_tasks;
use crate::workers::util::PollingWorkerRunner;

const PERIOD: Duration = Duration::from_hours(8);
const SINGULAR_UPDATE_FALLBACK_PERIOD: Duration = Duration::from_days(2); // depends on singular update period in fact

pub struct BulkUpdateWorker {
    pool: PgPool,
    client: HttpClient,
    min_spread: u32,
}

impl BulkUpdateWorker {
    pub fn new(pool: PgPool, client: HttpClient, min_spread: u32) -> Self {
        Self {
            pool,
            client,
            min_spread,
        }
    }

    async fn iteration(&self, status: &BulkUpdateStatus) -> anyhow::Result<()> {
        let projects = api::repology::fetch_projects(
            &self.client,
            self.min_spread,
            status.last_project_name.as_deref(),
        )
        .await?;

        counter!("wiyci_daemon_repology_requests_total", "type" => "bulk").increment(1);

        let Some(last_project_name) = projects.keys().max() else {
            info!("update finished");
            db::bulk_update::finish_update(&self.pool, PERIOD).await?;
            return Ok(());
        };

        let mut num_active = 0;
        let mut num_inactive = 0;
        let mut num_tasks = 0;

        let mut tx = self.pool.begin().await?;
        for (project_name, repology_packages) in &projects {
            try {
                let tasks = generate_tasks(repology_packages);
                let latest_versions = get_latest_versions(repology_packages);

                db::projects::create_or_update(
                    &mut tx,
                    project_name,
                    tasks.len() as u32,
                    &latest_versions,
                    SINGULAR_UPDATE_FALLBACK_PERIOD,
                    !tasks.is_empty(),
                )
                .await?;
                db::fetch_tasks::update_tasks_for_project(&mut tx, project_name, &tasks).await?;
                // update latest_snippet_counts which depends on latest_versions which may've changed
                db::projects::update_snippet_counts(&mut tx, project_name).await?;

                if tasks.is_empty() {
                    num_inactive += 1;
                } else {
                    num_active += 1;
                }
                num_tasks += tasks.len() as u64;
            }
            .with_context(|| format!("in project {project_name}"))?;
        }
        db::bulk_update::finish_batch(&mut *tx, last_project_name).await?;
        tx.commit().await?;

        counter!("wiyci_daemon_update_projects_total", "activeness" => "active", "type" => "bulk")
            .increment(num_active);
        counter!("wiyci_daemon_update_projects_total", "activeness" => "inactive", "type" => "bulk")
            .increment(num_inactive);
        counter!("wiyci_daemon_update_tasks_total").increment(num_tasks);
        info!(
            num_projects = &projects.len(),
            start_project_name = status.last_project_name,
            end_project_name = last_project_name,
            "batch finished"
        );

        Ok(())
    }

    #[cfg_attr(not(coverage), tracing::instrument(name = "BulkUpdate", skip_all))]
    pub async fn run(&self) -> anyhow::Result<()> {
        PollingWorkerRunner::new(
            "BulkUpdate",
            async || Ok(db::bulk_update::get_pending_status(&self.pool).await?),
            async |status| self.iteration(status).await,
        )
        .with_span(|status| info_span!("status", last_project_name = status.last_project_name))
        .run()
        .await
    }
}
