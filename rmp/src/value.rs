//! Contains Value and ValueRef structs and its conversion traits.

use std::convert::From;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Integer {
    /// Every non-negative integer is treated as u64, even if it fits in i64.
    U64(u64),
    /// Every negative integer is treated as i64.
    I64(i64),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Float {
    F32(f32),
    F64(f64),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// Nil represents nil.
    Nil,
    /// Boolean represents true or false.
    Boolean(bool),
    /// Integer represents an integer.
    Integer(Integer),
    /// Float represents a floating point number.
    Float(Float),
    /// String extending Raw type represents a UTF-8 string.
    String(String),
    /// Binary extending Raw type represents a byte array.
    Binary(Vec<u8>),
    /// Array represents a sequence of objects.
    Array(Vec<Value>),
    /// Map represents key-value pairs of objects.
    Map(Vec<(Value, Value)>),
    /// Extended implements Extension interface: represents a tuple of type information and a byte
    /// array where type information is an integer whose meaning is defined by applications.
    Ext(i8, Vec<u8>),
}

impl Value {
    /// Returns true if the `Value` is a Null. Returns false otherwise.
    pub fn is_nil(&self) -> bool {
        if let Value::Nil = *self {
            true
        } else {
            false
        }
    }

    /// If the `Value` is a Boolean, returns the associated bool.
    /// Returns None otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rmp::Value;
    ///
    /// assert_eq!(Some(true), Value::Boolean(true).as_bool());
    /// assert_eq!(None, Value::Nil.as_bool());
    /// ```
    pub fn as_bool(&self) -> Option<bool> {
        if let Value::Boolean(val) = *self {
            Some(val)
        } else {
            None
        }
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Value {
        Value::Boolean(v)
    }
}

impl From<u8> for Value {
    fn from(v: u8) -> Value {
        Value::Integer(Integer::U64(From::from(v)))
    }
}

impl From<u16> for Value {
    fn from(v: u16) -> Value {
        Value::Integer(Integer::U64(From::from(v)))
    }
}

impl From<u32> for Value {
    fn from(v: u32) -> Value {
        Value::Integer(Integer::U64(From::from(v)))
    }
}

impl From<u64> for Value {
    fn from(v: u64) -> Value {
        Value::Integer(Integer::U64(From::from(v)))
    }
}

impl From<usize> for Value {
    fn from(v: usize) -> Value {
        Value::Integer(Integer::U64(v as u64))
    }
}

impl From<i8> for Value {
    fn from(v: i8) -> Value {
        Value::Integer(Integer::I64(From::from(v)))
    }
}

impl From<i16> for Value {
    fn from(v: i16) -> Value {
        Value::Integer(Integer::I64(From::from(v)))
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Value {
        Value::Integer(Integer::I64(From::from(v)))
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Value {
        Value::Integer(Integer::I64(From::from(v)))
    }
}

impl From<isize> for Value {
    fn from(v: isize) -> Value {
        Value::Integer(Integer::I64(v as i64))
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Value {
        Value::Float(Float::F32(v))
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Value {
        Value::Float(Float::F64(v))
    }
}

/// Implements human-readable value formatting.
impl ::std::fmt::Display for Value {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            Value::Nil => write!(f, "nil"),
            Value::Boolean(val) => write!(f, "{}", val),
            Value::Integer(Integer::U64(val)) => write!(f, "{}", val),
            Value::Integer(Integer::I64(val)) => write!(f, "{}", val),
            Value::Float(Float::F32(val)) => write!(f, "{}", val),
            Value::Float(Float::F64(val)) => write!(f, "{}", val),
            Value::String(ref val) => write!(f, "\"{}\"", val),
            Value::Binary(ref val) => write!(f, "{:?}", val),
            Value::Array(ref vec) => {
                // TODO: This can be slower than naive implementation. Need benchmarks for more
                // information.
                let res = vec.iter()
                    .map(|val| format!("{}", val))
                    .collect::<Vec<String>>()
                    .join(", ");

                write!(f, "[{}]", res)
            }
            Value::Map(ref vec) => {
                try!(write!(f, "{{"));

                match vec.iter().take(1).next() {
                    Some(&(ref k, ref v)) => {
                        try!(write!(f, "{}: {}", k, v));
                    }
                    None => {
                        try!(write!(f, ""));
                    }
                }

                for &(ref k, ref v) in vec.iter().skip(1) {
                    try!(write!(f, ", {}: {}", k, v));
                }

                write!(f, "}}")
            }
            Value::Ext(ty, ref data) => {
                write!(f, "[{}, {:?}]", ty, data)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueRef<'a> {
    /// Nil represents nil.
    Nil,
    /// Boolean represents true or false.
    Boolean(bool),
    /// Integer represents an integer.
    Integer(Integer),
    /// Float represents a floating point number.
    Float(Float),
    /// String extending Raw type represents a UTF-8 string.
    String(&'a str),
    /// Binary extending Raw type represents a byte array.
    Binary(&'a [u8]),
    /// Array represents a sequence of objects.
    Array(Vec<ValueRef<'a>>),
    /// Map represents key-value pairs of objects.
    Map(Vec<(ValueRef<'a>, ValueRef<'a>)>),
    /// Extended implements Extension interface: represents a tuple of type information and a byte
    /// array where type information is an integer whose meaning is defined by applications.
    Ext(i8, &'a [u8]),
}

impl<'a> ValueRef<'a> {
    /// Converts the current non-owning value to an owned Value.
    ///
    /// This is achieved by deep copying all underlying structures and borrowed buffers.
    ///
    /// # Panics
    ///
    /// Panics in unable to allocate memory to keep all internal structures and buffers.
    ///
    /// # Examples
    /// ```
    /// use rmp::{Value, ValueRef};
    /// use rmp::value::Integer;
    ///
    /// let val = ValueRef::Array(vec![
    ///    ValueRef::Nil,
    ///    ValueRef::Integer(Integer::U64(42)),
    ///    ValueRef::Array(vec![
    ///        ValueRef::String("le message"),
    ///    ])
    /// ]);
    ///
    /// let expected = Value::Array(vec![
    ///     Value::Nil,
    ///     Value::Integer(Integer::U64(42)),
    ///     Value::Array(vec![
    ///         Value::String("le message".to_string())
    ///     ])
    /// ]);
    ///
    /// assert_eq!(expected, val.to_owned());
    /// ```
    pub fn to_owned(&self) -> Value {
        match self {
            &ValueRef::Nil => Value::Nil,
            &ValueRef::Boolean(val) => Value::Boolean(val),
            &ValueRef::Integer(val) => Value::Integer(val),
            &ValueRef::Float(val) => Value::Float(val),
            &ValueRef::String(val) => Value::String(val.to_owned()),
            &ValueRef::Binary(val) => Value::Binary(val.to_vec()),
            &ValueRef::Array(ref val) => {
                Value::Array(val.iter().map(|v| v.to_owned()).collect())
            }
            &ValueRef::Map(ref val) => {
                Value::Map(val.iter().map(|&(ref k, ref v)| (k.to_owned(), v.to_owned())).collect())
            }
            &ValueRef::Ext(ty, buf) => Value::Ext(ty, buf.to_vec()),
        }
    }
}

// For some weird reasons I can't implement it manually.
// It gives: conflicting implementations for trait `collections::borrow::ToOwned`
// impl<'a> ToOwned for ValueRef<'a> {
//     type Owned = Value;
// }
