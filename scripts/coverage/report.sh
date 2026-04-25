#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
IN_LCOV="${ROOT_DIR}/coverage_results/merged.lcov"

OUT_DIR="${ROOT_DIR}/coverage_results"
OUT_BREC_LCOV="${OUT_DIR}/brec.lcov"
OUT_BREC_MACROS_LCOV="${OUT_DIR}/brec_macros.lcov"
OUT_WASM_LCOV="${OUT_DIR}/brec_in_wasm_lib.lcov"

HTML_ROOT="${OUT_DIR}/html"
HTML_BREC_DIR="${HTML_ROOT}/brec"
HTML_BREC_MACRO_DIR="${HTML_ROOT}/brec_macro"
HTML_WASM_DIR="${HTML_ROOT}/brec_in_wasm_lib"

if [[ ! -f "${IN_LCOV}" ]]; then
  echo "Merged LCOV not found: ${IN_LCOV}" >&2
  exit 1
fi

mkdir -p "${OUT_DIR}" "${HTML_ROOT}"
rm -f "${OUT_BREC_LCOV}" "${OUT_BREC_MACROS_LCOV}" "${OUT_WASM_LCOV}"
rm -rf "${HTML_BREC_DIR}" "${HTML_BREC_MACRO_DIR}" "${HTML_WASM_DIR}"

echo "Extracting brec coverage..."
lcov \
  --quiet \
  --extract "${IN_LCOV}" "${ROOT_DIR}/lib/core/*" \
  --output-file "${OUT_BREC_LCOV}"

echo "Extracting brec_macros coverage..."
lcov \
  --quiet \
  --extract "${IN_LCOV}" "${ROOT_DIR}/generator/macros/*" \
  --output-file "${OUT_BREC_MACROS_LCOV}"

echo "Extracting brec_in_wasm_lib coverage..."
lcov \
  --quiet \
  --extract "${IN_LCOV}" "${ROOT_DIR}/integration/wasm/lib/*" \
  --output-file "${OUT_WASM_LCOV}"

echo "Generating HTML report for brec..."
genhtml \
  --quiet \
  "${OUT_BREC_LCOV}" \
  --output-directory "${HTML_BREC_DIR}"

echo "Generating HTML report for brec_macros..."
genhtml \
  --quiet \
  "${OUT_BREC_MACROS_LCOV}" \
  --output-directory "${HTML_BREC_MACRO_DIR}"

echo "Generating HTML report for brec_in_wasm_lib..."
genhtml \
  --quiet \
  "${OUT_WASM_LCOV}" \
  --output-directory "${HTML_WASM_DIR}"

echo
echo "brec summary:"
lcov --summary "${OUT_BREC_LCOV}"

echo
echo "brec_macros summary:"
lcov --summary "${OUT_BREC_MACROS_LCOV}"

echo
echo "brec_in_wasm_lib summary:"
lcov --summary "${OUT_WASM_LCOV}"

echo
echo "Reports generated:"
echo "  LCOV brec: ${OUT_BREC_LCOV}"
echo "  LCOV brec_macros: ${OUT_BREC_MACROS_LCOV}"
echo "  LCOV brec_in_wasm_lib: ${OUT_WASM_LCOV}"
echo "  HTML brec: ${HTML_BREC_DIR}/index.html"
echo "  HTML brec_macro: ${HTML_BREC_MACRO_DIR}/index.html"
echo "  HTML brec_in_wasm_lib: ${HTML_WASM_DIR}/index.html"
