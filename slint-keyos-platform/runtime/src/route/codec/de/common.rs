// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{Error, ParseError};
use crate::route::codec::encode::is_reserved_char;

// Wrapper around &str with convenience methods.
#[derive(Debug)]
pub struct Deserializer<'de> {
    pub input: &'de str,
    // the current position
    pub position: usize,
    // the last confirmed position
    pub checkpoint: usize,
}

impl<'de> Deserializer<'de> {
    pub fn new(input: &'de str) -> Self { Self { input, position: 0, checkpoint: 0 } }

    // Look at the first character in the input without consuming it.
    pub fn peek_char(&mut self) -> Result<char, Error> {
        self.input.chars().next().ok_or(self.error(ParseError::Eof))
    }

    // Consume the first character in the input.
    pub fn next_char(&mut self) -> Result<char, Error> {
        self.checkpoint();

        let ch = self.peek_char()?;
        let ch_len = ch.len_utf8();
        self.input = &self.input[ch_len..];
        self.position += ch_len;
        Ok(ch)
    }

    #[inline]
    pub fn next_char_exact(&mut self, expected: char) -> Result<char, Error> {
        let ch = self.next_char()?;
        if ch == expected {
            Ok(ch)
        } else {
            Err(self.error(ParseError::UnexpectedChar { found: ch, expected: expected.into() }))
        }
    }

    #[inline]
    pub fn next_str_matches(&mut self, expected: &'static [&'static str]) -> Result<&str, Error> {
        let checkpoint = self.checkpoint;
        let s = self.take_next_str();
        if expected.contains(&s) {
            Ok(s)
        } else {
            let err = ParseError::UnexpectedStr { found: s.to_string(), expected };
            Err(Error::Parse { position: checkpoint, error: err, context: None })
        }
    }

    // Take the next string segment until a reserved character is found.
    pub fn take_next_str(&mut self) -> &str {
        self.checkpoint();

        let end = self
            .input
            .char_indices()
            .find(|(_, c)| is_reserved_char(*c))
            .map(|c| c.0)
            .unwrap_or(self.input.len());

        let part = &self.input[..end];
        self.input = &self.input[end..];
        self.position += end;
        part
    }

    pub fn checkpoint(&mut self) { self.checkpoint = self.position; }

    pub fn is_empty(&self) -> bool { self.input.is_empty() }

    // creates an error at the last checkpoint position
    pub fn error(&self, error: ParseError) -> Error {
        Error::Parse { position: self.checkpoint, error, context: None }
    }
}
