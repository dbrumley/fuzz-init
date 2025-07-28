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
LIBRARY_NAME="libexample-libfuzzer.a"
SOURCES="src/{{target_name}}.c"  # libFuzzer provides its own main
{{/if}}

{{#if (eq default_fuzzer "afl")}}
# AFL configuration
echo "Building with AFL support..."
COMPILER="afl-clang-fast"
FUZZER_FLAGS="-fsanitize=address"
TARGET_NAME="{{target_name}}-afl"
LIBRARY_NAME="libexample-afl.a"
SOURCES="driver/main.c src/{{target_name}}.c"
{{/if}}

{{#if (eq default_fuzzer "honggfuzz")}}
# HonggFuzz configuration
echo "Building with HonggFuzz support..."
COMPILER="hfuzz-clang"
FUZZER_FLAGS="-fsanitize=address"
TARGET_NAME="{{target_name}}-honggfuzz"
LIBRARY_NAME="libexample-honggfuzz.a"
SOURCES="driver/main.c src/{{target_name}}.c"
{{/if}}

{{#if (eq default_fuzzer "standalone")}}
# Standalone configuration
echo "Building standalone fuzzer..."
COMPILER="clang"
FUZZER_FLAGS="-fsanitize=address"
TARGET_NAME="{{target_name}}-standalone"
LIBRARY_NAME="libexample-fuzz.a"
SOURCES="driver/main.c src/{{target_name}}.c"
{{/if}}

# Common build settings
INCLUDES="-I../include"
CFLAGS="-g -O1 -Wall -Wextra"

# Check if fuzzing library exists
if [ ! -f "../${LIBRARY_NAME}" ]; then
    echo "Error: Required library ../${LIBRARY_NAME} not found."
    echo ""
{{#if (eq integration "make")}}
    echo "Please run the following in the parent directory first:"
    echo "  make lib-{{#if (eq default_fuzzer "standalone")}}fuzz{{else}}{{default_fuzzer}}{{/if}}"
{{else}}
    echo "Please ensure your project builds the fuzzing library: ${LIBRARY_NAME}"
    echo "This library should be compiled with the same sanitizer flags for"
    echo "consistent instrumentation."
{{/if}}
    echo ""
    echo "This ensures consistent sanitizer instrumentation between"
    echo "your project code and the fuzz harness."
    exit 1
fi

echo "✓ Found required library: ../${LIBRARY_NAME}"

# Build the fuzzer
echo "Compiling ${TARGET_NAME}..."

{{#if (eq default_fuzzer "libfuzzer")}}
# libFuzzer build
if ${COMPILER} ${CFLAGS} ${INCLUDES} ${FUZZER_FLAGS} \
   -DFUZZER_TYPE_LIBFUZZER \
   ${SOURCES} -L.. -lgps-libfuzzer -o bin/${TARGET_NAME}; then
    echo "✅ libFuzzer build successful!"
else
    echo "❌ libFuzzer build failed. Make sure clang supports -fsanitize=fuzzer"
    exit 1
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

if ${COMPILER} ${CFLAGS} ${INCLUDES} ${FUZZER_FLAGS} \
   -DFUZZER_TYPE_AFL \
   ${SOURCES} -L.. -lgps-afl -o bin/${TARGET_NAME}; then
    echo "✅ AFL build successful!"
else
    echo "❌ AFL build failed"
    exit 1
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

if ${COMPILER} ${CFLAGS} ${INCLUDES} ${FUZZER_FLAGS} \
   -DFUZZER_TYPE_HONGGFUZZ \
   ${SOURCES} -L.. -lgps-honggfuzz -o bin/${TARGET_NAME}; then
    echo "✅ HonggFuzz build successful!"
else
    echo "❌ HonggFuzz build failed"
    exit 1
fi
{{/if}}

{{#if (eq default_fuzzer "standalone")}}
# Standalone build
if ${COMPILER} ${CFLAGS} ${INCLUDES} ${FUZZER_FLAGS} \
   -DFUZZER_TYPE_STANDALONE \
   ${SOURCES} -L.. -lgps-fuzz -o bin/${TARGET_NAME}; then
    echo "✅ Standalone build successful!"
else
    echo "❌ Standalone build failed"
    exit 1
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