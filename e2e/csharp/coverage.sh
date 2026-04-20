#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
E2E_DIR="${ROOT_DIR}/e2e/csharp"
OUT_DIR="${E2E_DIR}/coverage_results"
LCOV_OUT="${OUT_DIR}/lcov.info"
LCOV_DEP_OUT="${OUT_DIR}/lcov.dep.brec.info"
WORKSPACE_MANIFEST="${E2E_DIR}/Cargo.toml"

mkdir -p "${OUT_DIR}"
rm -f "${LCOV_OUT}" "${LCOV_DEP_OUT}"

echo "Preparing coverage environment for e2e/csharp..."
ROOT_ENV="$(cargo llvm-cov show-env --sh --manifest-path "${ROOT_DIR}/Cargo.toml")"
WORKSPACE_ENV="$(cargo llvm-cov show-env --sh --manifest-path "${WORKSPACE_MANIFEST}")"

ROOT_CRATES="$(printf '%s\n' "${ROOT_ENV}" | sed -n "s/^export __CARGO_LLVM_COV_RUSTC_WRAPPER_CRATE_NAMES=//p" | tr -d "'\"")"
WORKSPACE_CRATES="$(printf '%s\n' "${WORKSPACE_ENV}" | sed -n "s/^export __CARGO_LLVM_COV_RUSTC_WRAPPER_CRATE_NAMES=//p" | tr -d "'\"")"

source <(printf '%s\n' "${ROOT_ENV}")

if [[ -n "${ROOT_CRATES}" && -n "${WORKSPACE_CRATES}" ]]; then
  export __CARGO_LLVM_COV_RUSTC_WRAPPER_CRATE_NAMES="${ROOT_CRATES},${WORKSPACE_CRATES}"
fi

export CARGO_TARGET_DIR="${CARGO_LLVM_COV_TARGET_DIR}"

echo "Cleaning old coverage artifacts for e2e/csharp workspace..."
cargo llvm-cov clean --workspace --manifest-path "${WORKSPACE_MANIFEST}"

echo "Running csharp e2e with coverage instrumentation..."
(
  cd "${E2E_DIR}"
  BINDINGS_PROFILE=debug bash "./test.sh"
)

echo "Generating e2e/csharp lcov report..."
cargo llvm-cov report \
  --manifest-path "${WORKSPACE_MANIFEST}" \
  --lcov \
  --output-path "${LCOV_OUT}" \
  --ignore-filename-regex '/rustc/|/\.cargo/(registry|git)/|/\.rustup/toolchains/'

echo "Generating dependency lcov report for brec..."
cargo llvm-cov report \
  --manifest-path "${WORKSPACE_MANIFEST}" \
  --dep-coverage brec \
  --lcov \
  --output-path "${LCOV_DEP_OUT}" \
  --ignore-filename-regex '/rustc/|/\.cargo/(registry|git)/|/\.rustup/toolchains/'

if [[ -s "${LCOV_DEP_OUT}" ]]; then
  if command -v lcov >/dev/null 2>&1; then
    echo "Merging dependency report into e2e/csharp lcov..."
    TMP_MERGED="${LCOV_OUT}.tmp"
    lcov \
      --quiet \
      --add-tracefile "${LCOV_OUT}" \
      --add-tracefile "${LCOV_DEP_OUT}" \
      --output-file "${TMP_MERGED}"
    mv "${TMP_MERGED}" "${LCOV_OUT}"
  else
    echo "lcov not found; keeping dependency report as separate file: ${LCOV_DEP_OUT}" >&2
  fi
else
  echo "Dependency report for brec is empty (no tracefiles matched --dep-coverage)." >&2
fi

echo "e2e/csharp coverage report generated:"
echo "  LCOV: ${LCOV_OUT}"
if [[ -f "${LCOV_DEP_OUT}" ]]; then
  echo "  LCOV dep(brec): ${LCOV_DEP_OUT}"
fi
