#![cfg_attr(feature = "serde_derive", feature(proc_macro, custom_derive))]

extern crate serde;
extern crate rmp;
extern crate rmp_serde;

#[cfg(feature = "serde_derive")]
mod night;
