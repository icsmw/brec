#!/bin/bash
set -e

cd ./stress_blocks
sh ./stress.sh
cd ..

cd ./stress_payloads
sh ./stress.sh
cd ..

cd ./stress_payloads_crypt
sh ./stress.sh
cd ..

cd ./stress_resilient
sh ./stress.sh
cd ..

cd ./stress_packets
sh ./stress.sh
cd ..

