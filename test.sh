#!/bin/bash
set -e

cd ./brec
cargo test --features locked_storage -- --nocapture
cd ..

cd ./tests
sh test.sh
