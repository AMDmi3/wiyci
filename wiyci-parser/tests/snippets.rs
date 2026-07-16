// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fmt::Debug;
use std::io::{BufReader, Cursor};

use indoc::indoc;
use itertools::Itertools as _;

use wiyci_parser::{LogParser, snippets::*};

#[track_caller]
fn parse_snippet<T>(text: &str) -> T
where
    T: Debug + StoredInSnippetStorage,
{
    LogParser::default()
        .parse(BufReader::new(Cursor::new(text)))
        .expect("parsing failed")
        .snippets
        .into_iter_of()
        .exactly_one()
        .expect("parser was expected to return exactly one snippet")
}

#[test]
fn test_compiler_warning() {
    let snippet: CompilerWarning = parse_snippet(indoc! {r#"
        c++ -c 1.cc
        1.cc: In function ‘int foo()’:
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

#[test]
fn test_pytest_failed_test() {
    let snippet: TestResult = parse_snippet(indoc! {r#"
        Tests/test_file_webp_animated.py::test_write_animation_L FAILED          [ 50%]
    "#});

    assert_eq!(
        snippet.name,
        "Tests/test_file_webp_animated.py::test_write_animation_L"
    );
    assert_eq!(snippet.outcome, TestOutcome::Failed);
}

#[test]
fn test_pytest_passed_test() {
    let snippet: TestResult = parse_snippet(indoc! {r#"
        Tests/test_webp_leaks.py::TestWebPLeaks::test_leak_load PASSED           [100%]
    "#});

    assert_eq!(
        snippet.name,
        "Tests/test_webp_leaks.py::TestWebPLeaks::test_leak_load"
    );
    assert_eq!(snippet.outcome, TestOutcome::Passed);
}

#[test]
fn test_pytest_skipped_test() {
    let snippet: TestResult = parse_snippet(indoc! {r#"
        Tests/test_imagewin.py::TestImageWinDib::test_dib_mode_string SKIPPED    [ 95%]
    "#});

    assert_eq!(
        snippet.name,
        "Tests/test_imagewin.py::TestImageWinDib::test_dib_mode_string"
    );
    assert_eq!(snippet.outcome, TestOutcome::Skipped);
}

#[test]
fn test_pytest_skipped_described_test() {
    let snippet: TestResult = parse_snippet(indoc! {r#"
        Tests/test_qt_image_toqimage.py::test_sanity[P] SKIPPED (Qt bindings...) [ 99%]
    "#});

    assert_eq!(
        snippet.name,
        "Tests/test_qt_image_toqimage.py::test_sanity[P]"
    );
    assert_eq!(snippet.outcome, TestOutcome::Skipped);
}
