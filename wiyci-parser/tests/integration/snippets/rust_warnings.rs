// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use indoc::indoc;

use crate::common::parse_snippet;

use wiyci_parser::snippets::CompilerWarning;

#[test]
fn test_rust() {
    let snippet: CompilerWarning = parse_snippet(indoc! {r#"
        warning: unexpected `cfg` condition value: `upload`
          --> src/lib.rs:44:7
           |
        44 | #[cfg(feature = "upload")]
           |       ^^^^^^^^^^^^^^^^^^
           |
           = note: expected values for `feature` are: `cli-completion`, `console`, `default`, `dialoguer`, `full`, `minijinja`, `scaffolding`, `schemars`, and `unicode-xid`
           = help: consider adding `upload` as a feature in `Cargo.toml`
           = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration
           = note: `#[warn(unexpected_cfgs)]` on by default
    "#});

    assert_eq!(snippet.path, "src/lib.rs");
    assert_eq!(snippet.line_number, 44);
    assert_eq!(snippet.category.as_deref(), Some("unexpected_cfgs"));
    assert_eq!(
        snippet.message,
        "unexpected `cfg` condition value: `upload`"
    );
    assert_eq!(
        snippet
            .lines
            .iter()
            .map(|line| line.as_str())
            .collect::<Vec<_>>(),
        vec![
            "warning: unexpected `cfg` condition value: `upload`",
            "  --> src/lib.rs:44:7",
            "   |",
            "44 | #[cfg(feature = \"upload\")]",
            "   |       ^^^^^^^^^^^^^^^^^^",
            "   |",
            "   = note: expected values for `feature` are: `cli-completion`, `console`, `default`, `dialoguer`, `full`, `minijinja`, `scaffolding`, `schemars`, and `unicode-xid`",
            "   = help: consider adding `upload` as a feature in `Cargo.toml`",
            "   = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration",
            "   = note: `#[warn(unexpected_cfgs)]` on by default",
        ],
    );
}
