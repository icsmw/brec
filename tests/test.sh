#!/bin/bash
set -e

cd ./stress_blocks
cargo test  --release -- --nocapture
cd ..

cd ./stress_payloads
cargo test  --release -- --nocapture
cd ..

cd ./stress_payloads_crypt
cargo test  --release -- --nocapture
cd ..

cd ./stress_packets
cargo test  --release -- --nocapture
cd ..

cd ./stress_resilient
cargo test  --release -- --nocapture
cd ..

cd ./locked_storage
cargo test  --release -- --nocapture
cd ..

cd ./observer
cargo test  --release -- --nocapture
cd ..

