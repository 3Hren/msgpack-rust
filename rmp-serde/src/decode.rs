use std::error;
use std::fmt::{self, Display, Formatter};
use std::io::{self, Read};
use std::str::{self, Utf8Error};

use byteorder::{self, ReadBytesExt};

use serde;
use serde::de::{self, Deserialize, DeserializeSeed, Visitor};

use rmp;
use rmp::Marker;
use rmp::decode::{MarkerReadError, DecodeStringError, ValueReadError, NumValueReadError,
                  read_array_len};

///
// TODO: Write docs.
#[derive(Debug)]
pub enum Error {
    InvalidMarkerRead(io::Error),
    InvalidDataRead(io::Error),
    /// The actual value type isn't equal with the expected one.
    TypeMismatch(Marker),
    /// Numeric cast failed due to out of range error.
    OutOfRange,
    LengthMismatch(u32),
    /// Uncategorized error.
    Uncategorized(String),
    Syntax(String),
    Utf8Error(Utf8Error),
    DepthLimitExceeded,
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "error while decoding value"
    }

    fn cause(&self) -> Option<&error::Error> {
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
        Error::Syntax(format!("{}", msg))
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        error::Error::description(self).fmt(fmt)
    }
}

impl From<MarkerReadError> for Error {
    fn from(err: MarkerReadError) -> Error {
        Error::InvalidMarkerRead(err.0)
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
            ValueReadError::TypeMismatch(marker)   => Error::TypeMismatch(marker),
            ValueReadError::InvalidMarkerRead(err) => Error::InvalidMarkerRead(err),
            ValueReadError::InvalidDataRead(err)   => Error::InvalidDataRead(err),
        }
    }
}

impl From<NumValueReadError> for Error {
    fn from(err: NumValueReadError) -> Error {
        match err {
            NumValueReadError::TypeMismatch(marker)   => Error::TypeMismatch(marker),
            NumValueReadError::InvalidMarkerRead(err) => Error::InvalidMarkerRead(err),
            NumValueReadError::InvalidDataRead(err)   => Error::InvalidDataRead(err),
            NumValueReadError::OutOfRange => Error::OutOfRange,
        }
    }
}

impl<'a> From<DecodeStringError<'a>> for Error {
    fn from(err: DecodeStringError) -> Error {
        match err {
            DecodeStringError::InvalidMarkerRead(err) => Error::InvalidMarkerRead(err),
            DecodeStringError::InvalidDataRead(..) => Error::Uncategorized("InvalidDataRead".to_string()),
            DecodeStringError::TypeMismatch(..) => Error::Uncategorized("TypeMismatch".to_string()),
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
pub struct Deserializer<R: Read> {
    rd: R,
    buf: Vec<u8>,
    // Cached marker in case of deserializing options.
    marker: Option<Marker>,
    depth: usize,
}

impl<R: Read> Deserializer<R> {
    /// Constructs a new deserializer by consuming the given reader.
    pub fn new(rd: R) -> Self {
        Deserializer {
            rd: rd,
            buf: Vec::with_capacity(128),
            marker: None,
            depth: 1024,
        }
    }

    /// Gets a reference to the underlying reader in this decoder.
    pub fn get_ref(&self) -> &R {
        &self.rd
    }

    /// Gets a mutable reference to the underlying reader in this decoder.
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.rd
    }

    /// Consumes this decoder returning the underlying reader.
    pub fn into_inner(self) -> R {
        self.rd
    }

    /// Changes the maximum nesting depth that is allowed
    pub fn set_max_depth(&mut self, depth: usize) {
        self.depth = depth;
    }

    fn read_str_data(&mut self, len: u32) -> Result<&str, Error> {
        self.buf.resize(len as usize, 0u8);

        self.rd.read_exact(&mut self.buf[..]).map_err(Error::InvalidDataRead)?;
        str::from_utf8(&self.buf).map_err(From::from)
    }

    fn read_bin_data(&mut self, len: u32) -> Result<&[u8], Error> {
        self.buf.resize(len as usize, 0u8);

        self.rd.read_exact(&mut self.buf[..len as usize]).map_err(Error::InvalidDataRead)?;
        Ok(&self.buf[..len as usize])
    }

    fn read_array<V>(&mut self, len: u32, visitor: V) -> Result<V::Value, Error>
        where V: Visitor
    {
        visitor.visit_seq(SeqVisitor::new(self, len))
    }

    fn read_map<V>(&mut self, len: u32, visitor: V) -> Result<V::Value, Error>
        where V: Visitor
    {
        visitor.visit_map(MapVisitor::new(self, len))
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

impl<'a, R: Read> serde::Deserializer for &'a mut Deserializer<R> {
    type Error = Error;

    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor
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
            Marker::FixStr(len) => visitor.visit_str(self.read_str_data(len as u32)?),
            Marker::Str8 => {
                let len = read_u8(&mut self.rd)?;
                visitor.visit_str(self.read_str_data(len as u32)?)
            }
            Marker::Str16 => {
                let len = read_u16(&mut self.rd)?;
                visitor.visit_str(self.read_str_data(len as u32)?)
            }
            Marker::Str32 => {
                let len = read_u32(&mut self.rd)?;
                visitor.visit_str(self.read_str_data(len)?)
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
                visitor.visit_bytes(self.read_bin_data(len as u32)?)
            }
            Marker::Bin16 => {
                let len = read_u16(&mut self.rd)?;
                visitor.visit_bytes(self.read_bin_data(len as u32)?)
            }
            Marker::Bin32 => {
                let len = read_u32(&mut self.rd)?;
                visitor.visit_bytes(self.read_bin_data(len)?)
            }
            Marker::Reserved => Err(Error::TypeMismatch(Marker::Reserved)),
            // TODO: Make something with exts.
            marker => Err(Error::TypeMismatch(marker)),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor
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
        where V: Visitor
    {
        match read_array_len(&mut self.rd)? {
            2 => visitor.visit_enum(VariantVisitor::new(self)),
            n => Err(Error::LengthMismatch(n as u32)),
        }
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Error>
        where V: Visitor
    {
        match read_array_len(&mut self.rd)? {
            1 => visitor.visit_newtype_struct(self),
            n => Err(Error::LengthMismatch(n as u32)),
        }
    }

    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char
        str string bytes byte_buf unit unit_struct seq seq_fixed_size map
        tuple_struct struct struct_field tuple
        ignored_any
    }
}

struct SeqVisitor<'a, R: Read + 'a> {
    de: &'a mut Deserializer<R>,
    len: u32,
    nleft: u32,
}

impl<'a, R: Read + 'a> SeqVisitor<'a, R> {
    fn new(de: &'a mut Deserializer<R>, len: u32) -> Self {
        SeqVisitor {
            de: de,
            len: len,
            nleft: len,
        }
    }
}

impl<'a, R: Read + 'a> de::SeqVisitor for SeqVisitor<'a, R> {
    type Error = Error;

    fn visit_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
        where T: DeserializeSeed
    {
        if self.nleft > 0 {
            self.nleft -= 1;
            Ok(Some(seed.deserialize(&mut *self.de)?))
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len as usize, Some(self.len as usize))
    }
}

struct MapVisitor<'a, R: Read + 'a> {
    de: &'a mut Deserializer<R>,
    len: u32,
    nleft: u32,
}

impl<'a, R: Read + 'a> MapVisitor<'a, R> {
    fn new(de: &'a mut Deserializer<R>, len: u32) -> Self {
        MapVisitor {
            de: de,
            len: len,
            nleft: len,
        }
    }
}

impl<'a, R: Read + 'a> de::MapVisitor for MapVisitor<'a, R> {
    type Error = Error;

    fn visit_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
        where K: DeserializeSeed
    {
        if self.nleft > 0 {
            self.nleft -= 1;
            Ok(Some(seed.deserialize(&mut *self.de)?))
        } else {
            Ok(None)
        }
    }

    fn visit_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
        where V: DeserializeSeed
    {
        Ok(seed.deserialize(&mut *self.de)?)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len as usize, Some(self.len as usize))
    }
}

/// Default variant visitor.
///
/// # Note
///
/// We use default behaviour for new type, which decodes enums with a single value as a tuple.
pub struct VariantVisitor<'a, R: Read + 'a> {
    de: &'a mut Deserializer<R>,
}

impl<'a, R: Read + 'a> VariantVisitor<'a, R> {
    pub fn new(de: &'a mut Deserializer<R>) -> Self {
        VariantVisitor {
            de: de,
        }
    }
}

impl<'a, R: Read> de::EnumVisitor for VariantVisitor<'a, R> {
    type Error = Error;
    type Variant = Self;

    fn visit_variant_seed<V>(self, seed: V) -> Result<(V::Value, Self), Error>
        where V: de::DeserializeSeed,
    {
        use serde::de::value::ValueDeserializer;

        let idx: u32 = serde::Deserialize::deserialize(&mut *self.de)?;
        let val: Result<_, Error> = seed.deserialize(idx.into_deserializer());
        Ok((val?, self))
    }
}

impl<'a, R: Read> de::VariantVisitor for VariantVisitor<'a, R> {
    type Error = Error;

    fn visit_unit(self) -> Result<(), Error> {
        type T = ();
        T::deserialize(self.de)
    }

    fn visit_newtype_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
        where T: DeserializeSeed
    {
        read_array_len(self.de.get_mut())?;
        seed.deserialize(self.de)
    }

    fn visit_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Error>
        where V: Visitor
    {
        de::Deserializer::deserialize_tuple(self.de, len, visitor)
    }

    fn visit_struct<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value, Error>
        where V: Visitor,
    {
        de::Deserializer::deserialize_tuple(self.de, fields.len(), visitor)
    }
}
