// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use wiyci_common::models::repology::RepologyPackage;

pub fn get_latest_versions(packages: &[RepologyPackage]) -> Vec<&str> {
    let mut res: Vec<&str> = packages
        .iter()
        .filter(|package| package.status == "newest")
        .map(|package| package.version.as_str())
        .collect();
    res.sort_unstable();
    res.dedup();
    res
}
