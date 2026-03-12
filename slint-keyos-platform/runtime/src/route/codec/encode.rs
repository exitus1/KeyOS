// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use percent_encoding::{percent_encode, AsciiSet, PercentEncode, CONTROLS};

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
const ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'/')
    .add(b'[')
    .add(b',')
    .add(b']')
    .add(b'{')
    .add(b':')
    .add(b'}')
    .add(b'?')
    .add(b'=')
    .add(b'&');

pub fn encode_bytes(bytes: &[u8]) -> PercentEncode<'_> { percent_encode(bytes, ENCODE_SET) }

pub fn is_reserved_char(c: char) -> bool {
    matches!(c, '/' | ',' | '[' | ']' | '{' | '}' | ':' | '?' | '=' | '&')
}

#[test]
fn test_encode() {
    assert_eq!(encode_bytes(b"hello world").to_string(), "hello%20world");
    assert_eq!(encode_bytes(b"hello/world").to_string(), "hello%2Fworld");
    assert_eq!(encode_bytes(b"hello:world").to_string(), "hello%3Aworld");
    assert_eq!(encode_bytes(b"hello,world").to_string(), "hello%2Cworld");
    assert_eq!(encode_bytes(b"hello[world").to_string(), "hello%5Bworld");
    assert_eq!(encode_bytes(b"hello]world").to_string(), "hello%5Dworld");
    assert_eq!(encode_bytes(b"hello\"world").to_string(), "hello%22world");
    assert_eq!(encode_bytes(b"hello{world").to_string(), "hello%7Bworld");
    assert_eq!(encode_bytes(b"hello}world").to_string(), "hello%7Dworld");
}
