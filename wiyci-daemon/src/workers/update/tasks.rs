// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

mod alpine;
mod fedora;
mod freebsd;

use std::collections::{BTreeSet, HashMap};

use libversion::Version;
use tracing::error;

use wiyci_common::models::fetch_tasks::NewFetchTask;
use wiyci_common::models::repology::RepologyPackage;

struct VersionStatistics<'a> {
    newest_version: Version<&'a str>,
}

impl<'a> VersionStatistics<'a> {
    fn gather(packages: &'a [RepologyPackage]) -> anyhow::Result<Self> {
        let mut newest_versions: BTreeSet<Version<&'a str>> = Default::default();
        for package in packages {
            if package.status == "newest" {
                newest_versions.insert(Version::new(&package.version));
            }
        }

        if newest_versions.is_empty() {
            anyhow::bail!("no multiple newest version");
        } else if newest_versions.len() > 1 {
            anyhow::bail!("multiple newest versions: {:?}", newest_versions);
        }

        Ok(Self {
            newest_version: newest_versions
                .into_iter()
                .next()
                .expect("element existence checked just before"),
        })
    }
}

#[derive(Default)]
struct LatestPackages<'a> {
    stable_packages: Vec<&'a RepologyPackage>,
}

impl<'a> LatestPackages<'a> {
    fn push(&mut self, package: &'a RepologyPackage, statistics: &VersionStatistics) {
        let this_version = Version::new(&package.version);

        if let Some(prev_version) = self
            .stable_packages
            .first()
            .map(|package| Version::new(&package.version))
        {
            match this_version.cmp(&prev_version) {
                std::cmp::Ordering::Greater if this_version > statistics.newest_version => {
                    return;
                }
                std::cmp::Ordering::Greater => self.stable_packages.clear(),
                std::cmp::Ordering::Equal => {}
                std::cmp::Ordering::Less => {
                    return;
                }
            }
        }

        self.stable_packages.push(package);
    }
}

pub fn generate_tasks(packages: &[RepologyPackage]) -> anyhow::Result<Vec<NewFetchTask>> {
    let statistics = VersionStatistics::gather(packages)?;
    let mut latest_per_repo: HashMap<String, LatestPackages> = Default::default();

    for package in packages {
        // TODO: we could support devel packages
        if package.status != "newest" && package.status != "outdated" && package.status != "legacy"
        {
            continue;
        }

        latest_per_repo
            .entry(package.repo.clone())
            .or_default()
            .push(package, &statistics);
    }

    let mut tasks: Vec<NewFetchTask> = vec![];

    for package in latest_per_repo
        .into_values()
        .flat_map(|latest| latest.stable_packages.into_iter())
    {
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

    Ok(tasks.into_iter().collect())
}
