// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, sqlx::Type)]
#[sqlx(type_name = "text")]
#[non_exhaustive]
pub enum FetchTaskKind {
    Alpine,
    Fedora,
    FreeBsd,
    Nix,
}

// Note: make sure to add skip_serializing_if for any new optional
// fields, so serialized params for existing tasks do not change,
// otherwise ALL logs will have to be refetched
#[derive(Debug, Eq, PartialEq, Hash, Default, Serialize, Deserialize)]
pub struct FetchTaskParams {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pkgname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(PartialEq, Eq, Hash)]
pub struct NewFetchTask {
    pub kind: FetchTaskKind,
    pub params: FetchTaskParams,
    pub version: String,
    pub variant: String,
    pub source_pkgname: Option<String>,
    pub binary_pkgname: Option<String>,
}

#[derive(FromRow)]
pub struct FetchTask {
    pub id: i32,
    pub created_at: OffsetDateTime,

    #[sqlx(json)]
    pub params: FetchTaskParams,
    pub project_name: String,
    pub version: String,
    pub variant: String,
    pub source_pkgname: Option<String>,
    pub binary_pkgname: Option<String>,

    #[sqlx(try_from = "i32")]
    pub num_attempts: u32,
    pub next_fetch_attempt_at: Option<OffsetDateTime>,
    pub last_fetch_attempted_at: Option<OffsetDateTime>,
    pub last_error: Option<String>,
    pub log_id: Option<i32>,
}
