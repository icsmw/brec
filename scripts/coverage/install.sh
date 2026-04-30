#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
LOCK_FILE="${ROOT_DIR}/Cargo.lock"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found in PATH" >&2
  exit 1
fi

if ! command -v rustup >/dev/null 2>&1; then
  echo "rustup not found in PATH" >&2
  exit 1
fi

if ! cargo llvm-cov --version >/dev/null 2>&1; then
  echo "Installing cargo-llvm-cov..."
  cargo install cargo-llvm-cov --locked
fi

echo "Ensuring nightly toolchain is installed..."
rustup toolchain install nightly

echo "Ensuring llvm-tools-preview is installed for stable and nightly..."
rustup component add llvm-tools-preview
rustup component add llvm-tools-preview --toolchain nightly

echo "Ensuring wasm32 target is installed for nightly..."
rustup target add wasm32-unknown-unknown --toolchain nightly

WASM_BINDGEN_VERSION="$(
  awk '
    $0 == "[[package]]" { in_pkg = 0 }
    $0 == "name = \"wasm-bindgen\"" { in_pkg = 1; next }
    in_pkg && /^version = / {
      gsub(/"/, "", $3)
      print $3
      exit
    }
  ' "${LOCK_FILE}"
)"

if [[ -z "${WASM_BINDGEN_VERSION}" ]]; then
  echo "Unable to determine wasm-bindgen version from ${LOCK_FILE}" >&2
  exit 1
fi

echo "Installing wasm-bindgen CLI ${WASM_BINDGEN_VERSION} (includes wasm-bindgen-test-runner)..."
cargo install wasm-bindgen-cli --locked --version "${WASM_BINDGEN_VERSION}"

if ! command -v lcov >/dev/null 2>&1; then
  echo "lcov not found in PATH, installing..."
  if command -v apt-get >/dev/null 2>&1; then
    sudo apt-get update
    sudo apt-get install -y lcov
  elif command -v dnf >/dev/null 2>&1; then
    sudo dnf install -y lcov
  elif command -v pacman >/dev/null 2>&1; then
    sudo pacman -Sy --noconfirm lcov
  elif command -v zypper >/dev/null 2>&1; then
    sudo zypper --non-interactive install lcov
  elif command -v brew >/dev/null 2>&1; then
    brew install lcov
  else
    echo "Unable to install lcov automatically: unsupported package manager." >&2
    echo "Please install lcov manually and rerun this script." >&2
    exit 1
  fi
fi

echo "Coverage toolchain is installed."
