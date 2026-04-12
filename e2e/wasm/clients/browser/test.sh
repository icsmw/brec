#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
WASM_DIR="$(cd -- "${SCRIPT_DIR}/../.." && pwd)"
SERVER_LOG="${SCRIPT_DIR}/.server_e2e.log"
CLIENT_LOG="${SCRIPT_DIR}/.client_e2e.log"
SERVER_PID=""
TAIL_PID=""
IMAGE_TAG="wasm-browser-e2e:local"

if ! command -v docker >/dev/null 2>&1; then
  echo "docker is required but not found in PATH" >&2
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required but not found in PATH" >&2
  exit 1
fi

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "wasm-pack is required but not found in PATH" >&2
  echo "Install it locally (e.g. cargo install wasm-pack) and rerun." >&2
  exit 1
fi

TEST_PACKAGE_COUNT="${TEST_PACKAGE_COUNT:-1000}"
SERVER_BIND_ADDR="${SERVER_BIND_ADDR:-0.0.0.0:19001}"
CLIENT_WS_ADDR="${CLIENT_WS_ADDR:-host.docker.internal:19001}"

cleanup() {
  if [[ -n "${TAIL_PID}" ]] && kill -0 "${TAIL_PID}" >/dev/null 2>&1; then
    kill "${TAIL_PID}" >/dev/null 2>&1 || true
    wait "${TAIL_PID}" >/dev/null 2>&1 || true
  fi
  if [[ -n "${SERVER_PID}" ]] && kill -0 "${SERVER_PID}" >/dev/null 2>&1; then
    kill "${SERVER_PID}" >/dev/null 2>&1 || true
    wait "${SERVER_PID}" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

echo "Checking local compilation before e2e..."
(
  cd "${WASM_DIR}"
  cargo check -p protocol --features test-utils
  cargo check -p bindings --target wasm32-unknown-unknown
  cargo test -p server --no-run
)

echo "Rebuilding binding/pkg via wasm-pack..."
(
  cd "${WASM_DIR}/binding"
  wasm-pack build --dev --target web --out-dir pkg --out-name wasmjs
)

if [[ ! -f "${WASM_DIR}/binding/pkg/wasmjs.js" ]] || [[ ! -f "${WASM_DIR}/binding/pkg/wasmjs_bg.wasm" ]]; then
  echo "binding/pkg build failed: missing wasmjs.js or wasmjs_bg.wasm" >&2
  exit 1
fi

echo "Starting local rust server test..."
(
  cd "${WASM_DIR}"
  BROWSER_E2E=1 \
  TEST_PACKAGE_COUNT="${TEST_PACKAGE_COUNT}" \
  TEST_WS_ADDR="${SERVER_BIND_ADDR}" \
  cargo test -p server server_roundtrip_binary_packets_external_client -- --nocapture
) >"${SERVER_LOG}" 2>&1 &
SERVER_PID=$!

READY=0
for _ in $(seq 1 120); do
  if ! kill -0 "${SERVER_PID}" >/dev/null 2>&1; then
    echo "Server process exited early. Log:" >&2
    cat "${SERVER_LOG}" >&2
    exit 1
  fi
  if grep -q "READY_WS_ADDR=" "${SERVER_LOG}"; then
    READY=1
    break
  fi
  sleep 1
done

if [[ "${READY}" -ne 1 ]]; then
  echo "Timeout waiting for server readiness. Log:" >&2
  cat "${SERVER_LOG}" >&2
  exit 1
fi

echo "Server is ready, tailing ${SERVER_LOG} ..."
tail -f "${SERVER_LOG}" &
TAIL_PID=$!

echo "Building browser-e2e image..."
docker build -t "${IMAGE_TAG}" -f "${WASM_DIR}/clients/browser/Dockerfile" "${WASM_DIR}"

echo "Running client e2e in container..."
docker run --rm \
  --add-host=host.docker.internal:host-gateway \
  -e CI=1 \
  -e TEST_PACKAGE_COUNT="${TEST_PACKAGE_COUNT}" \
  -e TEST_WS_ADDR="${CLIENT_WS_ADDR}" \
  "${IMAGE_TAG}" | tee "${CLIENT_LOG}"

echo "Waiting local server test to finish..."
wait "${SERVER_PID}"
SERVER_PID=""

if [[ -n "${TAIL_PID}" ]] && kill -0 "${TAIL_PID}" >/dev/null 2>&1; then
  kill "${TAIL_PID}" >/dev/null 2>&1 || true
  wait "${TAIL_PID}" >/dev/null 2>&1 || true
  TAIL_PID=""
fi

SERVER_SUMMARY_LINE="$(grep 'SERVER_SUMMARY ' "${SERVER_LOG}" | tail -n 1 || true)"
CLIENT_SUMMARY_LINE="$(grep 'CLIENT_SUMMARY ' "${CLIENT_LOG}" | tail -n 1 || true)"

echo "----- E2E SUMMARY -----"
if [[ -n "${SERVER_SUMMARY_LINE}" ]]; then
  echo "${SERVER_SUMMARY_LINE}"
else
  echo "SERVER_SUMMARY missing"
fi
if [[ -n "${CLIENT_SUMMARY_LINE}" ]]; then
  echo "${CLIENT_SUMMARY_LINE}"
else
  echo "CLIENT_SUMMARY missing"
fi

echo "E2E PASSED"
