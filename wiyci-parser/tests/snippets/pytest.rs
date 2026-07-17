// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use indoc::indoc;

use wiyci_parser::snippets::{TestOutcome, TestResult};

use crate::common::parse_snippet;

#[test]
fn test_failed() {
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
fn test_passed() {
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
fn test_skipped() {
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
fn test_skipped_described() {
    let snippet: TestResult = parse_snippet(indoc! {r#"
        Tests/test_qt_image_toqimage.py::test_sanity[P] SKIPPED (Qt bindings...) [ 99%]
    "#});

    assert_eq!(
        snippet.name,
        "Tests/test_qt_image_toqimage.py::test_sanity[P]"
    );
    assert_eq!(snippet.outcome, TestOutcome::Skipped);
}
