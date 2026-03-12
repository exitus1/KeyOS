// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

mod codec;
mod metadata;

pub use codec::*;
pub use metadata::*;

// Route Entry is not `Sync` because many Slint types (e.g. Array, String) are not Sync.
pub trait RouteEntry: RouteCodec {
    /// Rust doesn't have sufficient support to make this const yet.
    /// Tuples of the same length, but different types will have the same metadata if using a static once_cell
    /// in the implementation.
    fn metadata() -> RouteMetadata;
    fn route_id() -> String { Self::metadata().id().to_string() }
}

macro_rules! impl_route_codec_for_tuple {
    ($(($elem_ty:ident, $idx:tt)),+ $(,)?) => {
        impl<$($elem_ty),+> RouteCodec for ($($elem_ty),+,)
        where
            $($elem_ty: RouteCodec),+
        {
            type Path = ($($elem_ty::Path),+,);
            type PathRef<'a> = ($($elem_ty::PathRef<'a>),+,) where $($elem_ty: 'a),+;
            type Query = ($($elem_ty::Query),+,);
            type QueryRef<'a> = ($($elem_ty::QueryRef<'a>),+,) where $($elem_ty: 'a),+;


            fn from_parts(path: Self::Path, query: Self::Query) -> Self {
                ($(
                    $elem_ty::from_parts(path.$idx, query.$idx)
                ),+,)
            }

            fn path_ref(&self) -> Self::PathRef<'_> {
                ($(self.$idx.path_ref()),+,)
            }

            fn query_ref(&self) -> Self::QueryRef<'_> {
                ($(self.$idx.query_ref()),+,)
            }

            paste::paste! {
                fn into_parts(self) -> (Self::Path, Self::Query) {
                    let ($([<$elem_ty:snake>]),+,) = self;
                    $(
                        let ([<$elem_ty:snake _path>], [<$elem_ty:snake _query>]) = [<$elem_ty:snake>].into_parts();
                    )+
                    (
                        ($( [<$elem_ty:snake _path>] ,)+),
                        ($( [<$elem_ty:snake _query>] ,)+)
                    )
                }
            }
        }

        impl<$($elem_ty),+> RouteEntry for ($($elem_ty,)+)
        where
            $($elem_ty: RouteEntry),+
        {
            fn metadata() -> RouteMetadata {
                let mut metadata = RouteMetadata::default();
                $(
                    metadata = metadata.merge_ref(&<$elem_ty as RouteEntry>::metadata());
                )+
                metadata
            }
        }
    };
}

macro_rules! generate_for_tuples {
    ($name:ident) => {
        $name!((T0, 0));
        $name!((T0, 0), (T1, 1));
        $name!((T0, 0), (T1, 1), (T2, 2));
        $name!((T0, 0), (T1, 1), (T2, 2), (T3, 3));
        $name!((T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4));
        $name!((T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4), (T5, 5));
        $name!((T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4), (T5, 5), (T6, 6));
        $name!((T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4), (T5, 5), (T6, 6), (T7, 7));
        $name!((T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4), (T5, 5), (T6, 6), (T7, 7), (T8, 8));
        $name!((T0, 0), (T1, 1), (T2, 2), (T3, 3), (T4, 4), (T5, 5), (T6, 6), (T7, 7), (T8, 8), (T9, 9));
        $name!(
            (T0, 0),
            (T1, 1),
            (T2, 2),
            (T3, 3),
            (T4, 4),
            (T5, 5),
            (T6, 6),
            (T7, 7),
            (T8, 8),
            (T9, 9),
            (T10, 10)
        );
    };
}

generate_for_tuples!(impl_route_codec_for_tuple);

#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};

    use super::*;

    #[crate::route(path = "/one/{id}?{query_one}")]
    #[derive(Debug, PartialEq)]
    struct PartOne {
        id: String,
        query_one: String,
    }

    #[crate::route(path = "/two/{id}?{query_two}")]
    #[derive(Debug, PartialEq)]
    struct PartTwo {
        id: u32,
        query_two: String,
    }

    #[crate::route(path = "/three/{test}")]
    #[derive(Debug, PartialEq)]
    struct PartThree {
        test: TestStruct,
    }

    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct TestStruct {
        id: String,
        nested: NestedStruct,
    }

    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct NestedStruct {
        something: String,
    }

    #[crate::route(path = "/static")]
    struct Static {}

    #[test]
    fn test_id() {
        assert_eq!(PartOne::metadata().id(), "/one/{}");
        assert_eq!(PartTwo::metadata().id(), "/two/{}");
        assert_eq!(Static::metadata().id(), "/static");

        assert_eq!(<(PartOne,)>::metadata().id(), "/one/{}");
        assert_eq!(<(PartTwo,)>::metadata().id(), "/two/{}");
        assert_eq!(<(Static,)>::metadata().id(), "/static");

        assert_eq!(<(PartOne, PartTwo)>::metadata().id(), "/one/{}/two/{}");
        assert_eq!(<(PartTwo, PartOne)>::metadata().id(), "/two/{}/one/{}");
        assert_eq!(<(PartTwo, Static)>::metadata().id(), "/two/{}/static");
    }

    #[test]
    fn round_trip() {
        let route: (PartOne, PartTwo) = (
            PartOne { id: "1".into(), query_one: "one".into() },
            PartTwo { id: 2, query_two: "QueryParam".into() },
        );

        let serialized = route.ser_route().unwrap();
        assert_eq!(serialized, "/one/1/two/2?query_one=one&query_two=QueryParam");
        let deserialized =
            <(PartOne, PartTwo)>::de_route("/one/1/two/2?query_two=QueryParam&query_one=one").unwrap();
        assert_eq!(route, deserialized);
    }

    #[test]
    fn nested() {
        let route: PartThree = PartThree {
            test: TestStruct { id: "1".into(), nested: NestedStruct { something: "nested".into() } },
        };

        let serialized = route.ser_route().unwrap();
        assert_eq!(serialized, "/three/{id:1,nested:{something:nested}}");
        let deserialized = PartThree::de_route(&serialized).unwrap();
        assert_eq!(route, deserialized);
    }

    #[crate::route(path = "/envoy?{phone_name}&{last_communication}")]
    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct TestDoubleQuery {
        phone_name: String,
        last_communication: String,
    }

    #[test]
    fn double_query() {
        let query = TestDoubleQuery { phone_name: "Pixel".into(), last_communication: "2021-09-01".into() };

        let serialized = query.ser_route().unwrap();
        assert_eq!(serialized, "/envoy?phone_name=Pixel&last_communication=2021-09-01");
        let deserialized = TestDoubleQuery::de_route(&serialized).unwrap();
        assert_eq!(query, deserialized);
    }

    #[test]
    fn optional_query() {
        let deserialized = PartOne::de_route("/one/1").unwrap();
        assert_eq!(deserialized, PartOne { id: "1".into(), query_one: String::new() });
        let deserialized = <(PartOne, PartTwo)>::de_route("/one/1/two/2").unwrap();
        assert_eq!(
            deserialized,
            (
                PartOne { id: "1".into(), query_one: String::new() },
                PartTwo { id: 2, query_two: String::new() }
            )
        );
    }

    #[test]
    fn parse_error_path() {
        #[derive(Debug, PartialEq)]
        #[crate::route(path = "/test/{field}")]
        struct TestPath {
            field: u32,
        }

        let input = "/test/notanumber";
        let result = TestPath::de_route(input);
        let error = result.unwrap_err();
        let index = input.find("notanumber").unwrap();
        match error {
            de::Error::Parse { position, error: de::ParseError::Int(_), .. } => {
                assert_eq!(position, index)
            }
            _ => panic!("Expected a Parse error at index {index}"),
        }
    }

    #[test]
    fn parse_error_path_nested() {
        #[derive(Debug, PartialEq)]
        #[crate::route(path = "/test/{field}")]
        struct TestNested {
            field: NestedNumber,
        }

        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct NestedNumber {
            something: u32,
        }

        let input = "/test/{something:notanumber}";
        let result = TestNested::de_route(input);
        let error = result.unwrap_err();
        let index = input.find("notanumber").unwrap();

        match error {
            de::Error::Parse { position, error: de::ParseError::Int(_), .. } => {
                assert_eq!(position, index, "incorrect position")
            }
            _ => panic!("Expected a Parse error at index {index}"),
        }

        let input = "/test/{something:32{";
        let result = TestNested::de_route(input);
        let error = result.unwrap_err();
        println!("{error}",);
        let index = input.rfind('{').unwrap();
        match error {
            de::Error::Parse {
                position,
                error: de::ParseError::UnexpectedChar { found, expected: _ },
                ..
            } => {
                assert_eq!(position, index);
                assert_eq!(found, '{');
            }
            _ => panic!("Expected a Parse error at index {index}"),
        }

        let input = "/test/[one]";
        let result = TestNested::de_route(input);
        let error = result.unwrap_err();
        let index = input.rfind('[').unwrap();
        match error {
            de::Error::Parse {
                position,
                error: de::ParseError::UnexpectedChar { found, expected: _ },
                ..
            } => {
                assert_eq!(position, index);
                assert_eq!(found, '[');
            }
            _ => panic!("Expected a Parse error at index {index}"),
        }
    }

    #[test]
    fn required_query() {
        #[derive(Debug, PartialEq)]
        #[crate::route(required, path = "/required?{query}")]
        struct RequiredQuery {
            query: u32,
        }

        let deserialized = RequiredQuery::de_route("/required?query=123").unwrap();
        assert_eq!(deserialized, RequiredQuery { query: 123 });

        let result = RequiredQuery::de_route("/required");

        assert!(result.is_err());

        let input = "/required?query=notanumber";
        let result = RequiredQuery::de_route(input);
        let error = result.unwrap_err();

        let index = input.find("notanumber").unwrap();

        match error {
            de::Error::Parse { position, error: de::ParseError::Int(_), .. } => {
                assert_eq!(position, index, "incorrect position")
            }
            _ => panic!("Expected a Parse error at index {index}"),
        }
    }

    #[test]
    fn dev_flag() {
        #[crate::route(dev, path = "/dev")]
        #[derive(Debug, PartialEq)]
        struct Dev {}

        let deserialized = Dev::de_route("/dev").unwrap();
        assert_eq!(deserialized, Dev {});
    }

    #[test]
    fn raw_fields() {
        #[crate::route(path = "/test/{field-a}/{field_b}?{field-c}&{field_d}")]
        #[derive(Deserialize, Serialize, Debug, PartialEq)]
        struct TestRaw {
            r#field_a: u8,
            r#field_b: String,
            r#field_c: u8,
            r#field_d: String,
        }

        let base = TestRaw {
            r#field_a: 42,
            r#field_b: "test-value".to_string(),
            r#field_c: 10,
            r#field_d: "query-value".to_string(),
        };

        let serialized = base.ser_route().unwrap();
        assert_eq!(serialized, "/test/42/test-value?field_c=10&field_d=query-value");

        let deserialized = TestRaw::de_route(&serialized).unwrap();
        assert_eq!(base, deserialized);

        // Test with different values
        let another = TestRaw {
            r#field_a: 100,
            r#field_b: "another-test".to_string(),
            r#field_c: 20,
            r#field_d: "another-query".to_string(),
        };

        let another_serialized = another.ser_route().unwrap();
        assert_eq!(another_serialized, "/test/100/another-test?field_c=20&field_d=another-query");

        let another_deserialized = TestRaw::de_route(&another_serialized).unwrap();
        assert_eq!(another, another_deserialized);
    }
}
