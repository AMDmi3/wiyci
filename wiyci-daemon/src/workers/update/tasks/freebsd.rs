// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::bail;

use wiyci_common::models::fetch_tasks::NewFetchTask;
use wiyci_common::models::repology::RepologyPackage;

struct BuilderInfo {
    variant: &'static str,
    url: &'static str,
}

const BUILDERS: &[BuilderInfo] = &[
    BuilderInfo {
        variant: "150amd64",
        url: "https://pkg-status.freebsd.org/beefy23/data/latest-per-pkg/{binname}/{version}/150amd64-default.log",
    },
    BuilderInfo {
        variant: "150arm64",
        url: "https://pkg-status.freebsd.org/ampere5/data/latest-per-pkg/{binname}/{version}/150arm64-default.log",
    },
];

pub fn generate_tasks<C>(package: &RepologyPackage, tasks: &mut C) -> anyhow::Result<()>
where
    C: Extend<NewFetchTask>,
{
    let version = package.origversion.as_ref().unwrap_or(&package.version);

    let Some(binname) = &package.binname else {
        bail!("no binname");
    };

    for builder in BUILDERS {
        tasks.extend(std::iter::once(NewFetchTask {
            url: builder
                .url
                .replace("{binname}", binname)
                .replace("{version}", version),
            variant: Some(builder.variant.into()),
            version: Some(package.version.clone()),
        }));
    }

    Ok(())
}
