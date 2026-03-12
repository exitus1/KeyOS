// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Write;

use serde::{ser, Serializer};

use super::{segment::SegmentSerializer, unsupported, Error};

pub fn serialize_path(value: &impl ser::Serialize) -> Result<String, Error> {
    let mut writer = Vec::new();
    value.serialize(PathSerializer { writer: &mut writer })?;
    Ok(unsafe { String::from_utf8_unchecked(writer) })
}

pub fn serialize_path_partial(query: &impl ser::Serialize, writer: &mut impl Write) -> Result<(), Error> {
    let ser = PathSerializer { writer };
    query.serialize(ser)?;
    Ok(())
}

struct PathSerializer<'a, W> {
    writer: &'a mut W,
}

impl<'a, W> Serializer for PathSerializer<'a, W>
where
    W: Write,
{
    type Error = Error;
    type Ok = ();
    type SerializeMap = ser::Impossible<Self::Ok, Error>;
    type SerializeSeq = PathParamSerializer<'a, W>;
    type SerializeStruct = PathParamSerializer<'a, W>;
    type SerializeStructVariant = ser::Impossible<Self::Ok, Error>;
    type SerializeTuple = PathParamSerializer<'a, W>;
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

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(PathParamSerializer { writer: self.writer })
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(PathParamSerializer { writer: self.writer })
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

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(PathParamSerializer { writer: self.writer })
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

struct PathParamSerializer<'a, W> {
    writer: &'a mut W,
}

impl<'a, W> ser::SerializeStruct for PathParamSerializer<'a, W>
where
    W: Write,
{
    type Error = Error;
    type Ok = ();

    fn serialize_field<T: ?Sized + serde::ser::Serialize>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        self.writer.write_all(b"/")?;
        value.serialize(SegmentSerializer { writer: self.writer })?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> { Ok(()) }
}

impl<'a, W> ser::SerializeSeq for PathParamSerializer<'a, W>
where
    W: Write,
{
    type Error = Error;
    type Ok = ();

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ser::Serialize,
    {
        self.writer.write_all(b"/")?;
        value.serialize(SegmentSerializer { writer: self.writer })?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> { Ok(()) }
}

impl<'a, W> ser::SerializeTuple for PathParamSerializer<'a, W>
where
    W: Write,
{
    type Error = Error;
    type Ok = ();

    fn serialize_element<T: ?Sized + serde::ser::Serialize>(&mut self, value: &T) -> Result<(), Error> {
        value.serialize(PathSerializer { writer: self.writer })?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> { Ok(()) }
}

#[test]
fn test_path_ser() {
    #[derive(serde::Serialize)]
    struct PathA {
        a: i32,
    }

    #[derive(serde::Serialize)]
    struct PathB {
        b: String,
    }

    let path = (PathA { a: 1 }, PathB { b: "test".into() });

    let serialized = serialize_path(&path).unwrap();

    assert_eq!(serialized, "/1/test");
}
