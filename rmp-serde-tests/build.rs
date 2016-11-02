#[cfg(feature = "with-syntex")]
mod with_codegen {
    extern crate serde_codegen;

    use std::env;
    use std::path::Path;

    pub fn main() {
        let out_dir = env::var_os("OUT_DIR").unwrap();

        for &(src, dst) in &[("tests/derive/de.in.rs", "de.rs"),
                             ("tests/derive/se.in.rs", "se.rs"),
                             ("tests/derive/round.in.rs", "round.rs")] {
            let src = Path::new(src);
            let dst = Path::new(&out_dir).join(dst);

            serde_codegen::expand(&src, &dst).unwrap();
        }
    }
}

#[cfg(not(feature = "with-syntex"))]
mod with_codegen {
    pub fn main() {}
}

pub fn main() {
    with_codegen::main();
}
