# RMP - Rust MessagePack

RMP is a pure Rust [MessagePack](http://msgpack.org) implementation.

[![Build Status](https://travis-ci.org/3Hren/rust-msgpack.svg?branch=master)](https://travis-ci.org/3Hren/rust-msgpack)

- [API documentation]()

- [Crates.io](http://crates.io/crates/rmp)

## Usage

To use `rmp`, first add this to your `Cargo.toml`:

```toml
[dependencies.msgpack]
rmp = "0.1.0"
```

Then, add this to your crate root:

```rust
extern crate msgpack;
```

## Features

- **Convenient API**

  RMP is designed to be lightweight and straightforward. There are low-level API, which gives you
  full control on data encoding/decoding process and makes no heap allocations. On the other hand
  there are high-level API, which provides you convenient interface using Rust standard library and
  compiler reflection, allowing to encode/decode structures using `derive` attribute.

- **Clear error handling**

  RMP's error system guarantees that you never receive an error enum with unreachable variant.

- **Robust and tested**

  This project is developed using TDD and CI, so any found bugs will be fixed without breaking
  existing functionality.
