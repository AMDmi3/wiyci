// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::fmt::Debug;
use std::io::{BufReader, Cursor};
use std::ops::ControlFlow;

use indoc::indoc;
use itertools::Itertools as _;

use wiyci_parser::{LogParser, SnippetHandler, snippets::*};

#[derive(Default)]
struct SnippetSaver {
    snippets: Vec<Snippet>,
}

impl SnippetHandler for SnippetSaver {
    fn handle_snippet(&mut self, snippet: Snippet) -> ControlFlow<()> {
        self.snippets.push(snippet);
        ControlFlow::Continue(())
    }
}

#[track_caller]
fn parse_snippet<T>(text: &str) -> T
where
    T: Debug + TryFrom<Snippet>,
    <T as TryFrom<Snippet>>::Error: Debug,
{
    let mut saver = SnippetSaver::default();

    LogParser::default()
        .parse(BufReader::new(Cursor::new(text)), &mut saver)
        .expect("parsing failed");

    saver
        .snippets
        .into_iter()
        .exactly_one()
        .expect("parser was expected to return exactly one snippet")
        .try_into()
        .expect("got unexpected snippet type")
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
