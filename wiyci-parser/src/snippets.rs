// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use strum::EnumString;

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Clone)]
pub struct CompilerWarning {
    pub path: String,
    pub line_number: u32,
    pub category: String,
    pub message: String,
}

macro_rules! declare_snippets {
    ($($kind:ident,)+) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize, EnumString)]
        #[non_exhaustive]
        pub enum SnippetKind {
            $($kind),+
        }

        typed_storage!(
            #[derive(Clone, Debug)]
            #[non_exhaustive]
            pub SnippetStorage<Vec>{$($kind),+}
        );

        impl SnippetStorage {
            pub fn counts_per_kind(&self) -> HashMap<SnippetKind, u64> {
                [
                    $(
                        (SnippetKind::$kind, self.get::<$kind>().len() as u64),
                    ),+
                ].into_iter().collect()
            }

            pub fn is_empty(&self) -> bool {
                $(
                    self.get::<$kind>().is_empty() &&
                ),+
                true
            }

            pub fn len(&self) -> usize {
                $(
                    self.get::<$kind>().len() +
                ),+
                0
            }
        }
    }
}

declare_snippets! {
    CompilerWarning,
}
