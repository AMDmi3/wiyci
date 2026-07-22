// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use metrics::{counter, gauge};
use sqlx::PgPool;
use tracing::{info, info_span};

use wiyci_common::db;
use wiyci_common::models::logs::LogRemovalTask;
use wiyci_common::models::statistics::StatisticsDelta;

use crate::storage::LogStorage;
use crate::workers::util::PollingWorkerRunner;

pub struct RemoveLogsWorker {
    pool: PgPool,
    storage: LogStorage,
}

impl RemoveLogsWorker {
    pub fn new(pool: PgPool, storage: LogStorage) -> Self {
        Self { pool, storage }
    }

    async fn remove_log(&self, task: &LogRemovalTask) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        self.storage.remove(task.id as u64)?;

        #[expect(clippy::needless_update)]
        let statistics = db::statistics::apply_delta(
            &mut tx,
            &StatisticsDelta {
                stored_logs_size: -(task.size as i64),
                ..Default::default()
            },
        )
        .await?;

        db::logs::confirm_removal(&mut *tx, task.id).await?;

        tx.commit().await?;

        gauge!("wiyci_daemon_statistics_stored_logs_size_bytes")
            .set(statistics.stored_logs_size as f64);

        counter!("wiyci_daemon_logs_removed_total").increment(1);
        info!("log removed");

        Ok(())
    }

    #[cfg_attr(not(coverage), tracing::instrument(name = "RemoveLogs", skip_all))]
    pub async fn run(&self) -> anyhow::Result<()> {
        PollingWorkerRunner::new(
            "RemoveLogs",
            async || Ok(db::logs::get_next_for_removal(&self.pool).await?),
            async |task| self.remove_log(task).await,
        )
        .with_span(|task| info_span!("task", id = task.id))
        .run()
        .await
    }
}
