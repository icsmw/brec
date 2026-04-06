#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${COVERAGE_ROOT:-$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)}"
OUT_DIR="${ROOT_DIR}/coverage_results"
SOURCES_LIST="${OUT_DIR}/lcov_sources.list"
MERGED_LCOV="${OUT_DIR}/merged.lcov"

mkdir -p "${OUT_DIR}"
rm -f "${SOURCES_LIST}" "${MERGED_LCOV}"

if ! command -v lcov >/dev/null 2>&1; then
  echo "lcov is not installed. Please install it to merge tracefiles." >&2
  exit 1
fi

find "${ROOT_DIR}" -type f -path '*/coverage_results/lcov.info' 2>/dev/null | sort -u > "${SOURCES_LIST}"

if [[ ! -s "${SOURCES_LIST}" ]]; then
  echo "No lcov.info files found under ${ROOT_DIR}" >&2
  exit 1
fi

mapfile -t SOURCES < "${SOURCES_LIST}"
cp "${SOURCES[0]}" "${MERGED_LCOV}"

for ((i = 1; i < ${#SOURCES[@]}; i++)); do
  NEXT="${SOURCES[i]}"
  TMP_MERGED="${MERGED_LCOV}.tmp"
  lcov \
    --quiet \
    --add-tracefile "${MERGED_LCOV}" \
    --add-tracefile "${NEXT}" \
    --output-file "${TMP_MERGED}"
  mv "${TMP_MERGED}" "${MERGED_LCOV}"
done

echo "Merged LCOV report generated:"
echo "  Sources: ${SOURCES_LIST}"
echo "  Output: ${MERGED_LCOV}"
