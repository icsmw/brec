#!/bin/bash

cd ./brec
cargo +nightly clippy --tests --all --all-features -- -D warnings
cargo fmt --all --check
cd ..

cd ./brec_macros
cargo +nightly clippy --tests --all --all-features -- -D warnings
cargo fmt --all --check
cd ..

cd ./brec_consts
cargo +nightly clippy --tests --all --all-features -- -D warnings
cargo fmt --all --check
cd ..