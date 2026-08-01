// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![feature(duration_constructors)]
#![feature(const_trait_impl)]
#![feature(try_blocks)]
#![cfg_attr(test, feature(coverage_attribute))]

mod config;
mod init;
mod storage;
mod util;
mod workers;

use anyhow::Context as _;
use reqwest_middleware::ClientWithMiddleware as HttpClient;
use std::pin::Pin;
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
    // Note: it turned out to be really hard to implement dynamic worker spawning.
    // Neither of these approaches compile due to (higher-ranked) lifetime errors:
    // - Add Worker trait and store workers in a vec, then map these into futures,
    //   then futures::future::try_join_all these.
    // - Change run() signature to take `self`, so we could fill vec of futures directly,
    // - Use JoinSet and spawn a task for each worker, moving worker into it
    // So instead hardcode the workers unconditionally, but allow to run them conditionally.
    let preseed = workers::PreseedWorker::new(pool.clone());
    let singular_update = workers::SingularUpdateWorker::new(pool.clone(), client.clone());
    let bulk_update = workers::BulkUpdateWorker::new(pool.clone(), client.clone());
    let fetch = workers::FetchWorker::new(pool.clone(), client.clone(), storage.clone());
    let parse = workers::ParseWorker::new(pool.clone(), storage.clone());
    let metrics = workers::MetricsWorker::new(pool.clone());
    let remove_logs = workers::RemoveLogsWorker::new(pool.clone(), storage.clone());
    let expire_logs = workers::ExpireLogsWorker::new(pool.clone());

    let mut futures: Vec<Pin<Box<dyn Future<Output = anyhow::Result<()>>>>> = vec![
        Box::pin(preseed.run()),
        Box::pin(singular_update.run()),
        Box::pin(fetch.run()),
        Box::pin(parse.run()),
        Box::pin(metrics.run()),
        Box::pin(remove_logs.run()),
        Box::pin(expire_logs.run()),
    ];

    if config.enable_bulk_update {
        futures.push(Box::pin(bulk_update.run()));
    }

    futures::future::try_join_all(futures)
        .await
        .context("worker finished with error")?;

    Ok(())
}
