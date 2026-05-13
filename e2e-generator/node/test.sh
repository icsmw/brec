#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
SERVER_LOG="${ROOT_DIR}/.server_e2e.log"
CLIENT_LOG="${ROOT_DIR}/.client_e2e.log"
SERVER_PID=""
TAIL_PID=""
IMAGE_TAG="node-e2e:local"
SCHEME_PATH="${ROOT_DIR}/protocol/target/brec.scheme.json"
GENERATED_NPM_DIR="${ROOT_DIR}/generated/npm"
LOCAL_CARGO_DEPS="${ROOT_DIR}/local.deps.cargo.toml"

if ! command -v docker >/dev/null 2>&1; then
  echo "docker is required but not found in PATH" >&2
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required but not found in PATH" >&2
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

echo "Cleaning generated Node artifacts..."
rm -rf "${ROOT_DIR}/bindings" "${GENERATED_NPM_DIR}"
rm -f "${SCHEME_PATH}"

echo "Generating protocol scheme..."
(
  cd "${ROOT_DIR}"
  cargo clean -p protocol
  cargo check -p protocol --features test-utils
)

if [[ ! -f "${SCHEME_PATH}" ]]; then
  echo "Failed to generate scheme at ${SCHEME_PATH}" >&2
  exit 1
fi

echo "Generating Node bindings crate and npm package..."
(
  cd "${ROOT_DIR}"
  cargo run --manifest-path "${ROOT_DIR}/../../Cargo.toml" -p brec_node_cli -- \
    --scheme "${SCHEME_PATH}" \
    --protocol "${ROOT_DIR}/protocol" \
    --bindings-out "${ROOT_DIR}/bindings" \
    --npm-out "${GENERATED_NPM_DIR}" \
    --cargo-deps "${LOCAL_CARGO_DEPS}"
)

echo "Checking local rust compilation before e2e..."
(
  cd "${ROOT_DIR}"
  cargo check -p protocol --features test-utils
  cargo test -p server --no-run
)

echo "Compiling TypeScript node client..."
(
  cd "${ROOT_DIR}/client"
  rm -rf dist
  npm install --no-package-lock
  npm run build
)

echo "Starting local rust server external-client test..."
(
  cd "${ROOT_DIR}"
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

echo "Building node client image..."
docker build -t "${IMAGE_TAG}" -f "${ROOT_DIR}/client/Dockerfile" "${ROOT_DIR}"

echo "Running node client in container..."
DOCKER_RUN_ARGS=(
  --rm
  --add-host=host.docker.internal:host-gateway
  -e TEST_PACKAGE_COUNT="${TEST_PACKAGE_COUNT}"
  -e TEST_WS_ADDR="${CLIENT_WS_ADDR}"
)

if [[ -n "${LLVM_PROFILE_FILE:-}" ]]; then
  DOCKER_RUN_ARGS+=(-e "LLVM_PROFILE_FILE=${LLVM_PROFILE_FILE}")
fi

if [[ -n "${CARGO_LLVM_COV_TARGET_DIR:-}" ]]; then
  mkdir -p "${CARGO_LLVM_COV_TARGET_DIR}"
  DOCKER_RUN_ARGS+=(-v "${CARGO_LLVM_COV_TARGET_DIR}:${CARGO_LLVM_COV_TARGET_DIR}")
fi

docker run "${DOCKER_RUN_ARGS[@]}" "${IMAGE_TAG}" | tee "${CLIENT_LOG}"

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
