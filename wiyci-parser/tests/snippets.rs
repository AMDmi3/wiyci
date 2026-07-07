// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{BufReader, Cursor};

use indoc::indoc;

use wiyci_parser::{LogParser, snippets};

#[test]
fn test_compiler_warning() {
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

#[test]
fn test_pytest_failed_test() {
    let data = Cursor::new(indoc! {r#"
        Tests/test_file_webp_animated.py::test_n_frames PASSED                   [ 50%]
        Tests/test_file_webp_animated.py::test_write_animation_L FAILED          [ 50%]
        Tests/test_file_webp_animated.py::test_write_animation_RGB FAILED        [ 50%]
        Tests/test_file_webp_animated.py::test_timestamp_and_duration PASSED     [ 50%]
    "#});

    let res = LogParser::default().parse(BufReader::new(data)).unwrap();
    let snippets = res.snippets.get::<snippets::PytestFailedTest>();

    assert_eq!(snippets.len(), 2);

    assert_eq!(snippets[0].path, "Tests/test_file_webp_animated.py");
    assert_eq!(snippets[0].rest_of_nodeid, "test_write_animation_L");
    assert_eq!(snippets[1].path, "Tests/test_file_webp_animated.py");
    assert_eq!(snippets[1].rest_of_nodeid, "test_write_animation_RGB");
}
