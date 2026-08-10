// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use dom_query::Document;
use http::StatusCode;

use wiyci_common::models::fetch_tasks::FetchTask;

use crate::HttpClient;
use crate::workers::fetch::{FetchImpl, FetchReject};

pub struct NixFetchImpl;

async fn fetch_job(client: &HttpClient, url: &str) -> Result<String, FetchReject> {
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

    for row in document.select("table > tbody > tr").iter() {
        if row
            .select(r#"td:nth-child(1) > img[title="Succeeded"]"#)
            .is_empty()
        {
            continue;
        }
        let _pkgname = row.select("td:nth-child(4)").text().to_string(); // TODO: match with real package url

        if let Some(url) = row
            .select("td:nth-child(2) > a")
            .attr("href")
            .map(|v| v.to_string())
        {
            return Ok(url);
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
        let build_url = fetch_job(client, &fetch_task.url).await?;
        let log_url = fetch_build(client, &build_url).await?;

        match client.get(&log_url).send().await {
            Ok(response) => Ok(response),
            Err(error) => Err(FetchReject::RequestFailed(error)),
        }
    }
}
