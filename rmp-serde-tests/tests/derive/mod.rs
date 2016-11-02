#[cfg(feature = "with-syntex")]
mod de {
    include!(concat!(env!("OUT_DIR"), "/de.rs"));
}

#[cfg(feature = "with-syntex")]
mod se {
    include!(concat!(env!("OUT_DIR"), "/se.rs"));
}

#[cfg(feature = "with-syntex")]
mod round {
    include!(concat!(env!("OUT_DIR"), "/round.rs"));
}

#[cfg(not(feature = "with-syntex"))]
mod de {
    include!("de.in.rs");
}

#[cfg(not(feature = "with-syntex"))]
mod se {
    include!("se.in.rs");
}

#[cfg(not(feature = "with-syntex"))]
mod round {
    include!("round.in.rs");
}
