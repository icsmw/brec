#!/bin/bash
set -e

cd ./wasm
bash ./test.sh
cd ..

cd ./node
bash ./test.sh
cd ..

cd ./java
bash ./test.sh
cd ..

cd ./csharp
bash ./test.sh
cd ..
