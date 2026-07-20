// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::time::Duration;

use metrics::gauge;
use sqlx::PgPool;

use wiyci_common::db;

use crate::workers::util::PeriodicWorkerRunner;

const PERIOD: Duration = Duration::from_secs(10);

pub struct MetricsWorker {
    pool: PgPool,
}

impl MetricsWorker {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn iteration(&self) -> anyhow::Result<()> {
        let statistics = db::statistics::get(&self.pool).await?;

        gauge!("wiyci_daemon_statistics_stored_logs_size_bytes")
            .set(statistics.stored_logs_size as f64);

        Ok(())
    }

    #[cfg_attr(not(coverage), tracing::instrument(name = "Metrics", skip_all))]
    pub async fn run(&self) -> anyhow::Result<()> {
        PeriodicWorkerRunner::new("Metrics", async || self.iteration().await, PERIOD)
            .run()
            .await
    }
}
