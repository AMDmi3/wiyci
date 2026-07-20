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
    assert_eq!(snippet.category.as_deref(), Some("-Wreturn-type"));
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

#[test]
fn test_xs() {
    let snippet: CompilerWarning = parse_snippet(indoc! {r#"
        Util.xs: In function 'XS_NetAddr__IP__Util_comp128':
        Util.xs:501:17: warning: format '%d' expects argument of type 'int', but argument 4 has type 'STRLEN' {aka 'long unsigned int'} [-Wformat=]
          501 |           croak("Bad arg length for %s%s, length is %d, should be %d",
              |                 ^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
          502 |                 "NetAddr::IP::Util::",subname,len *8,128);
              |                                               ~~~~~~
              |                                                   |
              |                                                   STRLEN {aka long unsigned int}
    "#});

    assert_eq!(snippet.path, "Util.xs");
}
