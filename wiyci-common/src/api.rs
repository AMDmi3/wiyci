// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

pub mod repology {
    use std::collections::HashMap;

    use crate::models::repology::RepologyPackage;

    pub async fn fetch_project_packages(
        client: &reqwest_middleware::ClientWithMiddleware,
        project_name: &str,
    ) -> reqwest_middleware::Result<Vec<RepologyPackage>> {
        let packages: Vec<RepologyPackage> = client
            .get(format!(
                "https://repology.org/api/v1/project/{}",
                project_name
            ))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(packages)
    }

    pub async fn fetch_projects(
        client: &reqwest_middleware::ClientWithMiddleware,
        min_spread: u32,
        prev_project_name: Option<&str>,
    ) -> reqwest_middleware::Result<HashMap<String, Vec<RepologyPackage>>> {
        let url = if let Some(prev_project_name) = prev_project_name {
            format!(
                "https://repology.org/api/v1/projects/{}/?families={}-",
                prev_project_name, min_spread
            )
        } else {
            format!(
                "https://repology.org/api/v1/projects/?families={}-",
                min_spread
            )
        };

        let mut packages: HashMap<String, Vec<RepologyPackage>> = client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        if let Some(prev_project_name) = prev_project_name {
            // the reply includes prev package name, so filter it out
            packages.retain(|k, _| *k != prev_project_name)
        }

        Ok(packages)
    }
}
