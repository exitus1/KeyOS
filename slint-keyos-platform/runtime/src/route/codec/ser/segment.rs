// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Write;

use serde::{ser, Serializer};

use super::*;
use crate::route::codec::encode::encode_bytes;

pub struct SegmentSerializer<'a, W> {
    pub writer: &'a mut W,
}

macro_rules! serialize_num {
        (Int => $($ty:ty => $meth:ident,)*) => {
            $(
                fn $meth(self, v: $ty) -> Result<Self::Ok, Self::Error> {
                    let mut buf = itoa::Buffer::new();
                    let part = buf.format(v);
                    ser::Serializer::serialize_str(self, part)
                }
            )*
        };

        (Float => $($ty:ty => $meth:ident,)*) => {
            $(
                fn $meth(self, v: $ty) -> Result<Self::Ok, Self::Error> {
                    let mut buf = ryu::Buffer::new();
                    let part = buf.format(v);
                    ser::Serializer::serialize_str(self, part)
                }
            )*
        };
    }

impl<'a, W> ser::Serializer for SegmentSerializer<'a, W>
where
    W: Write,
{
    type Error = Error;
    type Ok = ();
    type SerializeMap = ser::Impossible<Self::Ok, Error>;
    type SerializeSeq = SeqSerializer<'a, W>;
    type SerializeStruct = StructSerializer<'a, W>;
    type SerializeStructVariant = ser::Impossible<Self::Ok, Error>;
    type SerializeTuple = ser::Impossible<Self::Ok, Error>;
    type SerializeTupleStruct = ser::Impossible<Self::Ok, Error>;
    type SerializeTupleVariant = ser::Impossible<Self::Ok, Error>;

    serialize_num! {
        Int =>
        u8 => serialize_u8,
        i8 => serialize_i8,
        u16 => serialize_u16,
        i16 => serialize_i16,
        u32 => serialize_u32,
        i32 => serialize_i32,
        u64 => serialize_u64,
        i64 => serialize_i64,
    }

    serialize_num! {
        Float =>
        f32 => serialize_f32,
        f64 => serialize_f64,
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        ser::Serializer::serialize_bytes(self, v.as_bytes())
    }

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        ser::Serializer::serialize_bytes(self, if v { b"true" } else { b"false" })
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Serializer::serialize_bytes(self, variant.as_bytes())
    }

    // TODO: more efficiency? Is this even necessary?
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        ser::Serializer::serialize_str(self, v.to_string().as_str())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        for byte in encode_bytes(v) {
            self.writer.write_all(byte.as_bytes())?;
        }
        Ok(())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.writer.write_all(b"[")?;
        Ok(SeqSerializer { len, writer: self.writer })
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct, Self::Error> {
        self.writer.write_all(b"{")?;
        Ok(StructSerializer { writer: self.writer, len })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        unsupported!(map)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> { unsupported!(unit_struct) }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        unsupported!(unit_struct)
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

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> { unsupported!(tuple) }

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

pub struct SeqSerializer<'a, W> {
    len: Option<usize>,
    writer: &'a mut W,
}

impl<'a, W> serde::ser::SerializeSeq for SeqSerializer<'a, W>
where
    W: Write,
{
    type Error = Error;
    type Ok = ();

    fn serialize_element<T: ?Sized + serde::ser::Serialize>(
        &mut self,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(SegmentSerializer { writer: self.writer })?;

        if let Some(ref mut len) = self.len {
            if *len > 1 {
                self.writer.write_all(b",")?;
                *len -= 1
            }
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_all(b"]")?;
        Ok(())
    }
}

pub struct StructSerializer<'a, W> {
    writer: &'a mut W,
    len: usize,
}

impl<'a, W> ser::SerializeStruct for StructSerializer<'a, W>
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
        Serializer::serialize_str(SegmentSerializer { writer: self.writer }, key)?;
        self.writer.write_all(b":")?;
        value.serialize(SegmentSerializer { writer: self.writer })?;
        self.len -= 1;
        if self.len > 0 {
            self.writer.write_all(b",")?;
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_all(b"}")?;
        Ok(())
    }
}
