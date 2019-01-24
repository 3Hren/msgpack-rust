no_std
======

This is a shim that provides alternate impls of some `std` traits so that `rmp`
can be used in a `no_std` target, when `std` trait is omitted.

This is based on work of "Vadzim Dambrouski <pftbest@gmail.com>"
From this repo: https://github.com/pftbest/msgpack-rust at branch `no_std`

with some light refactor / updates.
