// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(coverage_attribute)]

#[macro_use]
mod typed_storage;

mod snippets;

use std::io::BufRead;
use std::sync::LazyLock;

use regex::Regex;

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
    pub const VERSION: u32 = 1;

    pub fn parse(&self, reader: impl BufRead) -> std::io::Result<LogParseReport> {
        let lines = reader.lines();
        let mut res = LogParseReport::default();

        for line in lines {
            let line = line?;
            res.parsed_lines += 1;

            if let Some(r#match) = COMPILER_WARNING_REGEX.captures(&line) {
                let warning = snippets::CompilerWarning {
                    path: r#match
                        .get(1)
                        .expect("capture group presence ensured by the regex")
                        .as_str()
                        .to_string(),
                    line_number: r#match
                        .get(2)
                        .expect("capture group presence ensured by the regex")
                        .as_str()
                        .parse()
                        .expect("parsable integer ensured by the regex"),
                    category: r#match
                        .get(4)
                        .expect("capture group presence ensured by the regex")
                        .as_str()
                        .to_string(),
                    message: r#match
                        .get(3)
                        .expect("capture group presence ensured by the regex")
                        .as_str()
                        .to_string(),
                };

                res.snippets.get_mut().push(warning);
            }
        }

        Ok(res)
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use std::io::{BufReader, Cursor};

    use indoc::indoc;

    use super::*;

    #[test]
    fn test_simple() {
        let data = Cursor::new(indoc! {r#"
            c++ -c 1.cc
            1.cc: In function ‘int foo()’:
            1.cc:1:12: warning: no return statement in function returning non-void [-Wreturn-type]
                1 | int foo() {}
                  |            ^
        "#});

        let res = LogParser::default().parse(BufReader::new(data)).unwrap();

        insta::assert_debug_snapshot!(res, @r#"
            LogParseReport {
                parsed_lines: 5,
                snippets: SnippetStorage {
                    compiler_warnings: [
                        CompilerWarning {
                            path: "1.cc",
                            line_number: 1,
                            category: "-Wreturn-type",
                            message: "warning: no return statement in function returning non-void [-Wreturn-type]",
                        },
                    ],
                },
            }
        "#);
    }
}
