#!/bin/sh

echo "RMP"
cd rmp && cargo test -- -q && cd ..

echo "RMP SERIALIZE"
cd rmp-serialize && cargo test -- -q && cd ..

echo "RMP SERDE"
cd rmp-serde && cargo test -- -q && cd ..

echo "RMP SERDE CODEGEN & NIGHTLY"
cd rmp-serde-tests && cargo test --features=with-syntex --no-default-features -- -q && rustup run nightly cargo test -- -q && cd ..

echo "RMP VALUE"
cd rmpv && cargo test -- -q && cd .. && \
cd rmpv-tests && cargo test --features=with-serde -- -q && cd ..
