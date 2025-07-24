#!/bin/bash
set -e
mkdir -p bin

# Detect which fuzzer/compiler to use
if command -v afl-clang >/dev/null 2>&1 && [ "$USE_AFL" = "1" ]; then
    echo "Building with AFL support..."
    afl-clang -g -O1 -fsanitize=address \
        driver/main.c src/{{target_name}}.c -o bin/{{target_name}}
        
elif [ "$USE_LIBFUZZER" = "1" ]; then
    echo "Building with libFuzzer..."
    clang -g -O1 -fsanitize=fuzzer,address \
        src/{{target_name}}.c -o bin/{{target_name}}
        
else
    echo "Building standalone binary..."
    clang -g -O1 -fsanitize=address \
        driver/main.c src/{{target_name}}.c -o bin/{{target_name}}
fi

echo "Build complete: bin/{{target_name}}"
