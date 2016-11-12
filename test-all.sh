#!/bin/sh

(cd rmp && cargo test -- -q) && \

(cd rmp-serialize && cargo test -- -q) && \

(cd rmp-serde && cargo test -- -q) && \
(cd rmp-serde-tests && cargo test --features=with-syntex --no-default-features -- -q)
(cd rmp-serde-tests && rustup run nightly cargo test -- -q)
