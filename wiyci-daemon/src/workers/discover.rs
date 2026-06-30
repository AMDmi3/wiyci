// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;
use tracing::info;

use wiyci_common::db::projects;

static DEFAULT_PROJECTS: &[&str] = &["kio"];

pub struct DiscoverProjectsWorker {
    pool: PgPool,
}

impl DiscoverProjectsWorker {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[cfg_attr(
        not(coverage),
        tracing::instrument(name = "DiscoverProjectsWorker", skip_all)
    )]
    pub async fn run(&self) -> anyhow::Result<()> {
        for project in DEFAULT_PROJECTS {
            info!("adding default project {}", project);
            projects::create(&self.pool, project).await?;
        }

        Ok(())
    }
}
