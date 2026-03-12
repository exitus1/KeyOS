// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{
    de::{self, Visitor},
    forward_to_deserialize_any, Deserialize,
};

use super::{common::Deserializer, segment::SegmentDeserializer, Error, ParseError};

pub fn deserialize_path<'de, T>(path: &'de str) -> Result<(T, &'de str), Error>
where
    T: Deserialize<'de>,
{
    let mut deserializer = Deserializer::new(path);
    let value = T::deserialize(RootDeserializer { de: &mut deserializer })?;
    let rest = deserializer.input;
    Ok((value, rest))
}

struct RootDeserializer<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> de::Deserializer<'de> for RootDeserializer<'a, 'de> {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct
        map struct enum identifier ignored_any tuple_struct
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let deserializer = SingleDeserializer { de: self.de };
        deserializer.deserialize_seq(visitor)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let deserializer = ComposedDeserializer { de: self.de };
        deserializer.deserialize_tuple(len, visitor)
    }

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(self.de.error(ParseError::UnexpectedType { expected: &["sequence", "tuple"] }))
    }
}

struct ComposedDeserializer<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> de::Deserializer<'de> for ComposedDeserializer<'a, 'de> {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct
        map struct enum identifier ignored_any seq tuple_struct
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(ComposedRouteDeserializer { de: self.de })
    }

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(self.de.error(ParseError::UnexpectedType { expected: &["tuple"] }))
    }
}

struct ComposedRouteDeserializer<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> de::SeqAccess<'de> for ComposedRouteDeserializer<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.de.is_empty() {
            return Ok(None);
        }

        match self.de.peek_char()? {
            '/' => seed.deserialize(SingleDeserializer { de: self.de }).map(Some),
            c => Err(self.de.error(ParseError::UnexpectedChar { found: c, expected: '/'.into() })),
        }
    }
}

struct SingleDeserializer<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> de::Deserializer<'de> for SingleDeserializer<'a, 'de> {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct
        tuple_struct map struct enum identifier ignored_any tuple
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(RouteDeserializer { de: self.de })
    }

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(self.de.error(ParseError::UnexpectedType { expected: &["sequence"] }))
    }
}

struct RouteDeserializer<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> de::SeqAccess<'de> for RouteDeserializer<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.de.is_empty() {
            return Ok(None);
        }

        match self.de.next_char()? {
            '?' => Ok(None),
            '/' => seed.deserialize(SegmentDeserializer { de: self.de }).map(Some),
            c => {
                static EXPECTED: &[char] = &['/', '?'];
                Err(self.de.error(ParseError::UnexpectedChar { found: c, expected: EXPECTED.into() }))
            }
        }
    }
}

#[cfg(test)]
mod test {
    pub struct OnePath {
        one: String,
    }

    impl<'de> serde::de::Deserialize<'de> for OnePath {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::de::Deserializer<'de>,
        {
            struct PathVisitor;

            impl<'de> serde::de::Visitor<'de> for PathVisitor {
                type Value = OnePath;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("struct OnePath")
                }

                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    let static_segment: String =
                        seq.next_element()?.ok_or_else(|| serde::de::Error::invalid_length(2usize, &self))?;
                    let static_segment = static_segment.as_str();
                    if static_segment != "one" {
                        return Err(serde::de::Error::invalid_value(
                            serde::de::Unexpected::Str(static_segment),
                            &"one",
                        ));
                    }
                    let one =
                        seq.next_element()?.ok_or_else(|| serde::de::Error::invalid_length(2usize, &self))?;
                    Ok(OnePath { one })
                }
            }
            deserializer.deserialize_seq(PathVisitor)
        }
    }

    pub struct TwoPath {
        two: String,
    }

    impl<'de> serde::de::Deserialize<'de> for TwoPath {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::de::Deserializer<'de>,
        {
            struct PathVisitor;

            impl<'de> serde::de::Visitor<'de> for PathVisitor {
                type Value = TwoPath;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("struct TwoPath")
                }

                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    let static_segment: String =
                        seq.next_element()?.ok_or_else(|| serde::de::Error::invalid_length(2usize, &self))?;
                    let static_segment = static_segment.as_str();
                    if static_segment != "two" {
                        return Err(serde::de::Error::invalid_value(
                            serde::de::Unexpected::Str(static_segment),
                            &"two",
                        ));
                    }
                    let two =
                        seq.next_element()?.ok_or_else(|| serde::de::Error::invalid_length(2usize, &self))?;
                    Ok(TwoPath { two })
                }
            }
            deserializer.deserialize_seq(PathVisitor)
        }
    }

    #[test]
    fn test_deserialize() {
        let path = "/one/a/two/b";
        let (route, rest) = super::deserialize_path::<(OnePath, TwoPath)>(path).unwrap();
        assert_eq!(route.0.one, "a");
        assert_eq!(route.1.two, "b");
        assert_eq!(rest, "");
    }
}
