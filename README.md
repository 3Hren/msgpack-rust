# RMP - Rust MessagePack

RMP is a pure Rust [MessagePack](http://msgpack.org) implementation.

[![Build Status](https://travis-ci.org/3Hren/msgpack-rust.svg?branch=master)](https://travis-ci.org/3Hren/msgpack-rust)
[![Coverage Status][coveralls-img]][coveralls-url]

This repository consists of three separate crates: the RMP core and two implementations to ease serializing and
deserializing Rust structs.

|                   | Crates.io | API Documentation |
|-------------------|-----------|-------------------|
| **rmp**           | [![][crates-rmp-img]][crates-rmp-url] | [RMP][rmp-docs-url] |
| **rmp-serialize** | [![][crates-rmp-ser-img]][crates-rmp-ser-url] | [RMP Serialize][rmp-ser-docs-url] |
| **rmp-serde**     | [![][crates-rmp-serde-img]][crates-rmp-serde-url] | [RMP Serde][rmp-serde-docs-url] |

## Usage

To use `rmp`, first add this to your `Cargo.toml`:

```toml
[dependencies.rmp]
rmp = "^0.7"
```

Then, add this to your crate root:

```rust
extern crate rmp as msgpack; // Or just `rmp`.
```

## Features

- **Convenient API**

  RMP is designed to be lightweight and straightforward. There are low-level API, which gives you
  full control on data encoding/decoding process and makes no heap allocations. On the other hand
  there are high-level API, which provides you convenient interface using Rust standard library and
  compiler reflection, allowing to encode/decode structures using `derive` attribute.

- **Zero-copy value decoding**

  RMP allows to decode bytes from a buffer in a zero-copy manner easily and blazingly fast, while Rust
  static checks guarantees that the data will be valid until buffer lives.

- **Clear error handling**

  RMP's error system guarantees that you never receive an error enum with unreachable variant.

- **Robust and tested**

  This project is developed using TDD and CI, so any found bugs will be fixed without breaking
  existing functionality.

## Examples

Let's try to encode a tuple of int and string.

```rust
extern crate rmp_serialize as msgpack;
extern crate rustc_serialize;

use rustc_serialize::Encodable;
use msgpack::Encoder;

fn main() {
    let val = (42u8, "the Answer");

    // The encoder borrows the bytearray buffer.
    let mut buf = [0u8; 13];

    val.encode(&mut Encoder::new(&mut &mut buf[..]));

    assert_eq!([0x92, 0x2a, 0xaa, 0x74, 0x68, 0x65, 0x20, 0x41, 0x6e, 0x73, 0x77, 0x65, 0x72], buf);
}
```

Now we have an encoded buffer, which we can decode the same way:

```rust
extern crate rmp_serialize as msgpack;
extern crate rustc_serialize;

use rustc_serialize::Decodable;
use msgpack::Decoder;

fn main() {
    let buf = [0x92, 0x2a, 0xaa, 0x74, 0x68, 0x65, 0x20, 0x41, 0x6e, 0x73, 0x77, 0x65, 0x72];

    let mut decoder = Decoder::new(&buf[..]);

    let res: (u8, String) = Decodable::decode(&mut decoder).unwrap();

    assert_eq!((42u8, "the Answer".to_string()), res);
}
```

RMP also allows to automatically serialize/deserialize custom structures using rustc_serialize
reflection. To enable this feature, derive RustcEncodable and RustcDecodable attributes as
shown in the following example:

```rust
extern crate rmp_serialize as msgpack;
extern crate rustc_serialize;

use rustc_serialize::{Encodable, Decodable};
use msgpack::{Encoder, Decoder};

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
struct Custom {
    id: u32,
    key: String,
}

fn main() {
    let val = Custom { id: 42u32, key: "the Answer".to_string() };

    let mut buf = [0u8; 13];

    val.encode(&mut Encoder::new(&mut &mut buf[..]));

    assert_eq!([0x92, 0x2a, 0xaa, 0x74, 0x68, 0x65, 0x20, 0x41, 0x6e, 0x73, 0x77, 0x65, 0x72], buf);

    // Now try to unpack the buffer into the initial struct.
    let mut decoder = Decoder::new(&buf[..]);
    let res: Custom = Decodable::decode(&mut decoder).ok().unwrap();

    assert_eq!(val, res);
}
```

## Versioning

This project adheres to [Semantic Versioning](http://semver.org/). However until 1.0.0 comes there
will be the following rules:

 - Any API/ABI breaking changes will be notified in the changelog explicitly and results in minor
   version bumping.
 - API extending features results in patch version bumping.
 - Non-breaking bug fixes and performance improving results in patch version bumping.

[coveralls-img]: https://coveralls.io/repos/3Hren/msgpack-rust/badge.svg?branch=master&service=github
[coveralls-url]: https://coveralls.io/github/3Hren/msgpack-rust?branch=master

[rmp-docs-url]: https://docs.rs/rmp/0.7.5/rmp/
[rmp-ser-docs-url]: https://docs.rs/rmp-serialize/0.7.0/rmp_serialize/
[rmp-serde-docs-url]: https://docs.rs/rmp-serde/0.10.0/rmp_serde/

[crates-rmp-img]: http://meritbadge.herokuapp.com/rmp
[crates-rmp-url]: https://crates.io/crates/rmp

[crates-rmp-ser-img]: http://meritbadge.herokuapp.com/rmp-serialize
[crates-rmp-ser-url]: https://crates.io/crates/rmp-serialize

[crates-rmp-serde-img]: http://meritbadge.herokuapp.com/rmp-serde
[crates-rmp-serde-url]: https://crates.io/crates/rmp-serde
