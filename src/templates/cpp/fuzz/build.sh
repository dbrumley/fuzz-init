#!/bin/bash
set -e
mkdir -p bin

# Check if we're using this for AFL or libFuzzer
if command -v afl-clang++ >/dev/null 2>&1 && [ "$USE_AFL" = "1" ]; then
    echo "Building with AFL support..."
    afl-clang++ -g -O1 -fsanitize=address \
        driver/afl_driver.cpp src/{{target_name}}.cpp -o bin/{{target_name}}
else
    echo "Building with libFuzzer (no AFL driver needed)..."
    clang++ -g -O1 -fsanitize=fuzzer,address \
        src/{{target_name}}.cpp -o bin/{{target_name}}
fi
