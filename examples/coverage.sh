#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPTS_DIR="${ROOT_DIR}/scripts/coverage"

echo "Cleaning coverage artifacts for: ${ROOT_DIR}/examples/Cargo.toml"
cargo llvm-cov clean --workspace --manifest-path "${ROOT_DIR}/examples/Cargo.toml"

export COVERAGE_WORKSPACE="${ROOT_DIR}/examples"

export COVERAGE_TARGET="bincode_feat"
export COVERAGE_TARGET_DIR="${ROOT_DIR}/examples/bincode_feat"
bash "${SCRIPTS_DIR}/coverage.sh"

export COVERAGE_TARGET="crypt"
export COVERAGE_TARGET_DIR="${ROOT_DIR}/examples/crypt"
bash "${SCRIPTS_DIR}/coverage.sh"

export COVERAGE_TARGET="ctx"
export COVERAGE_TARGET_DIR="${ROOT_DIR}/examples/ctx"
bash "${SCRIPTS_DIR}/coverage.sh"

export COVERAGE_TARGET="packets"
export COVERAGE_TARGET_DIR="${ROOT_DIR}/examples/packets"
bash "${SCRIPTS_DIR}/coverage.sh"
