#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
IN_LCOV="${ROOT_DIR}/coverage_results/merged.lcov"

OUT_DIR="${ROOT_DIR}/coverage_results"
HTML_ROOT="${OUT_DIR}/html"

REPORT_NAMES=(
  "brec"
  "brec_macros"
  "brec_in_node_lib"
  "brec_in_java_lib"
  "brec_in_csharp_lib"
  "brec_in_wasm_lib"
  "integration_libs"
  "integrations"
)

REPORT_PATTERNS=(
  "${ROOT_DIR}/lib/core/*"
  "${ROOT_DIR}/generator/macros/*"
  "${ROOT_DIR}/integration/node/lib/*"
  "${ROOT_DIR}/integration/java/lib/*"
  "${ROOT_DIR}/integration/csharp/lib/*"
  "${ROOT_DIR}/integration/wasm/lib/*"
  "${ROOT_DIR}/integration/node/lib/* ${ROOT_DIR}/integration/java/lib/* ${ROOT_DIR}/integration/csharp/lib/* ${ROOT_DIR}/integration/wasm/lib/*"
  "${ROOT_DIR}/integration/*/gen/* ${ROOT_DIR}/integration/*/lib/* ${ROOT_DIR}/integration/*/macro/*"
)

if [[ ! -f "${IN_LCOV}" ]]; then
  echo "Merged LCOV not found: ${IN_LCOV}" >&2
  exit 1
fi

mkdir -p "${OUT_DIR}" "${HTML_ROOT}"

for name in "${REPORT_NAMES[@]}"; do
  rm -f "${OUT_DIR}/${name}.lcov"
  rm -rf "${HTML_ROOT}/${name}"
done
rm -rf "${HTML_ROOT}/brec_macro"

for i in "${!REPORT_NAMES[@]}"; do
  name="${REPORT_NAMES[i]}"
  output_lcov="${OUT_DIR}/${name}.lcov"
  output_html="${HTML_ROOT}/${name}"

  read -r -a patterns <<< "${REPORT_PATTERNS[i]}"

  echo "Extracting ${name} coverage..."
  lcov \
    --quiet \
    --extract "${IN_LCOV}" "${patterns[@]}" \
    --output-file "${output_lcov}"

  echo "Generating HTML report for ${name}..."
  genhtml \
    --quiet \
    "${output_lcov}" \
    --output-directory "${output_html}"

  echo
  echo "${name} summary:"
  lcov --summary "${output_lcov}"
done

echo
echo "Reports generated:"
for name in "${REPORT_NAMES[@]}"; do
  echo "  LCOV ${name}: ${OUT_DIR}/${name}.lcov"
  echo "  HTML ${name}: ${HTML_ROOT}/${name}/index.html"
done
