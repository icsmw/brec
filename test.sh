#!/bin/bash
set -e

cd ./brec
cargo test -- --nocapture
cargo test --features locked_storage -- --nocapture
cargo test --features observer -- --nocapture

cd ..

cd ./tests
sh test.sh
