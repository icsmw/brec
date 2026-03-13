#!/bin/bash

if ! command -v wasm-pack &> /dev/null; then
  echo "wasm-pack not found. Installing..."
  curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
else
  echo "wasm-pack is already installed."
fi

# Check nodejs target
wasm-pack build --target nodejs

# Check bundler
wasm-pack build --target bundler

# Check web target
wasm-pack build --target web