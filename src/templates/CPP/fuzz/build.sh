#!/bin/bash
# Universal fuzzing build script for {{project_name}}
# Links against standard lib{{project_name}}.a with intelligent fuzzer detection

set -e

# =============================================================================
# Environment Detection
# =============================================================================

echo "🔍 Detecting fuzzing environment..."

# Detect available compilers and fuzzing tools
HAVE_CLANG=""
HAVE_AFL=""
HAVE_HFUZZ=""

if command -v clang++ >/dev/null 2>&1; then
    HAVE_CLANG="yes"
    echo "   clang++: ✅ Available"
else
    echo "   clang++: ❌ Not found"
fi

if command -v afl-clang-fast++ >/dev/null 2>&1; then
    HAVE_AFL="yes"
    echo "   AFL++: ✅ Available"
else
    echo "   AFL++: ❌ Not found"
fi

if command -v hfuzz-clang++ >/dev/null 2>&1; then
    HAVE_HFUZZ="yes"
    echo "   HonggFuzz: ✅ Available"
else
    echo "   HonggFuzz: ❌ Not found"
fi

# =============================================================================
# Library Path Configuration
# =============================================================================

{{#unless minimal}}
# Full mode - library paths relative to parent project
BUILD_DIR="../build"
INCLUDES="-I../include"
LIBRARY_DIR="$BUILD_DIR"
{{else}}
# Minimal mode - discover user's existing library
BUILD_DIR="./build"
INCLUDES="-I./include -I../include"

# Try to find user's library
DETECTED_LIB=""
for lib_pattern in "../lib*.a" "../build/lib*.a" "../build/*/lib*.a"; do
    for lib_file in $lib_pattern; do
        if [ -f "$lib_file" ] && [[ "$lib_file" != *"/fuzz/"* ]]; then
            DETECTED_LIB="$lib_file"
            break 2
        fi
    done
done

if [ -n "$DETECTED_LIB" ]; then
    LIBRARY_DIR=$(dirname "$DETECTED_LIB")
    INTEGRATION_MODE="library"
    echo "   Integration: Using detected library ($DETECTED_LIB)"
else
    LIBRARY_DIR="$BUILD_DIR"
    INTEGRATION_MODE="standalone"
    echo "   Integration: Standalone (no library found)"
fi
{{/unless}}

# Standard library naming (overridable via LIBRARY_PATH)
LIBRARY_PATH="${LIBRARY_PATH:-$LIBRARY_DIR/lib{{project_name}}.a}"

# =============================================================================
# Compiler Configuration
# =============================================================================

# Use clang++ for best fuzzing support, fallback to g++
if [ -n "$HAVE_CLANG" ]; then
    CXX="${CXX:-clang++}"
    CC="${CC:-clang}"
else
    CXX="${CXX:-g++}"
    CC="${CC:-gcc}"
fi

echo "   Compiler: $CXX"
echo "   Library: $LIBRARY_PATH"

# Standard flags for harness compilation
CXXFLAGS="-g -O2 -Wall -Wextra -std=c++11 $INCLUDES"
CFLAGS="-g -O2 -Wall -Wextra $INCLUDES"

# =============================================================================
# Source Files and Targets
# =============================================================================

FUZZ_SRC="src/{{target_name}}.cpp"
DRIVER_SRC="driver/main.c"

# Target executables with standard naming
TARGET_LIBFUZZER="$BUILD_DIR/{{target_name}}-libfuzzer"
TARGET_AFL="$BUILD_DIR/{{target_name}}-afl"
TARGET_HONGGFUZZ="$BUILD_DIR/{{target_name}}-honggfuzz"
TARGET_STANDALONE="$BUILD_DIR/{{target_name}}-standalone"

# Object files
HARNESS_OBJ="$BUILD_DIR/harness.o"
DRIVER_OBJ="$BUILD_DIR/driver.o"

# =============================================================================
# Build Functions
# =============================================================================

create_build_dir() {
    mkdir -p "$BUILD_DIR"
}

build_objects() {
    echo "🔨 Compiling harness and driver..."
    
    # Compile harness
    $CXX $CXXFLAGS -c "$FUZZ_SRC" -o "$HARNESS_OBJ"
    
    # Compile driver (for non-libFuzzer targets)
    $CC $CFLAGS -c "$DRIVER_SRC" -o "$DRIVER_OBJ"
    
    echo "✅ Objects compiled"
}

build_libfuzzer() {
    echo "🔨 Building libFuzzer target..."
    
    if [ -n "$HAVE_CLANG" ]; then
        clang++ $CXXFLAGS -fsanitize=address,undefined,fuzzer \
            "$FUZZ_SRC" "$LIBRARY_PATH" -o "$TARGET_LIBFUZZER"
        echo "✅ Built: $TARGET_LIBFUZZER"
    else
        echo "❌ libFuzzer requires clang++ but only found: $CXX"
        echo "💡 Install clang: sudo apt install clang (Ubuntu) or xcode-select --install (macOS)"
        return 1
    fi
}

build_afl() {
    echo "🔨 Building AFL target..."
    
    # Choose best available AFL compiler
    if [ -n "$HAVE_AFL" ]; then
        AFL_CXX="afl-clang-fast++"
        AFL_C="afl-clang-fast"
        AFL_FLAGS="-fsanitize=address,undefined"
    elif [ -n "$HAVE_CLANG" ]; then
        AFL_CXX="clang++"
        AFL_C="clang"
        AFL_FLAGS="-fsanitize=address,undefined"
    else
        AFL_CXX="$CXX"
        AFL_C="$CC"
        AFL_FLAGS=""
    fi
    
    # Compile with AFL compiler
    $AFL_CXX $CXXFLAGS $AFL_FLAGS -c "$FUZZ_SRC" -o "${HARNESS_OBJ/.o/-afl.o}"
    $AFL_C $CFLAGS $AFL_FLAGS -c "$DRIVER_SRC" -o "${DRIVER_OBJ/.o/-afl.o}"
    
    # Link
    $AFL_CXX $AFL_FLAGS \
        "${HARNESS_OBJ/.o/-afl.o}" "${DRIVER_OBJ/.o/-afl.o}" "$LIBRARY_PATH" \
        -o "$TARGET_AFL"
    
    echo "✅ Built: $TARGET_AFL"
}

build_honggfuzz() {
    echo "🔨 Building HonggFuzz target..."
    
    if [ -n "$HAVE_HFUZZ" ]; then
        hfuzz-clang++ $CXXFLAGS -fsanitize=address,undefined \
            "$HARNESS_OBJ" "$DRIVER_OBJ" "$LIBRARY_PATH" \
            -o "$TARGET_HONGGFUZZ"
        echo "✅ Built: $TARGET_HONGGFUZZ"
    else
        echo "❌ HonggFuzz not found. Install: apt install honggfuzz"
        return 1
    fi
}

build_standalone() {
    echo "🔨 Building standalone target..."
    
    $CXX -g \
        "$HARNESS_OBJ" "$DRIVER_OBJ" "$LIBRARY_PATH" \
        -o "$TARGET_STANDALONE"
    
    echo "✅ Built: $TARGET_STANDALONE"
}

# =============================================================================
# Testing Functions
# =============================================================================

test_libfuzzer() {
    if [ -f "$TARGET_LIBFUZZER" ]; then
        echo "🧪 Testing libFuzzer with sample inputs..."
        "$TARGET_LIBFUZZER" testsuite/ -runs=10 -max_total_time=5
        echo "✅ libFuzzer test passed!"
    fi
}

test_afl() {
    if [ -f "$TARGET_AFL" ]; then
        echo "🧪 Testing AFL target with sample input..."
        echo "test input" | "$TARGET_AFL"
        echo "✅ AFL test passed!"
    fi
}

test_standalone() {
    if [ -f "$TARGET_STANDALONE" ]; then
        echo "🧪 Testing standalone target..."
        echo "test input" | "$TARGET_STANDALONE"
        echo "✅ Standalone test passed!"
    fi
}

run_best_fuzzer() {
    echo "🚀 Running best available fuzzer..."
    
    if [ -f "$TARGET_LIBFUZZER" ] && [ -n "$HAVE_CLANG" ]; then
        echo "   Using libFuzzer..."
        mkdir -p corpus
        "$TARGET_LIBFUZZER" corpus/ -dict=dictionaries/{{target_name}}.dict -max_total_time=60
    elif [ -f "$TARGET_AFL" ]; then
        echo "   Using AFL..."
        if [ -n "$HAVE_AFL" ]; then
            mkdir -p findings
            echo "Note: AFL requires 'echo core | sudo tee /proc/sys/kernel/core_pattern' on Linux"
            afl-fuzz -i testsuite -o findings -- "$TARGET_AFL"
        else
            echo "Manual AFL testing (no afl-fuzz found):"
            echo "  echo 'test input' | $TARGET_AFL"
        fi
    elif [ -f "$TARGET_STANDALONE" ]; then
        echo "   Using standalone..."
        "$TARGET_STANDALONE" testsuite/
    else
        echo "❌ No fuzzer targets available"
        return 1
    fi
}

# =============================================================================
# Environment Checks
# =============================================================================

check_environment() {
    echo "🔍 Fuzzing environment:"
    echo "   Mode: {{#if minimal}}minimal{{else}}full{{/if}}"
    echo "   Compiler: $CXX"
    echo "   Library: $LIBRARY_PATH"
    
    if [ -n "$HAVE_CLANG" ]; then
        echo "   clang++: ✅"
    else
        echo "   clang++: ❌ (affects libFuzzer support)"
    fi
    
    if [ -n "$HAVE_AFL" ]; then
        echo "   AFL++: ✅"
    else
        echo "   AFL++: ❌"
    fi
    
    if [ -n "$HAVE_HFUZZ" ]; then
        echo "   HonggFuzz: ✅"
    else
        echo "   HonggFuzz: ❌"
    fi
}

check_library() {
    {{#unless minimal}}
    if [ ! -f "$LIBRARY_PATH" ]; then
        echo "❌ Library not found: $LIBRARY_PATH"
        echo "💡 Run 'make lib-fuzz' in parent directory first"
        echo ""
        echo "ℹ️  Fuzzing requires your library to be built with sanitizers:"
        echo "   cd .. && make lib-fuzz"
        return 1
    fi
    echo "✅ Found library: $LIBRARY_PATH"
    {{else}}
    if [ "$INTEGRATION_MODE" = "library" ]; then
        if [ ! -f "$DETECTED_LIB" ]; then
            echo "❌ Previously detected library no longer exists: $DETECTED_LIB"
            echo "💡 Rebuild your project with sanitizer flags"
            return 1
        fi
        echo "✅ Using library: $DETECTED_LIB"
    else
        echo "ℹ️  Minimal mode: Using built-in demonstration code"
        echo "🔄 To integrate with your library:"
        echo "   1. Build your library with sanitizer flags"
        echo "   2. Place it where we can find it (../lib{{project_name}}.a)"
        echo "   3. Edit src/{{target_name}}.cpp to call your functions"
    fi
    {{/unless}}
}

# =============================================================================
# Main Build Logic
# =============================================================================

build_all_fuzzers() {
    echo "🎯 Building all compatible fuzzing targets..."
    
    create_build_dir
    check_library
    build_objects
    
    # Build fuzzers based on available tools
    if [ -n "$HAVE_CLANG" ]; then
        build_libfuzzer
    fi
    
    build_afl
    
    if [ -n "$HAVE_HFUZZ" ]; then
        build_honggfuzz
    fi
    
    build_standalone
    
    echo "🎯 Fuzzing build complete!"
}

build_specific_fuzzer() {
    local fuzzer="$1"
    
    create_build_dir
    check_library
    
    case "$fuzzer" in
        libfuzzer)
            build_libfuzzer
            ;;
        afl)
            build_objects
            build_afl
            ;;
        honggfuzz)
            build_objects
            build_honggfuzz
            ;;
        standalone)
            build_objects
            build_standalone
            ;;
        *)
            echo "❌ Unknown fuzzer: $fuzzer"
            echo "Available: libfuzzer, afl, honggfuzz, standalone"
            return 1
            ;;
    esac
}

run_tests() {
    echo "🧪 Running tests..."
    
    # Test the best available fuzzer
    if [ -n "$HAVE_CLANG" ]; then
        test_libfuzzer
    else
        test_standalone
    fi
    
    echo "✅ Testing complete"
}

show_help() {
    echo "Universal Fuzzing Build Script for {{project_name}}"
    echo ""
    echo "Usage: $0 [command] [options]"
    echo ""
    echo "Commands:"
    echo "  all                  Build all compatible fuzzers (default)"
    echo "  libfuzzer            Build libFuzzer target only"
    echo "  afl                  Build AFL target only"
    echo "  honggfuzz            Build HonggFuzz target only"
    echo "  standalone           Build standalone target only"
    echo "  test                 Build and test best available fuzzer"
    echo "  run                  Build and run best available fuzzer"
    echo "  clean                Clean build artifacts"
    echo "  check-env            Show environment information"
    echo "  help                 Show this help"
    echo ""
    echo "Environment variables:"
    echo "  CXX                  C++ compiler (default: auto-detected)"
    echo "  CC                   C compiler (default: auto-detected)"
    echo "  LIBRARY_PATH         Path to library to link against"
    echo ""
{{#if minimal}}
    echo "Minimal mode: Edit src/{{target_name}}.cpp to integrate with your code"
{{else}}
    echo "Library: $LIBRARY_PATH"
{{/if}}
}

show_summary() {
    echo ""
    echo "📋 Summary:"
    echo "   Built targets:"
    
    [ -f "$TARGET_LIBFUZZER" ] && echo "     ✅ libFuzzer: $TARGET_LIBFUZZER"
    [ -f "$TARGET_AFL" ] && echo "     ✅ AFL: $TARGET_AFL"
    [ -f "$TARGET_HONGGFUZZ" ] && echo "     ✅ HonggFuzz: $TARGET_HONGGFUZZ"
    [ -f "$TARGET_STANDALONE" ] && echo "     ✅ Standalone: $TARGET_STANDALONE"
    
    echo ""
    echo "🚀 Quick start:"
    echo "   $0 test      # Test fuzzing setup"
    echo "   $0 run       # Start fuzzing"
    echo ""
}

# =============================================================================
# Command Line Interface
# =============================================================================

case "${1:-all}" in
    all)
        build_all_fuzzers
        show_summary
        ;;
    libfuzzer)
        build_specific_fuzzer libfuzzer
        ;;
    afl)
        build_specific_fuzzer afl
        ;;
    honggfuzz)
        build_specific_fuzzer honggfuzz
        ;;
    standalone)
        build_specific_fuzzer standalone
        ;;
    test)
        build_all_fuzzers
        run_tests
        ;;
    run)
        build_all_fuzzers
        run_best_fuzzer
        ;;
    clean)
        echo "🧹 Cleaning build artifacts..."
        rm -f "$TARGET_LIBFUZZER" "$TARGET_AFL" "$TARGET_HONGGFUZZ" "$TARGET_STANDALONE"
        rm -f "$HARNESS_OBJ" "$DRIVER_OBJ"
        rm -f "${HARNESS_OBJ/.o/-afl.o}" "${DRIVER_OBJ/.o/-afl.o}"
        rm -rf corpus findings
{{#if minimal}}
        rm -rf "$BUILD_DIR"
{{/if}}
        echo "✅ Clean complete"
        ;;
    check-env)
        check_environment
        check_library
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        echo "❌ Unknown command: $1"
        echo "Run '$0 help' for usage information"
        exit 1
        ;;
esac