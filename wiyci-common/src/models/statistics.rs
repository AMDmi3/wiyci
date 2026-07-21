// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::HashMap;

use sqlx::FromRow;

use crate::models::snippets::SnippetKind;

#[derive(FromRow)]
pub struct Statistics {
    #[sqlx(try_from = "i64")]
    pub stored_logs_size: u64,
}

#[derive(Default)]
pub struct StatisticsDelta {
    pub stored_logs_size: i64,
}

pub struct SnippetCountStatistics {
    pub num_projects: u64,
    pub num_snippets: HashMap<SnippetKind, u64>,
    pub num_projects_by_snippet: HashMap<SnippetKind, u64>,
}
