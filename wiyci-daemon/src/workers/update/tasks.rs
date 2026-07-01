// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

mod alpine;
mod fedora;
mod freebsd;

use std::collections::HashSet;

use tracing::error;

use wiyci_common::models::fetch_tasks::NewFetchTask;
use wiyci_common::models::repology::RepologyPackage;

pub fn generate_tasks(packages: &[RepologyPackage]) -> Vec<NewFetchTask> {
    let mut tasks: HashSet<NewFetchTask> = Default::default();

    for package in packages {
        // TODO: may include devel here in the future, as long as
        // multiple versions can by handled in the webapp consistently
        if package.status != "newest" {
            continue;
        }
        let res = match package.repo.as_ref() {
            "alpine_edge" => alpine::generate_tasks(package, &mut tasks),
            "fedora_rawhide" => fedora::generate_tasks(package, &mut tasks),
            "freebsd" => freebsd::generate_tasks(package, &mut tasks),
            _ => continue,
        };

        if let Err(err) = res {
            error!(
                "error processing package in repository {}: {:?}",
                package.repo, err
            );
        }
    }

    tasks.into_iter().collect()
}
