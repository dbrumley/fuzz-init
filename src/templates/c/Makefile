# Makefile for {{project_name}} with fuzzing support

# Default compiler for normal builds
CC ?= cc

AR ?= ar

SRC_DIR := src
INC_DIR := include
BUILD_DIR := build
OBJ_DIR := $(BUILD_DIR)/obj
BIN_DIR := $(BUILD_DIR)/bin

CFLAGS := -g -O0 -I$(INC_DIR)
FUZZER_CFLAGS := -g -O1 -I$(INC_DIR) -fsanitize=address

LIB_SOURCES := $(SRC_DIR)/lib.c
MAIN_SRC := $(SRC_DIR)/main.c

LIBRARY := $(BUILD_DIR)/lib{{project_name}}.a
FUZZ_LIBRARY := $(BUILD_DIR)/lib{{project_name}}-$(LIBRARY_SUFFIX).a
TARGET := $(BIN_DIR)/{{target_name}}

LIB_OBJECTS := $(patsubst $(SRC_DIR)/%.c,$(OBJ_DIR)/%.o,$(LIB_SOURCES))
FUZZ_OBJECTS := $(patsubst $(SRC_DIR)/%.c,$(OBJ_DIR)/%-$(LIBRARY_SUFFIX).o,$(LIB_SOURCES))

.PHONY: all clean test lib lib-$(LIBRARY_SUFFIX) fuzz fuzz-test fuzz-clean help

.DEFAULT_GOAL := all

all: $(LIBRARY) $(TARGET)

$(TARGET): $(MAIN_SRC) $(LIBRARY)
	@mkdir -p $(BIN_DIR)
	$(CC) $(CFLAGS) -o $@ $(MAIN_SRC) $(LIBRARY)

$(LIBRARY): $(LIB_OBJECTS)
	@mkdir -p $(BUILD_DIR)
	$(AR) rcs $@ $^

$(FUZZ_LIBRARY): $(FUZZ_OBJECTS)
	@mkdir -p $(BUILD_DIR)
	$(AR) rcs $@ $^

$(OBJ_DIR)/%.o: $(SRC_DIR)/%.c
	@mkdir -p $(OBJ_DIR)
	$(CC) $(CFLAGS) -c -o $@ $<

$(OBJ_DIR)/%-$(LIBRARY_SUFFIX).o: $(SRC_DIR)/%.c
	@mkdir -p $(OBJ_DIR)
	$(FUZZ_CC) $(FUZZER_CFLAGS) -c -o $@ $< 2>/dev/null || clang $(FUZZER_CFLAGS) -c -o $@ $<

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

fuzz:
	@echo "🔨 Building all available fuzzing targets..."
	@echo "This will build each fuzzer with its appropriate compiler and flags..."
	@$(MAKE) fuzz-libfuzzer || true
	@$(MAKE) fuzz-afl || true
	@$(MAKE) fuzz-honggfuzz || true
	@$(MAKE) fuzz-standalone || true
	@echo "🎯 Fuzzing build complete! Check above for which fuzzers were built."

fuzz-test: $(LIBRARY)
	@echo "Running fuzz tests..."
	@$(MAKE) -C fuzz test

fuzz-clean:
	$(MAKE) -C fuzz clean

fuzz-help:
	$(MAKE) -C fuzz help



lib: $(LIBRARY)
lib-$(LIBRARY_SUFFIX): $(FUZZ_LIBRARY)

# Fuzzer-specific targets - build library with appropriate compiler/flags then only that fuzzer
fuzz-libfuzzer:
	$(MAKE) LIBRARY_SUFFIX=libfuzzer FUZZ_CC=clang FUZZER_CFLAGS="-g -O1 -I$(INC_DIR) -fsanitize=address,fuzzer-no-link" lib-libfuzzer
	$(MAKE) -C fuzz libfuzzer LIBRARY_SUFFIX=libfuzzer

fuzz-afl:
	$(MAKE) LIBRARY_SUFFIX=afl FUZZ_CC=afl-clang-fast FUZZER_CFLAGS="-g -O1 -I$(INC_DIR) -fsanitize=address" lib-afl
	$(MAKE) -C fuzz afl LIBRARY_SUFFIX=afl

fuzz-honggfuzz:
	$(MAKE) LIBRARY_SUFFIX=honggfuzz FUZZ_CC=hfuzz-clang FUZZER_CFLAGS="-g -O1 -I$(INC_DIR) -fsanitize=address" lib-honggfuzz
	$(MAKE) -C fuzz honggfuzz LIBRARY_SUFFIX=honggfuzz

fuzz-standalone:
	$(MAKE) LIBRARY_SUFFIX=fuzz FUZZ_CC=clang FUZZER_CFLAGS="-g -O1 -I$(INC_DIR)" lib-fuzz
	$(MAKE) -C fuzz standalone LIBRARY_SUFFIX=fuzz

{{#unless minimal}}
clean: 
	rm -rf $(BUILD_DIR)
	$(MAKE) -C test clean
{{else}}
clean: 
	rm -rf $(BUILD_DIR)
{{/unless}}

help:
	@echo "Targets:"
	@echo "  all          - Build normal library and executable"
	@echo "  lib          - Build normal library"
	@echo "  lib-afl      - Build AFL-instrumented library"
	@echo "  lib-libfuzzer - Build libFuzzer-instrumented library"
	@echo "  lib-honggfuzz - Build HonggFuzz-instrumented library"
	@echo "  lib-standalone - Build standalone fuzzing library"
{{#unless minimal}}
	@echo "  test         - Build and run unit tests"
	@echo "  integration-test - Build and test main executable with test data"
{{else}}
	@echo "  test         - Show testing info (minimal mode)"
{{/unless}}
	@echo "  fuzz         - Build all available fuzzers"
	@echo "  fuzz-afl     - Build AFL fuzzer"
	@echo "  fuzz-libfuzzer - Build libFuzzer"
	@echo "  fuzz-honggfuzz - Build HonggFuzz fuzzer"
	@echo "  fuzz-standalone - Build standalone fuzzer"
	@echo "  fuzz-test    - Run fuzzer smoke test"
	@echo "  clean        - Remove all build artifacts"
