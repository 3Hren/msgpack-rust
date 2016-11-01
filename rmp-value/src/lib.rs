//! Contains Value and ValueRef structs and its conversion traits.
//!
//! # Examples
//!
//! ```
//! ```

use std::ops::Index;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// Nil represents nil.
    Nil,
    /// Boolean represents true or false.
    Boolean(bool),
    /// Unsigned integer.
    U64(u64),
    /// Signed integer.
    I64(i64),
    /// A 32-bit floating point number.
    F32(f32),
    /// A 64-bit floating point number.
    F64(f64),
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
    ///
    /// # Examples
    ///
    /// ```
    /// use rmp_value::Value;
    ///
    /// assert!(Value::Nil.is_nil());
    /// ```
    pub fn is_nil(&self) -> bool {
        if let Value::Nil = *self {
            true
        } else {
            false
        }
    }

    /// Returns true if the `Value` is a Boolean. Returns false otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rmp_value::Value;
    ///
    /// assert!(Value::Boolean(true).is_bool());
    ///
    /// assert!(!Value::Nil.is_bool());
    /// ```
    pub fn is_bool(&self) -> bool {
        self.as_bool().is_some()
    }

    /// Returns true if (and only if) the `Value` is a i64. Returns false otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rmp_value::Value;
    ///
    /// assert!(Value::I64(42).is_i64());
    ///
    /// assert!(!Value::U64(42).is_i64());
    /// assert!(!Value::F32(42.0).is_i64());
    /// assert!(!Value::F64(42.0).is_i64());
    /// ```
    pub fn is_i64(&self) -> bool {
        if let Value::I64(..) = *self {
            true
        } else {
            false
        }
    }

    /// Returns true if (and only if) the `Value` is a u64. Returns false otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rmp_value::Value;
    ///
    /// assert!(Value::U64(42).is_u64());
    ///
    /// assert!(!Value::I64(42).is_u64());
    /// assert!(!Value::F32(42.0).is_u64());
    /// assert!(!Value::F64(42.0).is_u64());
    /// ```
    pub fn is_u64(&self) -> bool {
        if let Value::U64(..) = *self {
            true
        } else {
            false
        }
    }

    /// Returns true if (and only if) the `Value` is a f32. Returns false otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rmp_value::Value;
    ///
    /// assert!(Value::F32(42.0).is_f32());
    ///
    /// assert!(!Value::I64(42).is_f32());
    /// assert!(!Value::U64(42).is_f32());
    /// assert!(!Value::F64(42.0).is_f32());
    /// ```
    pub fn is_f32(&self) -> bool {
        if let Value::F32(..) = *self {
            true
        } else {
            false
        }
    }

    /// Returns true if (and only if) the `Value` is a f64. Returns false otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rmp_value::Value;
    ///
    /// assert!(Value::F64(42.0).is_f64());
    ///
    /// assert!(!Value::I64(42).is_f64());
    /// assert!(!Value::U64(42).is_f64());
    /// assert!(!Value::F32(42.0).is_f64());
    /// ```
    pub fn is_f64(&self) -> bool {
        if let Value::F64(..) = *self {
            true
        } else {
            false
        }
    }

    /// Returns true if the `Value` is a Number. Returns false otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rmp_value::Value;
    ///
    /// assert!(Value::I64(42).is_number());
    /// assert!(Value::U64(42).is_number());
    /// assert!(Value::F32(42.0).is_number());
    /// assert!(Value::F64(42.0).is_number());
    ///
    /// assert!(!Value::Nil.is_number());
    /// ```
    pub fn is_number(&self) -> bool {
        match *self {
            Value::U64(..) | Value::I64(..) | Value::F32(..) | Value::F64(..) => true,
            _ => false,
        }
    }

    /// Returns true if the `Value` is a String. Returns false otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rmp_value::Value;
    ///
    /// assert!(Value::String("value".into()).is_str());
    ///
    /// assert!(!Value::Nil.is_str());
    /// ```
    pub fn is_str(&self) -> bool {
        self.as_str().is_some()
    }

    /// Returns true if the `Value` is a Binary. Returns false otherwise.
    pub fn is_bin(&self) -> bool {
        self.as_slice().is_some()
    }

    /// Returns true if the `Value` is an Array. Returns false otherwise.
    pub fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    /// Returns true if the `Value` is a Map. Returns false otherwise.
    pub fn is_map(&self) -> bool {
        self.as_map().is_some()
    }

    /// Returns true if the `Value` is an Ext. Returns false otherwise.
    pub fn is_ext(&self) -> bool {
        self.as_ext().is_some()
    }

    /// If the `Value` is a Boolean, returns the associated bool.
    /// Returns None otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rmp_value::Value;
    ///
    /// assert_eq!(Some(true), Value::Boolean(true).as_bool());
    ///
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
    /// use rmp_value::Value;
    ///
    /// assert_eq!(Some(42i64), Value::I64(42).as_i64());
    /// assert_eq!(Some(42i64), Value::U64(42).as_i64());
    ///
    /// assert_eq!(None, Value::F64(42.0).as_i64());
    /// ```
    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            Value::I64(n) => Some(n),
            Value::U64(n) if n <= i64::max_value() as u64 => Some(n as i64),
            _ => None,
        }
    }

    /// If the `Value` is an integer, return or cast it to a u64.
    /// Returns None otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rmp_value::Value;
    ///
    /// assert_eq!(Some(42u64), Value::I64(42).as_u64());
    /// assert_eq!(Some(42u64), Value::U64(42).as_u64());
    ///
    /// assert_eq!(None, Value::I64(-42).as_u64());
    /// assert_eq!(None, Value::F64(42.0).as_u64());
    /// ```
    pub fn as_u64(&self) -> Option<u64> {
        match *self {
            Value::I64(n) if 0 <= n => Some(n as u64),
            Value::U64(n) => Some(n),
            _ => None,
        }
    }

    /// If the `Value` is a number, return or cast it to a f64.
    /// Returns None otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rmp_value::Value;
    ///
    /// assert_eq!(Some(42.0), Value::I64(42).as_f64());
    /// assert_eq!(Some(42.0), Value::U64(42).as_f64());
    /// assert_eq!(Some(42.0), Value::F32(42.0f32).as_f64());
    /// assert_eq!(Some(42.0), Value::F64(42.0f64).as_f64());
    ///
    /// assert_eq!(Some(2147483647.0), Value::I64(i32::max_value() as i64).as_f64());
    ///
    /// assert_eq!(None, Value::Nil.as_f64());
    ///
    /// assert_eq!(None, Value::I64(i32::max_value() as i64 + 1).as_f64());
    /// ```
    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            Value::I64(n) if (i32::min_value() as i64 <= n) && (n <= i32::max_value() as i64) => {
                Some(From::from(n as i32))
            }
            Value::U64(n) if n <= u32::max_value() as u64 => {
                Some(From::from(n as u32))
            }
            Value::F32(n) => Some(From::from(n)),
            Value::F64(n) => Some(n),
            _ => None,
        }
    }

    /// If the `Value` is a String, returns the associated str.
    /// Returns None otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rmp_value::Value;
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

    /// If the `Value` is a Binary, returns the associated slice.
    /// Returns None otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rmp_value::Value;
    ///
    /// assert_eq!(Some(&[1, 2, 3, 4, 5][..]), Value::Binary(vec![1, 2, 3, 4, 5]).as_slice());
    ///
    /// assert_eq!(None, Value::Boolean(true).as_slice());
    /// ```
    pub fn as_slice(&self) -> Option<&[u8]> {
        if let Value::Binary(ref val) = *self {
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
    /// use rmp_value::Value;
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
    /// use rmp_value::Value;
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

    /// If the `Value` is an Ext, returns the associated tuple with a ty and slice.
    /// Returns None otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rmp_value::Value;
    ///
    /// assert_eq!(Some((42, &[1, 2, 3, 4, 5][..])), Value::Ext(42, vec![1, 2, 3, 4, 5]).as_ext());
    ///
    /// assert_eq!(None, Value::Boolean(true).as_ext());
    /// ```
    pub fn as_ext(&self) -> Option<(i8, &[u8])> {
        if let Value::Ext(ty, ref buf) = *self {
            Some((ty, buf))
        } else {
            None
        }
    }
}

static NIL: Value = Value::Nil;

impl Index<usize> for Value {
    type Output = Value;

    fn index(&self, index: usize) -> &Value {
        self.as_array().and_then(|v| v.get(index)).unwrap_or(&NIL)
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Value {
        Value::Boolean(v)
    }
}

impl From<u8> for Value {
    fn from(v: u8) -> Value {
        Value::U64(From::from(v))
    }
}

impl From<u16> for Value {
    fn from(v: u16) -> Value {
        Value::U64(From::from(v))
    }
}

impl From<u32> for Value {
    fn from(v: u32) -> Value {
        Value::U64(From::from(v))
    }
}

impl From<u64> for Value {
    fn from(v: u64) -> Value {
        Value::U64(From::from(v))
    }
}

impl From<usize> for Value {
    fn from(v: usize) -> Value {
        Value::U64(v as u64)
    }
}

impl From<i8> for Value {
    fn from(v: i8) -> Value {
        if v < 0 {
            Value::I64(From::from(v))
        } else {
            Value::from(v as u8)
        }
    }
}

impl From<i16> for Value {
    fn from(v: i16) -> Value {
        if v < 0 {
            Value::I64(From::from(v))
        } else {
            Value::from(v as u16)
        }
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Value {
        if v < 0 {
            Value::I64(From::from(v))
        } else {
            Value::from(v as u32)
        }
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Value {
        if v < 0 {
            Value::I64(From::from(v))
        } else {
            Value::from(v as u64)
        }
    }
}

impl From<isize> for Value {
    fn from(v: isize) -> Value {
        if v < 0 {
            Value::I64(v as i64)
        } else {
            Value::from(v as usize)
        }
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Value {
        Value::F32(v)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Value {
        Value::F64(v)
    }
}

/// Implements human-readable value formatting.
impl ::std::fmt::Display for Value {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            Value::Nil => write!(f, "nil"),
            Value::Boolean(val) => write!(f, "{}", val),
            Value::U64(val) => write!(f, "{}", val),
            Value::I64(val) => write!(f, "{}", val),
            Value::F32(val) => write!(f, "{}", val),
            Value::F64(val) => write!(f, "{}", val),
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
    /// Unsigned integer.
    U64(u64),
    /// Signed integer.
    I64(i64),
    /// A 32-bit floating point number.
    F32(f32),
    /// A 64-bit floating point number.
    F64(f64),
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
    /// use rmp_value::{Value, ValueRef};
    ///
    /// let val = ValueRef::Array(vec![
    ///    ValueRef::Nil,
    ///    ValueRef::U64(42),
    ///    ValueRef::Array(vec![
    ///        ValueRef::String("le message"),
    ///    ])
    /// ]);
    ///
    /// let expected = Value::Array(vec![
    ///     Value::Nil,
    ///     Value::U64(42),
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
            &ValueRef::U64(val) => Value::U64(val),
            &ValueRef::I64(val) => Value::I64(val),
            &ValueRef::F32(val) => Value::F32(val),
            &ValueRef::F64(val) => Value::F64(val),
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
