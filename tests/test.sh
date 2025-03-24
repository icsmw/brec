#!/bin/bash

cd ./stress_blocks
cargo test  --release -- --nocapture
cd ..

cd ./stress_payloads
cargo test  --release -- --nocapture
cd ..

cd ./stress_packets
cargo test  --release -- --nocapture
cd ..
