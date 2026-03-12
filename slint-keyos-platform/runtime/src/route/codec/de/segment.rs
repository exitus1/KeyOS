// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{
    de::{self, IntoDeserializer, Visitor},
    forward_to_deserialize_any,
};

use super::{common::Deserializer, Error, ParseError};

pub struct SegmentDeserializer<'a, 'de: 'a> {
    pub de: &'a mut Deserializer<'de>,
}

impl<'de, 'a> de::Deserializer<'de> for SegmentDeserializer<'a, 'de> {
    type Error = Error;

    forward_to_deserialize_any! {
        i128 u128 char
        bytes byte_buf unit unit_struct newtype_struct tuple
        tuple_struct
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(self.de.error(ParseError::UnexpectedType { expected: &["list", "struct", "number", "string"] }))
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let _ = self.de.next_char_exact('[')?;
        let value = visitor.visit_seq(CommaSeparated::new(self.de))?;
        let _ = self.de.next_char_exact(']')?;
        Ok(value)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let _ = self.de.next_char_exact('{')?;
        let value = visitor.visit_map(CommaSeparated::new(self.de))?;
        let _ = self.de.next_char_exact('}')?;
        Ok(value)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let decoded = match percent_encoding::percent_decode_str(self.de.take_next_str()).decode_utf8() {
            Ok(decoded) => decoded,
            Err(_) => return Err(self.de.error(ParseError::InvalidUtf8)),
        };
        visitor.visit_str(decoded.as_ref())
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let decoded = match percent_encoding::percent_decode_str(self.de.take_next_str()).decode_utf8() {
            Ok(decoded) => decoded,
            Err(_) => return Err(self.de.error(ParseError::InvalidUtf8)),
        };
        visitor.visit_string(decoded.to_string())
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        // only allow unit enums.
        visitor.visit_enum(self.de.take_next_str().into_deserializer())
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let value = self.de.next_str_matches(&["true", "false"])?;
        visitor.visit_bool(value == "true")
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let num = match self.de.take_next_str().parse() {
            Ok(n) => n,
            Err(e) => return Err(self.de.error(ParseError::from(e))),
        };
        visitor.visit_i8(num)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let num = match self.de.take_next_str().parse() {
            Ok(n) => n,
            Err(e) => return Err(self.de.error(ParseError::from(e))),
        };
        visitor.visit_i16(num)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let num = match self.de.take_next_str().parse() {
            Ok(n) => n,
            Err(e) => return Err(self.de.error(ParseError::from(e))),
        };
        visitor.visit_i32(num)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let num = match self.de.take_next_str().parse() {
            Ok(n) => n,
            Err(e) => return Err(self.de.error(ParseError::from(e))),
        };
        visitor.visit_i64(num)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let num = match self.de.take_next_str().parse() {
            Ok(n) => n,
            Err(e) => return Err(self.de.error(ParseError::from(e))),
        };
        visitor.visit_f32(num)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let num = match self.de.take_next_str().parse() {
            Ok(n) => n,
            Err(e) => return Err(self.de.error(ParseError::from(e))),
        };
        visitor.visit_f64(num)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let num = match self.de.take_next_str().parse() {
            Ok(n) => n,
            Err(e) => return Err(self.de.error(ParseError::from(e))),
        };
        visitor.visit_u8(num)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let num = match self.de.take_next_str().parse() {
            Ok(n) => n,
            Err(e) => return Err(self.de.error(ParseError::from(e))),
        };
        visitor.visit_u16(num)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let num = match self.de.take_next_str().parse() {
            Ok(n) => n,
            Err(e) => return Err(self.de.error(ParseError::from(e))),
        };
        visitor.visit_u32(num)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let num = match self.de.take_next_str().parse() {
            Ok(n) => n,
            Err(e) => return Err(self.de.error(ParseError::from(e))),
        };
        visitor.visit_u64(num)
    }
}

struct CommaSeparated<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    first: bool,
}

impl<'a, 'de> CommaSeparated<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self { CommaSeparated { de, first: true } }
}

impl<'de, 'a> de::SeqAccess<'de> for CommaSeparated<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        // Check if there are no more elements.
        if self.de.peek_char()? == ']' {
            return Ok(None);
        }

        // Comma is required before every element except the first.
        if !self.first {
            let _ = self.de.next_char_exact(',')?;
        } else {
            self.first = false;
        }

        seed.deserialize(SegmentDeserializer { de: self.de }).map(Some)
    }
}

impl<'de, 'a> de::MapAccess<'de> for CommaSeparated<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        // Check if there are no more entries.
        if self.de.peek_char()? == '}' {
            return Ok(None);
        }
        // Comma is required before every entry except the first.
        if !self.first {
            let _ = self.de.next_char_exact(',')?;
        } else {
            self.first = false;
        }

        seed.deserialize(SegmentDeserializer { de: self.de }).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let _ = self.de.next_char_exact(':')?;
        // Deserialize a map value.
        seed.deserialize(SegmentDeserializer { de: self.de })
    }
}
