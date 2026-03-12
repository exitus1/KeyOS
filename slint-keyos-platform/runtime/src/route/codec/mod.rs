// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod de;
mod encode;
pub mod ser;

use serde::{de::DeserializeOwned, Serialize};

// A composable route codec that can be used to serialize and deserialize routes.
// You can combine RouteCodecs by using tuples.
// Paths will be concatenated in the tuple order.
// Queries will be appended to the end of the combined path.
// Query variables have no order.
pub trait RouteCodec: Sized {
    type Path: DeserializeOwned;
    type Query: DeserializeOwned;

    type PathRef<'a>: Serialize
    where
        Self: 'a;
    type QueryRef<'a>: Serialize
    where
        Self: 'a;

    fn from_parts(path: Self::Path, query: Self::Query) -> Self;
    fn path_ref(&self) -> Self::PathRef<'_>;
    fn query_ref(&self) -> Self::QueryRef<'_>;
    fn into_parts(self) -> (Self::Path, Self::Query);

    fn de_route(route: &str) -> Result<Self, de::Error> {
        let (path, rest) = de::deserialize_path::<Self::Path>(route)?;
        let start = route.len() - rest.len();
        let query = de::deserialize_query::<Self::Query>(rest).map_err(|e| match e {
            de::Error::Parse { position, error, context } => {
                de::Error::Parse { position: start + position, error, context }
            }
            e => e,
        })?;
        Ok(Self::from_parts(path, query))
    }

    fn ser_route(&self) -> Result<String, ser::Error> {
        let path = self.path_ref();
        let query = self.query_ref();
        let mut writer = Vec::new();
        ser::serialize_path_partial(&path, &mut writer)?;
        ser::serialize_query_partial(&query, &mut writer)?;
        Ok(unsafe { String::from_utf8_unchecked(writer) })
    }
}

#[cfg(test)]
mod round_trip {
    use crate::{route, route::*};

    #[route(path = "/test/{test_id}/{float_id}?{users}")]
    #[derive(Debug, PartialEq)]
    struct RouteOne {
        test_id: String,
        float_id: f32,
        users: Vec<User>,
    }

    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct User {
        id: u32,
        name: String,
    }

    #[route(path = "/static/")]
    struct RouteTwo {}

    #[test]
    fn round_trip() {
        let route = RouteOne {
            test_id: "test1".into(),
            float_id: 2.5,
            users: vec![User { id: 1, name: "nico".to_string() }],
        };

        let serialized = route.ser_route().unwrap();
        println!("{serialized}");
        let deserialized = RouteOne::de_route(&serialized).unwrap();

        assert_eq!(route, deserialized);
    }
}
