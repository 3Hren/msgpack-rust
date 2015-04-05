// Hack, because of PartialEq trait removal from io::Error.
#[macro_export]
macro_rules! assert_err {
    ($expected:pat, $actual:expr) => ({
        match $actual {
            Err($expected) => {}
            c @ _ => panic!("assertion failed: `(expected == actual)` (actual: `{:?}`)", c)
        }
    })
}
