#!/usr/bin/env bash
set -euo pipefail

if [[ -z "${COVERAGE_TARGET:-}" ]]; then
  echo "COVERAGE_TARGET environment variable is not set" >&2
  exit 1
fi

if [[ -z "${COVERAGE_TARGET_DIR:-}" ]]; then
  echo "COVERAGE_TARGET_DIR environment variable is not set" >&2
  exit 1
fi

if [[ -z "${COVERAGE_WORKSPACE:-}" ]]; then
  echo "COVERAGE_WORKSPACE environment variable is not set" >&2
  exit 1
fi

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${COVERAGE_TARGET_DIR}/coverage_results"
LCOV_OUT="${OUT_DIR}/lcov.info"
WORKSPACE_MANIFEST="${COVERAGE_WORKSPACE}/Cargo.toml"

mkdir -p "${OUT_DIR}"
rm -f "${LCOV_OUT}"

echo "Preparing external coverage environment..."
ROOT_ENV="$(cargo llvm-cov show-env --sh --manifest-path "${ROOT_DIR}/Cargo.toml")"
WORKSPACE_ENV="$(cargo llvm-cov show-env --sh --manifest-path "${WORKSPACE_MANIFEST}")"

ROOT_CRATES="$(printf '%s\n' "${ROOT_ENV}" | sed -n "s/^export __CARGO_LLVM_COV_RUSTC_WRAPPER_CRATE_NAMES=//p" | tr -d "'\"")"
WORKSPACE_CRATES="$(printf '%s\n' "${WORKSPACE_ENV}" | sed -n "s/^export __CARGO_LLVM_COV_RUSTC_WRAPPER_CRATE_NAMES=//p" | tr -d "'\"")"

source <(printf '%s\n' "${ROOT_ENV}")

if [[ -n "${ROOT_CRATES}" && -n "${WORKSPACE_CRATES}" ]]; then
  export __CARGO_LLVM_COV_RUSTC_WRAPPER_CRATE_NAMES="${ROOT_CRATES},${WORKSPACE_CRATES}"
fi

# Keep build/test/report in the same target dir from show-env.
export CARGO_TARGET_DIR="${CARGO_LLVM_COV_TARGET_DIR}"

echo "Cleaning old coverage artifacts..."
cargo llvm-cov clean --workspace --manifest-path "${WORKSPACE_MANIFEST}"

echo "Running ${COVERAGE_TARGET} tests with coverage..."
cargo test \
  --manifest-path "${WORKSPACE_MANIFEST}" \
  -p "${COVERAGE_TARGET}" \
  --all-features \
  -- --nocapture

echo "Generating lcov report..."
cargo llvm-cov report \
  --manifest-path "${WORKSPACE_MANIFEST}" \
  --lcov \
  --output-path "${LCOV_OUT}" \
  --ignore-filename-regex '/rustc/|/\.cargo/(registry|git)/|/\.rustup/toolchains/'

echo "${COVERAGE_TARGET} coverage report generated:"
echo "  LCOV: ${LCOV_OUT}"
