// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::io::{BufReader, Cursor};
use std::ops::ControlFlow;

use indoc::indoc;

use wiyci_parser::{
    LogParser, SnippetHandler,
    snippets::{Snippet, TestResult},
};

use crate::common::SnippetSaver;

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

#[test]
fn test_unicalize_not() {
    let mut saver = SnippetSaver::default();

    LogParser::default()
        .parse(
            BufReader::new(Cursor::new(indoc! {r#"
            [       OK ] Foo (0 ms)
            [       OK ] Foo (0 ms)
            [       OK ] Foo (0 ms)
            [       OK ] Bar (0 ms)
            [       OK ] Bar (0 ms)
        "#})),
            &mut saver,
        )
        .expect("parsing failed");

    assert_eq!(saver.snippets.len(), 5);
}

#[test]
fn test_unicalize() {
    let mut saver = SnippetSaver::default();

    LogParser::default()
        .with_unicalize(true)
        .parse(
            BufReader::new(Cursor::new(indoc! {r#"
            [       OK ] Foo (0 ms)
            [       OK ] Foo (0 ms)
            [       OK ] Foo (0 ms)
            [       OK ] Bar (0 ms)
            [       OK ] Bar (0 ms)
        "#})),
            &mut saver,
        )
        .expect("parsing failed");

    assert_eq!(saver.snippets.len(), 2);
}

#[test]
#[ignore]
fn test_preserves_snippet_order() {
    let mut saver = SnippetSaver::default();

    LogParser::default()
        .with_unicalize(true)
        .parse(
            BufReader::new(Cursor::new(indoc! {r#"
            [       OK ] 0 (0 ms)
            [       OK ] 1 (0 ms)
            [       OK ] 2 (0 ms)
            [       OK ] 3 (0 ms)
            [       OK ] 4 (0 ms)
        "#})),
            &mut saver,
        )
        .expect("parsing failed");

    let texts: Vec<String> = saver
        .snippets
        .into_iter()
        .map(|snippet| TestResult::try_from(snippet).unwrap().name)
        .collect();
    assert_eq!(texts, ["0", "1", "2", "3", "4"]);
}
