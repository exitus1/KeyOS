// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::borrow::Cow;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct RouteMetadata {
    id: String,
    path: Vec<PathSegment>,
    query: Vec<QueryParameter>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PathSegment {
    Capture,
    Static(Cow<'static, str>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct QueryParameter {
    pub key: Cow<'static, str>,
}

impl RouteMetadata {
    pub fn new(path: Vec<PathSegment>, query: Vec<QueryParameter>) -> Self {
        // Route ID:
        // - starts with a leading slash
        // - does not consider query params
        // - does not have a trailing slash
        let id = path.iter().enumerate().fold(String::from("/"), |mut acc, (idx, segment)| {
            if idx > 0 {
                acc.push('/');
            }
            match segment {
                PathSegment::Static(static_part) => {
                    acc.push_str(static_part);
                }
                PathSegment::Capture => {
                    acc.push_str("{}");
                }
            }
            acc
        });

        Self { id, path, query }
    }

    pub fn matches<'a>(&self, path: &'a str) -> Option<&'a str> { route_path_matches(path, &self.path) }

    pub fn full_matches(&self, path: &str) -> bool {
        matches!(route_path_matches(path, &self.path), Some(""))
    }

    pub fn merge_ref(self, other: &Self) -> Self { self.merge(other.clone()) }

    pub fn merge(self, other: Self) -> Self {
        let mut path = Vec::with_capacity(self.path.len() + other.path.len());
        path.extend(self.path);
        path.extend(other.path);
        let mut query = Vec::with_capacity(self.query.len() + other.query.len());
        query.extend(self.query);
        query.extend(other.query);
        Self::new(path, query)
    }

    pub fn id(&self) -> &str { &self.id }
}

#[inline]
fn route_path_matches<'a, 'b>(path: &'a str, pattern: &'b [PathSegment]) -> Option<&'a str> {
    let path = path.split_once('?').map(|(p, _q)| p).unwrap_or(path);

    if pattern.is_empty() {
        // TODO: Revise this
        if path.is_empty() || path == "/" {
            return Some("");
        }
        return None;
    }

    let path = strip_single(path, '/');
    let mut path_parts = path.split('/');
    let mut matched_len = 0;

    for part in pattern {
        match (part, path_parts.next()) {
            (PathSegment::Static(static_part), Some(path_part)) if static_part == path_part => {
                // +1 for the '/'
                matched_len += path_part.len() + 1;
            }
            (PathSegment::Capture, Some(path_part)) => {
                // +1 for the '/'
                matched_len += path_part.len() + 1;
            }
            _ => {
                return None;
            }
        }
    }

    matched_len -= 1;

    Some(&path[matched_len..])
}

fn strip_single(s: &str, c: char) -> &str {
    let s = s.strip_prefix(c).unwrap_or(s);
    s.strip_suffix(c).unwrap_or(s)
}

#[test]
fn test_strip_single() {
    assert_eq!(strip_single("foo/", '/'), "foo");
    assert_eq!(strip_single("/foo", '/'), "foo");
    assert_eq!(strip_single("/foo/bar/", '/'), "foo/bar");
    assert_eq!(strip_single("//", '/'), "");
    assert_eq!(strip_single("", '/'), "");

    assert_eq!(strip_single("//foo//", '/'), "/foo/");
}

#[test]
fn test_matches() {
    let path = RouteMetadata::new(
        vec![
            PathSegment::Static(Cow::Borrowed("foo")),
            PathSegment::Capture,
            PathSegment::Static(Cow::Borrowed("two")),
        ],
        vec![],
    );
    assert_eq!(path.matches("foo/2/two"), Some(""));
    assert_eq!(path.matches("foo/2/two/"), Some(""));
    assert_eq!(path.matches("/foo/2/two"), Some(""));
    assert_eq!(path.matches("/foo/2/two/"), Some(""));
    assert_eq!(path.matches("foo/2/two/extra"), Some("/extra"));

    assert_eq!(path.matches("foo/1"), None);
    assert_eq!(path.matches("foo/1/twoo"), None);
    assert_eq!(path.matches("foo"), None);
    assert_eq!(path.matches("foo/"), None);
    assert_eq!(path.matches("/"), None);
    assert_eq!(path.matches(""), None);
}

#[test]
fn matches_query() {
    let route = RouteMetadata::new(
        vec![PathSegment::Static(Cow::Borrowed("foo")), PathSegment::Capture],
        vec![QueryParameter { key: Cow::Borrowed("bar") }],
    );

    assert!(route.full_matches("foo/2?bar=1"));
    assert!(route.full_matches("foo/2"));
    assert!(route.full_matches("foo/2?bar=1&baz=2"));
}

#[test]
fn test_id() {
    let path = RouteMetadata::new(
        vec![
            PathSegment::Static(Cow::Borrowed("users")),
            PathSegment::Capture,
            PathSegment::Static(Cow::Borrowed("profile")),
        ],
        vec![],
    );
    assert_eq!(path.id(), "/users/{}/profile");

    let empty_path = RouteMetadata::new(vec![], vec![]);
    assert_eq!(empty_path.id(), "/");

    let only_capture = RouteMetadata::new(vec![PathSegment::Capture], vec![]);
    assert_eq!(only_capture.id(), "/{}");

    let with_query = RouteMetadata::new(
        vec![PathSegment::Static(Cow::Borrowed("foo")), PathSegment::Capture],
        vec![QueryParameter { key: Cow::Borrowed("bar") }],
    );
    assert_eq!(with_query.id(), "/foo/{}");
}

#[test]
fn test_empty_pattern_matches() {
    let empty_pattern = RouteMetadata::new(vec![], vec![]);
    assert_eq!(empty_pattern.id(), "/");
    assert_eq!(empty_pattern.matches("/"), Some(""));
    assert_eq!(empty_pattern.matches(""), Some(""));
    assert_eq!(empty_pattern.matches("/settings"), None);
}
