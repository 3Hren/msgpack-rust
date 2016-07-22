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

    /// If the `Value` is an integer, return or cast it to a i64.
    /// Returns None otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rmp::Value;
    /// use rmp::value::{Float, Integer};
    ///
    /// assert_eq!(Some(42i64), Value::Integer(Integer::I64(42)).as_i64());
    /// assert_eq!(Some(42i64), Value::Integer(Integer::U64(42)).as_i64());
    ///
    /// assert_eq!(None, Value::Float(Float::F64(42.0)).as_i64());
    /// ```
    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            Value::Integer(Integer::I64(n)) => Some(n),
            Value::Integer(Integer::U64(n)) if n <= i64::max_value() as u64 => Some(n as i64),
            _ => None,
        }
    }

    /// If the `Value` is an integer, return or cast it to a u64.
    /// Returns None otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rmp::Value;
    /// use rmp::value::{Float, Integer};
    ///
    /// assert_eq!(Some(42u64), Value::Integer(Integer::I64(42)).as_u64());
    /// assert_eq!(Some(42u64), Value::Integer(Integer::U64(42)).as_u64());
    ///
    /// assert_eq!(None, Value::Integer(Integer::I64(-42)).as_u64());
    /// assert_eq!(None, Value::Float(Float::F64(42.0)).as_u64());
    /// ```
    pub fn as_u64(&self) -> Option<u64> {
        match *self {
            Value::Integer(Integer::I64(n)) if 0 <= n => Some(n as u64),
            Value::Integer(Integer::U64(n)) => Some(n),
            _ => None,
        }
    }

    /// If the `Value` is a number, return or cast it to a f64.
    /// Returns None otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rmp::Value;
    /// use rmp::value::{Float, Integer};
    ///
    /// assert_eq!(Some(42.0), Value::Integer(Integer::I64(42)).as_f64());
    /// assert_eq!(Some(42.0), Value::Integer(Integer::U64(42)).as_f64());
    /// assert_eq!(Some(42.0), Value::Float(Float::F32(42.0f32)).as_f64());
    /// assert_eq!(Some(42.0), Value::Float(Float::F64(42.0f64)).as_f64());
    ///
    /// assert_eq!(Some(2147483647.0), Value::Integer(Integer::I64(i32::max_value() as i64)).as_f64());
    ///
    /// assert_eq!(None, Value::Nil.as_f64());
    ///
    /// assert_eq!(None, Value::Integer(Integer::I64(i32::max_value() as i64 + 1)).as_f64());
    /// ```
    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            Value::Integer(Integer::I64(n)) if (i32::min_value() as i64 <= n) && (n <= i32::max_value() as i64) => {
                Some(From::from(n as i32))
            }
            Value::Integer(Integer::U64(n)) if n <= u32::max_value() as u64 => {
                Some(From::from(n as u32))
            }
            Value::Float(Float::F32(n)) => Some(From::from(n)),
            Value::Float(Float::F64(n)) => Some(n),
            _ => None,
        }
    }

    /// If the `Value` is a String, returns the associated str.
    /// Returns None otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rmp::Value;
    ///
    /// assert_eq!(Some("le message"), Value::String("le message".into()).as_str());
    ///
    /// assert_eq!(None, Value::Boolean(true).as_str());
    /// ```
    pub fn as_str(&self) -> Option<&str> {
        if let Value::String(ref val) = *self {
            Some(val)
        } else {
            None
        }
    }

    /// If the `Value` is an Array, returns the associated vector.
    /// Returns None otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rmp::Value;
    ///
    /// let val = Value::Array(vec![Value::Nil, Value::Boolean(true)]);
    ///
    /// assert_eq!(Some(&vec![Value::Nil, Value::Boolean(true)]), val.as_array());
    ///
    /// assert_eq!(None, Value::Nil.as_array());
    /// ```
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        if let Value::Array(ref array) = *self {
            Some(&*array)
        } else {
            None
        }
    }

    /// If the `Value` is a Map, returns the associated vector of key-value tuples.
    /// Returns None otherwise.
    ///
    /// # Note
    ///
    /// MessagePack represents map as a vector of key-value tuples.
    ///
    /// # Examples
    ///
    /// ```
    /// use rmp::Value;
    ///
    /// let val = Value::Map(vec![(Value::Nil, Value::Boolean(true))]);
    ///
    /// assert_eq!(Some(&vec![(Value::Nil, Value::Boolean(true))]), val.as_map());
    ///
    /// assert_eq!(None, Value::Nil.as_map());
    /// ```
    pub fn as_map(&self) -> Option<&Vec<(Value, Value)>> {
        if let Value::Map(ref map) = *self {
            Some(map)
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
