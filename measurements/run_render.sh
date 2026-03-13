#!/bin/bash
set -euo pipefail

MEASUREMENTS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RENDER_DIR="${MEASUREMENTS_DIR}/render"
VENV_DIR="${RENDER_DIR}/.venv"
REQ_FILE="${RENDER_DIR}/requirements.txt"
RENDER_SCRIPT="${RENDER_DIR}/render.py"

if [[ ! -d "${VENV_DIR}" ]]; then
  python3 -m venv "${VENV_DIR}"
fi

if ! "${VENV_DIR}/bin/python" -c "import matplotlib" >/dev/null 2>&1; then
  "${VENV_DIR}/bin/python" -m pip install -r "${REQ_FILE}"
fi

# Keep defaults aligned with measurements/runner/measurements_light.sh.
BREC_TEST_MEASUREMENTS_ITERATIONS=${BREC_TEST_MEASUREMENTS_ITERATIONS:-10} \
BREC_TEST_MEASUREMENTS_ITERATIONS_CRYPT=${BREC_TEST_MEASUREMENTS_ITERATIONS_CRYPT:-10} \
BREC_TEST_MEASUREMENTS_PACKAGES=${BREC_TEST_MEASUREMENTS_PACKAGES:-500} \
BREC_TEST_MEASUREMENTS_RECORDS=${BREC_TEST_MEASUREMENTS_RECORDS:-5000} \
BREC_SESSION_REUSE_LIMIT=${BREC_SESSION_REUSE_LIMIT:-500} \
BREC_DECRYPT_CACHE_LIMIT=${BREC_DECRYPT_CACHE_LIMIT:-500} \
"${VENV_DIR}/bin/python" "${RENDER_SCRIPT}"
