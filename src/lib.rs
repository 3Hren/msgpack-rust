#![crate_name = "msgpack"]

//#![feature(collections)]
//#![cfg_attr(test, feature(test))]

/// Unstable: this library is still in rapid development and everything may change until 1.0 comes.

extern crate byteorder;
extern crate rustc_serialize as serialize;

pub const MSGPACK_VERSION : u32 = 5;

pub mod core;

pub use core::{
    Marker,
};

pub use core::encode;

pub use core::encode::{
    FixedValueWriteError,
    write_u8,
    write_i8,
};

pub use core::decode;

pub use core::decode::serialize::{
    Decoder,
};

pub use core::encode::serialize::{
    Encoder,
};

/// Temporary. Move to error.rs module.
pub use core::{Error, ReadError};

//#[cfg(test)]
//mod bench;

// RC
// + 1. Core.
// 2. Low-level encode.
// 3. Serialization.
// 4. Low-level decode.
// 5. Deserialization.
// 6. Value.
// 7. ValueRef.
// 8. Module.
// 9. Serialize/Deserialize enums.
// 10. Unimplemented.
// 11. TODO's.

// NOTE: Write about strict integer typing. Sized integers always encoded as sized even if they are
// fit in unsized, i.e 100 -> i32 -> posfix.

// Write about error style.
