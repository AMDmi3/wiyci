// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::io::Cursor;
use std::ops::ControlFlow;

use indoc::indoc;

use wiyci_parser::{LogParser, SnippetHandler, snippets::Snippet};

const SAMPLE: &str = indoc! {"
    main.py::foo FAILED [  0%]
    main.py::bar FAILED [  1%]
    main.py::baz FAILED [  2%]
"};

struct DummyHandler;

impl SnippetHandler for DummyHandler {
    fn handle_snippet(&mut self, _snippet: Snippet) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}

struct InterruptingHandler;

impl SnippetHandler for InterruptingHandler {
    fn handle_snippet(&mut self, _snippet: Snippet) -> ControlFlow<()> {
        ControlFlow::Break(())
    }
}

#[test]
fn test_normal_operation() {
    let res = LogParser::default()
        .parse(Cursor::new(SAMPLE), &mut DummyHandler)
        .unwrap();

    assert_eq!(res.num_parsed_lines, 3);
    assert_eq!(res.num_truncated_lines, 0);
    assert_eq!(res.num_invalid_utf8_lines, 0);
    assert!(!res.is_truncated);
    assert!(!res.is_interrupted);
}

#[test]
fn test_interruption() {
    let res = LogParser::default()
        .parse(Cursor::new(SAMPLE), &mut InterruptingHandler)
        .unwrap();

    assert_eq!(res.num_parsed_lines, 1);
    assert_eq!(res.num_truncated_lines, 0);
    assert_eq!(res.num_invalid_utf8_lines, 0);
    assert!(!res.is_truncated);
    assert!(res.is_interrupted);
}

#[test]
fn test_max_lines_not_triggered() {
    let res = LogParser::default()
        .with_max_lines(Some(3))
        .parse(Cursor::new("1\n2\n3\n"), &mut DummyHandler)
        .unwrap();

    assert!(!res.is_truncated);
    assert_eq!(res.num_parsed_lines, 3);
}

#[test]
fn test_max_lines_triggered() {
    let res = LogParser::default()
        .with_max_lines(Some(2))
        .parse(Cursor::new(SAMPLE), &mut DummyHandler)
        .unwrap();

    assert!(res.is_truncated);
    assert_eq!(res.num_parsed_lines, 2);
}

#[test]
fn test_max_line_length_not_triggered() {
    let res = LogParser::default()
        .with_max_line_length(Some(9))
        .parse(Cursor::new("foobarbaz"), &mut DummyHandler)
        .unwrap();

    assert_eq!(res.num_truncated_lines, 0);
}

#[test]
fn test_max_line_length_triggered() {
    let res = LogParser::default()
        .with_max_line_length(Some(8))
        .parse(Cursor::new("foobarbaz"), &mut DummyHandler)
        .unwrap();

    assert_eq!(res.num_truncated_lines, 1);
}
