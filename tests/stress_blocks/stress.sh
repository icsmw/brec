#!/bin/bash

export BREC_STRESS_BLOCKS_CASES=500
export BREC_STRESS_BLOCKS_MAX_COUNT=2000

cargo test  --release -- --nocapture
