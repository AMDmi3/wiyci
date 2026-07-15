// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use sqlx::PgPool;
use tracing::info;

use wiyci_common::db::projects;

// https://repology.org/projects/?families=57- + some manual additions
static DEFAULT_PROJECTS: &[&str] = &[
    "binutils",
    "bzip2",
    "cmake",
    "curl",
    "expat",
    "ffmpeg",
    "file",
    "flac",
    "gcc",
    "glib",
    "gmp",
    "gperf",
    "lame",
    "libffi",
    "libpng",
    "libwebp",
    "libxml2",
    "libxslt",
    "lua",
    "m4",
    "make",
    "nano",
    "openssh",
    "openssl",
    "pcre2",
    "protobuf",
    "python",
    "python:pillow", // for failing pytest tests
    "sdl2",
    "sed",
    "sqlite",
    "vim",
    "xz",
    "zstd",
];

pub struct DiscoverProjectsWorker {
    pool: PgPool,
}

impl DiscoverProjectsWorker {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[cfg_attr(
        not(coverage),
        tracing::instrument(name = "DiscoverProjects", skip_all)
    )]
    pub async fn run(&self) -> anyhow::Result<()> {
        for project in DEFAULT_PROJECTS {
            info!("adding default project {}", project);
            projects::create(&self.pool, project).await?;
        }

        Ok(())
    }
}
