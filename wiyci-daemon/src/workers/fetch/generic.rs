// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use wiyci_common::models::fetch_tasks::FetchTask;

use crate::HttpClient;
use crate::workers::fetch::{FetchImpl, FetchReject};

pub struct GenericFetchImpl;

impl FetchImpl for GenericFetchImpl {
    async fn fetch(
        &self,
        client: &HttpClient,
        fetch_task: &FetchTask,
    ) -> Result<reqwest::Response, FetchReject> {
        match client.get(&fetch_task.params.url).send().await {
            Ok(response) => Ok(response),
            Err(error) => Err(FetchReject::RequestFailed(error)),
        }
    }
}
