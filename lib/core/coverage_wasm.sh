#!/usr/bin/env bash
set -euo pipefail

CORE_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd -- "${CORE_DIR}/../.." && pwd)"
OUT_DIR="${ROOT_DIR}/coverage_results"
LCOV_OUT="${OUT_DIR}/wasm.lcov.info"
WORKSPACE_MANIFEST="${ROOT_DIR}/Cargo.toml"
TOOLCHAIN="${TOOLCHAIN:-nightly}"
WASM_TARGET="wasm32-unknown-unknown"

# Use a dedicated coverage target directory to avoid mixing with normal builds.
export CARGO_LLVM_COV_TARGET_DIR="${ROOT_DIR}/target/llvm-cov-wasm"
export CARGO_TARGET_DIR="${CARGO_LLVM_COV_TARGET_DIR}"

mkdir -p "${OUT_DIR}"
rm -f "${LCOV_OUT}"

if ! command -v wasm-bindgen-test-runner >/dev/null 2>&1; then
  echo "wasm-bindgen-test-runner is required for wasm coverage." >&2
  echo "Install it, for example: cargo install wasm-bindgen-cli --locked" >&2
  exit 1
fi

if ! rustup target list --installed --toolchain "${TOOLCHAIN}-x86_64-unknown-linux-gnu" | grep -qx "${WASM_TARGET}"; then
  echo "Rust target ${WASM_TARGET} is not installed for toolchain ${TOOLCHAIN}." >&2
  echo "Install it with: rustup target add ${WASM_TARGET} --toolchain ${TOOLCHAIN}" >&2
  exit 1
fi

# Official wasm-bindgen-test coverage path (experimental).
export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER="wasm-bindgen-test-runner"
export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS="-Cinstrument-coverage -Zno-profiler-runtime -Clink-args=--no-gc-sections --cfg=wasm_bindgen_unstable_test_coverage"

echo "Cleaning old coverage artifacts for wasm-focused run..."
cargo +"${TOOLCHAIN}" llvm-cov clean --workspace --manifest-path "${WORKSPACE_MANIFEST}"

echo "Running wasm32 tests with coverage instrumentation..."
cargo +"${TOOLCHAIN}" llvm-cov test \
  --manifest-path "${WORKSPACE_MANIFEST}" \
  -p brec \
  --features wasm \
  --target "${WASM_TARGET}" \
  --coverage-target-only \
  --lcov \
  --output-path "${LCOV_OUT}" \
  --ignore-filename-regex '/rustc/|/\.cargo/(registry|git)/|/\.rustup/toolchains/' \
  -- --nocapture

echo "wasm-focused brec coverage report generated:"
echo "  LCOV: ${LCOV_OUT}"
echo "  Target dir: ${CARGO_TARGET_DIR}"
