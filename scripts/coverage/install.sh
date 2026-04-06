#!/usr/bin/env bash
set -euo pipefail

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

echo "Ensuring llvm-tools-preview is installed..."
rustup component add llvm-tools-preview

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
