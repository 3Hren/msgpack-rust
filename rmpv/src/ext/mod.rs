use std::error;
use std::fmt::{self, Display, Formatter};

use serde::de::Unexpected;

use crate::{IntPriv, Integer, Value, ValueRef};

pub use self::de::{deserialize_from, from_value, EnumRefDeserializer};
pub use self::se::to_value;

mod de;
mod se;

#[derive(Debug)]
pub enum Error {
    Syntax(String),
}

impl Display for Error {
    #[cold]
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match *self {
            Self::Syntax(ref err) => write!(fmt, "error while decoding value: {err}"),
        }
    }
}

impl error::Error for Error {}

trait ValueExt {
    fn unexpected(&self) -> Unexpected<'_>;
}

impl ValueExt for Value {
    #[cold]
    fn unexpected(&self) -> Unexpected<'_> {
        match *self {
            Self::Nil => Unexpected::Unit,
            Self::Boolean(v) => Unexpected::Bool(v),
            Self::Integer(Integer { n }) => match n {
                IntPriv::PosInt(v) => Unexpected::Unsigned(v),
                IntPriv::NegInt(v) => Unexpected::Signed(v),
            },
            Self::F32(v) => Unexpected::Float(f64::from(v)),
            Self::F64(v) => Unexpected::Float(v),
            Self::String(ref v) => match v.s {
                Ok(ref v) => Unexpected::Str(v),
                Err(ref v) => Unexpected::Bytes(&v.0[..]),
            },
            Self::Binary(ref v) => Unexpected::Bytes(v),
            Self::Array(..) => Unexpected::Seq,
            Self::Map(..) => Unexpected::Map,
            Self::Ext(..) => Unexpected::Seq,
        }
    }
}

impl ValueExt for ValueRef<'_> {
    #[cold]
    fn unexpected(&self) -> Unexpected<'_> {
        match *self {
            ValueRef::Nil => Unexpected::Unit,
            ValueRef::Boolean(v) => Unexpected::Bool(v),
            ValueRef::Integer(Integer { n }) => match n {
                IntPriv::PosInt(v) => Unexpected::Unsigned(v),
                IntPriv::NegInt(v) => Unexpected::Signed(v),
            },
            ValueRef::F32(v) => Unexpected::Float(f64::from(v)),
            ValueRef::F64(v) => Unexpected::Float(v),
            ValueRef::String(ref v) => match v.s {
                Ok(v) => Unexpected::Str(v),
                Err(ref v) => Unexpected::Bytes(v.0),
            },
            ValueRef::Binary(v) => Unexpected::Bytes(v),
            ValueRef::Array(..) => Unexpected::Seq,
            ValueRef::Map(..) => Unexpected::Map,
            ValueRef::Ext(..) => Unexpected::Seq,
        }
    }
}
