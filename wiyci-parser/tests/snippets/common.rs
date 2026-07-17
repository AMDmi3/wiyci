// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::fmt::Debug;
use std::io::{BufReader, Cursor};
use std::ops::ControlFlow;

use itertools::Itertools as _;

use wiyci_parser::{LogParser, SnippetHandler, snippets::*};

#[derive(Default)]
pub struct SnippetSaver {
    pub snippets: Vec<Snippet>,
}

impl SnippetHandler for SnippetSaver {
    fn handle_snippet(&mut self, snippet: Snippet) -> ControlFlow<()> {
        self.snippets.push(snippet);
        ControlFlow::Continue(())
    }
}

#[track_caller]
pub fn parse_snippet<T>(text: &str) -> T
where
    T: Debug + TryFrom<Snippet>,
    <T as TryFrom<Snippet>>::Error: Debug,
{
    parse_snippets(text)
        .into_iter()
        .exactly_one()
        .expect("parser was expected to return exactly one snippet")
}

#[track_caller]
pub fn parse_snippets<T>(text: &str) -> Vec<T>
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
        .map(|snippet| snippet.try_into())
        .try_collect()
        .expect("got unexpected snippet type")
}
