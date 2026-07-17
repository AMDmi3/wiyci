// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::io::{BufReader, Cursor};

use indoc::indoc;

use wiyci_parser::LogParser;
use wiyci_parser::snippets::{TestOutcome, TestResult};

use crate::common::{SnippetSaver, parse_snippet};

#[test]
fn test_passed() {
    let snippet: TestResult = parse_snippet(indoc! {r#"
        [ RUN      ] RebufferModule.FixedToVariable
        [       OK ] RebufferModule.FixedToVariable (0 ms)
    "#});

    assert_eq!(snippet.name, "RebufferModule.FixedToVariable");
    assert_eq!(snippet.outcome, TestOutcome::Passed);
}

#[test]
fn test_skipped() {
    let snippet: TestResult = parse_snippet(indoc! {r#"
        [ RUN      ] GenericStream/TlsConnectGeneric.CertificateCompressionTLS12AndBelow/0
        Version: TLS 1.3
        ssl_certificate_compression_unittest.cc:929: Skipped
        [  SKIPPED ] GenericStream/TlsConnectGeneric.CertificateCompressionTLS12AndBelow/0 (0 ms)
    "#});

    assert_eq!(
        snippet.name,
        "GenericStream/TlsConnectGeneric.CertificateCompressionTLS12AndBelow/0"
    );
    assert_eq!(snippet.outcome, TestOutcome::Skipped);
}

#[test]
fn test_failed() {
    let snippet: TestResult = parse_snippet(indoc! {r#"
        [ RUN      ] utHMPImportExport.importHMPFromFileTest
        Info,  T0: Load /builddir/build/BUILD/assimp-6.0.4-build/assimp-6.0.4/test/models/HMP/terrain.hmp
        Debug, T0: Assimp 6.0.0 <unknown architecture> gcc debug shared singlethreadedsingle :
        Info,  T0: Found a matching importer for this file format: 3D GameStudio Heightmap (HMP) Importer.
        Info,  T0: Import root directory is '/builddir/build/BUILD/assimp-6.0.4-build/assimp-6.0.4/test/models/HMP/'
        Debug, T0: HMP subtype: 3D GameStudio A7, magic word is HMP7
        Error, T0: Number of triangles in either x or y direction is zero
        /builddir/build/BUILD/assimp-6.0.4-build/assimp-6.0.4/test/unit/utHMPImportExport.cpp:59: Failure
        Value of: importerTest()
          Actual: false
        Expected: true
        [  FAILED  ] utHMPImportExport.importHMPFromFileTest (0 ms)
    "#});

    assert_eq!(snippet.name, "utHMPImportExport.importHMPFromFileTest");
    assert_eq!(snippet.outcome, TestOutcome::Failed);
}

#[test]
fn test_no_false_positives() {
    let mut saver = SnippetSaver::default();

    LogParser::default()
        .parse(
            BufReader::new(Cursor::new(indoc! {r#"
        [  PASSED  ] 0 tests.
        [  FAILED  ] 1 test, listed below:
        [  FAILED  ] utHMPImportExport.importHMPFromFileTest
    "#})),
            &mut saver,
        )
        .expect("parsing failed");

    assert_eq!(saver.snippets.len(), 0);
}
