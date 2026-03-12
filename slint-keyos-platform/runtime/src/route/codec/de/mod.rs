// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

mod common;
mod path;
mod query;
mod segment;

use std::borrow::Cow;

pub use path::deserialize_path;
pub use query::deserialize_query;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Message(String),
    #[error("{error} at position {position}")]
    Parse { position: usize, error: ParseError, context: Option<Cow<'static, str>> },
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ParseError {
    #[error("Unexpected char '{found}' expected {expected}")]
    UnexpectedChar { found: char, expected: ExpectedChar },

    // TODO: remove String for &str
    #[error("Unexpected string '{found}' expected one of {expected:?}")]
    UnexpectedStr { found: String, expected: &'static [&'static str] },

    #[error("Expected type {expected:?}")]
    UnexpectedType { expected: &'static [&'static str] },

    #[error("Unexpected end of input")]
    Eof,

    #[error("{0}")]
    Int(#[from] std::num::ParseIntError),

    #[error("{0}")]
    Float(#[from] std::num::ParseFloatError),

    #[error("Invalid UTF-8")]
    InvalidUtf8,
}

#[derive(Debug, PartialEq)]
pub enum ExpectedChar {
    Exact(char),
    OneOf(&'static [char]),
}

impl std::fmt::Display for ExpectedChar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpectedChar::Exact(ch) => write!(f, "'{}'", ch),
            ExpectedChar::OneOf(chars) => write!(f, "{:?}", chars),
        }
    }
}

impl From<char> for ExpectedChar {
    fn from(ch: char) -> Self { ExpectedChar::Exact(ch) }
}

impl From<&'static [char]> for ExpectedChar {
    fn from(chars: &'static [char]) -> Self { ExpectedChar::OneOf(chars) }
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Char(char),
    Str(&'static str),
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Char(c) => write!(f, "'{}'", c),
            Token::Str(s) => write!(f, "'{}'", s),
        }
    }
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Error::Message(msg.to_string())
    }
}
