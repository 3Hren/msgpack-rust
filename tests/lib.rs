#![cfg_attr(feature = "serde_macros", feature(custom_derive, plugin))]
#![cfg_attr(feature = "serde_macros", plugin(serde_macros))]

extern crate rmp as msgpack;
extern crate rustc_serialize;
extern crate serde;

mod func;
