// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::HashMap;

use sqlx::types::Json;

use crate::models::snippets::SnippetKind;

pub fn parse_snippet_counts(json: Json<HashMap<String, i64>>) -> HashMap<SnippetKind, u64> {
    json.into_inner()
        .into_iter()
        .filter_map(|(k, v)| k.parse().ok().map(|k| (k, v as u64)))
        .collect()
}
