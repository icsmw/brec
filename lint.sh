#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

paths=(
    "lib"
    "generator"
    "integration"
)

for path in "${paths[@]}"; do
    echo "==> $path/lint.sh"
    (
        cd "$path"
        bash lint.sh
    )
done
