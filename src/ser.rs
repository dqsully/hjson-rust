// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Serialize a Rust data structure into JSON data.

use std::fmt;
use std::io;
use std::num::FpCategory;
use std::str;

use serde::ser::{self, Impossible};
use super::error::{Error, ErrorCode, Result};

use itoa;
use dtoa;

use regex::Regex;

lazy_static! {
    // If a string matches this, go to RE_STR_DOUBLE
    static ref RE_VALUE: Regex = Regex::new(r#"^(?:-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?|true|false|null)(?:\s*$|\s*(?:[,}\]#/]|/[^/*]))"#).unwrap();
    // If a string doesn't match this, go to RE_STR_DOUBLE, else use no quotes
    static ref RE_STR_NONE: Regex = Regex::new(r#"^(?:[^\x00-\x1f\s"'{}\[\],:/#]|/[^\x00-\x1f\t\n/*])(?:[^\x00-\x1f\t\n]*[^\x00-\x1f\s"])?$"#).unwrap();
    // If a string doesn't match this, go to RE_STR_MULTILINE, else use quotes and whitespace escapes
    static ref RE_STR_DOUBLE: Regex = Regex::new(r#"^([^\t\n"](?:[^\t\n"]*[^\t\n"])?|[\s\x08]+)$"#).unwrap();
    // If a quoted string matches this, go to RE_STR_MULTILINE
    static ref RE_HAS_NEWLINE: Regex = Regex::new(r#"\n"#).unwrap();
    // If a string doesn't match this, use double quotes, else use multiline quotes
    static ref RE_STR_MULTILINE: Regex = Regex::new(r#"^([^']|'[^']|''[^'])+$"#).unwrap();

    // If a member string doesn't match this, use double quotes, else use no quotes
    static ref RE_MEMBER_NONE: Regex = Regex::new(r#"^(?:[^\x00-\x1f\s"'{}\[\],:/#]|/[^\x00-\x1f\s"'{}\[\],:/*])(?:(?:[^\x00-\x1f\s"'{}\[\],:/#]|/[^\x00-\x1f\s"'{}\[\],:/*])*[^\x00-\x1f\s"'{}\[\],:/#])?$"#).unwrap();
}

/// A structure for serializing Rust values into JSON.
pub struct Serializer<W, F = CompactFormatter> {
    writer: W,
    formatter: F,
}

impl<W> Serializer<W>
where
    W: io::Write,
{
    /// Creates a new JSON serializer.
    #[inline]
    pub fn new(writer: W) -> Self {
        Serializer::with_formatter(writer, CompactFormatter)
    }
}

impl<'a, W> Serializer<W, PrettyFormatter<'a>>
where
    W: io::Write,
{
    /// Creates a new JSON pretty print serializer.
    #[inline]
    pub fn pretty(writer: W) -> Self {
        Serializer::with_formatter(writer, PrettyFormatter::new())
    }
}

impl<W, F> Serializer<W, F>
where
    W: io::Write,
    F: Formatter,
{
    /// Creates a new JSON visitor whose output will be written to the writer
    /// specified.
    #[inline]
    pub fn with_formatter(writer: W, formatter: F) -> Self {
        Serializer {
            writer: writer,
            formatter: formatter,
        }
    }

    /// Unwrap the `Writer` from the `Serializer`.
    #[inline]
    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl<'a, W, F> ser::Serializer for &'a mut Serializer<W, F>
where
    W: io::Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Compound<'a, W, F>;
    type SerializeTuple = Compound<'a, W, F>;
    type SerializeTupleStruct = Compound<'a, W, F>;
    type SerializeTupleVariant = Compound<'a, W, F>;
    type SerializeMap = Compound<'a, W, F>;
    type SerializeStruct = Compound<'a, W, F>;
    type SerializeStructVariant = Compound<'a, W, F>;

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<()> {
        try!(
            self.formatter
                .write_bool(&mut self.writer, value)
                .map_err(Error::io)
        );
        Ok(())
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<()> {
        try!(
            self.formatter
                .write_i8(&mut self.writer, value)
                .map_err(Error::io)
        );
        Ok(())
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<()> {
        try!(
            self.formatter
                .write_i16(&mut self.writer, value)
                .map_err(Error::io)
        );
        Ok(())
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<()> {
        try!(
            self.formatter
                .write_i32(&mut self.writer, value)
                .map_err(Error::io)
        );
        Ok(())
    }

    #[inline]
    fn serialize_i64(self, value: i64) -> Result<()> {
        try!(
            self.formatter
                .write_i64(&mut self.writer, value)
                .map_err(Error::io)
        );
        Ok(())
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<()> {
        try!(
            self.formatter
                .write_u8(&mut self.writer, value)
                .map_err(Error::io)
        );
        Ok(())
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<()> {
        try!(
            self.formatter
                .write_u16(&mut self.writer, value)
                .map_err(Error::io)
        );
        Ok(())
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<()> {
        try!(
            self.formatter
                .write_u32(&mut self.writer, value)
                .map_err(Error::io)
        );
        Ok(())
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<()> {
        try!(
            self.formatter
                .write_u64(&mut self.writer, value)
                .map_err(Error::io)
        );
        Ok(())
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<()> {
        match value.classify() {
            FpCategory::Nan | FpCategory::Infinite => {
                try!(
                    self.formatter
                        .write_null(&mut self.writer)
                        .map_err(Error::io)
                );
            }
            _ => {
                try!(
                    self.formatter
                        .write_f32(&mut self.writer, value)
                        .map_err(Error::io)
                );
            }
        }
        Ok(())
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<()> {
        match value.classify() {
            FpCategory::Nan | FpCategory::Infinite => {
                try!(
                    self.formatter
                        .write_null(&mut self.writer)
                        .map_err(Error::io)
                );
            }
            _ => {
                try!(
                    self.formatter
                        .write_f64(&mut self.writer, value)
                        .map_err(Error::io)
                );
            }
        }
        Ok(())
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<()> {
        // A char encoded as UTF-8 takes 4 bytes at most.
        let mut buf = [0; 4];
        self.serialize_str(value.encode_utf8(&mut buf))
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<()> {
        try!(format_escaped_str(&mut self.writer, &mut self.formatter, value).map_err(Error::io));
        Ok(())
    }

    #[inline]
    fn serialize_bytes(self, value: &[u8]) -> Result<()> {
        use serde::ser::SerializeSeq;
        let mut seq = try!(self.serialize_seq(Some(value.len())));
        for byte in value {
            try!(seq.serialize_element(byte));
        }
        seq.end()
    }

    #[inline]
    fn serialize_unit(self) -> Result<()> {
        try!(
            self.formatter
                .write_null(&mut self.writer)
                .map_err(Error::io)
        );
        Ok(())
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    /// Serialize newtypes without an object wrapper.
    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ser::Serialize,
    {
        try!(
            self.formatter
                .begin_object(&mut self.writer)
                .map_err(Error::io)
        );
        try!(
            self.formatter
                .begin_object_key(&mut self.writer, true)
                .map_err(Error::io)
        );
        try!(self.serialize_str(variant));
        try!(
            self.formatter
                .end_object_key(&mut self.writer)
                .map_err(Error::io)
        );
        try!(
            self.formatter
                .begin_object_value(&mut self.writer)
                .map_err(Error::io)
        );
        try!(value.serialize(&mut *self));
        try!(
            self.formatter
                .end_object_value(&mut self.writer)
                .map_err(Error::io)
        );
        try!(
            self.formatter
                .end_object(&mut self.writer)
                .map_err(Error::io)
        );
        Ok(())
    }

    #[inline]
    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        if len == Some(0) {
            try!(
                self.formatter
                    .begin_array(&mut self.writer)
                    .map_err(Error::io)
            );
            try!(
                self.formatter
                    .end_array(&mut self.writer)
                    .map_err(Error::io)
            );
            Ok(
                Compound {
                    ser: self,
                    state: State::Empty,
                },
            )
        } else {
            try!(
                self.formatter
                    .begin_array(&mut self.writer)
                    .map_err(Error::io)
            );
            Ok(
                Compound {
                    ser: self,
                    state: State::First,
                },
            )
        }
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        try!(
            self.formatter
                .begin_object(&mut self.writer)
                .map_err(Error::io)
        );
        try!(
            self.formatter
                .begin_object_key(&mut self.writer, true)
                .map_err(Error::io)
        );
        try!(self.serialize_str(variant));
        try!(
            self.formatter
                .end_object_key(&mut self.writer)
                .map_err(Error::io)
        );
        try!(
            self.formatter
                .begin_object_value(&mut self.writer)
                .map_err(Error::io)
        );
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        if len == Some(0) {
            try!(
                self.formatter
                    .begin_object(&mut self.writer)
                    .map_err(Error::io)
            );
            try!(
                self.formatter
                    .end_object(&mut self.writer)
                    .map_err(Error::io)
            );
            Ok(
                Compound {
                    ser: self,
                    state: State::Empty,
                },
            )
        } else {
            try!(
                self.formatter
                    .begin_object(&mut self.writer)
                    .map_err(Error::io)
            );
            Ok(
                Compound {
                    ser: self,
                    state: State::First,
                },
            )
        }
    }

    #[inline]
    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        try!(
            self.formatter
                .begin_object(&mut self.writer)
                .map_err(Error::io)
        );
        try!(
            self.formatter
                .begin_object_key(&mut self.writer, true)
                .map_err(Error::io)
        );
        try!(self.serialize_str(variant));
        try!(
            self.formatter
                .end_object_key(&mut self.writer)
                .map_err(Error::io)
        );
        try!(
            self.formatter
                .begin_object_value(&mut self.writer)
                .map_err(Error::io)
        );
        self.serialize_map(Some(len))
    }

    fn collect_str<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: fmt::Display,
    {
        use std::fmt::Write;

        struct Adapter<'ser, W: 'ser, F: 'ser> {
            writer: &'ser mut W,
            formatter: &'ser mut F,
            error: Option<io::Error>,
        }

        impl<'ser, W, F> Write for Adapter<'ser, W, F>
        where
            W: io::Write,
            F: Formatter,
        {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                assert!(self.error.is_none());
                match self.formatter.write_string(self.writer, s) {
                    Ok(()) => Ok(()),
                    Err(err) => {
                        self.error = Some(err);
                        Err(fmt::Error)
                    }
                }
            }
        }

        let mut adapter = Adapter {
            writer: &mut self.writer,
            formatter: &mut self.formatter,
            error: None,
        };
        match write!(adapter, "{}", value) {
            Ok(()) => assert!(adapter.error.is_none()),
            Err(fmt::Error) => {
                return Err(Error::io(adapter.error.expect("there should be an error")));
            }
        }
        Ok(())
    }
}

#[doc(hidden)]
#[derive(Eq, PartialEq)]
pub enum State {
    Empty,
    First,
    Rest,
}

#[doc(hidden)]
pub struct Compound<'a, W: 'a, F: 'a> {
    ser: &'a mut Serializer<W, F>,
    state: State,
}

impl<'a, W, F> ser::SerializeSeq for Compound<'a, W, F>
where
    W: io::Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        try!(
            self.ser
                .formatter
                .begin_array_value(&mut self.ser.writer, self.state == State::First)
                .map_err(Error::io)
        );
        self.state = State::Rest;
        try!(value.serialize(&mut *self.ser));
        try!(
            self.ser
                .formatter
                .end_array_value(&mut self.ser.writer)
                .map_err(Error::io)
        );
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<()> {
        match self.state {
            State::Empty => {}
            _ => {
                try!(
                    self.ser
                        .formatter
                        .end_array(&mut self.ser.writer)
                        .map_err(Error::io)
                )
            }
        }
        Ok(())
    }
}

impl<'a, W, F> ser::SerializeTuple for Compound<'a, W, F>
where
    W: io::Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a, W, F> ser::SerializeTupleStruct for Compound<'a, W, F>
where
    W: io::Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a, W, F> ser::SerializeTupleVariant for Compound<'a, W, F>
where
    W: io::Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<()> {
        match self.state {
            State::Empty => {}
            _ => {
                try!(
                    self.ser
                        .formatter
                        .end_array(&mut self.ser.writer)
                        .map_err(Error::io)
                )
            }
        }
        try!(
            self.ser
                .formatter
                .end_object_value(&mut self.ser.writer)
                .map_err(Error::io)
        );
        try!(
            self.ser
                .formatter
                .end_object(&mut self.ser.writer)
                .map_err(Error::io)
        );
        Ok(())
    }
}

impl<'a, W, F> ser::SerializeMap for Compound<'a, W, F>
where
    W: io::Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        try!(
            self.ser
                .formatter
                .begin_object_key(&mut self.ser.writer, self.state == State::First)
                .map_err(Error::io)
        );
        self.state = State::Rest;

        try!(key.serialize(MapKeySerializer { ser: self.ser }));

        try!(
            self.ser
                .formatter
                .end_object_key(&mut self.ser.writer)
                .map_err(Error::io)
        );
        Ok(())
    }

    #[inline]
    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        try!(
            self.ser
                .formatter
                .begin_object_value(&mut self.ser.writer)
                .map_err(Error::io)
        );
        try!(value.serialize(&mut *self.ser));
        try!(
            self.ser
                .formatter
                .end_object_value(&mut self.ser.writer)
                .map_err(Error::io)
        );
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<()> {
        match self.state {
            State::Empty => {}
            _ => {
                try!(
                    self.ser
                        .formatter
                        .end_object(&mut self.ser.writer)
                        .map_err(Error::io)
                )
            }
        }
        Ok(())
    }
}

impl<'a, W, F> ser::SerializeStruct for Compound<'a, W, F>
where
    W: io::Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        try!(ser::SerializeMap::serialize_key(self, key));
        ser::SerializeMap::serialize_value(self, value)
    }

    #[inline]
    fn end(self) -> Result<()> {
        ser::SerializeMap::end(self)
    }
}

impl<'a, W, F> ser::SerializeStructVariant for Compound<'a, W, F>
where
    W: io::Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        ser::SerializeStruct::serialize_field(self, key, value)
    }

    #[inline]
    fn end(self) -> Result<()> {
        match self.state {
            State::Empty => {}
            _ => {
                try!(
                    self.ser
                        .formatter
                        .end_object(&mut self.ser.writer)
                        .map_err(Error::io)
                )
            }
        }
        try!(
            self.ser
                .formatter
                .end_object_value(&mut self.ser.writer)
                .map_err(Error::io)
        );
        try!(
            self.ser
                .formatter
                .end_object(&mut self.ser.writer)
                .map_err(Error::io)
        );
        Ok(())
    }
}

struct MapKeySerializer<'a, W: 'a, F: 'a> {
    ser: &'a mut Serializer<W, F>,
}

fn key_must_be_a_string() -> Error {
    Error::syntax(ErrorCode::KeyMustBeAString, 0, 0)
}

impl<'a, W, F> ser::Serializer for MapKeySerializer<'a, W, F>
where
    W: io::Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_str(self, value: &str) -> Result<()> {
        self.ser.formatter.write_member_string(&mut self.ser.writer, value)
            .map_err(Error::io)
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.ser.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        value.serialize(self)
    }

    type SerializeSeq = Impossible<(), Error>;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Impossible<(), Error>;
    type SerializeStruct = Impossible<(), Error>;
    type SerializeStructVariant = Impossible<(), Error>;

    fn serialize_bool(self, _value: bool) -> Result<()> {
        Err(key_must_be_a_string())
    }

    fn serialize_i8(self, value: i8) -> Result<()> {
        try!(
            self.ser
                .formatter
                .begin_string(&mut self.ser.writer)
                .map_err(Error::io)
        );
        try!(
            self.ser
                .formatter
                .write_i8(&mut self.ser.writer, value)
                .map_err(Error::io)
        );
        try!(
            self.ser
                .formatter
                .end_string(&mut self.ser.writer)
                .map_err(Error::io)
        );
        Ok(())
    }

    fn serialize_i16(self, value: i16) -> Result<()> {
        try!(
            self.ser
                .formatter
                .begin_string(&mut self.ser.writer)
                .map_err(Error::io)
        );
        try!(
            self.ser
                .formatter
                .write_i16(&mut self.ser.writer, value)
                .map_err(Error::io)
        );
        try!(
            self.ser
                .formatter
                .end_string(&mut self.ser.writer)
                .map_err(Error::io)
        );
        Ok(())
    }

    fn serialize_i32(self, value: i32) -> Result<()> {
        try!(
            self.ser
                .formatter
                .begin_string(&mut self.ser.writer)
                .map_err(Error::io)
        );
        try!(
            self.ser
                .formatter
                .write_i32(&mut self.ser.writer, value)
                .map_err(Error::io)
        );
        try!(
            self.ser
                .formatter
                .end_string(&mut self.ser.writer)
                .map_err(Error::io)
        );
        Ok(())
    }

    fn serialize_i64(self, value: i64) -> Result<()> {
        try!(
            self.ser
                .formatter
                .begin_string(&mut self.ser.writer)
                .map_err(Error::io)
        );
        try!(
            self.ser
                .formatter
                .write_i64(&mut self.ser.writer, value)
                .map_err(Error::io)
        );
        try!(
            self.ser
                .formatter
                .end_string(&mut self.ser.writer)
                .map_err(Error::io)
        );
        Ok(())
    }

    fn serialize_u8(self, value: u8) -> Result<()> {
        try!(
            self.ser
                .formatter
                .begin_string(&mut self.ser.writer)
                .map_err(Error::io)
        );
        try!(
            self.ser
                .formatter
                .write_u8(&mut self.ser.writer, value)
                .map_err(Error::io)
        );
        try!(
            self.ser
                .formatter
                .end_string(&mut self.ser.writer)
                .map_err(Error::io)
        );
        Ok(())
    }

    fn serialize_u16(self, value: u16) -> Result<()> {
        try!(
            self.ser
                .formatter
                .begin_string(&mut self.ser.writer)
                .map_err(Error::io)
        );
        try!(
            self.ser
                .formatter
                .write_u16(&mut self.ser.writer, value)
                .map_err(Error::io)
        );
        try!(
            self.ser
                .formatter
                .end_string(&mut self.ser.writer)
                .map_err(Error::io)
        );
        Ok(())
    }

    fn serialize_u32(self, value: u32) -> Result<()> {
        try!(
            self.ser
                .formatter
                .begin_string(&mut self.ser.writer)
                .map_err(Error::io)
        );
        try!(
            self.ser
                .formatter
                .write_u32(&mut self.ser.writer, value)
                .map_err(Error::io)
        );
        try!(
            self.ser
                .formatter
                .end_string(&mut self.ser.writer)
                .map_err(Error::io)
        );
        Ok(())
    }

    fn serialize_u64(self, value: u64) -> Result<()> {
        try!(
            self.ser
                .formatter
                .begin_string(&mut self.ser.writer)
                .map_err(Error::io)
        );
        try!(
            self.ser
                .formatter
                .write_u64(&mut self.ser.writer, value)
                .map_err(Error::io)
        );
        try!(
            self.ser
                .formatter
                .end_string(&mut self.ser.writer)
                .map_err(Error::io)
        );
        Ok(())
    }

    fn serialize_f32(self, _value: f32) -> Result<()> {
        Err(key_must_be_a_string())
    }

    fn serialize_f64(self, _value: f64) -> Result<()> {
        Err(key_must_be_a_string())
    }

    fn serialize_char(self, _value: char) -> Result<()> {
        Err(key_must_be_a_string())
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<()> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit(self) -> Result<()> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Err(key_must_be_a_string())
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()>
    where
        T: ser::Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_none(self) -> Result<()> {
        Err(key_must_be_a_string())
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(key_must_be_a_string())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(key_must_be_a_string())
    }
}

/// This trait abstracts away serializing the JSON control characters, which allows the user to
/// optionally pretty print the JSON output.
pub trait Formatter {
    /// Writes a `null` value to the specified writer.
    #[inline]
    fn write_null<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        writer.write_all(b"null")
    }

    /// Writes a `true` or `false` value to the specified writer.
    #[inline]
    fn write_bool<W: ?Sized>(&mut self, writer: &mut W, value: bool) -> io::Result<()>
    where
        W: io::Write,
    {
        let s = if value {
            b"true" as &[u8]
        } else {
            b"false" as &[u8]
        };
        writer.write_all(s)
    }

    /// Writes an integer value like `-123` to the specified writer.
    #[inline]
    fn write_i8<W: ?Sized>(&mut self, writer: &mut W, value: i8) -> io::Result<()>
    where
        W: io::Write,
    {
        itoa::write(writer, value).map(|_| ())
    }

    /// Writes an integer value like `-123` to the specified writer.
    #[inline]
    fn write_i16<W: ?Sized>(&mut self, writer: &mut W, value: i16) -> io::Result<()>
    where
        W: io::Write,
    {
        itoa::write(writer, value).map(|_| ())
    }

    /// Writes an integer value like `-123` to the specified writer.
    #[inline]
    fn write_i32<W: ?Sized>(&mut self, writer: &mut W, value: i32) -> io::Result<()>
    where
        W: io::Write,
    {
        itoa::write(writer, value).map(|_| ())
    }

    /// Writes an integer value like `-123` to the specified writer.
    #[inline]
    fn write_i64<W: ?Sized>(&mut self, writer: &mut W, value: i64) -> io::Result<()>
    where
        W: io::Write,
    {
        itoa::write(writer, value).map(|_| ())
    }

    /// Writes an integer value like `123` to the specified writer.
    #[inline]
    fn write_u8<W: ?Sized>(&mut self, writer: &mut W, value: u8) -> io::Result<()>
    where
        W: io::Write,
    {
        itoa::write(writer, value).map(|_| ())
    }

    /// Writes an integer value like `123` to the specified writer.
    #[inline]
    fn write_u16<W: ?Sized>(&mut self, writer: &mut W, value: u16) -> io::Result<()>
    where
        W: io::Write,
    {
        itoa::write(writer, value).map(|_| ())
    }

    /// Writes an integer value like `123` to the specified writer.
    #[inline]
    fn write_u32<W: ?Sized>(&mut self, writer: &mut W, value: u32) -> io::Result<()>
    where
        W: io::Write,
    {
        itoa::write(writer, value).map(|_| ())
    }

    /// Writes an integer value like `123` to the specified writer.
    #[inline]
    fn write_u64<W: ?Sized>(&mut self, writer: &mut W, value: u64) -> io::Result<()>
    where
        W: io::Write,
    {
        itoa::write(writer, value).map(|_| ())
    }

    /// Writes a floating point value like `-31.26e+12` to the specified writer.
    #[inline]
    fn write_f32<W: ?Sized>(&mut self, writer: &mut W, value: f32) -> io::Result<()>
    where
        W: io::Write,
    {
        dtoa::write(writer, value).map(|_| ())
    }

    /// Writes a floating point value like `-31.26e+12` to the specified writer.
    #[inline]
    fn write_f64<W: ?Sized>(&mut self, writer: &mut W, value: f64) -> io::Result<()>
    where
        W: io::Write,
    {
        dtoa::write(writer, value).map(|_| ())
    }

    /// Called before each series of `write_string_fragment` and
    /// `write_char_escape`.  Writes a `"` to the specified writer.
    #[inline]
    fn begin_string<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        writer.write_all(b"\"")
    }

    /// Called after each series of `write_string_fragment` and
    /// `write_char_escape`.  Writes a `"` to the specified writer.
    #[inline]
    fn end_string<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        writer.write_all(b"\"")
    }

    /// Writes a full string including starting and ending quotes
    #[inline]
    fn write_string<W: ?Sized>(&mut self, writer: &mut W, string: &str) -> io::Result<()>
    where
        W: io::Write,
    {
        try!(writer.write_all(b"\""));


        let bytes = string.as_bytes();

        let mut start = 0;
        let mut skip = 0;

        for (i, &byte) in bytes.iter().enumerate() {
            if skip > 1 {
                skip -= 1;
                start = i + 1;
                continue;
            }

            let escape = ESCAPE[byte as usize];
            if escape == 0 {
                continue;
            }

            if start < i {
                try!(writer.write_all(&bytes[start..i]));
            }

            try!(self.write_char_escape(writer, &bytes[i..], &mut skip));

            start = i + 1;
        }

        if start != bytes.len() {
            try!(writer.write_all(&bytes[start..]));
        }

        writer.write_all(b"\"")
    }

    /// Writes a full member string including starting and ending quotes
    #[inline]
    fn write_member_string<W: ?Sized>(&mut self, writer: &mut W, string: &str) -> io::Result<()>
    where
        W: io::Write,
    {
        self.write_string(writer, string)
    }

    /// Writes a character escape code to the specified writer.
    #[inline]
    fn write_char_escape<W: ?Sized>(
        &mut self,
        writer: &mut W,
        bytes: &[u8],
        bytes_read: &mut usize,
    ) -> io::Result<()>
    where
        W: io::Write,
    {
        let unicode_escape: [u8; 10];
        static HEX_DIGITS: [u8; 16] = *b"0123456789abcdef";

        let mut ch: u16 = bytes[0] as u16;

        *bytes_read = 1;

        if bytes.len() >= 2 {
            ch <<= 8;
            ch |= bytes[1] as u16;
            *bytes_read = 2;
        }

        let escape: &[u8] = match ch {
            0x7f...0x9f |
            0xad |
            0x0600...0x0604 |
            0x070f |
            0x17b4 |
            0x17b5 |
            0x200c...0x200f |
            0x2028...0x202f |
            0x2060...0x206f |
            0xfeff |
            0xfff0...0xffff => {
                unicode_escape = [
                    b'\\',
                    b'u',
                    HEX_DIGITS[(ch >> 12) as usize],
                    HEX_DIGITS[(ch >> 8 & 0b00001111) as usize],
                    HEX_DIGITS[(ch >> 4 & 0b00001111) as usize],
                    HEX_DIGITS[(ch & 0b00001111) as usize],
                    0,0,0,0
                ];
                &unicode_escape[0..6]
            },
            _ => {
                *bytes_read = 1;

                match ESCAPE[bytes[0] as usize] {
                    BB => br"\b",
                    TT => br"\t",
                    NN => br"\n",
                    FF => br"\f",
                    RR => br"\r",
                    QU => br#"\""#,
                    BS => br"\\",
                    U => {
                        unicode_escape = [
                            b'\\',
                            b'u',
                            b'0',
                            b'0',
                            HEX_DIGITS[(bytes[0] >> 4) as usize],
                            HEX_DIGITS[(bytes[0] & 0b00001111) as usize],
                            0,0,0,0
                        ];
                        &unicode_escape[0..6]
                    }
                    _ if (bytes[0] >> 5 == 0b110) && (bytes.len() >= 2) && (bytes[1] >> 6 == 0b10) => {
                        *bytes_read = 2;

                        let ch: u16 = ((bytes[0] as u16) & 0b00011111 << 6) | ((bytes[1] as u16) & 0b00111111);

                        match ch {
                            0x7f...0x9f |
                            0xad |
                            0x0600...0x0604 |
                            0x070f |
                            0x17b4 |
                            0x17b5 |
                            0x200c...0x200f |
                            0x2028...0x202f |
                            0x2060...0x206f |
                            0xfeff |
                            0xfff0...0xffff => {
                                unicode_escape = [
                                    b'\\',
                                    b'u',
                                    HEX_DIGITS[(ch >> 12) as usize],
                                    HEX_DIGITS[(ch >> 8 & 0b00001111) as usize],
                                    HEX_DIGITS[(ch >> 4 & 0b00001111) as usize],
                                    HEX_DIGITS[(ch & 0b00001111) as usize],
                                    0,0,0,0
                                ];
                                &unicode_escape[0..6]
                            },
                            _ => {
                                &bytes[0..2]
                            }
                        }
                    }
                    _ if (bytes[0] >> 4 == 0b1110) && (bytes.len() >= 3) && (bytes[1] >> 6 == 0b10) && (bytes[2] >> 6 == 0b10) => {
                        *bytes_read = 3;

                        let ch: u16 = ((((bytes[0] << 4) | (bytes[1] >> 2 & 0b00001111)) as u16) << 8) | (((bytes[1] << 6 & 0b11000000) | (bytes[2] & 0b00111111)) as u16);

                        match ch {
                            0x7f...0x9f |
                            0xad |
                            0x0600...0x0604 |
                            0x070f |
                            0x17b4 |
                            0x17b5 |
                            0x200c...0x200f |
                            0x2028...0x202f |
                            0x2060...0x206f |
                            0xfeff |
                            0xfff0...0xffff => {
                                unicode_escape = [
                                    b'\\',
                                    b'u',
                                    HEX_DIGITS[(ch >> 12) as usize],
                                    HEX_DIGITS[(ch >> 8 & 0b00001111) as usize],
                                    HEX_DIGITS[(ch >> 4 & 0b00001111) as usize],
                                    HEX_DIGITS[(ch & 0b00001111) as usize],
                                    0,0,0,0
                                ];
                                &unicode_escape[0..6]
                            },
                            _ => {
                                &bytes[0..3]
                            }
                        }
                    }
                    _ if (bytes[0] >> 3 == 0b11110) && (bytes.len() >= 4) && (bytes[1] >> 6 == 0b10) && (bytes[2] >> 6 == 0b10) && (bytes[3] >> 6 == 0b10) => {
                        *bytes_read = 4;
                        unicode_escape = [
                            b'\\',
                            b'u',
                            b'{',
                            HEX_DIGITS[(bytes[0] >> 2 & 0b00000001) as usize],
                            HEX_DIGITS[((bytes[0] << 2 & 0b00001100) | (bytes[1] >> 4 & 0b00000011)) as usize],
                            HEX_DIGITS[(bytes[1] & 0b00001111) as usize],
                            HEX_DIGITS[(bytes[2] >> 2 & 0b00001111) as usize],
                            HEX_DIGITS[((bytes[2] << 2 & 0b00001100) | (bytes[3] >> 4 & 0b00000011)) as usize],
                            HEX_DIGITS[(bytes[3] & 0b00001111) as usize],
                            b'}'
                        ];
                        &unicode_escape[0..]
                    }
                    _ => {
                        &bytes[0..1]
                    }
                }
            }
        };

        writer.write_all(escape)
    }

    /// Called before every array.  Writes a `[` to the specified
    /// writer.
    #[inline]
    fn begin_array<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        writer.write_all(b"[")
    }

    /// Called after every array.  Writes a `]` to the specified
    /// writer.
    #[inline]
    fn end_array<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        writer.write_all(b"]")
    }

    /// Called before every array value.  Writes a `,` if needed to
    /// the specified writer.
    #[inline]
    fn begin_array_value<W: ?Sized>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: io::Write,
    {
        if first {
            Ok(())
        } else {
            writer.write_all(b",")
        }
    }

    /// Called after every array value.
    #[inline]
    fn end_array_value<W: ?Sized>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        Ok(())
    }

    /// Called before every object.  Writes a `{` to the specified
    /// writer.
    #[inline]
    fn begin_object<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        writer.write_all(b"{")
    }

    /// Called after every object.  Writes a `}` to the specified
    /// writer.
    #[inline]
    fn end_object<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        writer.write_all(b"}")
    }

    /// Called before every object key.
    #[inline]
    fn begin_object_key<W: ?Sized>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: io::Write,
    {
        if first {
            Ok(())
        } else {
            writer.write_all(b",")
        }
    }

    /// Called after every object key.  A `:` should be written to the
    /// specified writer by either this method or
    /// `begin_object_value`.
    #[inline]
    fn end_object_key<W: ?Sized>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        Ok(())
    }

    /// Called before every object value.  A `:` should be written to
    /// the specified writer by either this method or
    /// `end_object_key`.
    #[inline]
    fn begin_object_value<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        writer.write_all(b":")
    }

    /// Called after every object value.
    #[inline]
    fn end_object_value<W: ?Sized>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        Ok(())
    }
}

/// This structure compacts a JSON value with no extra whitespace.
#[derive(Clone, Debug)]
pub struct CompactFormatter;

impl Formatter for CompactFormatter {}

/// This structure pretty prints a JSON value to make it human readable.
#[derive(Clone, Debug)]
pub struct PrettyFormatter<'a> {
    current_indent: usize,
    has_value: bool,
    in_object: bool,
    indent: &'a [u8],
    next_bracket: Option<u8>,
}

impl<'a> PrettyFormatter<'a> {
    /// Construct a pretty printer formatter that defaults to using two spaces for indentation.
    pub fn new() -> Self {
        PrettyFormatter::with_indent(b"  ")
    }

    /// Construct a pretty printer formatter that uses the `indent` string for indentation.
    pub fn with_indent(indent: &'a [u8]) -> Self {
        PrettyFormatter {
            current_indent: 0,
            has_value: false,
            in_object: false,
            indent: indent,
            next_bracket: None,
        }
    }

    fn try_write_bracket<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        if let Some(bracket) = self.next_bracket {
            if self.in_object {
                try!(writer.write_all(b"\n"));
                try!(indent(writer, self.current_indent, self.indent));
                self.in_object = false;
            }

            self.current_indent += 1;
            self.has_value = false;
            self.next_bracket = None;

            try!(writer.write_all(&[bracket]));
        }

        Ok(())
    }
}

impl<'a> Default for PrettyFormatter<'a> {
    fn default() -> Self {
        PrettyFormatter::new()
    }
}

impl<'a> Formatter for PrettyFormatter<'a> {
    #[inline]
    fn write_null<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        if self.in_object {
            try!(writer.write_all(b" "));
            self.in_object = false;
        }

        writer.write_all(b"null")
    }

    #[inline]
    fn write_bool<W: ?Sized>(&mut self, writer: &mut W, value: bool) -> io::Result<()>
    where
        W: io::Write,
    {
        if self.in_object {
            try!(writer.write_all(b" "));
            self.in_object = false;
        }

        let s = if value {
            b"true" as &[u8]
        } else {
            b"false" as &[u8]
        };
        writer.write_all(s)
    }

    #[inline]
    fn write_i8<W: ?Sized>(&mut self, writer: &mut W, value: i8) -> io::Result<()>
    where
        W: io::Write,
    {
        if self.in_object {
            try!(writer.write_all(b" "));
            self.in_object = false;
        }

        itoa::write(writer, value).map(|_| ())
    }

    #[inline]
    fn write_i16<W: ?Sized>(&mut self, writer: &mut W, value: i16) -> io::Result<()>
    where
        W: io::Write,
    {
        if self.in_object {
            try!(writer.write_all(b" "));
            self.in_object = false;
        }

        itoa::write(writer, value).map(|_| ())
    }

    #[inline]
    fn write_i32<W: ?Sized>(&mut self, writer: &mut W, value: i32) -> io::Result<()>
    where
        W: io::Write,
    {
        if self.in_object {
            try!(writer.write_all(b" "));
            self.in_object = false;
        }

        itoa::write(writer, value).map(|_| ())
    }

    #[inline]
    fn write_i64<W: ?Sized>(&mut self, writer: &mut W, value: i64) -> io::Result<()>
    where
        W: io::Write,
    {
        if self.in_object {
            try!(writer.write_all(b" "));
            self.in_object = false;
        }

        itoa::write(writer, value).map(|_| ())
    }

    #[inline]
    fn write_u8<W: ?Sized>(&mut self, writer: &mut W, value: u8) -> io::Result<()>
    where
        W: io::Write,
    {
        if self.in_object {
            try!(writer.write_all(b" "));
            self.in_object = false;
        }

        itoa::write(writer, value).map(|_| ())
    }

    #[inline]
    fn write_u16<W: ?Sized>(&mut self, writer: &mut W, value: u16) -> io::Result<()>
    where
        W: io::Write,
    {
        if self.in_object {
            try!(writer.write_all(b" "));
            self.in_object = false;
        }

        itoa::write(writer, value).map(|_| ())
    }

    #[inline]
    fn write_u32<W: ?Sized>(&mut self, writer: &mut W, value: u32) -> io::Result<()>
    where
        W: io::Write,
    {
        if self.in_object {
            try!(writer.write_all(b" "));
            self.in_object = false;
        }

        itoa::write(writer, value).map(|_| ())
    }

    #[inline]
    fn write_u64<W: ?Sized>(&mut self, writer: &mut W, value: u64) -> io::Result<()>
    where
        W: io::Write,
    {
        if self.in_object {
            try!(writer.write_all(b" "));
            self.in_object = false;
        }

        itoa::write(writer, value).map(|_| ())
    }

    #[inline]
    fn write_f32<W: ?Sized>(&mut self, writer: &mut W, value: f32) -> io::Result<()>
    where
        W: io::Write,
    {
        if self.in_object {
            try!(writer.write_all(b" "));
            self.in_object = false;
        }

        dtoa::write(writer, value).map(|_| ())
    }

    #[inline]
    fn write_f64<W: ?Sized>(&mut self, writer: &mut W, value: f64) -> io::Result<()>
    where
        W: io::Write,
    {
        if self.in_object {
            try!(writer.write_all(b" "));
            self.in_object = false;
        }

        dtoa::write(writer, value).map(|_| ())
    }

    #[inline]
    fn begin_string<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        if self.in_object {
            try!(writer.write_all(b" "));
            self.in_object = false;
        }

        writer.write_all(b"\"")
    }

    #[inline]
    fn end_string<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        writer.write_all(b"\"")
    }

    #[inline]
    fn write_string<W: ?Sized>(&mut self, writer: &mut W, string: &str) -> io::Result<()>
    where
        W: io::Write,
    {
        enum StringKind {
            Unquoted,
            DoubleQuoted,
            TripleQuoted,
            MultilineTripleQuoted,
        }

        // Use regexes to determine the kind of string to write
        let kind: StringKind = {
            if RE_VALUE.is_match(string) || !RE_STR_NONE.is_match(string) {
                if !RE_STR_DOUBLE.is_match(string) {
                    if RE_HAS_NEWLINE.is_match(string) {
                        if !RE_STR_MULTILINE.is_match(string) {
                            StringKind::DoubleQuoted
                        } else {
                            StringKind::MultilineTripleQuoted
                        }
                    } else {
                        StringKind::TripleQuoted
                    }
                } else {
                    StringKind::DoubleQuoted
                }
            } else {
                StringKind::Unquoted
            }
        };


        match kind {
            // Write the string verbatim
            StringKind::Unquoted => {
                if self.in_object {
                    try!(writer.write_all(b" "));
                    self.in_object = false;
                }

                writer.write_all(string.as_bytes())
            }
            // Write the string as a JSON string
            StringKind::DoubleQuoted => {
                if self.in_object {
                    try!(writer.write_all(b" "));
                    self.in_object = false;
                }

                try!(writer.write_all(b"\""));

                let bytes = string.as_bytes();

                let mut start = 0;
                let mut skip = 0;

                for(i, &byte) in bytes.iter().enumerate() {
                    if skip > 1 {
                        skip -= 1;
                        start = i + 1;
                        continue;
                    }

                    let escape = ESCAPE[byte as usize];
                    if escape == 0 {
                        continue;
                    }

                    if start < i {
                        try!(writer.write_all(&bytes[start..i]));
                    }

                    try!(self.write_char_escape(writer, &bytes[i..], &mut skip));

                    start = i + 1;
                }

                if start != bytes.len() {
                    try!(writer.write_all(&bytes[start..]));
                }

                writer.write_all(b"\"")
            }
            // Write the string wrapped in triple-apostraphes with no escapes
            StringKind::TripleQuoted => {
                if self.in_object {
                    try!(writer.write_all(b" "));
                    self.in_object = false;
                }

                try!(writer.write_all(b"'''"));

                try!(writer.write_all(string.as_bytes()));

                writer.write_all(b"'''")
            }
            // Write the string at the proper indentation level
            StringKind::MultilineTripleQuoted => {
                if self.in_object {
                    self.current_indent += 1;

                    try!(writer.write_all(b"\n"));
                    try!(indent(writer, self.current_indent, self.indent));
                }
                try!(writer.write_all(b"'''\n"));


                let bytes = string.as_bytes();

                let mut start = 0;
                let mut has_content = false;
                let mut has_newline = false;

                for(i, &byte) in bytes.iter().enumerate() {
                    if byte == b'\n' {
                        if has_content {
                            try!(indent(writer, self.current_indent, self.indent));
                            try!(writer.write_all(&bytes[start..i + 1]));
                        } else {
                            try!(writer.write_all(b"\n"));
                        }

                        start = i + 1;
                        has_content = false;
                        has_newline = true;
                    } else if byte == b'\t' || byte == b' ' || byte == b'\r' {
                        has_content = true;
                    }
                }

                if start != bytes.len() {
                    try!(indent(writer, self.current_indent, self.indent));
                    try!(writer.write_all(&bytes[start..]));
                }
                if has_newline {
                    try!(writer.write_all(b"\n"));
                }

                try!(indent(writer, self.current_indent, self.indent));
                self.current_indent -= 1;
                writer.write_all(b"'''")
            }
        }
    }

    #[inline]
    fn write_member_string<W: ?Sized>(&mut self, writer: &mut W, string: &str) -> io::Result<()>
    where
        W: io::Write,
    {
        if RE_MEMBER_NONE.is_match(string) {
            // Unquoted
            writer.write_all(string.as_bytes())
        } else {
            // Double quoted
            try!(writer.write_all(b"\""));

            let bytes = string.as_bytes();

            let mut skip = 0;
            let mut start = 0;

            for(i, &byte) in bytes.iter().enumerate() {
                if skip > 1 {
                    skip -= 1;
                    start = i + 1;
                    continue;
                }

                let escape = ESCAPE[byte as usize];
                if escape == 0 {
                    continue;
                }

                if start < i {
                    try!(writer.write_all(&bytes[start..i]));
                }

                try!(self.write_char_escape(writer, &bytes[i..], &mut skip));

                start = i + 1;
            }

            if start != bytes.len() {
                try!(writer.write_all(&bytes[start..]));
            }

            writer.write_all(b"\"")
        }
    }

    #[inline]
    fn begin_array<W: ?Sized>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        self.next_bracket = Some(b'[');
        Ok(())
    }

    #[inline]
    fn end_array<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        if let Some(_) = self.next_bracket {
            self.next_bracket = None;

            if self.in_object {
                try!(writer.write_all(b" "));
            }

            writer.write_all(b"[]")
        } else {
            self.current_indent -= 1;

            if self.has_value {
                try!(writer.write_all(b"\n"));
                try!(indent(writer, self.current_indent, self.indent));
            }

            writer.write_all(b"]")
        }
    }

    #[inline]
    fn begin_array_value<W: ?Sized>(&mut self, writer: &mut W, _first: bool) -> io::Result<()>
    where
        W: io::Write,
    {
        try!(self.try_write_bracket(writer));

        self.in_object = false;
        try!(writer.write_all(b"\n"));
        try!(indent(writer, self.current_indent, self.indent));
        Ok(())
    }

    #[inline]
    fn end_array_value<W: ?Sized>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        self.has_value = true;
        Ok(())
    }

    #[inline]
    fn begin_object<W: ?Sized>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        self.next_bracket = Some(b'{');
        Ok(())
    }

    #[inline]
    fn end_object<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        if let Some(_) = self.next_bracket {
            self.next_bracket = None;

            if self.in_object {
                try!(writer.write_all(b" "));
            }

            writer.write_all(b"{}")
        } else {
            self.current_indent -= 1;

            if self.has_value {
                try!(writer.write_all(b"\n"));
                try!(indent(writer, self.current_indent, self.indent));
            }

            writer.write_all(b"}")
        }
    }

    #[inline]
    fn begin_object_key<W: ?Sized>(&mut self, writer: &mut W, _first: bool) -> io::Result<()>
    where
        W: io::Write,
    {
        try!(self.try_write_bracket(writer));
        try!(writer.write_all(b"\n"));
        indent(writer, self.current_indent, self.indent)
    }

    #[inline]
    fn begin_object_value<W: ?Sized>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        self.in_object = true;

        writer.write_all(b":")
    }

    #[inline]
    fn end_object_value<W: ?Sized>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        self.has_value = true;
        Ok(())
    }
}

fn format_escaped_str<W: ?Sized, F: ?Sized>(
    writer: &mut W,
    formatter: &mut F,
    value: &str,
) -> io::Result<()>
where
    W: io::Write,
    F: Formatter,
{
    formatter.write_string(writer, value)
}

const BB: u8 = b'b'; // \x08
const TT: u8 = b't'; // \x09
const NN: u8 = b'n'; // \x0A
const FF: u8 = b'f'; // \x0C
const RR: u8 = b'r'; // \x0D
const QU: u8 = b'"'; // \x22
const BS: u8 = b'\\'; // \x5C
const U: u8 = b'u'; // \x00...\x1F except the ones above
const UU: u8 = 1; // Any characters that may possibly be a unicode escape

// Lookup table of escape sequences. A value of b'x' at index i means that byte
// i is escaped as "\x" in JSON. A value of 0 means that byte i is not escaped.
#[cfg_attr(rustfmt, rustfmt_skip)]
static ESCAPE: [u8; 256] = [
    //  1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
    U,  U,  U,  U,  U,  U,  U,  U, BB, TT, NN,  U, FF, RR,  U,  U,  // 0
    U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  // 1
    0,  0, QU,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  // 2
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  // 3
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  // 4
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, BS,  0,  0,  0,  // 5
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  // 6
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  // 7
    UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, // 8
    UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, // 9
    UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, // A
    UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, // B
    UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, // C
    UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, // D
    UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, // E
    UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, // F
];

/// Serialize the given data structure as JSON into the IO stream.
///
/// # Errors
///
/// Serialization can fail if `T`'s implementation of `Serialize` decides to
/// fail, or if `T` contains a map with non-string keys.
#[inline]
pub fn to_writer<W, T: ?Sized>(writer: W, value: &T) -> Result<()>
where
    W: io::Write,
    T: ser::Serialize,
{
    let mut ser = Serializer::new(writer);
    try!(value.serialize(&mut ser));
    Ok(())
}

/// Serialize the given data structure as pretty-printed JSON into the IO
/// stream.
///
/// # Errors
///
/// Serialization can fail if `T`'s implementation of `Serialize` decides to
/// fail, or if `T` contains a map with non-string keys.
#[inline]
pub fn to_writer_pretty<W, T: ?Sized>(writer: W, value: &T) -> Result<()>
where
    W: io::Write,
    T: ser::Serialize,
{
    let mut ser = Serializer::pretty(writer);
    try!(value.serialize(&mut ser));
    Ok(())
}

/// Serialize the given data structure as a JSON byte vector.
///
/// # Errors
///
/// Serialization can fail if `T`'s implementation of `Serialize` decides to
/// fail, or if `T` contains a map with non-string keys.
#[inline]
pub fn to_vec<T: ?Sized>(value: &T) -> Result<Vec<u8>>
where
    T: ser::Serialize,
{
    let mut writer = Vec::with_capacity(128);
    try!(to_writer(&mut writer, value));
    Ok(writer)
}

/// Serialize the given data structure as a pretty-printed JSON byte vector.
///
/// # Errors
///
/// Serialization can fail if `T`'s implementation of `Serialize` decides to
/// fail, or if `T` contains a map with non-string keys.
#[inline]
pub fn to_vec_pretty<T: ?Sized>(value: &T) -> Result<Vec<u8>>
where
    T: ser::Serialize,
{
    let mut writer = Vec::with_capacity(128);
    try!(to_writer_pretty(&mut writer, value));
    Ok(writer)
}

/// Serialize the given data structure as a String of JSON.
///
/// # Errors
///
/// Serialization can fail if `T`'s implementation of `Serialize` decides to
/// fail, or if `T` contains a map with non-string keys.
#[inline]
pub fn to_string<T: ?Sized>(value: &T) -> Result<String>
where
    T: ser::Serialize,
{
    let vec = try!(to_vec(value));
    let string = unsafe {
        // We do not emit invalid UTF-8.
        String::from_utf8_unchecked(vec)
    };
    Ok(string)
}

/// Serialize the given data structure as a pretty-printed String of JSON.
///
/// # Errors
///
/// Serialization can fail if `T`'s implementation of `Serialize` decides to
/// fail, or if `T` contains a map with non-string keys.
#[inline]
pub fn to_string_pretty<T: ?Sized>(value: &T) -> Result<String>
where
    T: ser::Serialize,
{
    let vec = try!(to_vec_pretty(value));
    let string = unsafe {
        // We do not emit invalid UTF-8.
        String::from_utf8_unchecked(vec)
    };
    Ok(string)
}

fn indent<W: ?Sized>(wr: &mut W, n: usize, s: &[u8]) -> io::Result<()>
where
    W: io::Write,
{
    for _ in 0..n {
        try!(wr.write_all(s));
    }

    Ok(())
}
