#!/bin/bash
set -e

cd ./brec
sh test.sh
cd ..

cd ./brec_macros
cargo test --all-features
cd ..

cd ./examples
sh test.sh
cd ..

cd ./tests
sh test.sh
cd ..

cd ./measurements
sh test.sh
cd ..