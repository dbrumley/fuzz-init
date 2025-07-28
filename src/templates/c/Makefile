# Makefile for {{project_name}} with {{default_fuzzer}} fuzzing support

{{#if (eq default_fuzzer "libfuzzer")}}
# Compiler optimized for libFuzzer
CC ?= clang
FUZZ_CC = clang
FUZZER_CFLAGS = -g -O1 -I$(INC_DIR) -fsanitize=address,fuzzer-no-link
LIBRARY_SUFFIX = libfuzzer
{{/if}}

{{#if (eq default_fuzzer "afl")}}
# Compiler optimized for AFL
CC ?= cc
FUZZ_CC = afl-clang-fast
FUZZER_CFLAGS = -g -O1 -I$(INC_DIR) -fsanitize=address
LIBRARY_SUFFIX = afl
{{/if}}

{{#if (eq default_fuzzer "honggfuzz")}}
# Compiler optimized for HonggFuzz
CC ?= cc
FUZZ_CC = hfuzz-clang
FUZZER_CFLAGS = -g -O1 -I$(INC_DIR) -fsanitize=address
LIBRARY_SUFFIX = honggfuzz
{{/if}}

{{#if (eq default_fuzzer "standalone")}}
# Compiler optimized for standalone fuzzing
CC ?= clang
FUZZ_CC = clang
FUZZER_CFLAGS = -g -O1 -I$(INC_DIR) -fsanitize=address
LIBRARY_SUFFIX = fuzz
{{/if}}

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

test: $(TARGET)
	@echo "=== Valid input ==="; ./$(TARGET) test_data/valid.txt
	@echo "=== Out of bound write (OOB Write) ==="; ./$(TARGET) test_data/oob_write.txt

fuzz: lib-$(LIBRARY_SUFFIX)
	$(MAKE) -C fuzz LIBRARY_SUFFIX=$(LIBRARY_SUFFIX)

fuzz-test: lib-$(LIBRARY_SUFFIX)
	$(MAKE) -C fuzz test LIBRARY_SUFFIX=$(LIBRARY_SUFFIX)

fuzz-clean:
	$(MAKE) -C fuzz clean

fuzz-help:
	$(MAKE) -C fuzz help



lib: $(LIBRARY)
lib-$(LIBRARY_SUFFIX): $(FUZZ_LIBRARY)

# Fuzzer-specific targets
fuzz-afl:
	$(MAKE) LIBRARY_SUFFIX=afl FUZZ_CC=afl-clang-fast FUZZER_CFLAGS="-g -O1 -I$(INC_DIR) -fsanitize=address" lib-afl
	$(MAKE) -C fuzz LIBRARY_SUFFIX=afl

fuzz-libfuzzer:
	$(MAKE) LIBRARY_SUFFIX=libfuzzer FUZZ_CC=clang FUZZER_CFLAGS="-g -O1 -I$(INC_DIR) -fsanitize=address,fuzzer-no-link" lib-libfuzzer
	$(MAKE) -C fuzz LIBRARY_SUFFIX=libfuzzer

fuzz-honggfuzz:
	$(MAKE) LIBRARY_SUFFIX=honggfuzz FUZZ_CC=hfuzz-clang FUZZER_CFLAGS="-g -O1 -I$(INC_DIR) -fsanitize=address" lib-honggfuzz
	$(MAKE) -C fuzz LIBRARY_SUFFIX=honggfuzz

fuzz-standalone:
	$(MAKE) LIBRARY_SUFFIX=fuzz FUZZ_CC=clang FUZZER_CFLAGS="-g -O1 -I$(INC_DIR) -fsanitize=address" lib-fuzz
	$(MAKE) -C fuzz LIBRARY_SUFFIX=fuzz

clean: 
	rm -rf $(BUILD_DIR)

help:
	@echo "Targets:"
	@echo "  all          - Build normal library and executable"
	@echo "  lib          - Build normal library"
	@echo "  lib-afl      - Build AFL-instrumented library"
	@echo "  lib-libfuzzer - Build libFuzzer-instrumented library"
	@echo "  lib-honggfuzz - Build HonggFuzz-instrumented library"
	@echo "  lib-standalone - Build standalone fuzzing library"
	@echo "  test         - Run normal tests"
	@echo "  fuzz         - Build default fuzzer ({{default_fuzzer}})"
	@echo "  fuzz-afl     - Build AFL fuzzer"
	@echo "  fuzz-libfuzzer - Build libFuzzer"
	@echo "  fuzz-honggfuzz - Build HonggFuzz fuzzer"
	@echo "  fuzz-standalone - Build standalone fuzzer"
	@echo "  fuzz-test    - Run fuzzer smoke test"
	@echo "  clean        - Remove all build artifacts"
