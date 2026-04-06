#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPTS_DIR="${ROOT_DIR}/scripts/coverage"

echo "Cleaning coverage artifacts for: ${ROOT_DIR}/tests/Cargo.toml"
cargo llvm-cov clean --workspace --manifest-path "${ROOT_DIR}/tests/Cargo.toml"

export COVERAGE_WORKSPACE="${ROOT_DIR}/tests"

export COVERAGE_TARGET="stress_packets"
export COVERAGE_TARGET_DIR="${ROOT_DIR}/tests/stress_packets"
bash "${SCRIPTS_DIR}/coverage.sh"

export COVERAGE_TARGET="stress_blocks"
export COVERAGE_TARGET_DIR="${ROOT_DIR}/tests/stress_blocks"
bash "${SCRIPTS_DIR}/coverage.sh"

export COVERAGE_TARGET="stress_payloads"
export COVERAGE_TARGET_DIR="${ROOT_DIR}/tests/stress_payloads"
bash "${SCRIPTS_DIR}/coverage.sh"

export COVERAGE_TARGET="stress_payloads_crypt"
export COVERAGE_TARGET_DIR="${ROOT_DIR}/tests/stress_payloads_crypt"
bash "${SCRIPTS_DIR}/coverage.sh"

export COVERAGE_TARGET="stress_resilient"
export COVERAGE_TARGET_DIR="${ROOT_DIR}/tests/stress_resilient"
bash "${SCRIPTS_DIR}/coverage.sh"

export COVERAGE_TARGET="observer"
export COVERAGE_TARGET_DIR="${ROOT_DIR}/tests/observer"
bash "${SCRIPTS_DIR}/coverage.sh"

export COVERAGE_TARGET="locked_storage"
export COVERAGE_TARGET_DIR="${ROOT_DIR}/tests/locked_storage"
bash "${SCRIPTS_DIR}/coverage.sh"