// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::HashMap;

use time::OffsetDateTime;

use crate::models::snippets::SnippetKind;

pub struct Version {
    pub id: i32,
    pub created_at: OffsetDateTime,
    pub last_updated_at: OffsetDateTime,
    pub project_name: String,
    pub version: String,
    pub max_snippet_counts: HashMap<SnippetKind, u64>,
}
