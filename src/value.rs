#[derive(Clone, Debug, PartialEq)]
pub enum Integer {
    /// Every non-negative integer is treated as u64, even if it fits in i64.
    U64(u64),
    /// Every negative integer is treated as i64.
    I64(i64),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Integer(Integer),
    String(String),
    Array(Vec<Value>),
}
