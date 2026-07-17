// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use indoc::indoc;

use crate::common::parse_snippet;

use wiyci_parser::snippets::CompilerWarning;

#[test]
fn test_cc_simple() {
    let snippet: CompilerWarning = parse_snippet(indoc! {r#"
        c++ -c 1.cc
        1.cc:1:12: warning: no return statement in function returning non-void [-Wreturn-type]
            1 | int foo() {}
              |            ^
    "#});

    assert_eq!(snippet.path, "1.cc");
    assert_eq!(snippet.line_number, 1);
    assert_eq!(snippet.category, "-Wreturn-type");
    assert_eq!(
        snippet.message,
        "warning: no return statement in function returning non-void [-Wreturn-type]"
    );
    assert_eq!(
        snippet
            .lines
            .iter()
            .map(|line| line.as_str())
            .collect::<Vec<_>>(),
        vec![
            "1.cc:1:12: warning: no return statement in function returning non-void [-Wreturn-type]",
            "    1 | int foo() {}",
            "      |            ^",
        ],
    );
}
