//! Serialize a Rust data structure into MessagePack data.

use std::error;
use std::fmt::{self, Display};
use std::io::Write;

use serde;
use serde::Serialize;
use serde::ser::{SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
                 SerializeTuple, SerializeTupleStruct, SerializeTupleVariant};

use rmp::encode;
use rmp::encode::ValueWriteError;

use encode_config::{
    EnumVariantEncoding, StructFieldEncoding, StructMapEncoding, StructTupleEncoding,
    VariantIntegerEncoding, VariantStringEncoding,
};

/// This type represents all possible errors that can occur when serializing or
/// deserializing MessagePack data.
#[derive(Debug)]
pub enum Error {
    /// Failed to write a MessagePack value.
    InvalidValueWrite(ValueWriteError),
    /// Failed to serialize struct, sequence or map, because its length is unknown.
    UnknownLength,
    /// Depth limit exceeded
    DepthLimitExceeded,
    /// Catchall for syntax error messages.
    Syntax(String),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::InvalidValueWrite(..) => "invalid value write",
            Error::UnknownLength => {
                "attempt to serialize struct, sequence or map with unknown length"
            }
            Error::DepthLimitExceeded => "depth limit exceeded",
            Error::Syntax(..) => "syntax error",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::InvalidValueWrite(ref err) => Some(err),
            Error::UnknownLength => None,
            Error::DepthLimitExceeded => None,
            Error::Syntax(..) => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        error::Error::description(self).fmt(f)
    }
}

impl From<ValueWriteError> for Error {
    fn from(err: ValueWriteError) -> Error {
        Error::InvalidValueWrite(err)
    }
}

impl serde::ser::Error for Error {
    /// Raised when there is general error when deserializing a type.
    fn custom<T: Display>(msg: T) -> Error {
        Error::Syntax(msg.to_string())
    }
}

/// Obtain the underlying writer.
pub trait UnderlyingWrite {
    /// Underlying writer type.
    type Write: Write;

    /// Gets a reference to the underlying writer.
    fn get_ref(&self) -> &Self::Write;

    /// Gets a mutable reference to the underlying writer.
    ///
    /// It is inadvisable to directly write to the underlying writer.
    fn get_mut(&mut self) -> &mut Self::Write;

    /// Unwraps this `Serializer`, returning the underlying writer.
    fn into_inner(self) -> Self::Write;
}

/// Represents MessagePack serialization implementation.
///
/// # Note
///
/// MessagePack has no specification about how to encode enum types. Thus we are free to do
/// whatever we want, so the given chose may be not ideal for you.
///
/// An enum value is represented as a single-entry map whose key is the variant
/// id and whose value is a sequence containing all associated data. If the enum
/// does not have associated data, the sequence is empty.
///
/// There are some ways to customize this, however. By default `Serializer` uses
/// `StructTupleEncoding` and `VariantIntegerEncoding`, which encode structs as tuples with
/// indices and enum variants as their variant id respectively. By using `Serializer::with_struct_map`
/// and `Serializer::with_variant_string_encoding`, `Serializer` can be configured to encode structs as maps
/// with field names as the key, and to use the enum variant name rather than index to encode enums.
///
/// All instances of `ErrorKind::Interrupted` are handled by this function and the underlying
/// operation is retried.
// TODO: Docs. Examples.
#[derive(Debug)]
pub struct Serializer<W, FE = StructTupleEncoding, VE = VariantIntegerEncoding> {
    wr: W,
    depth: usize,
    struct_field_encoding: FE,
    enum_variant_encoding: VE,
}

impl<W: Write, FE, VE> Serializer<W, FE, VE> {
    /// Gets a reference to the underlying writer.
    pub fn get_ref(&self) -> &W {
        &self.wr
    }

    /// Gets a mutable reference to the underlying writer.
    ///
    /// It is inadvisable to directly write to the underlying writer.
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.wr
    }

    /// Unwraps this `Serializer`, returning the underlying writer.
    pub fn into_inner(self) -> W {
        self.wr
    }

    /// Changes the maximum nesting depth that is allowed.
    ///
    /// Currently unused.
    #[doc(hidden)]
    pub fn set_max_depth(&mut self, depth: usize) {
        self.depth = depth;
    }
}

impl<W> Serializer<W, StructTupleEncoding, VariantIntegerEncoding> {
    /// Constructs a new `MessagePack` serializer whose output will be written to the writer
    /// specified.
    ///
    /// # Note
    ///
    /// This is the default constructor, which returns a serializer that will serialize structs
    /// and enums using the most compact representation.
    pub fn new(wr: W) -> Self {
        Serializer {
            wr: wr,
            depth: 1024,
            struct_field_encoding: StructTupleEncoding,
            enum_variant_encoding: VariantIntegerEncoding,
        }
    }

    #[deprecated(note = "use `Serializer::new` instead")]
    #[doc(hidden)]
    pub fn compact(wr: W) -> Self {
        Serializer::new(wr)
    }
}

impl<W> Serializer<W, StructMapEncoding, VariantIntegerEncoding> {
    #[deprecated(note = "use `Serializer::with_struct_map()` instead")]
    #[doc(hidden)]
    pub fn new_named(wr: W) -> Self {
        Serializer::new(wr).with_struct_map()
    }
}

impl<'a, W: Write + 'a, FE, VE> Serializer<W, FE, VE> {
    #[inline]
    fn compound(&'a mut self) -> Result<Compound<'a, W, FE, VE>, Error> {
        let c = Compound { se: self };
        Ok(c)
    }
}

impl<W, FE, VE> Serializer<W, FE, VE> {
    /// Consumes this serializer returning the new one, which will serialize structs as a map.
    ///
    /// This is used, when you the default struct serialization as a tuple does not fit your
    /// requirements.
    pub fn with_struct_map(self) -> Serializer<W, StructMapEncoding, VE> {
        self.with_field_encoding(StructMapEncoding)
    }

    /// Consumes this serializer returning the new one, which will serialize structs as a tuple
    /// without field names.
    ///
    /// This is the default MessagePack serialization mechanism, emitting the most compact
    /// representation.
    pub fn with_struct_tuple(self) -> Serializer<W, StructTupleEncoding, VE> {
        self.with_field_encoding(StructTupleEncoding)
    }

    /// Consumes this serializer returning a new one which will either serialize structs as tuples
    /// or as maps depending on the input.
    ///
    /// This allows for configuration at runtime rather than at compile time if necessary. If
    /// using with const parameters, prefer `Serializer::with_struct_tuple` and `Serializer::with_struct_map`.
    ///
    /// Example usage:
    ///
    /// ```rust
    /// use rmp_serde::{Serializer, RuntimeDecidedFieldEncoding};
    ///
    /// let calculated_encoding = RuntimeDecidedFieldEncoding::Map;
    /// let mut buf = Vec::<u8>::new();
    /// let mut ser = Serializer::new(&mut buf).with_field_encoding(calculated_encoding);
    /// ```
    pub fn with_field_encoding<T>(self, encoding: T) -> Serializer<W, T, VE> {
        let Serializer {
            wr,
            depth,
            struct_field_encoding: _,
            enum_variant_encoding,
        } = self;
        Serializer {
            wr: wr,
            depth: depth,
            struct_field_encoding: encoding,
            enum_variant_encoding: enum_variant_encoding,
        }
    }

    /// Consumes this serializer returning the new one which will serialize enum variants as strings.
    ///
    /// This is used when you the default variant serialization as integers does not fit your
    /// requirements.
    pub fn with_variant_string_encoding(self) -> Serializer<W, FE, VariantStringEncoding> {
        self.with_variant_encoding(VariantStringEncoding)
    }

    /// Consumes this serializer returning the new one which will serialize enum variants as their
    /// integer indices.
    ///
    /// This is the default MessagePack serialization mechanism emitting the most compact
    /// representation.
    pub fn with_variant_integer_encoding(self) -> Serializer<W, FE, VariantIntegerEncoding> {
        self.with_variant_encoding(VariantIntegerEncoding)
    }

    /// Consumes this serializer returning a new one which will either serialize enum identifiers as integres
    /// or as strings depending on the input.
    ///
    /// This allows for configuration at runtime rather than at compile time if necessary. If
    /// using with const parameters, prefer `Serializer::with_variant_integer_encoding` and
    /// `Serializer::with_variant_string_encoding`.
    ///
    /// Example usage:
    ///
    /// ```rust
    /// use rmp_serde::{Serializer, RuntimeDecidedVariantEncoding};
    ///
    /// let calculated_encoding = RuntimeDecidedVariantEncoding::String;
    /// let mut buf = Vec::<u8>::new();
    /// let mut ser = Serializer::new(&mut buf).with_variant_encoding(calculated_encoding);
    /// ```
    pub fn with_variant_encoding<T>(self, encoding: T) -> Serializer<W, FE, T> {
        let Serializer {
            wr,
            depth,
            struct_field_encoding,
            enum_variant_encoding: _,
        } = self;
        Serializer {
            wr: wr,
            depth: depth,
            struct_field_encoding: struct_field_encoding,
            enum_variant_encoding: encoding,
        }
    }
}

impl<W: Write, FE, VE> UnderlyingWrite for Serializer<W, FE, VE> {
    type Write = W;

    fn get_ref(&self) -> &Self::Write {
        &self.wr
    }

    fn get_mut(&mut self) -> &mut Self::Write {
        &mut self.wr
    }

    fn into_inner(self) -> Self::Write {
        self.wr
    }
}

/// Part of serde serialization API.
#[derive(Debug)]
pub struct Compound<'a, W: 'a, FE: 'a, VE: 'a> {
    se: &'a mut Serializer<W, FE, VE>,
}

impl<'a, W: 'a, FE, VE> SerializeSeq for Compound<'a, W, FE, VE>
where
    W: Write,
    FE: StructFieldEncoding,
    VE: EnumVariantEncoding,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(&mut *self.se)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W: 'a, FE, VE> SerializeTuple for Compound<'a, W, FE, VE>
where
    W: Write,
    FE: StructFieldEncoding,
    VE: EnumVariantEncoding,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(&mut *self.se)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W: 'a, FE, VE> SerializeTupleStruct for Compound<'a, W, FE, VE>
where
    W: Write,
    FE: StructFieldEncoding,
    VE: EnumVariantEncoding,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(&mut *self.se)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W: 'a, FE, VE> SerializeTupleVariant for Compound<'a, W, FE, VE>
where
    W: Write,
    FE: StructFieldEncoding,
    VE: EnumVariantEncoding,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(&mut *self.se)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W: 'a, FE, VE> SerializeMap for Compound<'a, W, FE, VE>
where
    W: Write,
    FE: StructFieldEncoding,
    VE: EnumVariantEncoding,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        key.serialize(&mut *self.se)
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(&mut *self.se)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W: 'a, FE, VE> SerializeStruct for Compound<'a, W, FE, VE>
where
    W: Write,
    FE: StructFieldEncoding,
    VE: EnumVariantEncoding,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.se
            .struct_field_encoding
            .write_field(self.se, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W: 'a, FE, VE> SerializeStructVariant for Compound<'a, W, FE, VE>
where
    W: Write,
    FE: StructFieldEncoding,
    VE: EnumVariantEncoding,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.se
            .struct_field_encoding
            .write_field(self.se, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W, FE, VE> serde::Serializer for &'a mut Serializer<W, FE, VE>
where
    W: Write,
    FE: StructFieldEncoding,
    VE: EnumVariantEncoding,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Compound<'a, W, FE, VE>;
    type SerializeTuple = Compound<'a, W, FE, VE>;
    type SerializeTupleStruct = Compound<'a, W, FE, VE>;
    type SerializeTupleVariant = Compound<'a, W, FE, VE>;
    type SerializeMap = Compound<'a, W, FE, VE>;
    type SerializeStruct = Compound<'a, W, FE, VE>;
    type SerializeStructVariant = Compound<'a, W, FE, VE>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        encode::write_bool(&mut self.wr, v)
            .map_err(|err| Error::InvalidValueWrite(ValueWriteError::InvalidMarkerWrite(err)))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        encode::write_sint(&mut self.wr, v)?;
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        encode::write_uint(&mut self.wr, v)?;
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        encode::write_f32(&mut self.wr, v)?;
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        encode::write_f64(&mut self.wr, v)?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        // A char encoded as UTF-8 takes 4 bytes at most.
        let mut buf = [0; 4];
        self.serialize_str(v.encode_utf8(&mut buf))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        encode::write_str(&mut self.wr, v)?;
        Ok(())
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok, Self::Error> {
        encode::write_bin_len(&mut self.wr, value.len() as u32)?;
        self.wr
            .write_all(value)
            .map_err(|err| Error::InvalidValueWrite(ValueWriteError::InvalidDataWrite(err)))
    }

    fn serialize_none(self) -> Result<(), Self::Error> {
        self.serialize_unit()
    }

    fn serialize_some<T: ?Sized + serde::Serialize>(self, v: &T) -> Result<(), Self::Error> {
        v.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        encode::write_nil(&mut self.wr)
            .map_err(|err| Error::InvalidValueWrite(ValueWriteError::InvalidMarkerWrite(err)))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        encode::write_array_len(&mut self.wr, 0)?;
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        // encode as a map from variant idx (or name) to nil, like: {idx => nil}
        encode::write_map_len(&mut self.wr, 1)?;
        self.enum_variant_encoding.write_variant_identifier(self, variant_index, variant)?;
        self.serialize_unit()
    }

    fn serialize_newtype_struct<T: ?Sized + serde::Serialize>(self, _name: &'static str, value: &T) -> Result<(), Self::Error> {
        // Encode as if it's inner type.
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + serde::Serialize>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        // encode as a map from variant idx (or name) to its attributed data, like: {idx => value}
        encode::write_map_len(&mut self.wr, 1)?;
        self.enum_variant_encoding.write_variant_identifier(self, variant_index, variant)?;
        self.serialize_newtype_struct(name, value)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        let len = match len {
            Some(len) => len,
            None => return Err(Error::UnknownLength),
        };

        encode::write_array_len(&mut self.wr, len as u32)?;

        self.compound()
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(self, _name: &'static str, len: usize) ->
        Result<Self::SerializeTupleStruct, Self::Error>
    {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Error> {
        // encode as a map from variant idx (or name) to a sequence of its attributed data, like: {idx => [v1,...,vN]}
        encode::write_map_len(&mut self.wr, 1)?;
        self.enum_variant_encoding.write_variant_identifier(self, variant_index, variant)?;
        self.serialize_tuple_struct(name, len)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Error> {
        match len {
            Some(len) => {
                encode::write_map_len(&mut self.wr, len as u32)?;
                self.compound()
            }
            None => Err(Error::UnknownLength),
        }
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.struct_field_encoding.write_struct_header(self, name, len)?;
        self.compound()
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        // encode as a map from variant idx (or name) to it's value.
        // value can either be a sequence of its attributed data, like: {idx => [v1,...,vN]},
        // or a map from field names to values, like: {idx => {field => value, field2 => value2}}
        encode::write_map_len(&mut self.wr, 1)?;
        self.enum_variant_encoding.write_variant_identifier(self, variant_index, variant)?;
        self.serialize_struct(name, len)
    }
}

/// Serialize the given data structure as MessagePack into the I/O stream.
/// This function uses compact representation - structures as arrays
///
/// Serialization can fail if `T`'s implementation of `Serialize` decides to fail.
#[inline]
pub fn write<W, T>(wr: &mut W, val: &T) -> Result<(), Error>
where
    W: Write + ?Sized,
    T: Serialize + ?Sized
{
    val.serialize(&mut Serializer::new(wr))
}

/// Serialize the given data structure as MessagePack into the I/O stream.
/// This function serializes structures as maps
///
/// Serialization can fail if `T`'s implementation of `Serialize` decides to fail.
#[inline]
pub fn write_named<W, T>(wr: &mut W, val: &T) -> Result<(), Error>
where
    W: Write + ?Sized,
    T: Serialize + ?Sized
{
    let mut se = Serializer::new(wr)
        .with_struct_map();
    val.serialize(&mut se)
}

/// Serialize the given data structure as a MessagePack byte vector.
/// This method uses compact representation, structs are serialized as arrays
///
/// Serialization can fail if `T`'s implementation of `Serialize` decides to fail.
#[inline]
pub fn to_vec<T>(val: &T) -> Result<Vec<u8>, Error>
where
    T: Serialize + ?Sized
{
    let mut wr = Vec::with_capacity(128);
    write(&mut wr, val)?;
    Ok(wr)
}

/// Serializes data structure into byte vector as a map
/// Resulting MessagePack message will contain field names
///
/// # Errors
///
/// Serialization can fail if `T`'s implementation of `Serialize` decides to fail.
#[inline]
pub fn to_vec_named<T>(val: &T) -> Result<Vec<u8>, Error>
where
    T: Serialize + ?Sized
{
    let mut wr = Vec::with_capacity(128);
    write_named(&mut wr, val)?;
    Ok(wr)
}

/// Serialize the given data structure as MessagePack into the I/O stream.
///
/// This serializes structs as either maps or tuples and enum identifiers as
/// variant names or indices depending on the configuration.
///
/// Serialization can fail if `T`'s implementation of `Serialize` decides to fail.
///
/// See [`to_vec_custom`] documentation for more information.
#[inline]
pub fn write_custom<FE, VE, W, T>(
    struct_field_encoding: FE,
    enum_variant_encoding: VE,
    wr: &mut W,
    val: &T,
) -> Result<(), Error>
where
    FE: StructFieldEncoding,
    VE: EnumVariantEncoding,
    W: Write + ?Sized,
    T: Serialize + ?Sized,
{
    let mut se = Serializer::new(wr)
        .with_field_encoding(struct_field_encoding)
        .with_variant_encoding(enum_variant_encoding);
    val.serialize(&mut se)
}

/// Serializes the data into a byte vector, using given configuration rather than any preset.
///
/// See documentation for `StructMapEncoding`, `StructTupleEncoding`, `VariantIntegerEncoding`,
/// and `VariantStringEncoding` for the various options you can use.
///
/// # Errors
///
/// Serialization can fail if `T`'s implementation of `Serialize` decides to fail.
///
/// # Example
///
/// ```rust
/// # #[macro_use] extern crate serde_derive;
/// # extern crate rmp_serde;
/// # fn main() {
/// # fn sub_func_for_question_mark_operator() -> Result<(), Box<::std::error::Error>> {
/// use rmp_serde::{VariantStringEncoding, StructMapEncoding};
///
/// #[derive(Serialize)]
/// enum Color { Red, Blue }
/// #[derive(Serialize)]
/// struct MyStruct {
///     color1: Color,
///     color2: Color,
/// }
///
/// let struct1 = MyStruct {
///     color1: Color::Red,
///     color2: Color::Blue,
/// };
///
/// let serialized = rmp_serde::to_vec_custom(StructMapEncoding, VariantStringEncoding, &struct1)?;
///
/// // The configuration causes rmp_serde to use enum and field names to serialize, rather than
/// // indices. Thus you can deserialize into different structs with the same names, but different
/// // field or variant orders.
///
/// #[derive(Deserialize, PartialEq, Debug)]
/// enum ColorV2 { Green, Purple, Blue, Red }
/// #[derive(Deserialize, PartialEq, Debug)]
/// struct SecondStruct {
///     color2: ColorV2,
///     color1: ColorV2,
/// }
///
/// let deserialized: SecondStruct = rmp_serde::from_slice(&serialized)?;
///
/// assert_eq!(deserialized, SecondStruct { color2: ColorV2::Blue, color1: ColorV2::Red });
/// # Ok(())
/// # }
/// # sub_func_for_question_mark_operator().unwrap();
/// # }
/// ```
#[inline]
pub fn to_vec_custom<FE, VE, T>(
    struct_field_encoding: FE,
    enum_variant_encoding: VE,
    val: &T,
) -> Result<Vec<u8>, Error>
where
    FE: StructFieldEncoding,
    VE: EnumVariantEncoding,
    T: Serialize + ?Sized,
{
    let mut wr = Vec::with_capacity(128);

    write_custom(struct_field_encoding, enum_variant_encoding, &mut wr, val)?;
    Ok(wr)
}
