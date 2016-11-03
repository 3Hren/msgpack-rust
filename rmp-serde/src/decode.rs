use std::error;
use std::fmt::{self, Display, Formatter};
use std::io::Read;
use std::str::{self, Utf8Error};

use byteorder::{self, ReadBytesExt};

use serde;
use serde::de::{Deserialize, Visitor};

use rmp;
use rmp::Marker;
use rmp::decode::{MarkerReadError, DecodeStringError, ValueReadError, NumValueReadError, read_array_len};

/// Unstable: docs; incomplete
#[derive(Debug)]
pub enum Error {
    InvalidMarkerRead(::std::io::Error),
    InvalidDataRead(::std::io::Error),
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

impl serde::de::Error for Error {
    fn invalid_value(msg: &str) -> Error {
        Error::Syntax(format!("syntax error: {}", msg))
    }

    fn invalid_length(len: usize) -> Error {
        Error::LengthMismatch(len as u32)
    }

    fn invalid_type(ty: serde::de::Type) -> Error {
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
            serde::de::Type::Option => Error::TypeMismatch(Marker::Null),
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
            serde::de::Type::FieldName => Error::TypeMismatch(Marker::Str32),
            serde::de::Type::VariantName => Error::TypeMismatch(Marker::Str32),
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

     fn custom<T: Into<String>>(msg: T) -> Error {
        Error::Uncategorized(msg.into())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        error::Error::description(self).fmt(f)
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

/// Unstable: docs; incomplete
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

impl From<serde::de::value::Error> for Error {
    fn from(err: serde::de::value::Error) -> Error {
        use serde::de::Error as SerdeError;
        match err {
           serde::de::value::Error::Custom(e) => {
               Error::custom(e)
           }
           serde::de::value::Error::EndOfStream => {
               Error::end_of_stream()
           }
           serde::de::value::Error::InvalidType(ty) => {
               Error::invalid_type(ty)
           }
           serde::de::value::Error::InvalidValue(msg) => {
               Error::invalid_value(&msg)
           }
           serde::de::value::Error::InvalidLength(len) => {
               Error::invalid_length(len)
           }
           serde::de::value::Error::UnknownVariant(_) => {
               Error::Uncategorized("unknown variant".to_string())
           }
           serde::de::value::Error::UnknownField(field) => {
               Error::unknown_field(&field)
           }
           serde::de::value::Error::MissingField(field) => {
               Error::missing_field(field)
           }
       }
    }
}

/// # Note
///
/// All instances of `ErrorKind::Interrupted` are handled by this function and the underlying
/// operation is retried.
// TODO: Docs. Examples.
pub struct Deserializer<R: Read> {
    rd: R,
    buf: Vec<u8>,
    decoding_option: bool,
    depth: usize,
}

macro_rules! stack_protector(
    ($counter:expr, $expr:expr) => {
        {
            $counter -= 1;
            if $counter == 0 {
                return Err(Error::DepthLimitExceeded)
            }
            let res = $expr;
            $counter += 1;
            res
        }
    }
);

impl<R: Read> Deserializer<R> {
    // TODO: Docs.
    pub fn new(rd: R) -> Deserializer<R> {
        Deserializer {
            rd: rd,
            buf: Vec::with_capacity(128), // NOTE: Update changelog.
            decoding_option: false,
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

        try!(self.rd.read_exact(&mut self.buf[..]).map_err(Error::InvalidDataRead));
        str::from_utf8(&self.buf).map_err(From::from)
    }

    fn read_bin_data(&mut self, len: u32) -> Result<&[u8], Error> {
        self.buf.resize(len as usize, 0u8);

        try!(self.rd.read_exact(&mut self.buf[..len as usize]).map_err(Error::InvalidDataRead));
        Ok(&self.buf[..len as usize])
    }

    fn read_array<V>(&mut self, len: u32, mut visitor: V) -> Result<V::Value, Error>
        where V: Visitor
    {
        visitor.visit_seq(SeqVisitor::new(self, len))
    }

    fn read_map<V>(&mut self, len: u32, mut visitor: V) -> Result<V::Value, Error>
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

impl<R: Read> serde::Deserializer for Deserializer<R> {
    type Error = Error;

    fn deserialize<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: Visitor
    {
        match try!(rmp::decode::read_marker(&mut self.rd)) {
            Marker::Null => {
                if self.decoding_option {
                    visitor.visit_none()
                } else {
                    visitor.visit_unit()
                }
            }
            Marker::True => visitor.visit_bool(true),
            Marker::False => visitor.visit_bool(false),
            Marker::FixPos(val) => visitor.visit_u8(val),
            Marker::FixNeg(val) => visitor.visit_i8(val),
            Marker::U8 => visitor.visit_u8(try!(rmp::decode::read_data_u8(&mut self.rd))),
            Marker::U16 => visitor.visit_u16(try!(rmp::decode::read_data_u16(&mut self.rd))),
            Marker::U32 => visitor.visit_u32(try!(rmp::decode::read_data_u32(&mut self.rd))),
            Marker::U64 => visitor.visit_u64(try!(rmp::decode::read_data_u64(&mut self.rd))),
            Marker::I8 => visitor.visit_i8(try!(rmp::decode::read_data_i8(&mut self.rd))),
            Marker::I16 => visitor.visit_i16(try!(rmp::decode::read_data_i16(&mut self.rd))),
            Marker::I32 => visitor.visit_i32(try!(rmp::decode::read_data_i32(&mut self.rd))),
            Marker::I64 => visitor.visit_i64(try!(rmp::decode::read_data_i64(&mut self.rd))),
            Marker::F32 => visitor.visit_f32(try!(rmp::decode::read_data_f32(&mut self.rd))),
            Marker::F64 => visitor.visit_f64(try!(rmp::decode::read_data_f64(&mut self.rd))),
            Marker::FixStr(len) => visitor.visit_str(try!(self.read_str_data(len as u32))),
            Marker::Str8 => {
                let len = try!(read_u8(&mut self.rd));
                visitor.visit_str(try!(self.read_str_data(len as u32)))
            }
            Marker::Str16 => {
                let len = try!(read_u16(&mut self.rd));
                visitor.visit_str(try!(self.read_str_data(len as u32)))
            }
            Marker::Str32 => {
                let len = try!(read_u32(&mut self.rd));
                visitor.visit_str(try!(self.read_str_data(len)))
            }
            Marker::FixArray(len) => {
                self.read_array(len as u32, visitor)
            }
            Marker::Array16 => {
                let len = try!(read_u16(&mut self.rd));
                self.read_array(len as u32, visitor)
            }
            Marker::Array32 => {
                let len = try!(read_u32(&mut self.rd));
                self.read_array(len, visitor)
            }
            Marker::FixMap(len) => {
                self.read_map(len as u32, visitor)
            }
            Marker::Map16 => {
                let len = try!(read_u16(&mut self.rd));
                self.read_map(len as u32, visitor)
            }
            Marker::Map32 => {
                let len = try!(read_u32(&mut self.rd));
                self.read_map(len, visitor)
            }
            Marker::Bin8 => {
                let len = try!(read_u8(&mut self.rd));
                visitor.visit_bytes(try!(self.read_bin_data(len as u32)))
            }
            Marker::Bin16 => {
                let len = try!(read_u16(&mut self.rd));
                visitor.visit_bytes(try!(self.read_bin_data(len as u32)))
            }
            Marker::Bin32 => {
                let len = try!(read_u32(&mut self.rd));
                visitor.visit_bytes(try!(self.read_bin_data(len)))
            }
            Marker::Reserved => Err(Error::TypeMismatch(Marker::Reserved)),
            // TODO: Make something with exts.
            marker => Err(Error::TypeMismatch(marker)),
        }
    }

    /// We treat Value::Null as None.
    ///
    /// Note, that without using explicit option marker it's impossible to properly deserialize
    /// the following specific cases:
    ///  - `Option<()>`.
    ///  - nested optionals, like `Option<Option<...>>`.
    fn deserialize_option<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor
    {
        // Primarily try to read optimisticly.
        self.decoding_option = true;
        let res = match stack_protector!(self.depth, visitor.visit_some(self)) {
            Ok(val) => Ok(val),
            Err(Error::TypeMismatch(Marker::Null)) => visitor.visit_none(),
            Err(err) => Err(err)
        };
        self.decoding_option = false;

        res
    }

    fn deserialize_enum<V>(&mut self, _enum: &str, _variants: &[&str], mut visitor: V) -> Result<V::Value, Error>
        where V: serde::de::EnumVisitor
    {
        let len = try!(read_array_len(&mut self.rd));

        match len {
            2 => stack_protector!(self.depth, visitor.visit(VariantVisitor::new(self))),
            n => Err(Error::LengthMismatch(n as u32)),
        }
    }

    fn deserialize_newtype_struct<V>(&mut self, _name: &'static str, mut visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {

        let len = try!(rmp::decode::read_array_len(&mut self.rd));

        match len {
            1 => stack_protector!(self.depth, visitor.visit_newtype_struct(self)),
            n => Err(Error::LengthMismatch(n as u32)),
        }
    }

    forward_to_deserialize! {
        bool usize u8 u16 u32 u64 i8 i16 i32 i64 isize f32 f64
        char str string bytes unit unit_struct seq seq_fixed_size map
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

impl<'a, R: Read + 'a> serde::de::SeqVisitor for SeqVisitor<'a, R> {
    type Error = Error;

    fn visit<T>(&mut self) -> Result<Option<T>, Error>
        where T: Deserialize
    {
        if self.nleft > 0 {
            self.nleft -= 1;
            Ok(Some(try!(Deserialize::deserialize(self.de))))
        } else {
            Ok(None)
        }
    }

    fn end(&mut self) -> Result<(), Error> {
        if self.nleft == 0 {
            Ok(())
        } else {
            Err(Error::LengthMismatch(self.nleft))
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

impl<'a, R: Read + 'a> serde::de::MapVisitor for MapVisitor<'a, R> {
    type Error = Error;

    fn visit_key<K>(&mut self) -> Result<Option<K>, Error>
        where K: Deserialize,
    {
        if self.nleft > 0 {
            self.nleft -= 1;
            Ok(Some(try!(Deserialize::deserialize(self.de))))
        } else {
            Ok(None)
        }
    }

    fn visit_value<V>(&mut self) -> Result<V, Error>
        where V: Deserialize,
    {
        Ok(try!(Deserialize::deserialize(self.de)))
    }

    fn end(&mut self) -> Result<(), Error> {
        if self.nleft == 0 {
            Ok(())
        } else {
            Err(Error::LengthMismatch(self.len))
        }
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
    pub fn new(de: &'a mut Deserializer<R>) -> VariantVisitor<'a, R> {
        VariantVisitor {
            de: de,
        }
    }
}

impl<'a, R: Read> serde::de::VariantVisitor for VariantVisitor<'a, R> {
    type Error = Error;

    // Resolves an internal variant type by integer id.
    fn visit_variant<V>(&mut self) -> Result<V, Error>
        where V: serde::Deserialize
    {
        use serde::de::value::ValueDeserializer;

        let id: u32 = try!(serde::Deserialize::deserialize(self.de));
        println!("id: {}", id);
        let mut de = (id as usize).into_deserializer();

        V::deserialize(&mut de)
    }

    fn visit_unit(&mut self) -> Result<(), Error> {
        use serde::de::Deserialize;

        type T = ();
        T::deserialize(self.de)
    }

    fn visit_tuple<V>(&mut self, len: usize, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor,
    {
        serde::de::Deserializer::deserialize_tuple(self.de, len, visitor)
    }

    fn visit_newtype<T>(&mut self) -> Result<T, Error>
        where T: serde::de::Deserialize
    {
        try!(rmp::decode::read_array_len(self.de.get_mut()));
        T::deserialize(self.de)
    }

    fn visit_struct<V>(&mut self, fields: &'static [&'static str], visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor,
    {
        serde::de::Deserializer::deserialize_tuple(self.de, fields.len(), visitor)
    }
}
