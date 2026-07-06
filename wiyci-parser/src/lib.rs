// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(coverage_attribute)]

#[macro_use]
mod typed_storage;

mod matching;
pub mod snippets;

use std::io::BufRead;
use std::sync::LazyLock;

use regex::Regex;

use crate::matching::SimplifiedCaptures;
use crate::snippets::SnippetStorage;

// XXX: is position (second number after path) mandatory?
// Note: the regex ensures line number fits into any 32 bit integer
static COMPILER_WARNING_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(.*\.(?:c|cc|cxx|cpp|cc|h|hh|hpp|hxx)):([0-9]{1,9}):[0-9]+: (warning: .* \[(-W.*)\])$",
    )
    .unwrap()
});

#[derive(Debug, Default)]
pub struct LogParseReport {
    pub parsed_lines: u64,
    pub snippets: SnippetStorage,
}

#[derive(Default)]
pub struct LogParser;

impl LogParser {
    // Bump this on each change of parser output, so the daemon could reparse stored logs
    pub const VERSION: u32 = 2;

    pub fn parse(&self, reader: impl BufRead) -> std::io::Result<LogParseReport> {
        let lines = reader.lines();
        let mut res = LogParseReport::default();

        for line in lines {
            let line = strip_ansi_escapes::strip_str(line?);
            res.parsed_lines += 1;

            if let Some(r#match) = COMPILER_WARNING_REGEX.captures(&line) {
                let warning = snippets::CompilerWarning {
                    path: r#match.get_any(1),
                    line_number: r#match.get_any(2),
                    category: r#match.get_any(4),
                    message: r#match.get_any(3),
                };

                res.snippets.get_mut().push(warning);
            }
        }

        Ok(res)
    }
}
