//! Generic MessagePack deserialization.

use std::error;
use std::fmt::{self, Display, Formatter};
use std::io::{self, Cursor, ErrorKind, Read};
use std::str::{self, Utf8Error};

use byteorder::{self, ReadBytesExt};

use serde;
use serde::de::{self, Deserialize, DeserializeOwned, DeserializeSeed, Visitor};

use rmp;
use rmp::Marker;
use rmp::decode::{self, MarkerReadError, DecodeStringError, ValueReadError, NumValueReadError};

use crate::MSGPACK_EXT_STRUCT_NAME;

/// Enum representing errors that can occur while decoding MessagePack data.
#[derive(Debug)]
pub enum Error {
    /// The enclosed I/O error occurred while trying to read a MessagePack
    /// marker.
    InvalidMarkerRead(io::Error),
    /// The enclosed I/O error occurred while trying to read the encoded
    /// MessagePack data.
    InvalidDataRead(io::Error),
    /// A mismatch occurred between the decoded and expected value types.
    TypeMismatch(Marker),
    /// A numeric cast failed due to an out-of-range error.
    OutOfRange,
    /// A decoded array did not have the enclosed expected length.
    LengthMismatch(u32),
    /// An otherwise uncategorized error occurred. See the enclosed `String` for
    /// details.
    Uncategorized(String),
    /// A general error occurred while deserializing the expected type. See the
    /// enclosed `String` for details.
    Syntax(String),
    /// An encoded string could not be parsed as UTF-8.
    Utf8Error(Utf8Error),
    /// The depth limit was exceeded; not currently used.
    DepthLimitExceeded,
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "error while decoding value"
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            Error::TypeMismatch(..) => None,
            Error::InvalidMarkerRead(ref err) => Some(err),
            Error::InvalidDataRead(ref err) => Some(err),
            Error::LengthMismatch(..) => None,
            Error::OutOfRange => None,
            Error::Uncategorized(..) => None,
            Error::Syntax(..) => None,
            Error::Utf8Error(ref err) => Some(err),
            Error::DepthLimitExceeded => None,
        }
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Syntax(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match *self {
            Error::InvalidMarkerRead(ref err) => write!(fmt, "IO error while reading marker: {}", err),
            Error::InvalidDataRead(ref err) => write!(fmt, "IO error while reading data: {}", err),
            Error::TypeMismatch(ref actual_marker) => {
                write!(fmt, "wrong msgpack marker {:?}", actual_marker)
            }
            Error::OutOfRange => fmt.write_str("numeric cast found out of range"),
            Error::LengthMismatch(expected_length) => write!(
                fmt,
                "array had incorrect length, expected {}",
                expected_length
            ),
            Error::Uncategorized(ref msg) => write!(fmt, "uncategorized error: {}", msg),
            Error::Syntax(ref msg) => fmt.write_str(msg),
            Error::Utf8Error(ref err) => write!(fmt, "string found to be invalid utf8: {}", err),
            Error::DepthLimitExceeded => fmt.write_str("depth limit exceeded"),
        }
    }
}

impl From<MarkerReadError> for Error {
    fn from(err: MarkerReadError) -> Error {
        match err {
            MarkerReadError(err) => Error::InvalidMarkerRead(err),
        }
    }
}

impl From<Utf8Error> for Error {
    fn from(err: Utf8Error) -> Error {
        Error::Utf8Error(err)
    }
}

impl From<ValueReadError> for Error {
    fn from(err: ValueReadError) -> Error {
        match err {
            ValueReadError::TypeMismatch(marker) => Error::TypeMismatch(marker),
            ValueReadError::InvalidMarkerRead(err) => Error::InvalidMarkerRead(err),
            ValueReadError::InvalidDataRead(err) => Error::InvalidDataRead(err),
        }
    }
}

impl From<NumValueReadError> for Error {
    fn from(err: NumValueReadError) -> Error {
        match err {
            NumValueReadError::TypeMismatch(marker) => Error::TypeMismatch(marker),
            NumValueReadError::InvalidMarkerRead(err) => Error::InvalidMarkerRead(err),
            NumValueReadError::InvalidDataRead(err) => Error::InvalidDataRead(err),
            NumValueReadError::OutOfRange => Error::OutOfRange,
        }
    }
}

impl<'a> From<DecodeStringError<'a>> for Error {
    fn from(err: DecodeStringError<'_>) -> Error {
        match err {
            DecodeStringError::InvalidMarkerRead(err) => Error::InvalidMarkerRead(err),
            DecodeStringError::InvalidDataRead(err) => Error::InvalidDataRead(err),
            DecodeStringError::TypeMismatch(marker) => Error::TypeMismatch(marker),
            DecodeStringError::BufferSizeTooSmall(..) => Error::Uncategorized("BufferSizeTooSmall".to_string()),
            DecodeStringError::InvalidUtf8(..) => Error::Uncategorized("InvalidUtf8".to_string()),
        }
    }
}

/// A Deserializer that reads bytes from a buffer.
///
/// # Note
///
/// All instances of `ErrorKind::Interrupted` are handled by this function and the underlying
/// operation is retried.
#[derive(Debug)]
pub struct Deserializer<R> {
    rd: R,
    marker: Option<Marker>,
    depth: usize,
}

impl<R: Read> Deserializer<ReadReader<R>> {
    #[doc(hidden)]
    #[deprecated(note="use `Deserializer::new` instead")]
    pub fn from_read(rd: R) -> Self {
        Self::new(rd)
    }

    /// Constructs a new `Deserializer` by consuming the given reader.
    pub fn new(rd: R) -> Self {
        Self {
            rd: ReadReader::new(rd),
            // Cached marker in case of deserializing optional values.
            marker: None,
            depth: 1024,
        }
    }

    /// Gets a reference to the underlying reader in this decoder.
    pub fn get_ref(&self) -> &R {
        &self.rd.rd
    }

    /// Gets a mutable reference to the underlying reader in this decoder.
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.rd.rd
    }

    /// Consumes this deserializer returning the underlying reader.
    pub fn into_inner(self) -> R {
        self.rd.rd
    }
}

impl<R: AsRef<[u8]>> Deserializer<ReadReader<Cursor<R>>> {
    /// Returns the current position of this deserializer, i.e. how many bytes were read.
    pub fn position(&self) -> u64 {
        self.rd.rd.position()
    }
}

impl<'de, R> Deserializer<ReadRefReader<'de, R>>
    where
        R: AsRef<[u8]> + ?Sized
{
    /// Constructs a new `Deserializer` from the given byte slice.
    pub fn from_read_ref(rd: &'de R) -> Self {
        Deserializer {
            rd: ReadRefReader::new(rd),
            marker: None,
            depth: 1024,
        }
    }

    /// Gets a reference to the underlying reader in this decoder.
    pub fn get_ref(&self) -> &R {
        self.rd.rd
    }
}

impl<'de, R: ReadSlice<'de>> Deserializer<R> {
    /// Changes the maximum nesting depth that is allowed
    pub fn set_max_depth(&mut self, depth: usize) {
        self.depth = depth;
    }

    fn read_str_data<V>(&mut self, len: u32, visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        match self.read_bin_data(len as u32)? {
            Reference::Borrowed(buf) => {
                match str::from_utf8(buf) {
                    Ok(s) => visitor.visit_borrowed_str(s),
                    Err(err) => {
                        // Allow to unpack invalid UTF-8 bytes into a byte array.
                        match visitor.visit_borrowed_bytes::<Error>(buf) {
                            Ok(buf) => Ok(buf),
                            Err(..) => Err(Error::Utf8Error(err)),
                        }
                    }
                }
            }
            Reference::Copied(buf) => {
                match str::from_utf8(buf) {
                    Ok(s) => visitor.visit_str(s),
                    Err(err) => {
                        // Allow to unpack invalid UTF-8 bytes into a byte array.
                        match visitor.visit_bytes::<Error>(buf) {
                            Ok(buf) => Ok(buf),
                            Err(..) => Err(Error::Utf8Error(err)),
                        }
                    }
                }
            }
        }
    }

    fn read_bin_data<'a>(&'a mut self, len: u32) -> Result<Reference<'de,'a, [u8]>, Error> {
        self.rd.read_slice(len as usize).map_err(Error::InvalidDataRead)
    }

    fn read_ext<V>(&mut self, len: u32, visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        let ext_de = ExtDeserializer::new(self, len);
        visitor.visit_newtype_struct(ext_de)
    }

    fn read_array<V>(&mut self, len: u32, visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        visitor.visit_seq(SeqAccess::new(self, len as usize))
    }

    fn read_map<V>(&mut self, len: u32, visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        visitor.visit_map(MapAccess::new(self, len as usize))
    }

    fn read_bytes<V>(&mut self, len: u32, visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        match self.read_bin_data(len)? {
            Reference::Borrowed(buf) => visitor.visit_borrowed_bytes(buf),
            Reference::Copied(buf) => visitor.visit_bytes(buf),
        }
    }
}

fn read_u8<R: Read>(rd: &mut R) -> Result<u8, Error> {
    rd.read_u8().map_err(Error::InvalidDataRead)
}

fn read_u16<R: Read>(rd: &mut R) -> Result<u16, Error> {
    rd.read_u16::<byteorder::BigEndian>().map_err(Error::InvalidDataRead)
}

fn read_u32<R: Read>(rd: &mut R) -> Result<u32, Error> {
    rd.read_u32::<byteorder::BigEndian>().map_err(Error::InvalidDataRead)
}

#[derive(Debug)]
enum ExtDeserializerState {
    New,
    ReadTag,
    ReadBinary
}

#[derive(Debug)]
struct ExtDeserializer<'a, R> {
    rd: &'a mut R,
    len: u32,
    state: ExtDeserializerState
}

impl<'de, 'a, R: ReadSlice<'de> + 'a> ExtDeserializer<'a, R> {
    fn new(d: &'a mut Deserializer<R>, len: u32) -> Self {
        ExtDeserializer {
            rd: &mut d.rd,
            len,
            state: ExtDeserializerState::New,
        }
    }
}

impl<'de, 'a, R: ReadSlice<'de> + 'a> de::Deserializer<'de> for ExtDeserializer<'a, R> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        visitor.visit_seq(self)
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq bytes byte_buf map unit_struct newtype_struct
        struct identifier tuple enum ignored_any tuple_struct
    }
}

impl<'de, 'a, R: ReadSlice<'de> + 'a> de::SeqAccess<'de> for ExtDeserializer<'a, R> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.state {
            ExtDeserializerState::New | ExtDeserializerState::ReadTag => Ok(Some(seed.deserialize(self)?)),
            ExtDeserializerState::ReadBinary => Ok(None)
        }
    }
}


/// Deserializer for Ext SeqAccess
impl<'de, 'a, R: ReadSlice<'de> + 'a> de::Deserializer<'de> for &mut ExtDeserializer<'a, R> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        match self.state {
            ExtDeserializerState::New => {
                let tag = rmp::decode::read_data_i8(self.rd)?;
                self.state = ExtDeserializerState::ReadTag;
                visitor.visit_i8(tag)
            }
            ExtDeserializerState::ReadTag => {
                let data = self.rd.read_slice(self.len as usize).map_err(Error::InvalidDataRead)?;
                self.state = ExtDeserializerState::ReadBinary;
                match data {
                    Reference::Borrowed(bytes) => {
                        visitor.visit_borrowed_bytes(bytes)
                    }
                    Reference::Copied(bytes) => {
                        visitor.visit_bytes(bytes)
                    }
                }
            }
            ExtDeserializerState::ReadBinary => unreachable!()
        }
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq bytes byte_buf map unit_struct newtype_struct
        tuple_struct struct identifier tuple enum ignored_any
    }
}

impl<'de, 'a, R: ReadSlice<'de>> serde::Deserializer<'de> for &'a mut Deserializer<R> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        let marker = match self.marker.take() {
            Some(marker) => marker,
            None => rmp::decode::read_marker(&mut self.rd)?,
        };

        match marker {
            Marker::Null => visitor.visit_unit(),
            Marker::True => visitor.visit_bool(true),
            Marker::False => visitor.visit_bool(false),
            Marker::FixPos(val) => visitor.visit_u8(val),
            Marker::FixNeg(val) => visitor.visit_i8(val),
            Marker::U8 => visitor.visit_u8(rmp::decode::read_data_u8(&mut self.rd)?),
            Marker::U16 => visitor.visit_u16(rmp::decode::read_data_u16(&mut self.rd)?),
            Marker::U32 => visitor.visit_u32(rmp::decode::read_data_u32(&mut self.rd)?),
            Marker::U64 => visitor.visit_u64(rmp::decode::read_data_u64(&mut self.rd)?),
            Marker::I8 => visitor.visit_i8(rmp::decode::read_data_i8(&mut self.rd)?),
            Marker::I16 => visitor.visit_i16(rmp::decode::read_data_i16(&mut self.rd)?),
            Marker::I32 => visitor.visit_i32(rmp::decode::read_data_i32(&mut self.rd)?),
            Marker::I64 => visitor.visit_i64(rmp::decode::read_data_i64(&mut self.rd)?),
            Marker::F32 => visitor.visit_f32(rmp::decode::read_data_f32(&mut self.rd)?),
            Marker::F64 => visitor.visit_f64(rmp::decode::read_data_f64(&mut self.rd)?),
            Marker::FixStr(len) => {
                self.read_str_data(len as u32, visitor)
            }
            Marker::Str8 => {
                let len = read_u8(&mut self.rd)?;
                self.read_str_data(len as u32, visitor)
            }
            Marker::Str16 => {
                let len = read_u16(&mut self.rd)?;
                self.read_str_data(len as u32, visitor)
            }
            Marker::Str32 => {
                let len = read_u32(&mut self.rd)?;
                self.read_str_data(len as u32, visitor)
            }
            Marker::FixArray(len) => {
                self.read_array(len as u32, visitor)
            }
            Marker::Array16 => {
                let len = read_u16(&mut self.rd)?;
                self.read_array(len as u32, visitor)
            }
            Marker::Array32 => {
                let len = read_u32(&mut self.rd)?;
                self.read_array(len, visitor)
            }
            Marker::FixMap(len) => {
                self.read_map(len as u32, visitor)
            }
            Marker::Map16 => {
                let len = read_u16(&mut self.rd)?;
                self.read_map(len as u32, visitor)
            }
            Marker::Map32 => {
                let len = read_u32(&mut self.rd)?;
                self.read_map(len, visitor)
            }
            Marker::Bin8 => {
                let len = read_u8(&mut self.rd)?;
                self.read_bytes(len as u32, visitor)
            }
            Marker::Bin16 => {
                let len = read_u16(&mut self.rd)?;
                self.read_bytes(len as u32, visitor)
            }
            Marker::Bin32 => {
                let len = read_u32(&mut self.rd)?;
                self.read_bytes(len, visitor)
            }
            Marker::FixExt1 => {
                let len = 1 as usize;
                self.read_ext(len as u32, visitor)
            }
            Marker::FixExt2 => {
                let len = 2 as usize;
                self.read_ext(len as u32, visitor)
            }
            Marker::FixExt4 => {
                let len = 4 as usize;
                self.read_ext(len as u32, visitor)
            }
            Marker::FixExt8 => {
                let len = 8 as usize;
                self.read_ext(len as u32, visitor)
            }
            Marker::FixExt16 => {
                let len = 16 as usize;
                self.read_ext(len as u32, visitor)
            }
            Marker::Ext8 => {
                let len = read_u8(&mut self.rd)?;
                self.read_ext(len as u32, visitor)
            }
            Marker::Ext16 => {
                let len = read_u16(&mut self.rd)?;
                self.read_ext(len as u32, visitor)
            }
            Marker::Ext32 => {
                let len = read_u32(&mut self.rd)?;
                self.read_ext(len as u32, visitor)
            }
            Marker::Reserved => Err(Error::TypeMismatch(Marker::Reserved)),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        let marker = rmp::decode::read_marker(&mut self.rd)?;

        if marker == Marker::Null {
            visitor.visit_none()
        } else {
            self.marker = Some(marker);
            visitor.visit_some(self)
        }
    }

    fn deserialize_enum<V>(self, _name: &str, _variants: &[&str], visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        let marker = match self.marker.take() {
            Some(marker) => marker,
            None => rmp::decode::read_marker(&mut self.rd)?,
        };

        match rmp::decode::marker_to_len(&mut self.rd, marker)? {
            1 => visitor.visit_enum(VariantAccess::new(self)),
            n => Err(Error::LengthMismatch(n as u32)),
        }
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        if name == MSGPACK_EXT_STRUCT_NAME {
            let marker = match self.marker.take() {
                Some(marker) => marker,
                None => rmp::decode::read_marker(&mut self.rd)?,
            };
            let len = match marker {
                Marker::FixExt1 => {
                    1 as u32
                }
                Marker::FixExt2 => {
                    2 as u32
                }
                Marker::FixExt4 => {
                    4 as u32
                }
                Marker::FixExt8 => {
                    8 as u32
                }
                Marker::FixExt16 => {
                    16 as u32
                }
                Marker::Ext8 => {
                    read_u8(&mut self.rd)? as u32
                }
                Marker::Ext16 => {
                    read_u16(&mut self.rd)? as u32
                }
                Marker::Ext32 => {
                    read_u32(&mut self.rd)? as u32
                }
                _ => {
                    return Err(Error::TypeMismatch(marker))
                }
            };

            let ext_de = ExtDeserializer::new(self, len);
            return visitor.visit_newtype_struct(ext_de);
        }

        visitor.visit_newtype_struct(self)
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        // We need to special case this so that [] is treated as a unit struct when asked for,
        // but as a sequence otherwise. This is because we serialize unit structs as [] rather
        // than as 'nil'.
        let marker = match self.marker.take() {
            Some(marker) => marker,
            None => rmp::decode::read_marker(&mut self.rd)?,
        };
        match marker {
            Marker::Null | Marker::FixArray(0) => visitor.visit_unit(),
            marker => {
                self.marker = Some(marker);
                self.deserialize_any(visitor)
            }
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        // This special case allows us to decode some integer types as floats when safe, and
        // asked for.
        //
        // This allows interoperability with msgpack-lite, which performs an optimization of
        // storing rounded floats as integers.
        let marker = match self.marker.take() {
            Some(marker) => marker,
            None => rmp::decode::read_marker(&mut self.rd)?,
        };
        match marker {
            Marker::U8 => visitor.visit_f32(rmp::decode::read_data_u8(&mut self.rd)?.into()),
            Marker::U16 => visitor.visit_f32(rmp::decode::read_data_u16(&mut self.rd)?.into()),
            Marker::I8 => visitor.visit_f32(rmp::decode::read_data_i8(&mut self.rd)?.into()),
            Marker::I16 => visitor.visit_f32(rmp::decode::read_data_i16(&mut self.rd)?.into()),
            Marker::F32 => visitor.visit_f32(rmp::decode::read_data_f32(&mut self.rd)?),
            marker => {
                self.marker = Some(marker);
                self.deserialize_any(visitor)
            }
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        // This special case allows us to decode some integer types as floats when safe, and
        // asked for. This is here to be consistent with 'f32'.
        let marker = match self.marker.take() {
            Some(marker) => marker,
            None => rmp::decode::read_marker(&mut self.rd)?,
        };
        match marker {
            Marker::U8 => visitor.visit_f64(rmp::decode::read_data_u8(&mut self.rd)?.into()),
            Marker::U16 => visitor.visit_f64(rmp::decode::read_data_u16(&mut self.rd)?.into()),
            Marker::U32 => visitor.visit_f64(rmp::decode::read_data_u32(&mut self.rd)?.into()),
            Marker::I8 => visitor.visit_f64(rmp::decode::read_data_i8(&mut self.rd)?.into()),
            Marker::I16 => visitor.visit_f64(rmp::decode::read_data_i16(&mut self.rd)?.into()),
            Marker::I32 => visitor.visit_f64(rmp::decode::read_data_i32(&mut self.rd)?.into()),
            Marker::F32 => visitor.visit_f64(rmp::decode::read_data_f32(&mut self.rd)?.into()),
            Marker::F64 => visitor.visit_f64(rmp::decode::read_data_f64(&mut self.rd)?),
            marker => {
                self.marker = Some(marker);
                self.deserialize_any(visitor)
            }
        }
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 char
        str string bytes byte_buf unit seq map
        struct identifier tuple tuple_struct
        ignored_any
    }
}

struct SeqAccess<'a, R> {
    de: &'a mut Deserializer<R>,
    left: usize,
}

impl<'a, R: 'a> SeqAccess<'a, R> {
    fn new(de: &'a mut Deserializer<R>, len: usize) -> Self {
        SeqAccess {
            de,
            left: len,
        }
    }
}

impl<'de, 'a, R: ReadSlice<'de> + 'a> de::SeqAccess<'de> for SeqAccess<'a, R> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
        where T: DeserializeSeed<'de>
    {
        if self.left > 0 {
            self.left -= 1;
            Ok(Some(seed.deserialize(&mut *self.de)?))
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.left)
    }
}

struct MapAccess<'a, R> {
    de: &'a mut Deserializer<R>,
    left: usize,
}

impl<'a, R: 'a> MapAccess<'a, R> {
    fn new(de: &'a mut Deserializer<R>, len: usize) -> Self {
        MapAccess {
            de,
            left: len,
        }
    }
}

impl<'de, 'a, R: ReadSlice<'de> + 'a> de::MapAccess<'de> for MapAccess<'a, R> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
        where K: DeserializeSeed<'de>
    {
        if self.left > 0 {
            self.left -= 1;
            Ok(Some(seed.deserialize(&mut *self.de)?))
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
        where V: DeserializeSeed<'de>
    {
        Ok(seed.deserialize(&mut *self.de)?)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.left)
    }
}

struct VariantAccess<'a, R> {
    de: &'a mut Deserializer<R>,
}

impl<'a, R: 'a> VariantAccess<'a, R> {
    pub fn new(de: &'a mut Deserializer<R>) -> Self {
        VariantAccess {
            de,
        }
    }
}

impl<'de, 'a, R: ReadSlice<'de>> de::EnumAccess<'de> for VariantAccess<'a, R> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self), Error>
        where V: de::DeserializeSeed<'de>,
    {
        Ok((seed.deserialize(&mut *self.de)?, self))
    }
}

impl<'de, 'a, R: ReadSlice<'de>> de::VariantAccess<'de> for VariantAccess<'a, R> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        decode::read_nil(&mut self.de.rd)?;
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
        where T: DeserializeSeed<'de>
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        de::Deserializer::deserialize_tuple(self.de, len, visitor)
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        de::Deserializer::deserialize_tuple(self.de, fields.len(), visitor)
    }
}

/// Unification of both borrowed and non-borrowed reference types.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Reference<'b, 'c, T: ?Sized + 'static> {
    /// The reference is pointed at data that was borrowed.
    Borrowed(&'b T),
    /// The reference is pointed at data that was copied.
    Copied(&'c T),
}

/// Extends the `Read` trait by allowing to read slices directly by borrowing bytes.
///
/// Used to allow zero-copy reading.
pub trait ReadSlice<'de>: Read {
    /// Reads the exact number of bytes from the underlying byte-array.
    fn read_slice<'a>(&'a mut self, len: usize) -> Result<Reference<'de, 'a, [u8]>, io::Error>;
}

/// Owned reader wrapper.
#[derive(Debug)]
pub struct ReadReader<R: Read> {
    rd: R,
    buf: Vec<u8>,
}

impl<R: Read> ReadReader<R> {
    fn new(rd: R) -> Self {
        ReadReader {
            rd,
            buf: Vec::with_capacity(128),
        }
    }
}

impl<'de, R: Read> ReadSlice<'de> for ReadReader<R> {
    #[inline]
    fn read_slice<'a>(&'a mut self, len: usize) -> Result<Reference<'de, 'a, [u8]>, io::Error> {
        self.buf.resize(len, 0u8);
        self.rd.read_exact(&mut self.buf[..])?;

        Ok(Reference::Copied(&self.buf[..]))
    }
}

impl<R: Read> Read for ReadReader<R> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.rd.read(buf)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.rd.read_exact(buf)
    }
}

/// Borrowed reader wrapper.
#[derive(Debug)]
pub struct ReadRefReader<'a, R: ?Sized> {
    rd: &'a R,
    buf: &'a [u8],
}

impl<'a, T: AsRef<[u8]> + ?Sized> ReadRefReader<'a, T> {
    fn new(rd: &'a T) -> Self {
        Self {
            rd,
            buf: rd.as_ref()
        }
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> Read for ReadRefReader<'a, T> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        self.buf.read(buf)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), io::Error> {
        self.buf.read_exact(buf)
    }
}

impl<'de, T: AsRef<[u8]> + ?Sized> ReadSlice<'de> for ReadRefReader<'de, T> {
    #[inline]
    fn read_slice<'a>(&'a mut self, len: usize) -> Result<Reference<'de, 'a, [u8]>, io::Error> {
        if len > self.buf.len() {
            return Err(ErrorKind::UnexpectedEof.into());
        }
        let (a, b) = self.buf.split_at(len);
        self.buf = b;
        Ok(Reference::Borrowed(a))
    }
}

#[test]
fn test_as_ref_reader() {
    let buf = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut rd = ReadRefReader::new(&buf);

    assert_eq!(rd.read_slice(1).unwrap(), Reference::Borrowed(&[0][..]));
    assert_eq!(rd.read_slice(6).unwrap(), Reference::Borrowed(&[1, 2, 3, 4, 5, 6][..]));
    assert!(rd.read_slice(5).is_err());
    assert_eq!(rd.read_slice(4).unwrap(), Reference::Borrowed(&[7, 8, 9, 10][..]));
}

/// Deserialize an instance of type `T` from an I/O stream of MessagePack.
///
/// # Errors
///
/// This conversion can fail if the structure of the Value does not match the structure expected
/// by `T`. It can also fail if the structure is correct but `T`'s implementation of `Deserialize`
/// decides that something is wrong with the data, for example required struct fields are missing.
pub fn from_read<R, T>(rd: R) -> Result<T, Error>
where R: Read,
      T: DeserializeOwned
{
    Deserialize::deserialize(&mut Deserializer::new(rd))
}

/// Deserializes a byte slice into the desired type.
///
/// Currently deprecated, use more generic `from_read_ref` instead.
#[doc(hidden)]
pub fn from_slice<'a, T>(input: &'a [u8]) -> Result<T, Error>
where
    T: Deserialize<'a>
{
    from_read_ref(input)
}

/// Deserialize an instance of type `T` from a reference I/O reader of MessagePack.
///
/// Deserialization will be performed in zero-copy manner whenever it is possible, borrowing the
/// data from the reader itself. For example, strings and byte-arrays won't be not copied.
///
/// # Errors
///
/// This conversion can fail if the structure of the Value does not match the structure expected
/// by `T`. It can also fail if the structure is correct but `T`'s implementation of `Deserialize`
/// decides that something is wrong with the data, for example required struct fields are missing.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate serde_derive;
///
/// // Encoded `["Bobby", 8]`.
/// let buf = [0x92, 0xa5, 0x42, 0x6f, 0x62, 0x62, 0x79, 0x8];
///
/// #[derive(Debug, Deserialize, PartialEq)]
/// struct Dog<'a> {
///    name: &'a str,
///    age: u8,
/// }
///
/// assert_eq!(Dog { name: "Bobby", age: 8 }, rmp_serde::from_read_ref(&buf).unwrap());
/// ```
#[inline]
pub fn from_read_ref<'a, R, T>(rd: &'a R) -> Result<T, Error>
where
    R: AsRef<[u8]> + ?Sized,
    T: Deserialize<'a>,
{
    let mut de = Deserializer::from_read_ref(rd);
    Deserialize::deserialize(&mut de)
}
