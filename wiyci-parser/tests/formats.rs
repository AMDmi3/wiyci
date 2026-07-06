// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{BufReader, Cursor};

use indoc::indoc;

use wiyci_parser::{LogParser, snippets};

#[test]
fn test_plain() {
    let data = Cursor::new(indoc! {r#"
        c++ -c 1.cc
        1.cc: In function ‘int foo()’:
        1.cc:1:12: warning: no return statement in function returning non-void [-Wreturn-type]
            1 | int foo() {}
              |            ^
    "#});

    let res = LogParser::default().parse(BufReader::new(data)).unwrap();
    let warning = &res.snippets.get::<snippets::CompilerWarning>()[0];

    assert_eq!(
        warning.message,
        "warning: no return statement in function returning non-void [-Wreturn-type]"
    );
}

#[test]
fn test_ansi() {
    let data = Cursor::new(indoc! {"
        c++ -c 1.cc
        \x1b[01m\x1b[K1.cc:\x1b[m\x1b[K In function ‘\x1b[01m\x1b[Kint foo()\x1b[m\x1b[K’:
        \x1b[01m\x1b[K1.cc:1:12:\x1b[m\x1b[K \x1b[01;35m\x1b[Kwarning: \x1b[m\x1b[Kno return statement in function returning non-void [\x1b[01;35m\x1b[K-Wreturn-type\x1b[m\x1b[K]
            1 | int foo() {\x1b[01;35m\x1b[K}\x1b[m\x1b[K
              |            \x1b[01;35m\x1b[K^\x1b[m\x1b[K
    "});

    let res = LogParser::default().parse(BufReader::new(data)).unwrap();
    let warning = &res.snippets.get::<snippets::CompilerWarning>()[0];

    assert_eq!(
        warning.message,
        "warning: no return statement in function returning non-void [-Wreturn-type]"
    );
}
