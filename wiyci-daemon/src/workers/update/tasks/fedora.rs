// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use anyhow::bail;

use wiyci_common::models::fetch_tasks::NewFetchTask;
use wiyci_common::models::repology::RepologyPackage;

// XXX: Some logs (for instance, mingw-expat) can reside in noarch directory, should we add it?
const ARCHES: &[&str] = &["aarch64", "i686", "ppc64le", "s390x", "x86_64"];

fn extract_version_revision(raw_version: &str) -> Option<(&str, &str)> {
    raw_version
        .split_once(':')
        .map(|(_, version)| version)
        .unwrap_or(raw_version)
        .rsplit_once('-')
}

pub fn generate_tasks<C>(package: &RepologyPackage, tasks: &mut C) -> anyhow::Result<()>
where
    C: Extend<NewFetchTask>,
{
    let raw_version = package.origversion.as_ref().unwrap_or(&package.version);

    let Some(srcname) = &package.srcname else {
        bail!("no srcname");
    };

    for &arch in ARCHES {
        let Some((version, revision)) = extract_version_revision(raw_version) else {
            bail!("invalid version format");
        };

        tasks.extend(std::iter::once(NewFetchTask {
            url: format!(
                "https://kojipkgs.fedoraproject.org/packages/{srcname}/{version}/{revision}/data/logs/{arch}/build.log",
            ),
            variant: format!("Fedora {}", arch),
            version: package.version.clone(),
            source_pkgname: package.srcname.clone(),
            binary_pkgname: package.binname.clone(),
        }));
    }

    Ok(())
}
