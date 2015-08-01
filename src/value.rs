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
    /// Converts the current non-owning value to the owned one.
    // TODO: Documentation with examples.
    pub fn to_owned(&self) -> Value {
        match self {
            &ValueRef::Nil => Value::Nil,
            &ValueRef::Boolean(val) => Value::Boolean(val),
            &ValueRef::Integer(val) => Value::Integer(val),
            &ValueRef::Float(val) => Value::Float(val),
            &ValueRef::String(val) => Value::String(val.to_string()),
            &ValueRef::Binary(val) => Value::Binary(val.to_vec()),
            &ValueRef::Array(ref val) => {
                let mut vec = Vec::new();
                for item in val {
                    vec.push(item.to_owned());
                }

                Value::Array(vec)
            }
            &ValueRef::Map(ref val) => {
                let mut vec = Vec::new();
                for &(ref key, ref val) in val {
                    vec.push((key.to_owned(), val.to_owned()));
                }

                Value::Map(vec)
            }
            &ValueRef::Ext(ty, buf) => Value::Ext(ty, buf.to_vec()),
        }
    }
}

// For some weird reasons I can't implement it manually.
// It gives me: conflicting implementations for trait `collections::borrow::ToOwned`
// impl<'a> ToOwned for ValueRef<'a> {
//     type Owned = Value;
// }
