#!/bin/bash

cd ./brec
cargo +nightly clippy --tests --all --all-features -- -D warnings
cd ..

cd ./brec_macros
cargo +nightly clippy --tests --all --all-features -- -D warnings
cd ..
