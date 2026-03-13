#!/bin/bash
set -e

cargo test --release --workspace -- --nocapture
