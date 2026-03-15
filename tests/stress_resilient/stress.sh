#!/bin/bash

export BREC_STRESS_PACKETS_CASES=200
export BREC_STRESS_PACKETS_MAX_COUNT=2000

cargo test  --release -- --nocapture
