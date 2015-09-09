use std::convert::From;
use std::fmt;
use std::io::Read;
use std::result;

use serde;

use rmp::Marker;
use rmp::decode::{
    DecodeStringError,
    FixedValueReadError,
    MarkerReadError,
    ReadError,
    ValueReadError,
    read_numeric_data,
    read_str_data,
    read_marker,
    read_full,
};

/// Unstable: docs; incomplete
#[derive(Debug)]
pub enum Error {
    /// The actual value type isn't equal with the expected one.
    TypeMismatch(Marker),
    InvalidMarkerRead(ReadError),
    InvalidDataRead(ReadError),
    LengthMismatch(u32),
    /// Uncategorized error.
    Uncategorized(String),
    Syntax(String),
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str { "error while decoding value" }

    fn cause(&self) -> Option<&::std::error::Error> {
        use self::Error::*;
        match *self {
            TypeMismatch(_) => None,
            InvalidMarkerRead(ref err) => Some(err),
            InvalidDataRead(ref err) => Some(err),
            LengthMismatch(_) => None,
            Uncategorized(_) => None,
            Syntax(_) => None,
        }
    }
}

impl serde::de::Error for Error {
    fn syntax(msg: &str) -> Error {
        Error::Syntax(format!("syntax error: {}", msg))
    }

    fn length_mismatch(len: usize) -> Error {
        Error::LengthMismatch(len as u32)
    }

    fn type_mismatch(ty: serde::de::Type) -> Error {
        match ty {
            serde::de::Type::Bool => Error::TypeMismatch(Marker::True),
            serde::de::Type::Usize => Error::TypeMismatch(Marker::FixPos(0)),
            serde::de::Type::U8 => Error::TypeMismatch(Marker::U8),
            serde::de::Type::U16 => Error::TypeMismatch(Marker::U16),
            serde::de::Type::U32 => Error::TypeMismatch(Marker::U32),
            serde::de::Type::U64 => Error::TypeMismatch(Marker::U64),
            serde::de::Type::Isize => Error::TypeMismatch(Marker::FixNeg(0)),
            serde::de::Type::I8 => Error::TypeMismatch(Marker::I8),
            serde::de::Type::I16 => Error::TypeMismatch(Marker::I16),
            serde::de::Type::I32 => Error::TypeMismatch(Marker::I32),
            serde::de::Type::I64 => Error::TypeMismatch(Marker::I64),
            serde::de::Type::F32 => Error::TypeMismatch(Marker::F32),
            serde::de::Type::F64 => Error::TypeMismatch(Marker::F64),
            serde::de::Type::Char => Error::TypeMismatch(Marker::Str32),
            serde::de::Type::Str => Error::TypeMismatch(Marker::Str32),
            serde::de::Type::String => Error::TypeMismatch(Marker::Str32),
            serde::de::Type::Unit => Error::TypeMismatch(Marker::Null),
            serde::de::Type::Option => Error::TypeMismatch(Marker::True),
            serde::de::Type::Seq => Error::TypeMismatch(Marker::Array32),
            serde::de::Type::Map => Error::TypeMismatch(Marker::Map32),
            serde::de::Type::UnitStruct => Error::TypeMismatch(Marker::Null),
            serde::de::Type::NewtypeStruct => Error::TypeMismatch(Marker::Array32),
            serde::de::Type::TupleStruct => Error::TypeMismatch(Marker::Array32),
            serde::de::Type::Struct => Error::TypeMismatch(Marker::Map32),
            serde::de::Type::Tuple => Error::TypeMismatch(Marker::Array32),
            serde::de::Type::Enum => Error::TypeMismatch(Marker::Array32),
            serde::de::Type::StructVariant => Error::TypeMismatch(Marker::Map32),
            serde::de::Type::TupleVariant => Error::TypeMismatch(Marker::Array32),
            serde::de::Type::UnitVariant => Error::TypeMismatch(Marker::Array32),
            serde::de::Type::Bytes => Error::TypeMismatch(Marker::Array32),
        }
    }

    fn end_of_stream() -> Error {
        Error::Uncategorized("end of stream".to_string())
    }

    fn missing_field(_field: &str) -> Error {
        Error::Uncategorized("missing field".to_string())
    }

    fn unknown_field(_field: &str) -> Error {
        Error::Uncategorized("unknown field".to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        ::std::error::Error::description(self).fmt(f)
    }
}

impl From<FixedValueReadError> for Error {
    fn from(err: FixedValueReadError) -> Error {
        match err {
            FixedValueReadError::UnexpectedEOF => Error::InvalidMarkerRead(ReadError::UnexpectedEOF),
            FixedValueReadError::Io(err) => Error::InvalidMarkerRead(ReadError::Io(err)),
            FixedValueReadError::TypeMismatch(marker) => Error::TypeMismatch(marker),
        }
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

/// Unstable: docs; incomplete
impl<'a> From<DecodeStringError<'a>> for Error {
    fn from(err: DecodeStringError) -> Error {
        match err {
            DecodeStringError::InvalidMarkerRead(err) => Error::InvalidMarkerRead(err),
            DecodeStringError::InvalidDataRead(..) => Error::Uncategorized("InvalidDataRead".to_string()),
            DecodeStringError::TypeMismatch(..) => Error::Uncategorized("TypeMismatch".to_string()),
            DecodeStringError::BufferSizeTooSmall(..) => Error::Uncategorized("BufferSizeTooSmall".to_string()),
            DecodeStringError::InvalidDataCopy(..) => Error::Uncategorized("InvalidDataCopy".to_string()),
            DecodeStringError::InvalidUtf8(..) => Error::Uncategorized("InvalidUtf8".to_string()),
        }
    }
}

impl From<MarkerReadError> for Error {
    fn from(err: MarkerReadError) -> Error {
        Error::InvalidMarkerRead(From::from(err))
    }
}

impl From<serde::de::value::Error> for Error {
    fn from(err: serde::de::value::Error) -> Error {
        match err {
            serde::de::value::Error::SyntaxError => Error::Syntax("unknown".into()),
            _ => Error::Uncategorized("unknown".into()),
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

/// # Note
///
/// All instances of `ErrorKind::Interrupted` are handled by this function and the underlying
/// operation is retried.
// TODO: Docs. Examples.
pub struct Deserializer<R: Read> {
    rd: R,
    buf: Vec<u8>,
}

impl<R: Read> Deserializer<R> {
    // TODO: Docs.
    pub fn new(rd: R) -> Deserializer<R> {
        Deserializer {
            rd: rd,
            buf: Vec::new(),
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

    fn read_str<V>(&mut self, len: u32, mut visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor
    {
        self.buf.clear();
        self.buf.extend((0..len).map(|_| 0));
        visitor.visit_str(try!(read_str_data(&mut self.rd, len, &mut self.buf[..])))
    }

    fn read_array<V>(&mut self, len: u32, mut visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor
    {
        visitor.visit_seq(SeqVisitor {
            deserializer: self,
            len: len,
            actual: len,
        })
    }

    fn read_map<V>(&mut self, len: u32, mut visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor
    {
        visitor.visit_map(MapVisitor {
            deserializer: self,
            len: len,
            actual: len,
        })
    }

    fn read_bin_data<V>(&mut self, len: usize, mut visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor
    {
        self.buf.clear();
        self.buf.extend((0..len).map(|_| 0));

        match read_full(&mut self.rd, &mut self.buf[..]) {
            Ok(n) if n == self.buf.len() => (),
            Ok(..)   => return Err(Error::InvalidDataRead(ReadError::UnexpectedEOF)),
            Err(err) => return Err(Error::InvalidDataRead(ReadError::Io(err))),
        }

        visitor.visit_bytes(&mut self.buf[..])
    }
}

/// Unstable: docs; examples; incomplete
impl<R: Read> serde::Deserializer for Deserializer<R> {
    type Error = Error;

    fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor
    {
        let marker = try!(read_marker(&mut self.rd));

        match marker {
            Marker::Null => visitor.visit_unit(),
            Marker::True => visitor.visit_bool(true),
            Marker::False => visitor.visit_bool(false),
            Marker::FixPos(val) => visitor.visit_u8(val),
            Marker::FixNeg(val) => visitor.visit_i8(val),
            Marker::U8 => visitor.visit_u8(try!(read_numeric_data(&mut self.rd))),
            Marker::U16 => visitor.visit_u16(try!(read_numeric_data(&mut self.rd))),
            Marker::U32 => visitor.visit_u32(try!(read_numeric_data(&mut self.rd))),
            Marker::U64 => visitor.visit_u64(try!(read_numeric_data(&mut self.rd))),
            Marker::I8 => visitor.visit_i8(try!(read_numeric_data(&mut self.rd))),
            Marker::I16 => visitor.visit_i16(try!(read_numeric_data(&mut self.rd))),
            Marker::I32 => visitor.visit_i32(try!(read_numeric_data(&mut self.rd))),
            Marker::I64 => visitor.visit_i64(try!(read_numeric_data(&mut self.rd))),
            Marker::F32 => visitor.visit_f32(try!(read_numeric_data(&mut self.rd))),
            Marker::F64 => visitor.visit_f64(try!(read_numeric_data(&mut self.rd))),
            Marker::FixStr(len) => self.read_str(len as u32, visitor),
            Marker::Str8 => {
                let len: u8 = try!(read_numeric_data(&mut self.rd));
                self.read_str(len as u32, visitor)
            }
            Marker::Str16 => {
                let len: u16 = try!(read_numeric_data(&mut self.rd));
                self.read_str(len as u32, visitor)
            }
            Marker::Str32 => {
                let len: u32 = try!(read_numeric_data(&mut self.rd));
                self.read_str(len, visitor)
            }
            Marker::FixArray(len) => {
                self.read_array(len as u32, visitor)
            }
            Marker::Array16 => {
                let len: u16 = try!(read_numeric_data(&mut self.rd));
                self.read_array(len as u32, visitor)
            }
            Marker::Array32 => {
                let len: u32 = try!(read_numeric_data(&mut self.rd));
                self.read_array(len, visitor)
            }
            Marker::FixMap(len) => {
                self.read_map(len as u32, visitor)
            }
            Marker::Map16 => {
                let len: u16 = try!(read_numeric_data(&mut self.rd));
                self.read_map(len as u32, visitor)
            }
            Marker::Map32 => {
                let len: u32 = try!(read_numeric_data(&mut self.rd));
                self.read_map(len, visitor)
            }
            Marker::Bin8 => {
                let len: u8 = try!(read_numeric_data(&mut self.rd));
                self.read_bin_data(len as usize, visitor)
            }
            Marker::Bin16 => {
                let len: u16 = try!(read_numeric_data(&mut self.rd));
                self.read_bin_data(len as usize, visitor)
            }
            Marker::Bin32 => {
                let len: u32 = try!(read_numeric_data(&mut self.rd));
                self.read_bin_data(len as usize, visitor)
            }
            Marker::Reserved => Err(Error::TypeMismatch(Marker::Reserved)),
            // TODO: Make something with exts.
            marker => Err(From::from(FixedValueReadError::TypeMismatch(marker))),
        }
    }

    /// We treat Value::Null as None.
    fn visit_option<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor,
    {
        // Primarily try to read optimisticly.
        match visitor.visit_some(self) {
            Ok(val) => Ok(val),
            Err(Error::TypeMismatch(Marker::Null)) => visitor.visit_none(),
            Err(err) => Err(err)
        }
    }

    fn visit_enum<V>(&mut self, _enum: &str, _variants: &'static [&'static str], mut visitor: V)
        -> Result<V::Value>
        where V: serde::de::EnumVisitor
    {
        // TODO: Read array len instead.
        let marker = try!(read_marker(&mut self.rd));

        match marker {
            Marker::FixArray(2) => visitor.visit(VariantVisitor::new(self)),
            Marker::FixArray(n) => Err(Error::LengthMismatch(n as u32)),
            marker => Err(Error::TypeMismatch(marker)),
        }
    }
}

struct SeqVisitor<'a, R: Read + 'a> {
    deserializer: &'a mut Deserializer<R>,
    len: u32,
    actual: u32,
}

impl<'a, R: Read + 'a> serde::de::SeqVisitor for SeqVisitor<'a, R> {
    type Error = Error;

    fn visit<T>(&mut self) -> Result<Option<T>>
        where T: serde::de::Deserialize,
    {
        if self.len > 0 {
            self.len -= 1;
            let value = try!(serde::Deserialize::deserialize(self.deserializer));
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn end(&mut self) -> Result<()> {
        if self.len == 0 {
            Ok(())
        } else {
            Err(Error::LengthMismatch(self.actual))
        }
    }
}

struct MapVisitor<'a, R: Read + 'a> {
    deserializer: &'a mut Deserializer<R>,
    len: u32,
    actual: u32,
}

impl<'a, R: Read + 'a> serde::de::MapVisitor for MapVisitor<'a, R> {
    type Error = Error;

    fn visit_key<K>(&mut self) -> Result<Option<K>>
        where K: serde::de::Deserialize,
    {
        if self.len > 0 {
            self.len -= 1;
            let key = try!(serde::Deserialize::deserialize(self.deserializer));
            Ok(Some(key))
        } else {
            Ok(None)
        }
    }

    fn visit_value<V>(&mut self) -> Result<V>
        where V: serde::de::Deserialize,
    {
        let value = try!(serde::Deserialize::deserialize(self.deserializer));
        Ok(value)
    }

    fn end(&mut self) -> Result<()> {
        if self.len == 0 {
            Ok(())
        } else {
            Err(Error::LengthMismatch(self.actual))
        }
    }
}

/// Default variant visitor.
///
/// # Note
///
/// We use default behaviour for new type, which decodes enums with a single value as a tuple.
struct VariantVisitor<'a, R: Read + 'a> {
    de: &'a mut Deserializer<R>,
}

impl<'a, R: Read + 'a> VariantVisitor<'a, R> {
    pub fn new(de: &'a mut Deserializer<R>) -> VariantVisitor<'a, R> {
        VariantVisitor {
            de: de,
        }
    }
}

impl<'a, R: Read> serde::de::VariantVisitor for VariantVisitor<'a, R> {
    type Error = Error;

    // Resolves an internal variant type by integer id.
    fn visit_variant<V>(&mut self) -> Result<V>
        where V: serde::Deserialize
    {
        use serde::de::value::ValueDeserializer;

        let id: u32 = try!(serde::Deserialize::deserialize(self.de));

        let mut de = (id as usize).into_deserializer();
        Ok(try!(V::deserialize(&mut de)))
    }

    fn visit_unit(&mut self) -> Result<()> {
        use serde::de::Deserialize;

        type T = ();
        T::deserialize(self.de)
    }

    fn visit_tuple<V>(&mut self, len: usize, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor,
    {
        serde::de::Deserializer::visit_tuple(self.de, len, visitor)
    }

    fn visit_struct<V>(&mut self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor,
    {
        serde::de::Deserializer::visit_tuple(self.de, fields.len(), visitor)
    }
}
