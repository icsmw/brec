#!/bin/bash
set -e

cd ./lib/core
sh test.sh
cd ../..

cd ./generator/macros
cargo test --all-features
cd ../..

cd ./examples
sh test.sh
cd ..

cd ./tests
sh test.sh
cd ..

cd ./measurements
sh test.sh
cd ..