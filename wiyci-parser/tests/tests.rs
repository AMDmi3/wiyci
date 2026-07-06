// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{BufReader, Cursor};

use indoc::indoc;

use wiyci_parser::LogParser;

#[test]
fn test_basic_warning() {
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

#[test]
#[ignore]
fn test_clean_ansi() {
    let data = Cursor::new(indoc! {"
        c++ -c 1.cc
        \x1b[01m\x1b[K1.cc:\x1b[m\x1b[K In function ‘\x1b[01m\x1b[Kint foo()\x1b[m\x1b[K’:
        \x1b[01m\x1b[K1.cc:1:12:\x1b[m\x1b[K \x1b[01;35m\x1b[Kwarning: \x1b[m\x1b[Kno return statement in function returning non-void [\x1b[01;35m\x1b[K-Wreturn-type\x1b[m\x1b[K]
            1 | int foo() {\x1b[01;35m\x1b[K}\x1b[m\x1b[K
              |            \x1b[01;35m\x1b[K^\x1b[m\x1b[K
    "});

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
