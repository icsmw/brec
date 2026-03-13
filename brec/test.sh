#!/bin/bash
set -e

cargo test -- --nocapture
cargo test --features locked_storage -- --nocapture
cargo test --features observer -- --nocapture
cargo test --features crypt -- --nocapture
cargo test --features bincode -- --nocapture
