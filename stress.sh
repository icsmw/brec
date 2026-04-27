#!/bin/bash
set -e

cd ./tests
sh stress.sh
cd ..

cd ./generator/macros
sh stress.sh 
cd ../..
