// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::time::Duration;

use sqlx::PgPool;

use wiyci_common::db;

use crate::workers::util::PeriodicWorkerRunner;

const PERIOD: Duration = Duration::from_secs(60);

pub struct ExpireLogsWorker {
    pool: PgPool,
}

impl ExpireLogsWorker {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn iteration(&self) -> anyhow::Result<()> {
        db::logs::expire_obsolete(&self.pool).await?;
        Ok(())
    }

    #[cfg_attr(not(coverage), tracing::instrument(name = "ExpireLogs", skip_all))]
    pub async fn run(&self) -> anyhow::Result<()> {
        PeriodicWorkerRunner::new("ExpireLogs", async || self.iteration().await, PERIOD)
            .run()
            .await
    }
}
