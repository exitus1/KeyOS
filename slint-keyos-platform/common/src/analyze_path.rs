// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    collections::{HashMap, HashSet},
    fmt,
};

#[derive(Debug, Clone, PartialEq)]
pub struct AnalyzedPath {
    pub path: Vec<Segment>,
    pub query: Vec<String>,
}

impl std::fmt::Display for AnalyzedPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for segment in &self.path {
            f.write_str("/")?;
            match segment {
                Segment::Capture(capture) => write!(f, "{{{}}}", capture)?,
                Segment::Static(static_segment) => write!(f, "{}", static_segment)?,
            }
        }

        if !self.query.is_empty() {
            f.write_str("?")?;
            for (i, query) in self.query.iter().enumerate() {
                if i > 0 {
                    f.write_str("&")?;
                }
                write!(f, "{{{}}}", query)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Segment {
    Capture(String),
    Static(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum PathError {
    RootEmpty,
    InvalidQuery(String),
    InvalidPath(String),
    DuplicateCaptures(Vec<String>),
    MissingFields(Vec<String>),
    MissingParams(Vec<String>),
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathError::RootEmpty => write!(f, "Path must start with a `/`. Use \"/\" for root routes"),
            PathError::InvalidQuery(var) => write!(f, "Invalid query variable: {}", var),
            PathError::InvalidPath(details) => write!(f, "Invalid path: {}", details),
            PathError::DuplicateCaptures(captures) => {
                write!(f, "Duplicate path captures: {:?}", captures)
            }
            PathError::MissingFields(fields) => write!(f, "Field missing from path: {:?}", fields),
            PathError::MissingParams(params) => {
                write!(f, "Path params missing from fields: {:?}", params)
            }
        }
    }
}

impl std::error::Error for PathError {}

/// Fields are cleaned of the raw `r#` prefix before parsing.
pub fn validate(path: impl AsRef<str>, fields: Vec<String>) -> Result<AnalyzedPath, PathError> {
    let path = path.as_ref();

    let fields: Vec<String> = fields.iter().map(|f| f.trim_start_matches("r#").into()).collect();

    if path.is_empty() {
        return Err(PathError::RootEmpty);
    } else if !path.starts_with('/') {
        return Err(PathError::InvalidPath("Path must start with a `/`".to_string()));
    }

    let (path_str, query) = path.split_once('?').unwrap_or((path, ""));

    let raw_path_segment: Vec<RawPathSegment> = if path_str == "/" {
        vec![]
    } else {
        path_str
            .trim_start_matches('/')
            .trim_end_matches('/')
            .split('/')
            .map(|segment| {
                if segment.is_empty() {
                    return Err(PathError::InvalidPath("Empty segment in path".to_string()));
                }

                let capture = segment
                    .strip_prefix('{')
                    .and_then(|s| s.strip_suffix('}'))
                    .or_else(|| segment.strip_prefix(':'))
                    .map(to_snake_case);

                Ok(if let Some(capture) = capture {
                    RawPathSegment::Capture(capture)
                } else {
                    RawPathSegment::Static(segment.to_owned())
                })
            })
            .collect::<Result<Vec<RawPathSegment>, PathError>>()?
    };

    let query_variables = query
        .split('&')
        .filter(|query| !query.is_empty())
        .map(|query| {
            query
                .strip_prefix('{')
                .and_then(|q| q.strip_suffix('}'))
                .map(to_snake_case)
                .ok_or_else(|| PathError::InvalidQuery(query.to_string()))
        })
        .collect::<Result<Vec<String>, _>>()?;

    let all_captures: Vec<String> = {
        let path_captures = raw_path_segment.iter().filter_map(|segment| match segment {
            RawPathSegment::Capture(capture) => Some(capture.clone()),
            _ => None,
        });
        let query_captures = query_variables.iter().cloned();

        let mut seen = HashSet::new();
        let mut duplicates = Vec::new();
        let mut captures = Vec::new();

        for capture in path_captures.chain(query_captures) {
            if !seen.insert(capture.clone()) {
                duplicates.push(capture);
            } else {
                captures.push(capture);
            }
        }

        if !duplicates.is_empty() {
            return Err(PathError::DuplicateCaptures(duplicates));
        }

        captures
    };

    validate_fields(&fields, &all_captures)?;

    let mut name_to_field: HashMap<&str, String> =
        fields.iter().map(|ident| (ident.as_str(), ident.to_string())).collect();

    let path_segments: Vec<_> = raw_path_segment
        .into_iter()
        .map(|segment| match segment {
            RawPathSegment::Capture(q) => {
                Segment::Capture(name_to_field.remove(q.as_str()).expect("Path capture to exist as field"))
            }
            RawPathSegment::Static(f) => Segment::Static(f),
        })
        .collect();

    let query_segments = query_variables
        .iter()
        .map(|q| name_to_field.remove(q.as_str()).expect("Query capture to exist as field"))
        .collect();

    Ok(AnalyzedPath { path: path_segments, query: query_segments })
}

/// Ensures that all captures are in the fields
fn validate_fields(fields: &[String], captures: &[String]) -> Result<(), PathError> {
    let invalid_fields: Vec<String> = fields
        .iter()
        .filter(|ident| !captures.contains(ident))
        .map(|ident| ident.trim_start_matches("r#"))
        .map(Into::into)
        .collect();

    if !invalid_fields.is_empty() {
        return Err(PathError::MissingFields(invalid_fields));
    }

    let missing_struct_fields: Vec<String> = captures
        .iter()
        .filter(|&capture| !fields.contains(capture))
        .map(|ident| ident.trim_start_matches("r#"))
        .map(Into::into)
        .collect();

    if !missing_struct_fields.is_empty() {
        return Err(PathError::MissingParams(missing_struct_fields));
    }

    Ok(())
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    let mut needs_underscore = false;

    while let Some(c) = chars.next() {
        if c.is_uppercase() {
            if !result.is_empty() && needs_underscore {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
            needs_underscore = false;
        } else if c == '-' || c == '_' {
            if !result.is_empty() && needs_underscore {
                result.push('_');
            }
            while let Some(&next) = chars.peek() {
                if next == '-' || next == '_' {
                    chars.next();
                } else {
                    break;
                }
            }
            needs_underscore = false;
        } else {
            result.push(c);
            needs_underscore = true;
        }
    }

    result
}

enum RawPathSegment {
    Capture(String),
    Static(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_1() {
        let path = "/user/{user_id}";
        let fields = vec!["user_id".to_string()];
        let result = validate(path, fields).unwrap();

        assert_eq!(
            result,
            AnalyzedPath {
                path: vec![Segment::Static("user".to_string()), Segment::Capture("user_id".to_string())],
                query: vec![],
            }
        );
    }

    #[test]
    fn parse_kebab() {
        let path = "/user/{user-id}";
        let fields = vec!["user_id".to_string()];
        let result = validate(path, fields).unwrap();

        assert_eq!(
            result,
            AnalyzedPath {
                path: vec![Segment::Static("user".to_string()), Segment::Capture("user_id".to_string())],
                query: vec![],
            }
        );
    }

    #[test]
    fn parse_camel() {
        let path = "/user/{userId}";
        let fields = vec!["user_id".to_string()];
        let result = validate(path, fields).unwrap();

        assert_eq!(
            result,
            AnalyzedPath {
                path: vec![Segment::Static("user".to_string()), Segment::Capture("user_id".to_string())],
                query: vec![],
            }
        );
    }

    #[test]
    fn parse_colon() {
        let path = "/user/:user_id";
        let fields = vec!["user_id".to_string()];
        let result = validate(path, fields).unwrap();

        assert_eq!(
            result,
            AnalyzedPath {
                path: vec![Segment::Static("user".to_string()), Segment::Capture("user_id".to_string())],
                query: vec![],
            }
        );
    }

    #[test]
    fn parse_query() {
        let path = "/user/{user_id}?{name}";
        let fields = vec!["user_id".to_string(), "name".to_string()];
        let result = validate(path, fields).unwrap();

        assert_eq!(
            result,
            AnalyzedPath {
                path: vec![Segment::Static("user".to_string()), Segment::Capture("user_id".to_string())],
                query: vec!["name".to_string()],
            }
        );
    }

    #[test]
    fn parse_raw_fields() {
        let path = "/user/{user_id}";
        let fields = vec!["r#user_id".to_string()];
        let result = validate(path, fields).unwrap();

        assert_eq!(
            result,
            AnalyzedPath {
                path: vec![Segment::Static("user".to_string()), Segment::Capture("user_id".to_string())],
                query: vec![],
            }
        );
    }

    #[test]
    fn invalid_path_1() {
        let path = "user/{user_id}";
        let fields = vec!["user_id".to_string()];
        let result = validate(path, fields);

        assert!(matches!(result, Err(PathError::InvalidPath(msg)) if msg == "Path must start with a `/`"));
    }

    #[test]
    fn invalid_path_2() {
        let path = "/user//{user_id}/";
        let fields = vec!["user_id".to_string()];
        let result = validate(path, fields);

        assert!(matches!(dbg!(result), Err(PathError::InvalidPath(msg)) if msg == "Empty segment in path"));
    }

    #[test]
    fn duplicate_captures() {
        let path = "/user/{user_id}/{user_id}";
        let fields = vec!["user_id".to_string()];
        let result = validate(path, fields);

        assert!(
            matches!(result, Err(PathError::DuplicateCaptures(captures)) if captures == vec!["user_id".to_string()])
        );
    }

    #[test]
    fn missing_fields() {
        let path = "/user/{user_id}";
        let fields = vec!["id".to_string()];
        let result = validate(path, fields);

        assert!(matches!(result, Err(PathError::MissingFields(fields)) if fields == vec!["id".to_string()]));
    }

    #[test]
    fn missing_fields_2() {
        let path = "/user/{user_id}";
        let fields = vec!["user_id".to_string(), "extra".to_string()];
        let result = validate(path, fields);

        assert!(
            matches!(result, Err(PathError::MissingFields(fields)) if fields == vec!["extra".to_string()])
        );
    }

    #[test]
    fn missing_params() {
        let path = "/user/{user_id}";
        let fields = vec![];
        let result = validate(path, fields);

        assert!(
            matches!(result, Err(PathError::MissingParams(fields)) if fields == vec!["user_id".to_string()])
        );
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!("user_id", to_snake_case("UserId"));
        assert_eq!("user_id", to_snake_case("user-id"));
        assert_eq!("user_id", to_snake_case("UserID"));
        assert_eq!("user_id_two", to_snake_case("user_id__two"));
        assert_eq!("user_id_two", to_snake_case("user_id--two"));
        assert_eq!("user_id_two", to_snake_case("user_id-Two"));
    }
}
