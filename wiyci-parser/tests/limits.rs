// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use indoc::indoc;

use wiyci_parser::{Flags, LogParser};

#[test]
fn test_max_lines() {
    let data = Cursor::new("1\n2\n3\n");
    assert_eq!(
        LogParser::default()
            .with_max_lines(Some(3))
            .parse(data.clone())
            .unwrap()
            .flags,
        Flags::empty()
    );
    assert_eq!(
        LogParser::default()
            .with_max_lines(Some(2))
            .parse(data.clone())
            .unwrap()
            .flags,
        Flags::TOO_MANY_LINES
    );
}

#[test]
fn test_max_line_length() {
    let data = Cursor::new("foobarbaz");

    assert_eq!(
        LogParser::default()
            .with_max_line_length(Some(9))
            .parse(data.clone())
            .unwrap()
            .flags,
        Flags::empty()
    );
    assert_eq!(
        LogParser::default()
            .with_max_line_length(Some(8))
            .parse(data.clone())
            .unwrap()
            .flags,
        Flags::HAD_TRUNCATED_LINES
    );
}

#[test]
fn test_max_snippets() {
    let data = Cursor::new(indoc! {r#"
        1.cc:1:12: warning: no return statement in function returning non-void [-Wreturn-type]
        1.cc:2:12: warning: no return statement in function returning non-void [-Wreturn-type]
        1.cc:3:12: warning: no return statement in function returning non-void [-Wreturn-type]
    "#});

    assert_eq!(
        LogParser::default()
            .with_max_snippets(Some(3))
            .parse(data.clone())
            .unwrap()
            .flags,
        Flags::empty()
    );
    assert_eq!(
        LogParser::default()
            .with_max_snippets(Some(2))
            .parse(data.clone())
            .unwrap()
            .flags,
        Flags::TOO_MANY_SNIPPETS
    );
}
