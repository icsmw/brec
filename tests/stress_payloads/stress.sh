#!/bin/bash

export BREC_STRESS_PAYLOADS_CASES=500
export BREC_STRESS_PAYLOADS_MAX_COUNT=2000

cargo test  --release -- --nocapture
