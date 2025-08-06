#!/bin/bash
# Universal build script for {{project_name}} with intelligent fuzzing support
# Detects available compilers and builds appropriate libraries and targets

set -e

# =============================================================================
# Configuration
# =============================================================================

SRC_DIR="src"
INC_DIR="include"
BUILD_DIR="build"

# =============================================================================
# Environment Detection
# =============================================================================

echo "🔍 Detecting build environment..."

# Detect available compilers and fuzzing tools
HAVE_CLANG=""
HAVE_AFL=""
HAVE_HFUZZ=""

if command -v clang++ >/dev/null 2>&1; then
    HAVE_CLANG="yes"
    echo "   clang++: ✅ Found"
else
    echo "   clang++: ❌ Not found"
fi

if command -v afl-clang-fast++ >/dev/null 2>&1; then
    HAVE_AFL="yes" 
    echo "   AFL++: ✅ Found"
else
    echo "   AFL++: ❌ Not found"
fi

if command -v hfuzz-clang++ >/dev/null 2>&1; then
    HAVE_HFUZZ="yes"
    echo "   HonggFuzz: ✅ Found"
else
    echo "   HonggFuzz: ❌ Not found"
fi

# =============================================================================
# Compiler Selection
# =============================================================================

# Choose standard compiler
if [ -n "$HAVE_CLANG" ]; then
    CXX="${CXX:-clang++}"
    CC="${CC:-clang}"
    FUZZ_MODE="instrumented"
else
    CXX="${CXX:-g++}"
    CC="${CC:-gcc}"
    FUZZ_MODE="basic"
fi

# Choose fuzzing compiler and flags
if [ -n "$HAVE_CLANG" ]; then
    FUZZ_CXX="clang++"
    FUZZ_FLAGS="-fsanitize=address,undefined -g -O1"
else
    FUZZ_CXX="$CXX"
    FUZZ_FLAGS="-g -O1"
fi

echo "   Compiler: $CXX"
echo "   Fuzz compiler: $FUZZ_CXX"
echo "   Fuzz mode: $FUZZ_MODE"

# =============================================================================
# Build Flags
# =============================================================================

CXXFLAGS="-Wall -Wextra -g -O0 -std=c++11 -I$INC_DIR"
FUZZ_CXXFLAGS="-Wall -Wextra -std=c++11 -I$INC_DIR $FUZZ_FLAGS"

# =============================================================================
# Source Files
# =============================================================================

LIB_SOURCES="$SRC_DIR/lib.cpp"
MAIN_SRC="$SRC_DIR/main.cpp"

# =============================================================================
# Library Build Functions
# =============================================================================

build_normal_library() {
    echo "🔨 Building normal library..."
    mkdir -p "$BUILD_DIR"
    
    $CXX $CXXFLAGS -c $LIB_SOURCES -o "$BUILD_DIR/lib.o"
    ar rcs "$BUILD_DIR/lib{{project_name}}.a" "$BUILD_DIR/lib.o"
    
    echo "✅ Built: $BUILD_DIR/lib{{project_name}}.a"
}

build_fuzz_library() {
    echo "🔨 Building instrumented library for fuzzing..."
    mkdir -p "$BUILD_DIR/fuzz"
    
    $FUZZ_CXX $FUZZ_CXXFLAGS -c $LIB_SOURCES -o "$BUILD_DIR/fuzz/lib.o"
    ar rcs "$BUILD_DIR/fuzz/lib{{project_name}}.a" "$BUILD_DIR/fuzz/lib.o"
    
    echo "✅ Built: $BUILD_DIR/fuzz/lib{{project_name}}.a"
}

build_standalone_library() {
    echo "🔨 Building standalone library (no sanitizers)..."
    mkdir -p "$BUILD_DIR/standalone"
    
    $CXX -Wall -Wextra -g -O1 -std=c++11 -I$INC_DIR -c $LIB_SOURCES -o "$BUILD_DIR/standalone/lib.o"
    ar rcs "$BUILD_DIR/standalone/lib{{project_name}}.a" "$BUILD_DIR/standalone/lib.o"
    
    echo "✅ Built: $BUILD_DIR/standalone/lib{{project_name}}.a"
}

# =============================================================================
# Main Executable
# =============================================================================

build_executable() {
    echo "🔨 Building main executable..."
    mkdir -p "$BUILD_DIR/bin"
    
    $CXX $CXXFLAGS -o "$BUILD_DIR/bin/{{target_name}}" $MAIN_SRC "$BUILD_DIR/lib{{project_name}}.a"
    
    echo "✅ Built: $BUILD_DIR/bin/{{target_name}}"
}

# =============================================================================
# Fuzzing Integration
# =============================================================================

build_fuzz_integration() {
    echo "🎯 Setting up fuzzing integration..."
    
    if [ -d "fuzz" ]; then
        echo "   Found fuzz directory - building fuzzing targets..."
        
        # Build appropriate library for fuzzing
        if [ -n "$HAVE_CLANG" ]; then
            build_fuzz_library
        else
            build_standalone_library
            echo "   ⚠️  Using standalone library (no sanitizers - limited fuzzing effectiveness)"
        fi
        
        # Build fuzzing targets using the fuzz directory's build system
        cd fuzz
        
        if [ -f "build.sh" ]; then
            echo "   Using fuzz/build.sh..."
            ./build.sh
        elif [ -f "Makefile" ]; then
            echo "   Using fuzz/Makefile..."
            make all
        else
            echo "   ⚠️  No build system found in fuzz/ directory"
        fi
        
        cd ..
    else
        echo "   No fuzz directory found - skipping fuzzing setup"
    fi
}

# =============================================================================
# Testing
# =============================================================================

run_tests() {
    echo "🧪 Running tests..."
    
    {{#unless minimal}}
    # Run unit tests if available
    if [ -d "test" ] && [ -f "test/Makefile" ]; then
        echo "   Running unit tests..."
        make -C test test
    fi
    
    # Run integration tests if executable exists and test data available
    if [ -f "$BUILD_DIR/bin/{{target_name}}" ] && [ -d "test_data" ]; then
        echo "   Running integration tests..."
        echo "=== Valid input ==="
        "$BUILD_DIR/bin/{{target_name}}" test_data/valid.nmea || true
        echo "=== Invalid input (should trigger bug) ==="
        "$BUILD_DIR/bin/{{target_name}}" test_data/oob_read.nmea || true
    fi
    {{else}}
    echo "   Minimal mode: Unit tests not included"
    echo "   For testing guidance, see fuzz/INTEGRATION.md"
    {{/unless}}
    
    echo "✅ Testing complete"
}

# =============================================================================
# Information Display
# =============================================================================

show_summary() {
    echo ""
    echo "🎯 Build complete!"
    echo ""
    echo "📋 Summary:"
    echo "   Mode: $FUZZ_MODE fuzzing"
    echo "   Compiler: $CXX"
    echo "   Libraries built:"
    
    if [ -f "$BUILD_DIR/lib{{project_name}}.a" ]; then
        echo "     ✅ Normal: $BUILD_DIR/lib{{project_name}}.a"
    fi
    
    if [ -f "$BUILD_DIR/fuzz/lib{{project_name}}.a" ]; then
        echo "     ✅ Fuzzing: $BUILD_DIR/fuzz/lib{{project_name}}.a"
    fi
    
    if [ -f "$BUILD_DIR/standalone/lib{{project_name}}.a" ]; then
        echo "     ✅ Standalone: $BUILD_DIR/standalone/lib{{project_name}}.a"
    fi
    
    if [ -f "$BUILD_DIR/bin/{{target_name}}" ]; then
        echo "   Executable: ✅ $BUILD_DIR/bin/{{target_name}}"
    fi
    
    echo ""
    echo "🚀 Next steps:"
    
    if [ -d "fuzz" ]; then
        echo "   # Test fuzzing setup:"
        echo "   cd fuzz && make test"
        echo ""
        echo "   # Start fuzzing:"
        if [ -n "$HAVE_CLANG" ]; then
            echo "   cd fuzz && make run-libfuzzer"
        else
            echo "   cd fuzz && make run-standalone"
        fi
    else
        echo "   # Test the application:"
        echo "   $BUILD_DIR/bin/{{target_name}} test_data/valid.nmea"
        echo ""
        echo "   # Add fuzzing support:"
        echo "   fuzz-init . --minimal --language cpp"
    fi
    
    echo ""
}

# =============================================================================
# Main Execution
# =============================================================================

main() {
    echo "🏗️  Building {{project_name}}..."
    echo ""
    
    # Build core libraries
    build_normal_library
    build_executable
    
    # Build fuzzing libraries based on detected environment  
    if [ -n "$HAVE_CLANG" ]; then
        build_fuzz_library
    fi
    build_standalone_library
    
    # Set up fuzzing integration if fuzz directory exists
    build_fuzz_integration
    
    # Run tests
    run_tests
    
    # Show summary
    show_summary
}

# =============================================================================
# Script Entry Point
# =============================================================================

# Handle command line arguments
case "${1:-}" in
    --help|-h)
        echo "Universal build script for {{project_name}}"
        echo ""
        echo "Usage: $0 [options]"
        echo ""
        echo "Options:"
        echo "  --help, -h     Show this help"
        echo "  --info         Show environment information only"
        echo "  --clean        Clean build artifacts"
        echo ""
        echo "Environment variables:"
        echo "  CXX           C++ compiler (default: auto-detected)"
        echo "  CC            C compiler (default: auto-detected)"
        echo ""
        exit 0
        ;;
    --info)
        echo "🔍 Environment Information:"
        echo "   clang++: ${HAVE_CLANG:-❌}"
        echo "   AFL++: ${HAVE_AFL:-❌}" 
        echo "   HonggFuzz: ${HAVE_HFUZZ:-❌}"
        echo "   Selected CXX: $CXX"
        echo "   Fuzz mode: $FUZZ_MODE"
        exit 0
        ;;
    --clean)
        echo "🧹 Cleaning build artifacts..."
        rm -rf "$BUILD_DIR"
        echo "✅ Clean complete"
        exit 0
        ;;
    "")
        # Default: run main build
        main
        ;;
    *)
        echo "❌ Unknown option: $1"
        echo "Run '$0 --help' for usage information"
        exit 1
        ;;
esac