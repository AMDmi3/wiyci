// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::fmt::Debug;
use std::io::{BufReader, Cursor};
use std::ops::ControlFlow;

use indoc::indoc;
use itertools::Itertools as _;

use wiyci_parser::{LogParser, ParseStatus, SnippetHandler, snippets::*};

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
fn parse_snippet<S, T>(text: T) -> (S, ParseStatus)
where
    S: Debug + TryFrom<Snippet>,
    <S as TryFrom<Snippet>>::Error: Debug,
    T: AsRef<[u8]>,
{
    let mut saver = SnippetSaver::default();

    let status = LogParser::default()
        .parse(BufReader::new(Cursor::new(text)), &mut saver)
        .expect("parsing failed");

    (
        saver
            .snippets
            .into_iter()
            .exactly_one()
            .expect("parser was expected to return exactly one snippet")
            .try_into()
            .expect("got unexpected snippet type"),
        status,
    )
}

fn is_clean_parse(status: &ParseStatus) -> bool {
    status.num_truncated_lines == 0
        && status.num_invalid_utf8_lines == 0
        && !status.is_truncated
        && !status.is_interrupted
}

#[test]
fn test_plain() {
    let (snippet, status): (CompilerWarning, _) = parse_snippet(indoc! {"
        1.cc:1:12: warning: no return statement in function returning non-void [-Wreturn-type]
            1 | int foo() {}
              |            ^
    "});

    assert!(is_clean_parse(&status));
    assert_eq!(snippet.path, "1.cc");
    assert_eq!(
        snippet.message,
        "warning: no return statement in function returning non-void [-Wreturn-type]"
    );
}

#[test]
fn test_bad_utf8() {
    let (snippet, status): (CompilerWarning, _) = parse_snippet(indoc! {b"
        1.cc:1:12: warning: no return statement in function returning non-void [-Wreturn-type]
            1 | int foo() { const char *badutf = \"\xc3\"; }
              |            ^
    "});

    assert_eq!(status.num_invalid_utf8_lines, 1);
    assert_eq!(snippet.path, "1.cc");
    assert_eq!(
        snippet.message,
        "warning: no return statement in function returning non-void [-Wreturn-type]"
    );
}

#[test]
fn test_ansi() {
    let (snippet, status): (CompilerWarning, _) = parse_snippet(indoc! {"
        c++ -c 1.cc
        \x1b[01m\x1b[K1.cc:\x1b[m\x1b[K In function ‘\x1b[01m\x1b[Kint foo()\x1b[m\x1b[K’:
        \x1b[01m\x1b[K1.cc:1:12:\x1b[m\x1b[K \x1b[01;35m\x1b[Kwarning: \x1b[m\x1b[Kno return statement in function returning non-void [\x1b[01;35m\x1b[K-Wreturn-type\x1b[m\x1b[K]
            1 | int foo() {\x1b[01;35m\x1b[K}\x1b[m\x1b[K
              |            \x1b[01;35m\x1b[K^\x1b[m\x1b[K
    "});

    assert!(is_clean_parse(&status));
    assert_eq!(snippet.path, "1.cc");
    assert_eq!(
        snippet.message,
        "warning: no return statement in function returning non-void [-Wreturn-type]"
    );
}
