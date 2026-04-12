#!/bin/bash
set -e

cd ./wasm
bash ./test.sh
cd ..

cd ./node
bash ./test.sh
cd ..
