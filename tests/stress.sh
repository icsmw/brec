#!/bin/bash

cd ./stress_blocks
sh ./stress.sh
cd ..

cd ./stress_payloads
sh ./stress.sh
cd ..

cd ./stress_packets
sh ./stress.sh
cd ..

cd ./measurements
cargo test  --release -- --nocapture
cd ..
