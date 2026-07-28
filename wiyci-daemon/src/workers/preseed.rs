// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use sqlx::PgPool;
use tracing::info;

use wiyci_common::db::projects;

static PRESEED_PROJECTS: &[&str] = &["bzip2"];

pub struct PreseedWorker {
    pool: PgPool,
}

impl PreseedWorker {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[cfg_attr(not(coverage), tracing::instrument(name = "Preseed", skip_all))]
    pub async fn run(&self) -> anyhow::Result<()> {
        for project in PRESEED_PROJECTS {
            info!("adding default project {}", project);
            projects::create(&self.pool, project).await?;
        }

        Ok(())
    }
}
