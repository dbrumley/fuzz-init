# Universal Makefile for {{project_name}} with intelligent fuzzing support
# Detects available compilers and builds appropriate fuzzing targets

# =============================================================================
# Compiler Detection and Environment Setup
# =============================================================================

# Detect available compilers and fuzzing tools
HAVE_CLANG := $(shell which clang++ >/dev/null 2>&1 && echo yes)
HAVE_AFL := $(shell which afl-clang-fast++ >/dev/null 2>&1 && echo yes)
HAVE_HFUZZ := $(shell which hfuzz-clang++ >/dev/null 2>&1 && echo yes)

# Set standard compiler for normal builds
CXX ?= $(if $(HAVE_CLANG),clang++,c++)
AR ?= ar

# Choose instrumentation compiler and flags based on available tools
ifeq ($(HAVE_CLANG),yes)
    FUZZ_CXX = clang++
    FUZZ_FLAGS = -fsanitize=address,undefined -g -O1
    FUZZ_MODE = instrumented
else
    FUZZ_CXX = c++
    FUZZ_FLAGS = -g -O1
    FUZZ_MODE = basic
endif

# =============================================================================
# Directory Structure and Sources
# =============================================================================

SRC_DIR := src
INC_DIR := include
BUILD_DIR := build
OBJ_DIR := $(BUILD_DIR)/obj
BIN_DIR := $(BUILD_DIR)/bin

# Standard flags for normal builds
CXXFLAGS := -g -O0 -I$(INC_DIR) -std=c++11

# Library and executable definitions
LIB_SOURCES := $(SRC_DIR)/lib.cpp
MAIN_SRC := $(SRC_DIR)/main.cpp

# Standard library naming (no fuzzer-specific suffixes)
LIBRARY := $(BUILD_DIR)/lib{{project_name}}.a
FUZZ_LIBRARY := $(BUILD_DIR)/fuzz/lib{{project_name}}.a
STANDALONE_LIBRARY := $(BUILD_DIR)/standalone/lib{{project_name}}.a
TARGET := $(BIN_DIR)/{{target_name}}

# Object file paths
LIB_OBJECTS := $(patsubst $(SRC_DIR)/%.cpp,$(OBJ_DIR)/%.o,$(LIB_SOURCES))
FUZZ_OBJECTS := $(patsubst $(SRC_DIR)/%.cpp,$(OBJ_DIR)/fuzz/%.o,$(LIB_SOURCES))
STANDALONE_OBJECTS := $(patsubst $(SRC_DIR)/%.cpp,$(OBJ_DIR)/standalone/%.o,$(LIB_SOURCES))

# =============================================================================
# Main Targets
# =============================================================================

.PHONY: all clean test lib lib-fuzz lib-standalone fuzz fuzz-test help
.PHONY: fuzz-libfuzzer fuzz-afl fuzz-honggfuzz fuzz-standalone

.DEFAULT_GOAL := all

all: $(LIBRARY) $(TARGET)

# =============================================================================
# Library Build Targets
# =============================================================================

# Normal library (no instrumentation)
lib: $(LIBRARY)

# Instrumented library for fuzzing (works with libFuzzer, AFL, HonggFuzz)
lib-fuzz: $(FUZZ_LIBRARY)

# Standalone library (no sanitizers, for Mayhem/basic usage)
lib-standalone: $(STANDALONE_LIBRARY)

# =============================================================================
# Library Build Rules
# =============================================================================

$(LIBRARY): $(LIB_OBJECTS)
	@mkdir -p $(BUILD_DIR)
	$(AR) rcs $@ $^

$(FUZZ_LIBRARY): $(FUZZ_OBJECTS)
	@mkdir -p $(BUILD_DIR)/fuzz
	$(AR) rcs $@ $^

$(STANDALONE_LIBRARY): $(STANDALONE_OBJECTS)
	@mkdir -p $(BUILD_DIR)/standalone
	$(AR) rcs $@ $^

$(OBJ_DIR)/%.o: $(SRC_DIR)/%.cpp
	@mkdir -p $(OBJ_DIR)
	$(CXX) $(CXXFLAGS) -c -o $@ $<

$(OBJ_DIR)/fuzz/%.o: $(SRC_DIR)/%.cpp
	@mkdir -p $(OBJ_DIR)/fuzz
	$(FUZZ_CXX) $(FUZZ_FLAGS) $(CXXFLAGS) -c -o $@ $<

$(OBJ_DIR)/standalone/%.o: $(SRC_DIR)/%.cpp
	@mkdir -p $(OBJ_DIR)/standalone
	$(CXX) $(CXXFLAGS) -c -o $@ $<

# =============================================================================
# Main Executable Target
# =============================================================================

$(TARGET): $(MAIN_SRC) $(LIBRARY)
	@mkdir -p $(BIN_DIR)
	$(CXX) $(CXXFLAGS) -o $@ $(MAIN_SRC) $(LIBRARY)

# =============================================================================
# Testing Targets
# =============================================================================

{{#unless minimal}}
test: $(LIBRARY)
	@echo "Running unit tests..."
	$(MAKE) -C test test

integration-test: $(TARGET)
	@echo "Running integration tests with sample data..."
	@echo "=== Valid input ==="; ./$(TARGET) test_data/valid.nmea
	@echo "=== Out of bound write (OOB Write) ==="; ./$(TARGET) test_data/oob_read.nmea
{{else}}
test:
	@echo "Minimal mode: Unit tests not included"
	@echo "For testing guidance, see full mode template or fuzz/INTEGRATION.md"
{{/unless}}

# =============================================================================
# Universal Fuzzing Targets
# =============================================================================

# Build all compatible fuzzers based on detected environment
fuzz: fuzz-info
	@echo "🔨 Building all compatible fuzzing targets..."
ifeq ($(HAVE_CLANG),yes)
	@echo "✅ Building instrumented library for fuzzing..."
	@$(MAKE) lib-fuzz
	@echo "✅ Building libFuzzer target..."
	@$(MAKE) fuzz-libfuzzer
ifeq ($(HAVE_AFL),yes)
	@echo "✅ Building AFL target..."
	@$(MAKE) fuzz-afl
endif
ifeq ($(HAVE_HFUZZ),yes)
	@echo "✅ Building HonggFuzz target..."
	@$(MAKE) fuzz-honggfuzz
endif
	@echo "✅ Building standalone target..."
	@$(MAKE) fuzz-standalone-target
else
	@echo "⚠️  Building basic fuzzing targets (no clang++ found)..."
	@$(MAKE) lib-standalone
ifeq ($(HAVE_AFL),yes)
	@echo "✅ Building AFL target..."
	@$(MAKE) fuzz-afl-basic
endif
	@echo "✅ Building standalone target..."
	@$(MAKE) fuzz-standalone-target
endif
	@echo "🎯 Fuzzing build complete!"

# Individual fuzzer targets with smart error handling
fuzz-libfuzzer: lib-fuzz
ifeq ($(HAVE_CLANG),yes)
	@$(MAKE) -C fuzz libfuzzer LIBRARY_PATH=../$(FUZZ_LIBRARY)
else
	@echo "❌ libFuzzer requires clang++ but only found: $(CXX)"
	@echo "💡 Available options:"
ifeq ($(HAVE_AFL),yes)
	@echo "   make fuzz-afl        # AFL fuzzing (works with $(CXX))"
endif
	@echo "   make fuzz-standalone # Standalone binary fuzzing"
	@echo "📦 Install clang to use libFuzzer:"
	@echo "   Ubuntu/Debian: sudo apt install clang"
	@echo "   macOS:         xcode-select --install"
	@false
endif

fuzz-afl: lib-fuzz
ifeq ($(HAVE_AFL),yes)
	@$(MAKE) -C fuzz afl LIBRARY_PATH=../$(FUZZ_LIBRARY)
else ifeq ($(HAVE_CLANG),yes)
	@$(MAKE) -C fuzz afl LIBRARY_PATH=../$(FUZZ_LIBRARY)
else
	@$(MAKE) fuzz-afl-basic
endif

fuzz-afl-basic: lib-standalone
	@echo "⚠️  Building AFL with basic instrumentation (no sanitizers)"
	@$(MAKE) -C fuzz afl LIBRARY_PATH=../$(STANDALONE_LIBRARY) FUZZ_MODE=basic

fuzz-honggfuzz: lib-fuzz
ifeq ($(HAVE_HFUZZ),yes)
	@$(MAKE) -C fuzz honggfuzz LIBRARY_PATH=../$(FUZZ_LIBRARY)
else
	@echo "❌ HonggFuzz not found. Install honggfuzz to use this target."
	@echo "💡 Available alternatives: make fuzz-libfuzzer, make fuzz-afl"
	@false
endif

fuzz-standalone-target: lib-standalone
	@$(MAKE) -C fuzz standalone LIBRARY_PATH=../$(STANDALONE_LIBRARY)

# Legacy target names for backward compatibility
fuzz-standalone: fuzz-standalone-target

# Fuzzing utilities
fuzz-test: lib-fuzz
	@$(MAKE) -C fuzz test LIBRARY_PATH=../$(FUZZ_LIBRARY)

fuzz-clean:
	@$(MAKE) -C fuzz clean

fuzz-info:
	@echo "🔍 Fuzzing environment detected:"
	@echo "   Compiler:     $(CXX)"
	@echo "   Fuzz compiler: $(FUZZ_CXX)"
	@echo "   Fuzz mode:    $(FUZZ_MODE)"
ifeq ($(HAVE_CLANG),yes)
	@echo "   clang++:      ✅ Available"
else
	@echo "   clang++:      ❌ Not found"
endif
ifeq ($(HAVE_AFL),yes)
	@echo "   AFL++:        ✅ Available"
else
	@echo "   AFL++:        ❌ Not found"
endif
ifeq ($(HAVE_HFUZZ),yes)
	@echo "   HonggFuzz:    ✅ Available"
else
	@echo "   HonggFuzz:    ❌ Not found"
endif

# =============================================================================
# Cleanup
# =============================================================================

{{#unless minimal}}
clean: 
	rm -rf $(BUILD_DIR)
	$(MAKE) -C test clean
{{else}}
clean: 
	rm -rf $(BUILD_DIR)
{{/unless}}

# =============================================================================
# Help
# =============================================================================

help:
	@echo "Universal Makefile for {{project_name}} - Environment-driven fuzzing"
	@echo ""
	@echo "Main targets:"
	@echo "  all              - Build normal library and executable"
	@echo "  lib              - Build normal library"
	@echo "  lib-fuzz         - Build instrumented library (for fuzzing)"
	@echo "  lib-standalone   - Build standalone library (no sanitizers)"
	@echo ""
	@echo "Fuzzing targets:"
	@echo "  fuzz             - Build all compatible fuzzers (recommended)"
	@echo "  fuzz-libfuzzer   - Build libFuzzer target (requires clang++)"
	@echo "  fuzz-afl         - Build AFL target"
	@echo "  fuzz-honggfuzz   - Build HonggFuzz target (requires honggfuzz)"
	@echo "  fuzz-standalone  - Build standalone fuzzer binary"
	@echo ""
	@echo "Testing and utilities:"
{{#unless minimal}}
	@echo "  test             - Build and run unit tests"
	@echo "  integration-test - Build and test main executable with test data"
{{else}}
	@echo "  test             - Show testing info (minimal mode)"
{{/unless}}
	@echo "  fuzz-test        - Run fuzzer smoke test"
	@echo "  fuzz-info        - Show detected fuzzing environment"
	@echo "  clean            - Remove all build artifacts"
	@echo ""
	@echo "Current environment: $(FUZZ_MODE) mode with $(FUZZ_CXX)"
