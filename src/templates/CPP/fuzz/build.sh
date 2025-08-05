#!/bin/bash
# Build script for fuzzing {{project_name}} with {{default_fuzzer}}
# This script is optimized for your chosen fuzzer and integration

set -e
mkdir -p bin

{{#if (eq default_fuzzer "libfuzzer")}}
# libFuzzer configuration
echo "Building with libFuzzer support..."
COMPILER="clang"
FUZZER_FLAGS="-fsanitize=fuzzer,address"
TARGET_NAME="{{target_name}}-libfuzzer"
LIBRARY_NAME="lib{{project_name}}-libfuzzer.a"
SOURCES="src/{{target_name}}.c"  # libFuzzer provides its own main
{{/if}}

{{#if (eq default_fuzzer "afl")}}
# AFL configuration
echo "Building with AFL support..."
COMPILER="afl-clang-fast"
FUZZER_FLAGS="-fsanitize=address"
TARGET_NAME="{{target_name}}-afl"
LIBRARY_NAME="lib{{project_name}}-afl.a"
SOURCES="driver/main.c src/{{target_name}}.c"
{{/if}}

{{#if (eq default_fuzzer "honggfuzz")}}
# HonggFuzz configuration
echo "Building with HonggFuzz support..."
COMPILER="hfuzz-clang"
FUZZER_FLAGS="-fsanitize=address"
TARGET_NAME="{{target_name}}-honggfuzz"
LIBRARY_NAME="lib{{project_name}}-honggfuzz.a"
SOURCES="driver/main.c src/{{target_name}}.c"
{{/if}}

{{#if (eq default_fuzzer "standalone")}}
# Standalone configuration
echo "Building standalone fuzzer..."
COMPILER="clang"
FUZZER_FLAGS="-fsanitize=address"
TARGET_NAME="{{target_name}}-standalone"
LIBRARY_NAME="lib{{project_name}}-fuzz.a"
SOURCES="driver/main.c src/{{target_name}}.c"
{{/if}}

# Common build settings
INCLUDES="-I../include"
CFLAGS="-g -O1 -Wall -Wextra"

# Check if fuzzing library exists, fall back to direct source compilation
USE_DIRECT_SOURCES=false
if [ ! -f "../${LIBRARY_NAME}" ]; then
    echo "⚠️  Library ../${LIBRARY_NAME} not found."
    
    # Check if we can compile sources directly
    if [ -f "../src/lib.c" ] && [ -f "../include/lib.h" ]; then
        echo "✓ Found source files, will compile directly with same instrumentation"
        USE_DIRECT_SOURCES=true
        PROJECT_SOURCES="../src/lib.c"
        INCLUDES="${INCLUDES} -I../include"
    else
        echo "Error: Neither library nor source files found."
        echo ""
{{#if (eq integration "make")}}
        echo "For full integration, run:"
        echo "  make lib-{{#if (eq default_fuzzer "standalone")}}fuzz{{else}}{{default_fuzzer}}{{/if}}"
        echo ""
{{else}}
        echo "For integration with existing projects:"
        echo "  1. Build fuzzing library: ${LIBRARY_NAME}"
        echo "  2. Or place source files in ../src/lib.c and ../include/lib.h"
        echo ""
{{/if}}
        echo "This ensures consistent sanitizer instrumentation."
        exit 1
    fi
else
    echo "✓ Found required library: ../${LIBRARY_NAME}"
    PROJECT_SOURCES=""
fi

# Build the fuzzer
echo "Compiling ${TARGET_NAME}..."

{{#if (eq default_fuzzer "libfuzzer")}}
# libFuzzer build
if [ "$USE_DIRECT_SOURCES" = true ]; then
    # Compile sources directly
    if ${COMPILER} ${CFLAGS} ${INCLUDES} ${FUZZER_FLAGS} \
       -DFUZZER_TYPE_LIBFUZZER \
       ${SOURCES} ${PROJECT_SOURCES} -o bin/${TARGET_NAME}; then
        echo "✅ libFuzzer build successful (direct sources)!"
    else
        echo "❌ libFuzzer build failed. Make sure clang supports -fsanitize=fuzzer"
        exit 1
    fi
else
    # Link against library
    if ${COMPILER} ${CFLAGS} ${INCLUDES} ${FUZZER_FLAGS} \
       -DFUZZER_TYPE_LIBFUZZER \
       ${SOURCES} -L.. -l{{project_name}}-libfuzzer -o bin/${TARGET_NAME}; then
        echo "✅ libFuzzer build successful!"
    else
        echo "❌ libFuzzer build failed. Make sure clang supports -fsanitize=fuzzer"
        exit 1
    fi
fi
{{/if}}

{{#if (eq default_fuzzer "afl")}}
# AFL build with fallback
if command -v afl-clang-fast >/dev/null 2>&1; then
    COMPILER="afl-clang-fast"
elif command -v afl-clang >/dev/null 2>&1; then
    COMPILER="afl-clang"
    echo "Using afl-clang (afl-clang-fast not found)"
else
    echo "❌ AFL not found. Please install AFL++ or AFL."
    echo "On Ubuntu/Debian: sudo apt-get install afl++"
    echo "On macOS: brew install afl-fuzz"
    exit 1
fi

if [ "$USE_DIRECT_SOURCES" = true ]; then
    # Compile sources directly
    if ${COMPILER} ${CFLAGS} ${INCLUDES} ${FUZZER_FLAGS} \
       -DFUZZER_TYPE_AFL \
       ${SOURCES} ${PROJECT_SOURCES} -o bin/${TARGET_NAME}; then
        echo "✅ AFL build successful (direct sources)!"
    else
        echo "❌ AFL build failed"
        exit 1
    fi
else
    # Link against library
    if ${COMPILER} ${CFLAGS} ${INCLUDES} ${FUZZER_FLAGS} \
       -DFUZZER_TYPE_AFL \
       ${SOURCES} -L.. -l{{project_name}}-afl -o bin/${TARGET_NAME}; then
        echo "✅ AFL build successful!"
    else
        echo "❌ AFL build failed"
        exit 1
    fi
fi
{{/if}}

{{#if (eq default_fuzzer "honggfuzz")}}
# HonggFuzz build with fallback
if command -v hfuzz-clang >/dev/null 2>&1; then
    COMPILER="hfuzz-clang"
elif command -v clang >/dev/null 2>&1; then
    COMPILER="clang"
    echo "Warning: hfuzz-clang not found, using regular clang"
    echo "Install HonggFuzz for optimal performance: brew install honggfuzz"
else
    echo "❌ Neither hfuzz-clang nor clang found"
    exit 1
fi

if [ "$USE_DIRECT_SOURCES" = true ]; then
    # Compile sources directly
    if ${COMPILER} ${CFLAGS} ${INCLUDES} ${FUZZER_FLAGS} \
       -DFUZZER_TYPE_HONGGFUZZ \
       ${SOURCES} ${PROJECT_SOURCES} -o bin/${TARGET_NAME}; then
        echo "✅ HonggFuzz build successful (direct sources)!"
    else
        echo "❌ HonggFuzz build failed"
        exit 1
    fi
else
    # Link against library
    if ${COMPILER} ${CFLAGS} ${INCLUDES} ${FUZZER_FLAGS} \
       -DFUZZER_TYPE_HONGGFUZZ \
       ${SOURCES} -L.. -l{{project_name}}-honggfuzz -o bin/${TARGET_NAME}; then
        echo "✅ HonggFuzz build successful!"
    else
        echo "❌ HonggFuzz build failed"
        exit 1
    fi
fi
{{/if}}

{{#if (eq default_fuzzer "standalone")}}
# Standalone build
if [ "$USE_DIRECT_SOURCES" = true ]; then
    # Compile sources directly
    if ${COMPILER} ${CFLAGS} ${INCLUDES} ${FUZZER_FLAGS} \
       -DFUZZER_TYPE_STANDALONE \
       ${SOURCES} ${PROJECT_SOURCES} -o bin/${TARGET_NAME}; then
        echo "✅ Standalone build successful (direct sources)!"
    else
        echo "❌ Standalone build failed"
        exit 1
    fi
else
    # Link against library
    if ${COMPILER} ${CFLAGS} ${INCLUDES} ${FUZZER_FLAGS} \
       -DFUZZER_TYPE_STANDALONE \
       ${SOURCES} -L.. -l{{project_name}}-fuzz -o bin/${TARGET_NAME}; then
        echo "✅ Standalone build successful!"
    else
        echo "❌ Standalone build failed"
        exit 1
    fi
fi
{{/if}}

echo ""
echo "🎯 Build complete: bin/${TARGET_NAME}"
echo "Fuzzer: {{default_fuzzer}}"
echo "Compiler: ${COMPILER}"
echo "Sanitizers: AddressSanitizer{{#if (eq default_fuzzer "libfuzzer")}}, libFuzzer{{/if}}"

# Show usage instructions
echo ""
echo "🚀 Usage instructions:"

{{#if (eq default_fuzzer "libfuzzer")}}
echo "Run libFuzzer:"
echo "  ./bin/${TARGET_NAME} testsuite/ -dict=dictionaries/{{target_name}}.dict"
echo ""
echo "Quick test:"
echo "  ./bin/${TARGET_NAME} testsuite/ -runs=100"
echo ""
echo "Continuous fuzzing:"
echo "  mkdir -p corpus"
echo "  ./bin/${TARGET_NAME} corpus/ -dict=dictionaries/{{target_name}}.dict -max_total_time=300"
{{/if}}

{{#if (eq default_fuzzer "afl")}}
echo "Run AFL:"
echo "  mkdir -p input output"
echo "  cp testsuite/* input/"
echo "  afl-fuzz -i input -o output -- ./bin/${TARGET_NAME}"
echo ""
echo "Quick test:"
echo "  echo 'FUZZ' | ./bin/${TARGET_NAME}"
{{/if}}

{{#if (eq default_fuzzer "honggfuzz")}}
echo "Run HonggFuzz:"
echo "  mkdir -p corpus"
echo "  honggfuzz -i testsuite/ -W corpus/ -- ./bin/${TARGET_NAME}"
echo ""
echo "Quick test:"
echo "  echo 'FUZZ' | ./bin/${TARGET_NAME}"
{{/if}}

{{#if (eq default_fuzzer "standalone")}}
echo "Run standalone:"
echo "  ./bin/${TARGET_NAME} testsuite/*"
echo ""
echo "Quick test:"
echo "  echo 'FUZZ' | ./bin/${TARGET_NAME}"
echo ""
echo "Process files:"
echo "  ./bin/${TARGET_NAME} testsuite/sample1.txt testsuite/sample2.txt"
{{/if}}

echo ""
echo "📁 Input files are in: testsuite/"
echo "📖 Dictionary is in: dictionaries/{{target_name}}.dict"