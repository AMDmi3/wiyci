// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::bail;

use wiyci_common::models::fetch_tasks::NewFetchTask;
use wiyci_common::models::repology::RepologyPackage;

const ARCHES: &[&str] = &["aarch64", "i686", "ppc64le", "s390x", "x86_64"];

pub fn generate_tasks<C>(package: &RepologyPackage, tasks: &mut C) -> anyhow::Result<()>
where
    C: Extend<NewFetchTask>,
{
    let version = package.origversion.as_ref().unwrap_or(&package.version);

    let Some(srcname) = &package.srcname else {
        bail!("no srcname");
    };

    for &arch in ARCHES {
        tasks.extend(std::iter::once(NewFetchTask {
            url: format!(
                "https://kojipkgs.fedoraproject.org/packages/{}/{}/data/logs/{}/build.log",
                srcname,
                version.replace('-', "/"),
                arch
            ),
            variant: format!("Fedora {}", arch),
            version: package.version.clone(),
        }));
    }

    Ok(())
}
