// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use indoc::indoc;

use wiyci_parser::snippets::{TestOutcome, TestResult};

use crate::common::parse_snippet;

#[test]
#[ignore = "not implemented yet"]
fn test_passed() {
    let snippet: TestResult = parse_snippet(indoc! {r#"
        [line start marker]
              Start  8: 28-bit-perfect-65537-44100
         1/10 Test  #8: 28-bit-perfect-65537-44100 .......   Passed    0.06 sec
    "#});

    assert_eq!(snippet.name, "28-bit-perfect-65537-44100");
    assert_eq!(snippet.outcome, TestOutcome::Passed);
}

#[test]
#[ignore = "not implemented yet"]
fn test_skipped() {
    let snippet: TestResult = parse_snippet(indoc! {r#"
        [line start marker]
                Start   1: test001_ff
          1/133 Test   #1: test001_ff .......................***Skipped   0.01 sec
    "#});

    assert_eq!(snippet.name, "test001_ff");
    assert_eq!(snippet.outcome, TestOutcome::Skipped);
}

#[test]
#[ignore = "not implemented yet"]
fn test_not_run() {
    let snippet: TestResult = parse_snippet(indoc! {r#"
        [line start marker]
              Start 56: rustup_proxy_cleanup
         1/97 Test #56: rustup_proxy_cleanup .....................................***Not Run (Disabled)   0.00 sec
    "#});

    assert_eq!(snippet.name, "rustup_proxy_cleanup");
    assert_eq!(snippet.outcome, TestOutcome::Skipped);
}

#[test]
fn test_failed() {
    let snippet: TestResult = parse_snippet(indoc! {r#"
        [line start marker]
        21/21 Test #18: TerminalInterfaceTest ............***Failed   10.99 sec
    "#});

    assert_eq!(snippet.name, "TerminalInterfaceTest");
    assert_eq!(snippet.outcome, TestOutcome::Failed);
}

#[test]
fn test_exception() {
    let snippet: TestResult = parse_snippet(indoc! {r#"
        [line start marker]
                Start 300: utglTF2ImportExport.importglTF2AndExport_KHR_materials_pbrSpecularGlossiness
        299/563 Test #300: utglTF2ImportExport.importglTF2AndExport_KHR_materials_pbrSpecularGlossiness .........................................................................................................................................................................................................................................................................................***Exception: SegFault  0.09 sec
    "#});

    assert_eq!(
        snippet.name,
        "utglTF2ImportExport.importglTF2AndExport_KHR_materials_pbrSpecularGlossiness"
    );
    assert_eq!(snippet.outcome, TestOutcome::Failed);
}
