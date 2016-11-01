#![cfg_attr(not(feature = "with-syntex"), feature(proc_macro, plugin))]

#[macro_use]
extern crate serde;
extern crate rmp;
extern crate rmp_serde;

#[cfg(not(feature = "with-syntex"))]
#[macro_use]
extern crate serde_derive;

mod derive;
