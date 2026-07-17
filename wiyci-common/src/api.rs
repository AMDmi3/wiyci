// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

pub mod repology {
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
}
