// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use dom_query::Document;
use http::StatusCode;

use wiyci_common::models::fetch_tasks::{FetchTask, FetchTaskParams};

use crate::HttpClient;
use crate::workers::fetch::{FetchImpl, FetchReject};

pub struct NixFetchImpl;

async fn fetch_job(client: &HttpClient, params: &FetchTaskParams) -> Result<String, FetchReject> {
    let expected_version_suffix = if let Some(version) = &params.version {
        format!("-{}", version)
    } else {
        return Err(FetchReject::Internal(
            "incomplete fetch params for the fetch impl".into(),
        ));
    };

    let response = client
        .get(&params.url)
        .send()
        .await
        .map_err(FetchReject::RequestFailed)?;

    if response.status() != StatusCode::OK {
        return Err(FetchReject::BadHttpCode(response.status()));
    }

    let content = response
        .text()
        .await
        .map_err(|e| FetchReject::RequestFailed(e.into()))?;

    let document = Document::from(content);

    for row in document.select("table > tbody > tr").iter() {
        if row
            .select(r#"td:nth-child(1) > img[title="Succeeded"]"#)
            .is_empty()
        {
            continue;
        }
        if !row
            .select("td:nth-child(4)")
            .text()
            .as_ref()
            .ends_with(&expected_version_suffix)
        {
            continue;
        }

        if let Some(url) = row.select("td:nth-child(2) > a").attr("href").as_ref() {
            return Ok(url.into());
        }
    }

    Err(FetchReject::ParseError(
        "cannot parse last successful build link".into(),
    ))
}

async fn fetch_build(client: &HttpClient, url: &str) -> Result<String, FetchReject> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(FetchReject::RequestFailed)?;

    if response.status() != StatusCode::OK {
        return Err(FetchReject::BadHttpCode(response.status()));
    }

    let content = response
        .text()
        .await
        .map_err(|e| FetchReject::RequestFailed(e.into()))?;

    let document = Document::from(content);

    document
        .select(r#"a:has-text("raw")"#)
        .attr("href")
        .map(|v| v.to_string())
        .ok_or_else(|| FetchReject::ParseError("cannot parse last log link".into()))
}

impl FetchImpl for NixFetchImpl {
    async fn fetch(
        &self,
        client: &HttpClient,
        fetch_task: &FetchTask,
    ) -> Result<reqwest::Response, FetchReject> {
        let build_url = fetch_job(client, &fetch_task.params).await?;
        let log_url = fetch_build(client, &build_url).await?;

        match client.get(&log_url).send().await {
            Ok(response) => Ok(response),
            Err(error) => Err(FetchReject::RequestFailed(error)),
        }
    }
}
