#!/bin/bash
set -e
mkdir -p bin

# Default fuzzer type (set during template generation)
DEFAULT_FUZZER="{{default_fuzzer}}"

# Allow environment override
FUZZER_TYPE=${FUZZER_TYPE:-$DEFAULT_FUZZER}

show_usage() {
    echo "Usage: $0 [options]"
    echo ""
    echo "Fuzzer Options (set via FUZZER_TYPE environment variable):"
    echo "  afl        - AFL/AFL++ (best for applications)"
    echo "  libfuzzer  - libFuzzer (best for libraries)" 
    echo "  hongfuzz   - HonggFuzz (alternative to AFL and libFuzzer)"
    echo "  standalone - Standalone binary (no fuzzer dependencies)"
    echo ""
    echo "Current default: $DEFAULT_FUZZER"
    echo ""
    echo "Examples:"
    echo "  $0                           # Use default ($DEFAULT_FUZZER)"
    echo "  FUZZER_TYPE=afl $0           # Force AFL build"
    echo "  FUZZER_TYPE=libfuzzer $0     # Force libFuzzer build"
    echo "  FUZZER_TYPE=hongfuzz $0      # Force HonggFuzz build"
    echo "  FUZZER_TYPE=standalone $0    # Force standalone build"
}

# Handle help flag
if [ "$1" = "-h" ] || [ "$1" = "--help" ]; then
    show_usage
    exit 0
fi

# Build based on fuzzer type
case $FUZZER_TYPE in
    "afl"|"afl++")
        if command -v afl-clang >/dev/null 2>&1; then
            echo "Building with AFL/AFL++ support..."
            afl-clang -g -O1 -fsanitize=address \
                driver/main.c src/{{target_name}}.c -o bin/{{target_name}}
        else
            echo "Error: AFL not found. Install AFL/AFL++ or use a different fuzzer type."
            echo "Try: FUZZER_TYPE=standalone $0"
            exit 1
        fi
        ;;
        
    "libfuzzer")
        echo "Building with libFuzzer..."
        if clang -g -O1 -fsanitize=fuzzer,address \
           src/{{target_name}}.c -o bin/{{target_name}} 2>/dev/null; then
            echo "libFuzzer build successful"
        else
            echo "Error: libFuzzer not available in this clang installation."
            echo "Try: FUZZER_TYPE=standalone $0"
            exit 1
        fi
        ;;
        
    "hongfuzz")
        if command -v hfuzz-clang >/dev/null 2>&1; then
            echo "Building with HonggFuzz..."
            hfuzz-clang -g -O1 -fsanitize=address \
                driver/main.c src/{{target_name}}.c -o bin/{{target_name}}
        else
            echo "Error: HonggFuzz not found. Install HonggFuzz or use a different fuzzer type."
            echo "Try: FUZZER_TYPE=standalone $0"
            exit 1
        fi
        ;;
        
    "standalone")
        echo "Building standalone binary..."
        clang -g -O1 -fsanitize=address \
            driver/main.c src/{{target_name}}.c -o bin/{{target_name}}
        ;;
        
    *)
        echo "Error: Unknown fuzzer type '$FUZZER_TYPE'"
        echo ""
        show_usage
        exit 1
        ;;
esac

echo "Build complete: bin/{{target_name}}"
echo "Fuzzer type: $FUZZER_TYPE"

# Show usage hints based on fuzzer type
case $FUZZER_TYPE in
    "afl"|"afl++")
        echo ""
        echo "To run with AFL:"
        echo "  mkdir -p input output"
        echo "  echo 'test' > input/test.txt"
        echo "  afl-fuzz -i input -o output ./bin/{{target_name}}"
        ;;
    "libfuzzer")
        echo ""
        echo "To run with libFuzzer:"
        echo "  ./bin/{{target_name}} testsuite/{{target_name}}/ -dict=dictionaries/{{target_name}}.dict"
        ;;
    "hongfuzz")
        echo ""
        echo "To run with HonggFuzz:"
        echo "  mkdir -p input"
        echo "  echo 'test' > input/test.txt"
        echo "  honggfuzz -i input -- ./bin/{{target_name}}"
        ;;
    "standalone")
        echo ""
        echo "To test standalone:"
        echo "  echo 'FUZZ' | ./bin/{{target_name}}"
        echo "  ./bin/{{target_name}} testsuite/{{target_name}}/*"
        ;;
esac