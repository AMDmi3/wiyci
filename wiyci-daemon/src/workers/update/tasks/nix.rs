// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use anyhow::bail;

use wiyci_common::models::fetch_tasks::{FetchTaskKind, FetchTaskParams, NewFetchTask};
use wiyci_common::models::repology::RepologyPackage;

const SYSTEMS: &[&str] = &["x86_64-linux", "aarch64-linux"];

pub fn generate_tasks<C>(package: &RepologyPackage, tasks: &mut C) -> anyhow::Result<()>
where
    C: Extend<NewFetchTask>,
{
    let Some(srcname) = &package.srcname else {
        bail!("no srcname");
    };

    let version = package.origversion.as_ref().unwrap_or(&package.version);

    for &system in SYSTEMS {
        tasks.extend(std::iter::once(NewFetchTask {
            kind: FetchTaskKind::Nix,
            params: FetchTaskParams {
                url: format!(
                    "https://hydra.nixos.org/job/nixpkgs/unstable/{}.{}",
                    srcname, system
                ),
                version: Some(version.into()),
                pkgname: package.srcname.clone(),
            },
            variant: format!("Nix {}", system),
            version: package.version.clone(),
            source_pkgname: package.srcname.clone(),
            binary_pkgname: package.binname.clone(),
        }));
    }

    Ok(())
}
