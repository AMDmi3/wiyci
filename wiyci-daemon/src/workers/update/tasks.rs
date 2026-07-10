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

/// Get all desired versions for a set of packages.
///
/// Since we act like a CI, we want to present latest "runs" to users,
/// so we want all `newest` and `devel` versions (in Repology terms).
/// Note that there may be more that one of each due to alternative
/// versioning schemes.
fn get_desired_versions(packages: &[RepologyPackage]) -> BTreeSet<Version<&str>> {
    packages
        .iter()
        .filter(|package| {
            package.status == "unique" || package.status == "newest" || package.status == "devel"
        })
        .map(|package| Version::new(package.version.as_ref()))
        .collect()
}

/// Pick greatest version for each desired one.
fn filter_versions_by_desired<'a, 'b>(
    versions: impl IntoIterator<Item = Version<&'a str>>,
    desired_versions: impl IntoIterator<Item = &'b Version<&'b str>>,
) -> BTreeSet<Version<&'a str>> {
    let mut versions = versions.into_iter().peekable();
    let mut desired_versions = desired_versions.into_iter();
    let mut res: BTreeSet<Version<&str>> = Default::default();

    let Some(mut current) = versions.next() else {
        return res;
    };
    let Some(mut next_desired) = desired_versions.next() else {
        return res;
    };
    loop {
        if current > *next_desired {
            // version is past the next disired, advance desired
            match desired_versions.next() {
                Some(v) => {
                    next_desired = v;
                }
                None => {
                    return res;
                }
            }
        } else {
            // if the version is the last up to the next desired, pick it
            // and continue with the next version
            if versions.peek().is_none_or(|v| v > next_desired) {
                res.insert(current);
            }
            match versions.next() {
                Some(v) => {
                    current = v;
                }
                None => {
                    return res;
                }
            }
        }
        // loop is finite as either of iterators always advances
    }
}

/// Generate a list of allowed versions for each repository.
///
/// Though we're generally interested only in `newest` and `devel` versions
/// (in Repology terms), not all repositories may provide these at the given
/// moment, and if we strictly limit to `newest` and `devel`, we may lose
/// unique (though outdated) log content.
///
/// To fix that we loosen the requirement, and allow latest available version(s)
/// before each of desired version for a given repository.
fn get_allowed_versions_per_repo(
    packages: &[RepologyPackage],
) -> HashMap<String, BTreeSet<Version<&str>>> {
    let desired_versions = get_desired_versions(packages);

    let mut versions_per_repo: HashMap<String, BTreeSet<Version<&str>>> = Default::default();
    for package in packages {
        if package.status == "unique"
            || package.status == "newest"
            || package.status == "devel"
            || package.status == "outdated"
            || package.status == "legacy"
        {
            versions_per_repo
                .entry(package.repo.clone())
                .or_default()
                .insert(Version::new(&package.version));
        }
    }

    versions_per_repo
        .into_iter()
        .map(|(k, v)| (k, filter_versions_by_desired(v, &desired_versions)))
        .collect()
}

/// Generate log fetching tasks from Repology packages.
pub fn generate_tasks(packages: &[RepologyPackage]) -> Vec<NewFetchTask> {
    let allowed_versions_per_repo = get_allowed_versions_per_repo(packages);

    let mut tasks: Vec<NewFetchTask> = vec![];

    for package in packages {
        if allowed_versions_per_repo
            .get(&package.repo)
            .is_some_and(|allowed| allowed.contains(&Version::new(package.version.as_ref())))
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
    }

    tasks
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use libversion::Version;

    use super::*;

    #[test]
    fn test_filter_versions_by_desired_no_exact_match() {
        assert_eq!(
            filter_versions_by_desired(
                vec![
                    Version::new("1.0"),
                    Version::new("1.2"),
                    Version::new("2.0"),
                    Version::new("2.2"),
                ],
                &vec![Version::new("1.1"), Version::new("2.1")]
            )
            .into_iter()
            .collect::<Vec<_>>(),
            vec![Version::new("1.0"), Version::new("2.0")]
        );
    }

    #[test]
    fn test_filter_versions_by_desired_exact_match() {
        assert_eq!(
            filter_versions_by_desired(
                vec![
                    Version::new("1.0"),
                    Version::new("1.1"),
                    Version::new("1.2"),
                    Version::new("2.0"),
                    Version::new("2.1"),
                    Version::new("2.2"),
                ],
                &vec![Version::new("1.1"), Version::new("2.1")]
            )
            .into_iter()
            .collect::<Vec<_>>(),
            vec![Version::new("1.1"), Version::new("2.1")]
        );
    }

    #[test]
    fn test_filter_versions_by_desired_no_match() {
        assert_eq!(
            filter_versions_by_desired(
                vec![
                    Version::new("2.0"),
                    Version::new("2.1"),
                    Version::new("2.2"),
                ],
                &vec![Version::new("1.1")]
            )
            .into_iter()
            .collect::<Vec<_>>(),
            Vec::<Version<&str>>::new()
        );
    }

    #[test]
    fn test_filter_versions_by_desired_no_desired() {
        assert_eq!(
            filter_versions_by_desired(
                vec![
                    Version::new("1.0"),
                    Version::new("1.1"),
                    Version::new("1.2"),
                ],
                vec![]
            )
            .into_iter()
            .collect::<Vec<_>>(),
            Vec::<Version<&str>>::new()
        );
    }

    #[test]
    fn test_filter_versions_by_desired_empty_desired_range() {
        assert_eq!(
            filter_versions_by_desired(
                vec![Version::new("1.0"), Version::new("3.0"),],
                &vec![
                    Version::new("1.0"),
                    Version::new("2.0"),
                    Version::new("3.0")
                ]
            )
            .into_iter()
            .collect::<Vec<_>>(),
            vec![Version::new("1.0"), Version::new("3.0"),]
        );
    }
}
