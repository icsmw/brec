#!/bin/bash
set -e

cd ./brec
sh test.sh
cd ..

cd ./examples
sh test.sh
cd ..

cd ./tests
sh test.sh
cd ..

cd ./measurements
sh test.sh
cd ..