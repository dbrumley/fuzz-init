# Universal Fuzzing Makefile for {{project_name}}
# Links against standard lib{{project_name}}.a with intelligent fuzzer detection

# =============================================================================
# Environment Detection
# =============================================================================

# Detect available fuzzing tools
HAVE_CLANG := $(shell which clang++ >/dev/null 2>&1 && echo yes)
HAVE_AFL := $(shell which afl-clang-fast++ >/dev/null 2>&1 && echo yes)
HAVE_HFUZZ := $(shell which hfuzz-clang++ >/dev/null 2>&1 && echo yes)

# =============================================================================
# Library Path Configuration
# =============================================================================

{{#unless minimal}}
# Full mode - library paths relative to parent project
BUILD_DIR = ../build
INCLUDES = -I../include
LIBRARY_DIR = $(BUILD_DIR)
{{else}}
# Minimal mode - discover user's existing library
BUILD_DIR = ./build
INCLUDES = -I./include -I../include

# Try to find user's library (customizable)
DETECTED_LIB := $(shell find .. -maxdepth 3 -name "lib*.a" ! -path "*/fuzz/*" ! -path "*/.git/*" 2>/dev/null | head -1)
ifneq ($(DETECTED_LIB),)
    LIBRARY_DIR := $(dir $(DETECTED_LIB))
    DETECTED_LIB_NAME := $(notdir $(DETECTED_LIB))
    INTEGRATION_MODE = library
else
    LIBRARY_DIR = $(BUILD_DIR)
    INTEGRATION_MODE = standalone
endif
{{/unless}}

# Standard library naming (overridable via LIBRARY_PATH from parent)
LIBRARY_PATH ?= $(LIBRARY_DIR)/lib{{project_name}}.a

# =============================================================================
# Compiler Configuration
# =============================================================================

# Use clang++ for best fuzzing support, fallback to g++
CXX ?= $(if $(HAVE_CLANG),clang++,g++)
CC ?= $(if $(HAVE_CLANG),clang,gcc)

# Standard flags for harness compilation
CXXFLAGS = -g -O2 -Wall -Wextra -std=c++11 $(INCLUDES)
CFLAGS = -g -O2 -Wall -Wextra $(INCLUDES)

# =============================================================================
# Source Files and Targets
# =============================================================================

FUZZ_SRC = src/{{target_name}}.cpp
DRIVER_SRC = driver/main.c

# Target executables with standard naming
TARGET_LIBFUZZER = $(BUILD_DIR)/{{target_name}}-libfuzzer
TARGET_AFL = $(BUILD_DIR)/{{target_name}}-afl  
TARGET_HONGGFUZZ = $(BUILD_DIR)/{{target_name}}-honggfuzz
TARGET_STANDALONE = $(BUILD_DIR)/{{target_name}}-standalone

# Object files
HARNESS_OBJ = $(BUILD_DIR)/harness.o
DRIVER_OBJ = $(BUILD_DIR)/driver.o

# =============================================================================
# Main Targets
# =============================================================================

.PHONY: all clean test help libfuzzer afl honggfuzz standalone
.PHONY: check-library check-environment

# Default target builds the best available fuzzer
all: check-environment
ifeq ($(HAVE_CLANG),yes)
	@$(MAKE) libfuzzer
else
	@$(MAKE) standalone
endif

# =============================================================================
# Individual Fuzzer Targets
# =============================================================================

libfuzzer: check-library $(TARGET_LIBFUZZER)

afl: check-library $(TARGET_AFL)

honggfuzz: check-library $(TARGET_HONGGFUZZ)

standalone: check-library $(TARGET_STANDALONE)

# =============================================================================
# Build Rules
# =============================================================================

$(BUILD_DIR):
	@mkdir -p $(BUILD_DIR)

# libFuzzer target (no separate driver needed)
$(TARGET_LIBFUZZER): $(FUZZ_SRC) | $(BUILD_DIR)
ifeq ($(HAVE_CLANG),yes)
	@echo "🔨 Building libFuzzer target..."
	clang++ $(CXXFLAGS) -fsanitize=address,undefined,fuzzer \
		$(FUZZ_SRC) $(LIBRARY_PATH) -o $@
	@echo "✅ Built: $@"
else
	@echo "❌ libFuzzer requires clang++ but only found: $(CXX)"
	@echo "💡 Install clang: sudo apt install clang (Ubuntu) or xcode-select --install (macOS)"
	@false
endif

# AFL target (needs universal driver)
$(TARGET_AFL): $(HARNESS_OBJ) $(DRIVER_OBJ) | $(BUILD_DIR)
	@echo "🔨 Linking AFL target..."
ifeq ($(HAVE_AFL),yes)
	afl-clang-fast++ -fsanitize=address,undefined -g \
		$(HARNESS_OBJ) $(DRIVER_OBJ) $(LIBRARY_PATH) -o $@
else ifeq ($(HAVE_CLANG),yes)
	clang++ -fsanitize=address,undefined -g \
		$(HARNESS_OBJ) $(DRIVER_OBJ) $(LIBRARY_PATH) -o $@
else
	$(CXX) -g $(HARNESS_OBJ) $(DRIVER_OBJ) $(LIBRARY_PATH) -o $@
endif
	@echo "✅ Built: $@"

# HonggFuzz target (needs universal driver)
$(TARGET_HONGGFUZZ): $(HARNESS_OBJ) $(DRIVER_OBJ) | $(BUILD_DIR)
ifeq ($(HAVE_HFUZZ),yes)
	@echo "🔨 Linking HonggFuzz target..."
	hfuzz-clang++ -fsanitize=address,undefined \
		$(HARNESS_OBJ) $(DRIVER_OBJ) $(LIBRARY_PATH) -o $@
	@echo "✅ Built: $@"
else
	@echo "❌ HonggFuzz not found. Install: apt install honggfuzz"
	@false
endif

# Standalone target (needs universal driver, no sanitizers)
$(TARGET_STANDALONE): $(HARNESS_OBJ) $(DRIVER_OBJ) | $(BUILD_DIR)
	@echo "🔨 Building standalone target..."
	$(CXX) -g $(HARNESS_OBJ) $(DRIVER_OBJ) $(LIBRARY_PATH) -o $@
	@echo "✅ Built: $@"

# Compile harness object
$(HARNESS_OBJ): $(FUZZ_SRC) | $(BUILD_DIR)
	@echo "🔨 Compiling harness..."
	$(CXX) $(CXXFLAGS) -c $(FUZZ_SRC) -o $@

# Compile driver object (for non-libFuzzer targets)
$(DRIVER_OBJ): $(DRIVER_SRC) | $(BUILD_DIR)  
	@echo "🔨 Compiling universal driver..."
	$(CC) $(CFLAGS) -c $(DRIVER_SRC) -o $@

# =============================================================================
# Fuzzer Execution Targets
# =============================================================================

run-libfuzzer: $(TARGET_LIBFUZZER)
	@echo "🚀 Running libFuzzer..."
	@mkdir -p corpus
	$(TARGET_LIBFUZZER) corpus/ -dict=dictionaries/{{target_name}}.dict -max_total_time=60

run-afl: $(TARGET_AFL)
	@echo "🚀 Running AFL..."
	@mkdir -p findings
	@echo "Note: AFL requires 'echo core | sudo tee /proc/sys/kernel/core_pattern' on Linux"
ifeq ($(HAVE_AFL),yes)
	afl-fuzz -i testsuite -o findings -- $(TARGET_AFL)
else
	@echo "Manual AFL testing (no afl-fuzz found):"
	@echo "  echo 'test input' | $(TARGET_AFL)"
endif

run-standalone: $(TARGET_STANDALONE)
	@echo "🚀 Running standalone test..."
	$(TARGET_STANDALONE) testsuite/

# =============================================================================
# Testing and Validation
# =============================================================================

test: check-environment
ifeq ($(HAVE_CLANG),yes)
	@$(MAKE) test-libfuzzer
else
	@$(MAKE) test-standalone
endif

test-libfuzzer: $(TARGET_LIBFUZZER)
	@echo "🧪 Testing libFuzzer with sample inputs..."
	$(TARGET_LIBFUZZER) testsuite/ -runs=10 -max_total_time=5
	@echo "✅ libFuzzer test passed!"

test-afl: $(TARGET_AFL)
	@echo "🧪 Testing AFL target with sample input..."
	@echo "test input" | $(TARGET_AFL)
	@echo "✅ AFL test passed!"

test-standalone: $(TARGET_STANDALONE)
	@echo "🧪 Testing standalone target..."
	@echo "test input" | $(TARGET_STANDALONE)
	@echo "✅ Standalone test passed!"

# =============================================================================
# Environment Checks
# =============================================================================

check-environment:
	@echo "🔍 Fuzzing environment:"
	@echo "   Mode: {{#if minimal}}minimal{{else}}full{{/if}}"
	@echo "   Compiler: $(CXX)"
{{#unless minimal}}
	@echo "   Library: $(LIBRARY_PATH)"
{{else}}
ifeq ($(INTEGRATION_MODE),library)
	@echo "   Integration: Using detected library"
	@echo "   Library: $(DETECTED_LIB)"
else
	@echo "   Integration: Standalone (no library found)"
endif
{{/unless}}
ifeq ($(HAVE_CLANG),yes)
	@echo "   clang++: ✅"
else
	@echo "   clang++: ❌ (affects libFuzzer support)"
endif
ifeq ($(HAVE_AFL),yes)
	@echo "   AFL++: ✅"
else
	@echo "   AFL++: ❌"
endif

check-library:
{{#unless minimal}}
	@if [ ! -f "$(LIBRARY_PATH)" ]; then \
		echo "❌ Library not found: $(LIBRARY_PATH)"; \
		echo "💡 Run 'make lib-fuzz' in parent directory first"; \
		echo ""; \
		echo "ℹ️  Fuzzing requires your library to be built with sanitizers:"; \
		echo "   cd .. && make lib-fuzz"; \
		exit 1; \
	fi
	@echo "✅ Found library: $(LIBRARY_PATH)"
{{else}}
ifeq ($(INTEGRATION_MODE),library)
	@if [ ! -f "$(DETECTED_LIB)" ]; then \
		echo "❌ Previously detected library no longer exists: $(DETECTED_LIB)"; \
		echo "💡 Rebuild your project with sanitizer flags"; \
		exit 1; \
	fi
	@echo "✅ Using library: $(DETECTED_LIB)"
else
	@echo "ℹ️  Minimal mode: Using built-in demonstration code"
	@echo "🔄 To integrate with your library:"
	@echo "   1. Build your library with sanitizer flags"
	@echo "   2. Place it where we can find it (../lib{{project_name}}.a)"
	@echo "   3. Edit src/{{target_name}}.cpp to call your functions"
endif
{{/unless}}

# =============================================================================
# Cleanup and Help
# =============================================================================

clean:
	rm -f $(TARGET_LIBFUZZER) $(TARGET_AFL) $(TARGET_HONGGFUZZ) $(TARGET_STANDALONE)
	rm -f $(HARNESS_OBJ) $(DRIVER_OBJ)
	rm -rf corpus findings
{{#if minimal}}
	rm -rf $(BUILD_DIR)
{{/if}}

help:
	@echo "Universal Fuzzing Makefile for {{project_name}}"
	@echo ""
	@echo "Quick start:"
	@echo "  make                 - Build best available fuzzer"
	@echo "  make test            - Build and test"
	@echo ""
	@echo "Fuzzer targets:"
	@echo "  make libfuzzer       - Build libFuzzer target (requires clang++)"
	@echo "  make afl             - Build AFL target"  
	@echo "  make honggfuzz       - Build HonggFuzz target (requires honggfuzz)"
	@echo "  make standalone      - Build standalone binary"
	@echo ""
	@echo "Run fuzzers:"
	@echo "  make run-libfuzzer   - Run libFuzzer"
	@echo "  make run-afl         - Run AFL"
	@echo "  make run-standalone  - Run standalone test"
	@echo ""
	@echo "Utilities:"
	@echo "  make check-environment - Show detected environment"
	@echo "  make check-library     - Verify library availability"
	@echo "  make clean             - Remove build artifacts"
	@echo ""
{{#if minimal}}
	@echo "Minimal mode: Edit src/{{target_name}}.cpp to integrate with your code"
{{else}}
	@echo "Library: $(LIBRARY_PATH)"
{{/if}}