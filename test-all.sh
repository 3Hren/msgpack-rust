#!/bin/sh

echo "RMP"
(cd rmp && cargo test -- -q)
if [ $? -ne 0 ]; then
    echo "FAILED"
    exit 1
fi

echo "RMP SERIALIZE"
(cd rmp-serialize && cargo test -- -q)
if [ $? -ne 0 ]; then
    echo "FAILED"
    exit 1
fi

echo "RMP SERDE"
(cd rmp-serde && cargo test -- -q)
if [ $? -ne 0 ]; then
    echo "FAILED"
    exit 1
fi

echo "RMP SERDE CODEGEN & NIGHTLY"
(cd rmp-serde-tests && cargo test -- -q)
if [ $? -ne 0 ]; then
    echo "FAILED"
    exit 1
fi

echo "RMP VALUE"
(cd rmpv && cargo test -- -q)
if [ $? -ne 0 ]; then
    echo "FAILED"
    exit 1
fi

(cd rmpv-tests && cargo test -- -q)
if [ $? -ne 0 ]; then
    echo "FAILED"
    exit 1
fi
