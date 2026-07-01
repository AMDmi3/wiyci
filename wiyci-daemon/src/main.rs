// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(duration_constructors)]
#![cfg_attr(test, feature(coverage_attribute))]

mod config;
mod init;
mod storage;
mod workers;

use anyhow::Context as _;
use reqwest_middleware::ClientWithMiddleware as HttpClient;
use tracing::info;

use crate::config::Config;
use crate::init::{init_database, init_http_client, init_logging, init_metrics};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::parse().with_context(|| "failed to process configuration")?;

    init_logging(&config).with_context(|| "failed to init logging")?;
    init_metrics(&config).with_context(|| "failed to init metrics")?;
    let pool = init_database(&config)
        .await
        .with_context(|| "failed to init database")?;

    let client = init_http_client(&config).with_context(|| "failed to init HTTP client")?;

    let storage = storage::LogStorage::new(&config.storage_path);

    if !config.skip_migrations {
        info!("running migrations");

        sqlx::query("CREATE SCHEMA IF NOT EXISTS wiyci")
            .execute(&pool)
            .await
            .context("failed to create schema")?;

        wiyci_common::MIGRATOR
            .run(&pool)
            .await
            .context("failed to run migrations")?;
    }

    info!("running workers");
    let discover = workers::DiscoverProjectsWorker::new(pool.clone());
    let update = workers::UpdateProjectsWorker::new(pool.clone(), client.clone());
    let fetch = workers::FetchLogsWorker::new(pool.clone(), client.clone(), storage.clone());
    tokio::try_join!(discover.run(), update.run(), fetch.run())
        .context("worker finished with error")?;

    Ok(())
}
