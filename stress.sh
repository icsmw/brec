#!/bin/bash

cd ./tests
sh stress.sh
cd ..

cd ./brec_macros
sh stress.sh 
cd ..
