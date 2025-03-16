#!/bin/bash

TESTS_DIR="../../../gen_tests"

if [ -d "$TESTS_DIR" ]; then
    echo "Removing previous tests"
    rm -rf "$TESTS_DIR"
fi

echo "This test may take a significant amount of time, at least several minutes depending on the configuration. In the first stage, crates with a random protocol will be generated, then each crate will be compiled (which takes most of the time) and executed."
read -p "Do you want to continue? (y/n): " answer

if [[ "$answer" != "y" ]]; then
    exit 0
fi

echo "Generate tests"

cargo test --release -- --nocapture 

for folder in "$TESTS_DIR"/*/; do
    if [ -d "$folder" ]; then
        cd "$folder" || { echo "Fail to enter into $folder"; exit 1; }

        echo "Visit $folder"

        cargo run --release
        
        echo "Test is OK. Executable is:"
        
        ls -l ./target/release/test_case

        cd - > /dev/null
    fi
done