#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

(
  cd "${SCRIPT_DIR}/clients/browser"
  bash ./test.sh
)

(
  cd "${SCRIPT_DIR}/clients/node"
  bash ./test.sh
)
