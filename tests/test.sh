#!/bin/bash
set -e

for manifest in */Cargo.toml; do
    crate_dir="$(dirname "$manifest")"
    echo "Running tests in $crate_dir"
    (
        cd "$crate_dir"
        cargo test --release -- --nocapture
    )
done
