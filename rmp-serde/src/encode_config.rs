//! Module containing compile-time configuration traits and structures.
use serde::{Serialize, Serializer};

use rmp::encode;

use encode::{Error, UnderlyingWrite};

/// Trait controlling how enum variants are encoded.
///
/// The standard options are to encode it as the variant index (`VariantIntegerEncoding`)
/// or as as the variant name (`VariantStringEncoding`)
///
/// `Copy + 'static` restrictions are here for convenience. This is meant to be implemented mainly
/// by zero-size structs for compile time configuration.
pub trait EnumVariantEncoding: Copy + 'static {
    /// Write the variant name for the given enum variant.
    ///
    /// This doesn't control how the value is written as that will be different depending on whether
    /// it's a unit value, struct value or tuple struct value.
    fn write_variant_identifier<S>(
        self,
        s: &mut S,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<(), Error>
    where
        for<'b> &'b mut S: Serializer<Ok = (), Error = Error>,
        S: UnderlyingWrite;
}

/// Trait controlling how struct field names are encoded.
///
/// The standard options are to encode fields as a list of fields, identifying the fields by their
/// indices (`StructTupleEncoding`), or to encode fields as a map from field name to field, identifying
/// the fields by their names (`StructMapEncoding`).
///
/// `Copy + 'static` restrictions are here for convenience. This is meant to be implemented mainly
/// by zero-size structs for compile time configuration.
pub trait StructFieldEncoding: Copy + 'static {
    /// Write the header of a struct value which directly precedes all the values.
    fn write_struct_header<S>(self, s: &mut S, name: &'static str, len: usize) -> Result<(), Error>
    where
        for<'b> &'b mut S: Serializer<Ok = (), Error = Error>,
        S: UnderlyingWrite;

    /// Write a particular struct value. This will only be called directly after another call to this method
    /// or directly after `StructFieldEncoding::write_struct_header`.
    fn write_field<S, T: ?Sized>(
        self,
        s: &mut S,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error>
    where
        T: Serialize,
        for<'b> &'b mut S: Serializer<Ok = (), Error = Error>,
        S: UnderlyingWrite;
}

/// `EnumVariantEncoding` for identifying enum variants as their index.
#[derive(Copy, Clone, Debug, Default)]
pub struct VariantIntegerEncoding;
/// `EnumVariantEncoding` for identifying enum variants as their name.
#[derive(Copy, Clone, Debug, Default)]
pub struct VariantStringEncoding;
/// `StructFieldEncoding` for identifying fields by their index.
#[derive(Copy, Clone, Debug, Default)]
pub struct StructTupleEncoding;
/// `StructFieldEncoding` for identifying fields by their name.
#[derive(Copy, Clone, Debug, Default)]
pub struct StructMapEncoding;
/// `EnumVariantEncoding` for deciding whether to use `VariantIntegerEncoding`
/// or `VariantStringEncoding` at runtime rather than compile time.
#[derive(Copy, Clone, Debug)]
pub enum RuntimeDecidedVariantEncoding {
    /// Chooses to identify enum variants by integers at runtime. See `VariantIntegerEncoding`.
    Integer,
    /// Chooses to identify enum variants by strings at runtime. See `VariantStringEncoding`.
    String,
}
/// `StructFieldEncoding` for deciding whether to use `StructTupleEncoding`
/// or `StructMapEncoding` at runtime rather than compile time.
#[derive(Copy, Clone, Debug)]
pub enum RuntimeDecidedFieldEncoding {
    /// Chooses to encode structs as tuples at runtime. See `StructTupleEncoding`.
    Tuple,
    /// Chooses to encode structs as maps at runtime. See `StructMapEncoding`.
    Map,
}

impl EnumVariantEncoding for VariantIntegerEncoding {
    fn write_variant_identifier<S>(
        self,
        s: &mut S,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<(), Error>
    where
        for<'b> &'b mut S: Serializer<Ok = (), Error = Error>,
        S: UnderlyingWrite,
    {
        s.serialize_u32(variant_index)
    }
}

impl EnumVariantEncoding for VariantStringEncoding {
    fn write_variant_identifier<S>(
        self,
        s: &mut S,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<(), Error>
    where
        for<'b> &'b mut S: Serializer<Ok = (), Error = Error>,
        S: UnderlyingWrite,
    {
        s.serialize_str(variant)
    }
}

impl EnumVariantEncoding for RuntimeDecidedVariantEncoding {
    fn write_variant_identifier<S>(
        self,
        s: &mut S,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<(), Error>
    where
        for<'b> &'b mut S: Serializer<Ok = (), Error = Error>,
        S: UnderlyingWrite,
    {
        match self {
            RuntimeDecidedVariantEncoding::Integer => {
                VariantIntegerEncoding.write_variant_identifier(s, variant_index, variant)
            }
            RuntimeDecidedVariantEncoding::String => {
                VariantStringEncoding.write_variant_identifier(s, variant_index, variant)
            }
        }
    }
}

impl StructFieldEncoding for StructTupleEncoding {
    fn write_struct_header<S>(self, s: &mut S, _name: &'static str, len: usize) -> Result<(), Error>
    where
        for<'b> &'b mut S: Serializer<Ok = (), Error = Error>,
        S: UnderlyingWrite,
    {
        encode::write_array_len(s.get_mut(), len as u32)?;

        Ok(())
    }

    fn write_field<S, T: ?Sized>(
        self,
        s: &mut S,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Error>
    where
        T: Serialize,
        for<'b> &'b mut S: Serializer<Ok = (), Error = Error>,
        S: UnderlyingWrite,
    {
        value.serialize(s)
    }
}

impl StructFieldEncoding for StructMapEncoding {
    fn write_struct_header<S>(self, s: &mut S, _name: &'static str, len: usize) -> Result<(), Error>
    where
        for<'b> &'b mut S: Serializer<Ok = (), Error = Error>,
        S: UnderlyingWrite,
    {
        encode::write_map_len(s.get_mut(), len as u32)?;

        Ok(())
    }

    fn write_field<S, T: ?Sized>(self, s: &mut S, key: &'static str, value: &T) -> Result<(), Error>
    where
        T: Serialize,
        for<'b> &'b mut S: Serializer<Ok = (), Error = Error>,
        S: UnderlyingWrite,
    {
        encode::write_str(s.get_mut(), key)?;
        value.serialize(s)
    }
}

impl StructFieldEncoding for RuntimeDecidedFieldEncoding {
    fn write_struct_header<S>(self, s: &mut S, name: &'static str, len: usize) -> Result<(), Error>
    where
        for<'b> &'b mut S: Serializer<Ok = (), Error = Error>,
        S: UnderlyingWrite,
    {
        match self {
            RuntimeDecidedFieldEncoding::Tuple => {
                StructTupleEncoding.write_struct_header(s, name, len)
            }
            RuntimeDecidedFieldEncoding::Map => StructMapEncoding.write_struct_header(s, name, len),
        }
    }

    fn write_field<S, T: ?Sized>(self, s: &mut S, key: &'static str, value: &T) -> Result<(), Error>
    where
        T: Serialize,
        for<'b> &'b mut S: Serializer<Ok = (), Error = Error>,
        S: UnderlyingWrite,
    {
        match self {
            RuntimeDecidedFieldEncoding::Tuple => StructTupleEncoding.write_field(s, key, value),
            RuntimeDecidedFieldEncoding::Map => StructMapEncoding.write_field(s, key, value),
        }
    }
}
