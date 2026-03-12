// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Write;

use serde::{ser, Serializer};

use super::{segment::SegmentSerializer, unsupported, Error};

pub fn serialize_query_partial(query: &impl ser::Serialize, writer: &mut impl Write) -> Result<(), Error> {
    let mut first = true;
    let ser = QuerySerializer { writer, first: &mut first };
    query.serialize(ser)?;
    Ok(())
}

pub fn serialize_query(query: &impl ser::Serialize) -> Result<String, Error> {
    let mut writer = Vec::new();
    serialize_query_partial(query, &mut writer)?;
    Ok(unsafe { String::from_utf8_unchecked(writer) })
}

struct QuerySerializer<'a, W> {
    first: &'a mut bool,
    writer: &'a mut W,
}

impl<'a, W> Serializer for QuerySerializer<'a, W>
where
    W: Write,
{
    type Error = Error;
    type Ok = ();
    type SerializeMap = ser::Impossible<Self::Ok, Error>;
    type SerializeSeq = ser::Impossible<Self::Ok, Error>;
    type SerializeStruct = QueryParamSerializer<'a, W>;
    type SerializeStructVariant = ser::Impossible<Self::Ok, Error>;
    type SerializeTuple = QueryParamSerializer<'a, W>;
    type SerializeTupleStruct = ser::Impossible<Self::Ok, Error>;
    type SerializeTupleVariant = ser::Impossible<Self::Ok, Error>;

    unsupported! {
        bool => serialize_bool,
        i8 => serialize_i8,
        i16 => serialize_i16,
        i32 => serialize_i32,
        i64 => serialize_i64,
        i128 => serialize_i128,
        u8 => serialize_u8,
        u16 => serialize_u16,
        u32 => serialize_u32,
        u64 => serialize_u64,
        u128 => serialize_u128,
        f32 => serialize_f32,
        f64 => serialize_f64,
        char => serialize_char,
        &str => serialize_str,
        &[u8] => serialize_bytes,
        &'static str => serialize_unit_struct,
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(QueryParamSerializer { writer: self.writer, len, first: self.first })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(QueryParamSerializer { writer: self.writer, len, first: self.first })
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        unsupported!(seq)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> { unsupported!(unit_struct) }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        unsupported!(unit_variant)
    }

    fn serialize_newtype_struct<T: ?Sized + ser::Serialize>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        unsupported!(newtype_struct)
    }

    fn serialize_newtype_variant<T: ?Sized + ser::Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        unsupported!(newtype_variant)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> { unsupported!(none) }

    fn serialize_some<T: ?Sized + ser::Serialize>(self, _value: &T) -> Result<Self::Ok, Self::Error> {
        unsupported!(some)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        unsupported!(tuple_struct)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        unsupported!(tuple_variant)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        unsupported!(map)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        unsupported!(struct_variant)
    }
}

struct QueryParamSerializer<'a, W> {
    writer: &'a mut W,
    first: &'a mut bool,
    len: usize,
}

impl<'a, W> ser::SerializeStruct for QueryParamSerializer<'a, W>
where
    W: Write,
{
    type Error = Error;
    type Ok = ();

    fn serialize_field<T: ?Sized + serde::ser::Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        if *self.first {
            *self.first = false;
            self.writer.write_all(b"?")?;
        }
        // TODO: Do we need to encode key?
        self.writer.write_all(key.as_bytes())?;
        self.writer.write_all(b"=")?;

        value.serialize(SegmentSerializer { writer: self.writer })?;

        self.len -= 1;

        if self.len > 0 && !*self.first {
            self.writer.write_all(b"&")?;
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> { Ok(()) }
}

impl<'a, W> ser::SerializeTuple for QueryParamSerializer<'a, W>
where
    W: Write,
{
    type Error = Error;
    type Ok = ();

    fn serialize_element<T: ?Sized + serde::ser::Serialize>(
        &mut self,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(QuerySerializer { writer: self.writer, first: self.first })?;

        self.len -= 1;

        if self.len > 0 && !*self.first {
            self.writer.write_all(b"&")?;
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> { Ok(()) }
}

#[test]
fn test_query_ser() {
    #[derive(serde::Serialize)]
    struct PathA {
        a: i32,
    }

    #[derive(serde::Serialize)]
    struct PathB {
        b: String,
    }

    #[derive(serde::Serialize)]
    struct Empty {}

    #[derive(serde::Serialize)]
    struct Raw {
        r#field_a: String,
    }

    let path = PathA { a: 1 };

    assert_eq!(serialize_query(&path).unwrap(), "?a=1");

    let path = (PathA { a: 1 }, PathB { b: "test".into() });

    assert_eq!(serialize_query(&path).unwrap(), "?a=1&b=test");

    let path = (Empty {}, PathA { a: 1 });

    assert_eq!(serialize_query(&path).unwrap(), "?a=1");

    assert_eq!(serialize_query(&Empty {}).unwrap(), "");

    let path = Raw { r#field_a: "test".into() };

    assert_eq!(serialize_query(&path).unwrap(), "?field_a=test");
}
