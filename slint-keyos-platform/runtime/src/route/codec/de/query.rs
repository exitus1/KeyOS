// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::BTreeMap;

use serde::{
    de::{self, Visitor},
    Deserialize,
};

use super::{common::Deserializer, segment::SegmentDeserializer, Error, ParseError};

pub fn deserialize_query<'de, T>(query: &'de str) -> Result<T, Error>
where
    T: Deserialize<'de>,
{
    let params = {
        if query.is_empty() || query == "?" {
            BTreeMap::new()
        } else {
            let mut pos = 0;

            if query.starts_with('?') {
                pos += 1;
            }

            let mut params = BTreeMap::new();
            for part in query.trim_start_matches('?').split('&') {
                if let Some((key, value)) = part.split_once('=') {
                    params.insert(
                        key,
                        QueryEntry {
                            value,
                            // +1 for the '=' separator
                            position: pos + key.len() + 1,
                        },
                    );
                } else {
                    return Err(Error::Parse {
                        position: pos,
                        // TODO: this should be a custom error probably.
                        error: ParseError::UnexpectedStr { found: part.to_string(), expected: &["="] },
                        context: None,
                    });
                }
                // +1 for the '&' separator
                pos += part.len() + 1;
            }
            params
        }
    };

    let value = T::deserialize(RootDeserializer { params: &params })?;

    Ok(value)
}

#[derive(Debug)]
struct QueryEntry<'a> {
    value: &'a str,
    position: usize,
}

struct RootDeserializer<'a, 'de: 'a> {
    params: &'a BTreeMap<&'de str, QueryEntry<'de>>,
}

impl<'a, 'de> de::Deserializer<'de> for RootDeserializer<'a, 'de> {
    type Error = Error;

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct
        map enum identifier ignored_any tuple_struct seq
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let deserializer = ComposedQueryDeserializer { params: self.params };
        deserializer.deserialize_tuple(len, visitor)
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let deserializer = QueryDeserializer { params: self.params };
        deserializer.deserialize_struct(name, fields, visitor)
    }

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Parse {
            position: 0,
            error: ParseError::UnexpectedType { expected: &["struct", "tuple"] },
            context: None,
        })
    }
}

struct ComposedQueryDeserializer<'a, 'de: 'a> {
    params: &'a BTreeMap<&'de str, QueryEntry<'de>>,
}

impl<'a, 'de> de::Deserializer<'de> for ComposedQueryDeserializer<'a, 'de> {
    type Error = Error;

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct
        map struct enum identifier ignored_any seq tuple_struct
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Parse {
            position: 0,
            error: ParseError::UnexpectedType { expected: &["tuple"] },
            context: None,
        })
    }
}

impl<'a, 'de> de::SeqAccess<'de> for ComposedQueryDeserializer<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(QueryDeserializer { params: self.params }).map(Some)
    }
}

struct QueryDeserializer<'a, 'de: 'a> {
    params: &'a BTreeMap<&'de str, QueryEntry<'de>>,
}

impl<'a, 'de> de::Deserializer<'de> for QueryDeserializer<'a, 'de> {
    type Error = Error;

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct
        tuple enum identifier ignored_any seq tuple_struct map
    }

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Parse {
            position: 0,
            error: ParseError::UnexpectedType { expected: &["struct"] },
            context: None,
        })
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(MapDeserializer { fields, params: self.params, value: None, position: 0 })
    }
}

struct MapDeserializer<'a, 'de: 'a> {
    fields: &'static [&'static str],
    params: &'a BTreeMap<&'de str, QueryEntry<'de>>,
    value: Option<&'de str>,
    position: usize,
}

impl<'a, 'de> de::MapAccess<'de> for MapDeserializer<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.fields.is_empty() {
            return Ok(None);
        }

        let key = self.fields[0];
        self.fields = &self.fields[1..];

        // Find entry with matching key and store its position
        if let Some(entry) = self.params.get(key) {
            self.position = entry.position;
            self.value = Some(entry.value);
        } else {
            self.value = None;
            return Ok(None);
        }

        seed.deserialize(KeyDeserializer { key }).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let value = self.value.take().unwrap();
        let mut de = Deserializer::new(value);
        de.position = self.position;
        seed.deserialize(SegmentDeserializer { de: &mut de })
    }
}

struct KeyDeserializer<'de> {
    key: &'de str,
}

impl<'de> de::Deserializer<'de> for KeyDeserializer<'de> {
    type Error = Error;

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(self.key)
    }
}

#[test]
fn test_query() {
    #[derive(serde::Deserialize)]
    struct QueryA {
        a: i32,
    }

    let query = "a=42";
    let result = deserialize_query::<QueryA>(query).unwrap();
    assert_eq!(result.a, 42);

    #[derive(serde::Deserialize)]
    struct QueryB {
        b: String,
    }

    let query = "b=hello";
    let result = deserialize_query::<QueryB>(query).unwrap();
    assert_eq!(result.b, "hello");

    let query = "a=42&b=hello";
    let result = deserialize_query::<(QueryA, QueryB)>(query).unwrap();

    assert_eq!(result.0.a, 42);
    assert_eq!(result.1.b, "hello");

    let query = "b=hello&a=42";
    let result = deserialize_query::<(QueryA, QueryB)>(query).unwrap();

    assert_eq!(result.0.a, 42);
    assert_eq!(result.1.b, "hello");
}
