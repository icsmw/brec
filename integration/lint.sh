#!/bin/bash

cd ./csharp/gen
cargo +nightly clippy --tests --all --all-features -- -D warnings
cargo fmt --all --check
cd ../..

cd ./csharp/lib
cargo +nightly clippy --tests --all --all-features -- -D warnings
cargo fmt --all --check
cd ../..

cd ./java/gen
cargo +nightly clippy --tests --all --all-features -- -D warnings
cargo fmt --all --check
cd ../..

cd ./java/lib
cargo +nightly clippy --tests --all --all-features -- -D warnings
cargo fmt --all --check
cd ../..

cd ./java/macro
cargo +nightly clippy --tests --all --all-features -- -D warnings
cargo fmt --all --check
cd ../..

cd ./node/gen
cargo +nightly clippy --tests --all --all-features -- -D warnings
cargo fmt --all --check
cd ../..

cd ./node/macro
cargo +nightly clippy --tests --all --all-features -- -D warnings
cargo fmt --all --check
cd ../..

cd ./wasm/gen
cargo +nightly clippy --tests --all --all-features -- -D warnings
cargo fmt --all --check
cd ../..

cd ./wasm/macro
cargo +nightly clippy --tests --all --all-features -- -D warnings
cargo fmt --all --check
cd ../..

cd ./wasm/lib
cargo +nightly clippy --tests --all --all-features -- -D warnings
cargo fmt --all --check
cd ../..
