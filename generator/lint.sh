#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

excluded_paths=(
    "gen_tests"
)

for integration in $(ls -1); do
    [ -d "$integration" ] || continue
    [[ " ${excluded_paths[*]} " == *" $integration "* ]] && continue

    for crate in $(ls -1 "$integration"); do
        path="$integration/$crate"
        [ -d "$path" ] || continue
        [ -f "$path/Cargo.toml" ] || continue

        echo "==> $path"
        (
            cd "$path"
            cargo +nightly clippy --tests --all --all-features -- -D warnings
            cargo fmt --all --check
        )
    done
done
