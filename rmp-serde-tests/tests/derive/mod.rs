#[cfg(feature = "with-syntex")]
include!(concat!(env!("OUT_DIR"), "/de.rs"));

#[cfg(not(feature = "with-syntex"))]
include!("de.rs.in");
