#!/bin/bash
set -e

cd ./wasm_browser
bash ./test.sh
cd ..

cd ./node
bash ./test.sh
cd ..
