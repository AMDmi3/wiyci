// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::{HashMap, HashSet};

use time::OffsetDateTime;

use crate::models::snippets::SnippetKind;

#[derive(Debug)]
pub struct Project {
    pub name: String,
    pub created_at: OffsetDateTime,
    pub num_tasks: u32,
    pub next_update_at: OffsetDateTime,
    pub last_updated_at: Option<OffsetDateTime>,
    pub max_snippet_counts: HashMap<SnippetKind, u64>,
    pub latest_snippet_counts: HashMap<SnippetKind, u64>,
    pub latest_versions: HashSet<String>,
}
