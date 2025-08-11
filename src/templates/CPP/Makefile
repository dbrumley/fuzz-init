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
CXXFLAGS := -g -O0 -I$(INC_DIR) -std=c++17

# Library and executable definitions
LIB_SOURCES := $(SRC_DIR)/mylib.cpp
MAIN_SRC := $(SRC_DIR)/main.cpp

# Standard library naming to match CMake
LIBRARY := $(BUILD_DIR)/libmylib.a
TARGET := $(BIN_DIR)/hellofuzz

# Object file paths
LIB_OBJECTS := $(patsubst $(SRC_DIR)/%.cpp,$(OBJ_DIR)/%.o,$(LIB_SOURCES))

# =============================================================================
# Main Targets
# =============================================================================

.PHONY: all clean test lib fuzz fuzz-test help
.PHONY: fuzz-libfuzzer fuzz-afl fuzz-honggfuzz fuzz-standalone

.DEFAULT_GOAL := all

all: $(LIBRARY) $(TARGET)

# =============================================================================
# Library Build Targets
# =============================================================================

# Normal library
lib: $(LIBRARY)

# =============================================================================
# Library Build Rules
# =============================================================================

$(LIBRARY): $(LIB_OBJECTS)
	@mkdir -p $(BUILD_DIR)
	$(AR) rcs $@ $^

$(OBJ_DIR)/%.o: $(SRC_DIR)/%.cpp
	@mkdir -p $(OBJ_DIR)
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
# Fuzzing Targets
# =============================================================================

# Build fuzz targets (delegates to fuzz/Makefile)
fuzz: $(LIBRARY)
	@echo "🔨 Building fuzzing targets..."
	@$(MAKE) -C fuzz all
	@echo "🎯 Fuzzing build complete!"

# Per-fuzzer targets that rebuild library with appropriate compiler/flags
fuzz-libfuzzer:
	@echo "🔨 Building library and fuzz targets for libFuzzer..."
	@if [ "$(HAVE_CLANG)" = "yes" ]; then \
	  $(MAKE) clean-lib && \
	  $(MAKE) lib CXX=clang++ CXXFLAGS="-g -O1 -fsanitize=address,undefined -I$(INC_DIR) -std=c++17" && \
	  $(MAKE) -C fuzz libfuzzer LIBPART=../$(LIBRARY) CXX_CLANG=clang++; \
	else \
	  echo "⏭️  libFuzzer requires clang++"; \
	fi

fuzz-afl:
	@echo "🔨 Building library and fuzz targets for AFL++..."
	@if [ "$(HAVE_AFL)" = "yes" ]; then \
	  $(MAKE) clean-lib && \
	  $(MAKE) lib CXX=afl-clang-fast++ CXXFLAGS="-g -O1 -fsanitize=address,undefined -I$(INC_DIR) -std=c++17" && \
	  $(MAKE) -C fuzz afl LIBPART=../$(LIBRARY) CXX_AFL=afl-clang-fast++; \
	else \
	  echo "⏭️  AFL++ requires afl-clang-fast++"; \
	fi

fuzz-hongfuzz:
	@echo "🔨 Building library and fuzz targets for HonggFuzz..."
	@if [ "$(HAVE_HFUZZ)" = "yes" ]; then \
	  $(MAKE) clean-lib && \
	  $(MAKE) lib CXX=hfuzz-clang++ CXXFLAGS="-g -O1 -fsanitize=address,undefined -I$(INC_DIR) -std=c++17" && \
	  $(MAKE) -C fuzz hongfuzz LIBPART=../$(LIBRARY) CXX_HFUZZ=hfuzz-clang++; \
	else \
	  echo "⏭️  HonggFuzz requires hfuzz-clang++"; \
	fi

fuzz-standalone:
	@echo "🔨 Building library and fuzz targets for standalone/native..."
	$(MAKE) clean-lib && \
	$(MAKE) lib CXXFLAGS="-g -O1 -I$(INC_DIR) -std=c++17" && \
	$(MAKE) -C fuzz standalone LIBPART=../$(LIBRARY)

# Clean just the library
clean-lib:
	rm -f $(LIBRARY) $(LIB_OBJECTS)

# Clean fuzz targets
fuzz-clean:
	@$(MAKE) -C fuzz clean

# Show fuzz environment info
fuzz-info:
	@$(MAKE) -C fuzz env-summary

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
	@echo "Makefile for {{project_name}}"
	@echo ""
	@echo "Main targets:"
	@echo "  all              - Build library and executable"
	@echo "  lib              - Build library only"
	@echo "  $(TARGET)        - Build executable"
	@echo ""
	@echo "Fuzzing targets:"
	@echo "  fuzz             - Build all fuzz targets (using single library)"
	@echo "  fuzz-libfuzzer   - Rebuild library and fuzz with clang/ASan/UBSan"
	@echo "  fuzz-afl         - Rebuild library and fuzz with AFL++ instrumentation"
	@echo "  fuzz-hongfuzz    - Rebuild library and fuzz with HonggFuzz instrumentation"
	@echo "  fuzz-standalone  - Rebuild library and fuzz without instrumentation"
	@echo "  fuzz-info        - Show detected fuzzing environment"
	@echo "  fuzz-clean       - Clean fuzz build artifacts"
	@echo ""
	@echo "Testing and utilities:"
{{#unless minimal}}
	@echo "  test             - Build and run unit tests"
	@echo "  integration-test - Build and test main executable with test data"
{{else}}
	@echo "  test             - Show testing info (minimal mode)"
{{/unless}}
	@echo "  clean            - Remove all build artifacts"
	@echo ""
	@echo "Run 'make -C fuzz help' for detailed fuzzing options"
