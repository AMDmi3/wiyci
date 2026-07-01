// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context;
use clap::Parser;
use serde::Deserialize;
use url::Url;

const DEFAULT_DSN: &str = "postgresql://wiyci@localhost/wiyci";
const DEFAULT_HTTP_TIMEOUT: Duration = Duration::from_secs(60);
const DEFAULT_HTTP_DELAY: Duration = Duration::from_secs(3);

// Note: do not use default values for args which are also present in
// FileConfig, otherwise config settings will always be overwritten
// by default clap value. Also, since clap does not allow to provide
// default values but not use them, we have to fill default values
// in options docs manually,
#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// Path to configuration file with default and/or additional settings
    #[arg(short = 'c', long, value_name = "PATH")]
    config: Option<PathBuf>,

    /// PostgreSQL database DSN
    ///
    /// Default: postgresql://wiyci@localhost/wiyci
    #[arg(short = 'd', long = "dsn", value_name = "DSN")]
    dsn: Option<String>,

    /// Do not run database schema migrations
    #[arg(long)]
    skip_migrations: bool,

    /// Path to log directory
    ///
    /// When specified, output is redirected to a log file in the
    /// given directory with daily rotation and 14 kept rotated files.
    #[arg(long, value_name = "PATH")]
    log_directory: Option<PathBuf>,

    /// Loki log collector URL
    #[arg(long, value_name = "URL")]
    loki_url: Option<Url>,

    /// Socket address for serving Prometheus metrics
    #[arg(long, value_name = "ADDR:PORT")]
    prometheus_export: Option<SocketAddr>,

    /// Frontend hostname
    ///
    /// This is used in HTTP User-Agent header submitted by the daemon
    #[arg(long, value_name = "HOST")]
    frontend_hostname: Option<String>,

    /// Timeout for HTTP requests
    ///
    /// Default: 60 seconds
    #[arg(long)]
    http_timeout: Option<f64>,

    /// Delay between HTTP requests to the same host
    ///
    /// Default: 3 seconds
    #[arg(long)]
    http_delay: Option<f64>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
#[serde(deny_unknown_fields)]
struct FileConfig {
    dsn: Option<String>,
    log_directory: Option<PathBuf>,
    loki_url: Option<Url>,
    prometheus_export: Option<SocketAddr>,
    skip_migrations: bool,
    frontend_hostname: Option<String>,
    http_timeout: Option<f64>,
    http_delay: Option<f64>,
}

#[derive(Debug)]
pub struct Config {
    pub dsn: String,
    pub log_directory: Option<PathBuf>,
    pub loki_url: Option<Url>,
    pub prometheus_export: Option<SocketAddr>,
    pub skip_migrations: bool,
    pub frontend_hostname: Option<String>,
    pub http_timeout: Duration,
    pub http_delay: Duration,
}

impl Config {
    pub fn parse() -> anyhow::Result<Self> {
        let args = CliArgs::parse();

        let config: FileConfig = if let Some(path) = args.config {
            // XXX: a good case for try block to avoid with_context repetition, but heterogeneous
            // try blocks are currently broken, see https://github.com/rust-lang/rust/issues/149025
            let toml = std::fs::read(&path)
                .with_context(|| format!("cannot read config file {}", path.display()))?;
            let toml = std::str::from_utf8(&toml)
                .with_context(|| format!("cannot parse config file {}", path.display()))?;
            toml::from_str(toml)
                .with_context(|| format!("cannot parse config file {}", path.display()))?
        } else {
            Default::default()
        };

        Ok(Config {
            dsn: args
                .dsn
                .or(config.dsn)
                .unwrap_or_else(|| DEFAULT_DSN.to_string()),
            log_directory: args.log_directory.or(config.log_directory),
            loki_url: args.loki_url.or(config.loki_url),
            prometheus_export: args.prometheus_export.or(config.prometheus_export),
            skip_migrations: args.skip_migrations || config.skip_migrations,
            frontend_hostname: args
                .frontend_hostname
                .or(config.frontend_hostname)
                .map(|host| host.trim_end_matches('/').to_string()),
            http_timeout: args
                .http_timeout
                .or(config.http_timeout)
                .map(Duration::from_secs_f64)
                .unwrap_or(DEFAULT_HTTP_TIMEOUT),
            http_delay: args
                .http_delay
                .or(config.http_delay)
                .map(Duration::from_secs_f64)
                .unwrap_or(DEFAULT_HTTP_DELAY),
        })
    }
}
