#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
SCRIPTS_DIR="${ROOT_DIR}/scripts/coverage"

bash "${SCRIPTS_DIR}/install.sh"

bash "${SCRIPTS_DIR}/clean.sh"

bash "${ROOT_DIR}/tests/coverage.sh"

bash "${ROOT_DIR}/examples/coverage.sh"

bash "${ROOT_DIR}/e2e/node/coverage.sh"

export COVERAGE_WORKSPACE="${ROOT_DIR}"

export COVERAGE_TARGET="brec"
export COVERAGE_TARGET_DIR="${ROOT_DIR}/brec"
bash "${SCRIPTS_DIR}/coverage.sh"

export COVERAGE_TARGET="brec_macros"
export COVERAGE_TARGET_DIR="${ROOT_DIR}/brec_macros"
bash "${SCRIPTS_DIR}/coverage.sh"

bash "${SCRIPTS_DIR}/merge.sh"
bash "${SCRIPTS_DIR}/report.sh"
