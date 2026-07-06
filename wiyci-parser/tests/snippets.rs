// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{BufReader, Cursor};

use indoc::indoc;

use wiyci_parser::{LogParser, snippets};

#[test]
fn test_warning() {
    let data = Cursor::new(indoc! {r#"
        c++ -c 1.cc
        1.cc: In function ‘int foo()’:
        1.cc:1:12: warning: no return statement in function returning non-void [-Wreturn-type]
            1 | int foo() {}
              |            ^
    "#});

    let res = LogParser::default().parse(BufReader::new(data)).unwrap();
    let warnings = res.snippets.get::<snippets::CompilerWarning>();

    assert_eq!(warnings.len(), 1);

    let warning = &warnings[0];

    assert_eq!(warning.path, "1.cc");
    assert_eq!(warning.line_number, 1);
    assert_eq!(warning.category, "-Wreturn-type");
    assert_eq!(
        warning.message,
        "warning: no return statement in function returning non-void [-Wreturn-type]"
    );
}
