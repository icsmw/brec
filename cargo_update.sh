#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required but not found in PATH" >&2
  exit 1
fi

mapfile -t LOCK_DIRS < <(
  find "${ROOT_DIR}" -name Cargo.lock -type f -print0 \
    | xargs -0 -n1 dirname \
    | sort -u
)

if [[ "${#LOCK_DIRS[@]}" -eq 0 ]]; then
  echo "No Cargo.lock files found under ${ROOT_DIR}"
  exit 0
fi

echo "Found ${#LOCK_DIRS[@]} directories with Cargo.lock"

for dir in "${LOCK_DIRS[@]}"; do
  echo "=== cargo update: ${dir} ==="
  (
    cd "${dir}"
    cargo clean
    cargo update
  )
done

echo "Done"
