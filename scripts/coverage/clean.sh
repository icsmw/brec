#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"

mapfile -t COVERAGE_DIRS < <(find "${ROOT_DIR}" -type d -name "coverage_results" | sort)

if [[ ${#COVERAGE_DIRS[@]} -eq 0 ]]; then
  echo "No coverage_results directories found under ${ROOT_DIR}"
  exit 0
fi

for dir in "${COVERAGE_DIRS[@]}"; do
  rm -rf "${dir}"
  echo "Removed: ${dir}"
done

echo "Removed ${#COVERAGE_DIRS[@]} coverage_results directories."
