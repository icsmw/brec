#!/bin/bash

cd ./brec
cargo +nightly clippy --tests --all --all-features -- -D warnings
cargo test --release
cd ..

cd ./brec_macros
cargo +nightly clippy --tests --all --all-features -- -D warnings
cargo test --release
cd ..
