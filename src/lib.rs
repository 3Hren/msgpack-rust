#![crate_name = "msgpack"]

/// Unstable: this library is still in rapid development and everything may change until 1.0 comes.

extern crate byteorder;
extern crate rustc_serialize as serialize;

pub const MSGPACK_VERSION : u32 = 5;

pub use init::Marker;

pub use encode::{
    FixedValueWriteError,
    write_u8,
    write_i8,
};

pub use decode::serialize::{
    Decoder,
};

pub use encode::serialize::{
    Encoder,
};

mod init;
pub mod encode;
pub mod decode;
pub mod value;

// Suppressed due to instability.
// #[cfg(test)]
// mod bench;

// NOTE: Write about strict integer typing. Sized integers always encoded as sized even if they are
// fit in unsized, i.e 100 -> i32 -> posfix.

// Write about error style.
