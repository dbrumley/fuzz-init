# Makefile for fuzzing {{project_name}} with {{default_fuzzer}}
# This Makefile is optimized for your chosen fuzzer and integration approach

{{#if (eq default_fuzzer "libfuzzer")}}
# libFuzzer configuration
CC = clang
CFLAGS = -g -O2 -Wall -Wextra
FUZZER_FLAGS = -fsanitize=fuzzer,address
TARGET = {{target_name}}-libfuzzer
LIBRARY_SUFFIX = libfuzzer
{{/if}}

{{#if (eq default_fuzzer "afl")}}
# AFL configuration  
CC = afl-clang-fast
CFLAGS = -g -O2 -Wall -Wextra
FUZZER_FLAGS = -fsanitize=address
TARGET = {{target_name}}-afl
LIBRARY_SUFFIX = afl
{{/if}}

{{#if (eq default_fuzzer "honggfuzz")}}
# HonggFuzz configuration
CC = hfuzz-clang
CFLAGS = -g -O2 -Wall -Wextra
FUZZER_FLAGS = -fsanitize=address
TARGET = {{target_name}}-honggfuzz
LIBRARY_SUFFIX = honggfuzz
{{/if}}

{{#if (eq default_fuzzer "standalone")}}
# Standalone configuration
CC = clang
CFLAGS = -g -O2 -Wall -Wextra
FUZZER_FLAGS = -fsanitize=address
TARGET = {{target_name}}-standalone
LIBRARY_SUFFIX = fuzz
{{/if}}

# Build directories and paths
{{#unless minimal}}
# Full mode - integrated with parent project
BUILD_DIR = ../build
FUZZ_BUILD_DIR = ../build/fuzz
INCLUDES = -I../include
LIBPATH = -L$(BUILD_DIR)
LIBRARY_NAME = lib{{project_name}}-$(LIBRARY_SUFFIX).a
LIBRARY_PATH = $(BUILD_DIR)/$(LIBRARY_NAME)
{{else}}
# Minimal mode - standalone fuzz directory
BUILD_DIR = ./build
FUZZ_BUILD_DIR = $(BUILD_DIR)
INCLUDES = -I./include  # TODO: Add your project's include directories
LIBPATH = -L./lib       # TODO: Add path to your instrumented library
LIBRARY_NAME = lib{{project_name}}-$(LIBRARY_SUFFIX).a
LIBRARY_PATH = ./lib/$(LIBRARY_NAME)
{{/unless}}

# Source files
FUZZ_SRC = src/{{target_name}}.c
{{#if (eq default_fuzzer "libfuzzer")}}
# libFuzzer doesn't need driver (provides its own main)
SOURCES = $(FUZZ_SRC)
{{else}}
# Other fuzzers need the universal driver
DRIVER_SRC = driver/main.c
SOURCES = $(FUZZ_SRC) $(DRIVER_SRC)
{{/if}}

# Library to link against
LIBS = -l{{project_name}}-$(LIBRARY_SUFFIX)

# Build configuration based on integration type
{{#if (eq integration "make")}}
# Makefile integration - links against parent library
PROJECT_SOURCES =
PROJECT_OBJS = 
{{else}}
# For other integrations, you may need to adjust these paths:
# PROJECT_SOURCES = ../../src/parser.c ../../src/utils.c
# PROJECT_OBJS = ../../build/parser.o ../../build/utils.o
PROJECT_SOURCES =
PROJECT_OBJS = 
{{/if}}

# Default target - builds your chosen fuzzer
all: {{default_fuzzer}}

# Build {{default_fuzzer}} fuzzer
{{default_fuzzer}}: $(FUZZ_BUILD_DIR)/$(TARGET)

# Create output directory
$(FUZZ_BUILD_DIR):
	@mkdir -p $(FUZZ_BUILD_DIR)

$(FUZZ_BUILD_DIR)/$(TARGET): $(SOURCES) | check-library $(FUZZ_BUILD_DIR)
{{#if (eq default_fuzzer "libfuzzer")}}
	@echo "Building libFuzzer target..."
	$(CC) $(CFLAGS) $(INCLUDES) $(FUZZER_FLAGS) \
		-DFUZZER_TYPE_LIBFUZZER \
		$(FUZZ_SRC) $(PROJECT_SOURCES) $(PROJECT_OBJS) $(LIBPATH) $(LIBS) -o $@
{{else}}
	@echo "Building {{default_fuzzer}} target..."
	$(CC) $(CFLAGS) $(INCLUDES) $(FUZZER_FLAGS) \
		-DFUZZER_TYPE_{{default_fuzzer}} \
		$(SOURCES) $(PROJECT_SOURCES) $(PROJECT_OBJS) $(LIBPATH) $(LIBS) -o $@ 2>/dev/null || \
	clang $(CFLAGS) $(INCLUDES) $(FUZZER_FLAGS) \
		-DFUZZER_TYPE_{{default_fuzzer}} \
		$(SOURCES) $(PROJECT_SOURCES) $(PROJECT_OBJS) $(LIBPATH) $(LIBS) -o $@
{{/if}}

# Run {{default_fuzzer}}
{{#if (eq default_fuzzer "standalone")}}
run: $(FUZZ_BUILD_DIR)/$(TARGET)
	$(FUZZ_BUILD_DIR)/$(TARGET) testsuite/
{{/if}}

{{#if (eq default_fuzzer "afl")}}
run: $(FUZZ_BUILD_DIR)/$(TARGET)
	@echo "Setting up AFL fuzzing..."
	mkdir -p findings
	afl-fuzz -i testsuite -o findings -- $(FUZZ_BUILD_DIR)/$(TARGET)
{{/if}}

{{#if (eq default_fuzzer "libfuzzer")}}
run: $(FUZZ_BUILD_DIR)/$(TARGET)
	@echo "Running libFuzzer..."
	mkdir -p corpus
	$(FUZZ_BUILD_DIR)/$(TARGET) corpus/ -dict=dictionaries/{{target_name}}.dict -max_total_time=60
{{/if}}

{{#if (eq default_fuzzer "honggfuzz")}}
run: $(FUZZ_BUILD_DIR)/$(TARGET)
	@echo "Running HonggFuzz..."
	mkdir -p corpus
	honggfuzz -i testsuite/ -W corpus/ -- $(FUZZ_BUILD_DIR)/$(TARGET)
{{/if}}

# Test target - builds and runs basic smoke test
test: $(FUZZ_BUILD_DIR)/$(TARGET)
	@echo "Running smoke test with {{default_fuzzer}}..."
{{#if (eq default_fuzzer "libfuzzer")}}
	@echo "Testing sample inputs..."
	$(FUZZ_BUILD_DIR)/$(TARGET) testsuite/ -runs=10
{{else}}
	@echo "FUZZ" | $(FUZZ_BUILD_DIR)/$(TARGET)
{{/if}}
	@echo "Smoke test passed!"

# Clean up build artifacts
clean:
	rm -f $(FUZZ_BUILD_DIR)/$(TARGET)
	rm -rf findings corpus
{{#if (eq minimal true)}}
	rm -rf $(BUILD_DIR)
{{/if}}

# Library check - ensures appropriate fuzzing library exists
check-library:
{{#unless minimal}}
	@echo "Checking for instrumented library..."
	@if [ ! -f $(LIBRARY_PATH) ]; then \
		echo "Error: Instrumented library not found at $(LIBRARY_PATH)"; \
		echo "Run 'make lib-$(LIBRARY_SUFFIX)' in parent directory first."; \
		echo ""; \
		echo "Why? Fuzzing requires all code to be compiled with the same"; \
		echo "sanitizer flags (e.g., AddressSanitizer) for proper bug detection."; \
		exit 1; \
	fi
	@echo "✓ Library found: $(LIBRARY_PATH)"
{{else}}
	@echo "Minimal mode: Checking for library or source files..."
	@# Try to find library in common locations
	@if [ -f $(LIBRARY_PATH) ]; then \
		echo "✓ Found library: $(LIBRARY_PATH)"; \
	elif [ -f "../lib{{project_name}}-$(LIBRARY_SUFFIX).a" ]; then \
		echo "✓ Found library: ../lib{{project_name}}-$(LIBRARY_SUFFIX).a"; \
	elif [ -f "../src/lib.c" ] && [ -f "../include/lib.h" ]; then \
		echo "✓ Found source files, will compile directly"; \
		echo "  Sources: ../src/lib.c"; \
		echo "  Headers: ../include/lib.h"; \
	else \
		echo ""; \
		echo "⚠️  No library or source files found."; \
		echo ""; \
		echo "For fuzzing, you need either:"; \
		echo "  1. Instrumented library: $(LIBRARY_PATH)"; \
		echo "  2. Source files: ../src/lib.c and ../include/lib.h"; \
		echo ""; \
		echo "Example build command for your instrumented library:"; \
		echo "  $(CC) $(FUZZER_FLAGS) -c your_source.c -o your_source.o"; \
		echo "  ar rcs $(LIBRARY_PATH) your_source.o"; \
		echo ""; \
		echo "Note: Both library and sources must use same instrumentation!"; \
		exit 1; \
	fi
{{/unless}}

# Help target
help:
	@echo "Fuzzing Makefile for {{project_name}} ({{default_fuzzer}} mode)"
	@echo ""
	@echo "Available targets:"
	@echo "  all          - Build {{default_fuzzer}} fuzzer (default)"
	@echo "  {{default_fuzzer}}     - Build {{default_fuzzer}} fuzzer"
	@echo "  run          - Build and run {{default_fuzzer}}"
	@echo "  test         - Build and run smoke test"
	@echo "  clean        - Remove build artifacts"
	@echo "  check-library - Verify fuzzing library exists"
	@echo "  help         - Show this help"
	@echo ""
	@echo "Configuration:"
{{#if (eq default_fuzzer "libfuzzer")}}
	@echo "  Fuzzer:      libFuzzer (part of clang)"
	@echo "  Compiler:    clang with -fsanitize=fuzzer,address"
	@echo "  Library:     $(LIBRARY_NAME) (with sanitizer instrumentation)"
	@echo "  Usage:       $(FUZZ_BUILD_DIR)/{{target_name}}-libfuzzer corpus/ -dict=dictionaries/{{target_name}}.dict"
{{/if}}
{{#if (eq default_fuzzer "afl")}}
	@echo "  Fuzzer:      AFL++ (American Fuzzy Lop)"
	@echo "  Compiler:    afl-clang-fast with -fsanitize=address"  
	@echo "  Library:     $(LIBRARY_NAME) (with sanitizer instrumentation)"
	@echo "  Usage:       afl-fuzz -i testsuite -o findings -- $(FUZZ_BUILD_DIR)/{{target_name}}-afl"
{{/if}}
{{#if (eq default_fuzzer "honggfuzz")}}
	@echo "  Fuzzer:      HonggFuzz"
	@echo "  Compiler:    hfuzz-clang with -fsanitize=address"
	@echo "  Library:     $(LIBRARY_NAME) (with sanitizer instrumentation)"
	@echo "  Usage:       honggfuzz -i testsuite/ -W corpus/ -- $(FUZZ_BUILD_DIR)/{{target_name}}-honggfuzz"
{{/if}}
{{#if (eq default_fuzzer "standalone")}}
	@echo "  Fuzzer:      Standalone (no fuzzer engine)"
	@echo "  Compiler:    clang with -fsanitize=address"
	@echo "  Library:     $(LIBRARY_NAME) (with sanitizer instrumentation)"
	@echo "  Usage:       $(FUZZ_BUILD_DIR)/{{target_name}}-standalone testsuite/ or echo 'input' | $(FUZZ_BUILD_DIR)/{{target_name}}-standalone"
{{/if}}
	@echo ""
{{#if (eq integration "make")}}
	@echo "Integration: Makefile (parent Makefile builds library automatically)"
{{else}}
	@echo "Integration: {{integration}}"
{{/if}}
	@echo "See INTEGRATION.md for detailed setup instructions."

.PHONY: all {{default_fuzzer}} run test clean check-library help