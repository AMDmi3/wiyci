// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use anyhow::bail;

use wiyci_common::models::fetch_tasks::NewFetchTask;
use wiyci_common::models::repology::RepologyPackage;

const ARCHES: &[&str] = &[
    "aarch64",
    "armhf",
    "armv7",
    "loongarch64",
    "ppc64le",
    "riscv64",
    "s390x",
    "x86",
    "x86_64",
];

pub fn generate_tasks<C>(package: &RepologyPackage, tasks: &mut C) -> anyhow::Result<()>
where
    C: Extend<NewFetchTask>,
{
    let Some(subrepo) = &package.subrepo else {
        bail!("no subrepo");
    };
    let Some(srcname) = &package.srcname else {
        bail!("no srcname");
    };

    let version = package.origversion.as_ref().unwrap_or(&package.version);

    for &arch in ARCHES {
        tasks.extend(std::iter::once(NewFetchTask {
            url: format!(
                "https://build.alpinelinux.org/buildlogs/build-edge-{}/{}/{}/{}-{}.log",
                arch, subrepo, srcname, srcname, version
            ),
            variant: format!("Alpine {}", arch),
            version: package.version.clone(),
            source_pkgname: package.srcname.clone(),
            binary_pkgname: package.binname.clone(),
        }));
    }

    Ok(())
}
