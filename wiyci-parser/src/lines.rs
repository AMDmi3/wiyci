// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::borrow::Cow;
use std::io::BufRead;

pub struct SafeLines<R> {
    reader: R,
    max_line_length: Option<usize>,
}

impl<R: BufRead> SafeLines<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            max_line_length: None,
        }
    }

    pub fn with_max_line_length(mut self, max_line_length: Option<usize>) -> Self {
        self.max_line_length = max_line_length;
        self
    }
}

#[derive(Debug)]
pub struct Line {
    pub string: String,
    pub was_truncated: bool,
    pub had_invalid_utf8: bool,
}

impl Line {
    fn from_buffer(mut bytes: Vec<u8>, was_truncated: bool) -> Self {
        bytes.pop_if(|c| *c == b'\n');
        bytes.pop_if(|c| *c == b'\r');

        // the code below was taken from from_utf8_lossy_owned()
        if let Cow::Owned(string) = String::from_utf8_lossy(&bytes) {
            Line {
                string,
                was_truncated,
                had_invalid_utf8: true,
            }
        } else {
            // SAFETY: `String::from_utf8_lossy`'s contract ensures that if
            // it returns a `Cow::Borrowed`, it is a valid UTF-8 string.
            // Otherwise, it returns a new allocation of an owned `String`, with
            // replacement characters for invalid sequences, which is returned
            // above.
            Line {
                string: unsafe { String::from_utf8_unchecked(bytes) },
                was_truncated,
                had_invalid_utf8: false,
            }
        }
    }
}

impl<B: BufRead> Iterator for SafeLines<B> {
    type Item = std::io::Result<Line>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut accum: Vec<u8> = Vec::new();
        let mut was_truncated = false;

        loop {
            match self.reader.fill_buf() {
                Ok([]) if accum.is_empty() => return None,
                Ok([]) => {
                    return Some(Ok(Line::from_buffer(
                        std::mem::take(&mut accum),
                        was_truncated,
                    )));
                }
                Ok(more_bytes) => {
                    let eol_pos = memchr::memchr(b'\n', more_bytes);
                    let more_len = eol_pos.map(|pos| pos + 1).unwrap_or(more_bytes.len());

                    if let Some(max_line_length) = self.max_line_length
                        && accum.len() + more_len > max_line_length
                    {
                        was_truncated = true;
                        accum.extend_from_slice(&more_bytes[..max_line_length - accum.len()]);
                    } else {
                        accum.extend_from_slice(&more_bytes[..more_len]);
                    }

                    self.reader.consume(more_len);

                    if eol_pos.is_some() {
                        return Some(Ok(Line::from_buffer(
                            std::mem::take(&mut accum),
                            was_truncated,
                        )));
                    }
                }
                Err(ref err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(err) => return Some(Err(err)),
            };
        }
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use itertools::Itertools as _;
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_unlimited() {
        let lines: Vec<_> = SafeLines::new(Cursor::new(b"foo\nbar"))
            .with_max_line_length(Some(4))
            .try_collect()
            .unwrap();

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].string, "foo");
        assert!(!lines[0].was_truncated);
        assert_eq!(lines[1].string, "bar");
        assert!(!lines[1].was_truncated);
    }

    #[test]
    fn test_fits() {
        let lines: Vec<_> = SafeLines::new(Cursor::new(b"foo\nbar"))
            .with_max_line_length(Some(4))
            .try_collect()
            .unwrap();

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].string, "foo");
        assert!(!lines[0].was_truncated);
        assert_eq!(lines[1].string, "bar");
        assert!(!lines[1].was_truncated);
    }

    #[test]
    fn test_trucated_eol() {
        let lines: Vec<_> = SafeLines::new(Cursor::new(b"foo\nbar"))
            .with_max_line_length(Some(3))
            .try_collect()
            .unwrap();

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].string, "foo");
        assert!(lines[0].was_truncated);
        assert_eq!(lines[1].string, "bar");
        assert!(!lines[1].was_truncated);
    }

    #[test]
    fn test_trucated_characters() {
        let lines: Vec<_> = SafeLines::new(Cursor::new(b"foo\nbar"))
            .with_max_line_length(Some(2))
            .try_collect()
            .unwrap();

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].string, "fo");
        assert!(lines[0].was_truncated);
        assert_eq!(lines[1].string, "ba");
        assert!(lines[1].was_truncated);
    }

    #[test]
    fn test_trailing_newline() {
        let lines: Vec<_> = SafeLines::new(Cursor::new(b"foo\n")).try_collect().unwrap();

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].string, "foo");
    }

    #[test]
    fn test_trailing_newline_truncated() {
        let lines: Vec<_> = SafeLines::new(Cursor::new(b"foo\n"))
            .with_max_line_length(Some(3))
            .try_collect()
            .unwrap();

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].string, "foo");
    }

    #[test]
    fn test_empty_lines() {
        let lines: Vec<_> = SafeLines::new(Cursor::new(b"\nfoo\n\nbar\n\n"))
            .try_collect()
            .unwrap();

        assert_eq!(lines.len(), 5);
        assert_eq!(lines[0].string, "");
        assert_eq!(lines[1].string, "foo");
        assert_eq!(lines[2].string, "");
        assert_eq!(lines[3].string, "bar");
        assert_eq!(lines[4].string, "");
    }

    #[test]
    fn test_bad_utf8() {
        let lines: Vec<_> = SafeLines::new(Cursor::new(b"foo\xffbar"))
            .try_collect()
            .unwrap();

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].string, "foo\u{fffd}bar");
    }
}
