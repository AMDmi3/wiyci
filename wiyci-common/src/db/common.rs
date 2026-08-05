// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::HashMap;

use sqlx::types::Json;

use crate::models::snippets::SnippetKind;

pub fn convert_snippet_counts(
    input: Option<Json<HashMap<String, u64>>>,
) -> HashMap<SnippetKind, u64> {
    input
        .map(|json| {
            json.into_inner()
                .into_iter()
                .filter_map(|(k, v)| k.parse().ok().map(|k| (k, v)))
                .collect()
        })
        .unwrap_or_default()
}
