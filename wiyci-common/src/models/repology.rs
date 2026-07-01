// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RepologyPackage {
    pub repo: String,
    pub subrepo: Option<String>,
    pub srcname: Option<String>,
    pub binname: Option<String>,
    #[serde(default)]
    pub binnames: Vec<String>,
    pub version: String,
    pub origversion: Option<String>,
    pub status: String,
}
