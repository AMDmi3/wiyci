// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::ops::Range;
use std::fmt::Debug;
use std::str::FromStr;

use regex::Captures;

pub trait SimplifiedCaptures {
    fn get_str(&self, i: usize) -> &str;
    #[expect(unused)]
    fn get_range(&self, i: usize) -> Range<usize>;
    fn get_any<T>(&self, i: usize) -> T
    where
        T: FromStr,
        <T as FromStr>::Err: Debug;
}

impl SimplifiedCaptures for Captures<'_> {
    fn get_str(&self, i: usize) -> &str {
        self.get(i)
            .expect("capture group presence ensured by the regex")
            .as_str()
    }

    fn get_range(&self, i: usize) -> Range<usize> {
        self.get(i)
            .expect("capture group presence ensured by the regex")
            .range()
    }

    fn get_any<T>(&self, i: usize) -> T
    where
        T: FromStr,
        <T as FromStr>::Err: Debug,
    {
        self.get_str(i)
            .parse()
            .expect("parasable value ensured by the regex")
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use regex::Regex;

    use super::*;

    #[test]
    fn test_simplified_captures() {
        let regex = Regex::new(r"^([a-z]+)([0-9]+)$").unwrap();
        let captures = regex.captures("abc123").unwrap();
        assert_eq!(captures.get_str(1), "abc");
        assert_eq!(captures.get_str(2), "123");
        assert_eq!(captures.get_any::<u32>(2), 123);
    }

    #[test]
    #[should_panic]
    fn test_simplified_captures_panics_on_unknown_group() {
        let regex = Regex::new(r"^([a-z]+)([0-9]+)$").unwrap();
        let captures = regex.captures("abc123").unwrap();
        let _ = captures.get_str(3);
    }

    #[test]
    #[should_panic]
    fn test_simplified_captures_panics_on_bad_parse() {
        let regex = Regex::new(r"^([a-z]+)([0-9]+)$").unwrap();
        let captures = regex.captures("abc123").unwrap();
        let _ = captures.get_any::<u32>(1);
    }
}
