#!/bin/sh

(cd rmp && cargo test) && \
(cd rmp-serialize && cargo test) && \
(cd rmp-serde && cargo test) && \
(cd rmp-serde-tests && cargo test --features=with-syntex --no-default-features)
(cd rmp-serde-tests && rustup run nightly cargo test)
