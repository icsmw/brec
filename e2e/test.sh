#!/bin/bash
set -e

cd ./wasm_browser
sh ./test.sh
cd ..

cd ./node
sh ./test.sh
cd ..
