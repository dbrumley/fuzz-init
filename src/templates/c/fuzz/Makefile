SHELL := /bin/bash

# --- Detect available fuzzing tools ---
HAVE_CLANG := $(shell which clang >/dev/null 2>&1 && echo yes)
HAVE_AFL   := $(shell which afl-clang-fast >/dev/null 2>&1 && echo yes)
HAVE_HFUZZ := $(shell which hfuzz-clang >/dev/null 2>&1 && echo yes)

# --- Compiler settings for each fuzzer ---
CC_CLANG := clang
CC_AFL   := afl-clang-fast
CC_HFUZZ := hfuzz-clang
CC_PLAIN := $(if $(HAVE_CLANG),clang,gcc)

# --- Project layout and flags ---
BUILD_DIR  := build
DRIVER_SRC := driver/main.c
INCLUDES   := -I./include -I../include
COMMON     := -g -O1 -Wall -Wextra -fno-omit-frame-pointer
SAN        := -fsanitize=address,undefined

# --- Source files ---
HARNESS_SRC := src/fuzz_harness_1.c
HARNESS_NAME := fuzz_harness_1

# --- Library configuration ---
{{#if minimal}}
# Minimal mode: Look for user's library or use standalone
LIBPART ?= $(shell find .. -maxdepth 3 -name "lib{{project_name}}.a" -o -name "lib*.a" | grep -v "/fuzz/" | head -1)
# If no library found, we'll build standalone
{{else}}
# Full mode: Use parent library with appropriate suffix
# When building all targets, we don't need a specific suffix
LIBRARY_SUFFIX ?= all
ifeq ($(LIBRARY_SUFFIX),all)
  # Building all targets, library will be chosen per-fuzzer
  LIBPART := ../build/lib{{project_name}}.a
else
  # Building specific fuzzer with matching library
  LIBPART := ../build/lib{{project_name}}-$(LIBRARY_SUFFIX).a
endif
{{/if}}

# --- Build targets for each fuzzer ---
LIBFUZZER_BIN := $(BUILD_DIR)/$(HARNESS_NAME)-libfuzzer
AFL_BIN       := $(BUILD_DIR)/$(HARNESS_NAME)-afl
HFUZZ_BIN     := $(BUILD_DIR)/$(HARNESS_NAME)-honggfuzz
PLAIN_BIN     := $(BUILD_DIR)/$(HARNESS_NAME)-standalone

.PHONY: all env-summary summary clean help libfuzzer afl honggfuzz standalone

# Default target builds all available fuzzers
all: env-summary $(LIBFUZZER_BIN) $(AFL_BIN) $(HFUZZ_BIN) $(PLAIN_BIN) summary

# Individual fuzzer targets
libfuzzer: env-summary $(LIBFUZZER_BIN) summary
afl: env-summary $(AFL_BIN) summary
honggfuzz: env-summary $(HFUZZ_BIN) summary
standalone: env-summary $(PLAIN_BIN) summary

# --- Environment detection summary ---
env-summary:
	@echo "=== Fuzzing environment detection ==="
	@echo "  clang         : $(if $(HAVE_CLANG),yes,no)"
	@echo "  afl-clang-fast: $(if $(HAVE_AFL),yes,no)"
	@echo "  hfuzz-clang   : $(if $(HAVE_HFUZZ),yes,no)"
	@echo "  harness       : $(HARNESS_NAME)"
{{#if minimal}}
	@echo "  library       : $(if $(LIBPART),$(LIBPART),(none - will build standalone))"
{{else}}
	@echo "  library       : $(LIBPART)"
{{/if}}
	@echo

$(BUILD_DIR):
	@mkdir -p $@

# --- libFuzzer target (harness only, no driver needed) ---
$(BUILD_DIR)/%-libfuzzer: src/%.c | $(BUILD_DIR)
	@if [ "$(HAVE_CLANG)" = "yes" ]; then \
	  echo "[libFuzzer] Building $@ with AddressSanitizer+UndefinedBehaviorSanitizer"; \
{{#if minimal}}
	  if [ -f "$(LIBPART)" ]; then \
	    $(CC_CLANG) $(COMMON) $(SAN) -fsanitize=fuzzer $(INCLUDES) $< $(LIBPART) -o $@; \
	  else \
	    echo "[libFuzzer] No library found, building standalone demo"; \
	    $(CC_CLANG) $(COMMON) $(SAN) -fsanitize=fuzzer $(INCLUDES) -DMINIMAL_STANDALONE $< -o $@; \
	  fi; \
{{else}}
	  if [ -f "../build/lib{{project_name}}-libfuzzer.a" ]; then \
	    $(CC_CLANG) $(COMMON) $(SAN) -fsanitize=fuzzer $(INCLUDES) $< ../build/lib{{project_name}}-libfuzzer.a -o $@; \
	  else \
	    echo "[libFuzzer] Library not found, building with source files"; \
	    $(CC_CLANG) $(COMMON) $(SAN) -fsanitize=fuzzer $(INCLUDES) $< ../src/lib.c -o $@; \
	  fi; \
{{/if}}
	else \
	  echo "⏭️  libFuzzer skip (clang not found): $@"; \
	  echo "   Install clang to enable libFuzzer support"; \
	fi

# --- AFL++ target (harness + driver) ---
$(BUILD_DIR)/%.afl.harness.o: src/%.c | $(BUILD_DIR)
	@if [ "$(HAVE_AFL)" = "yes" ]; then \
	  echo "[AFL++] Compiling harness $< with afl-clang-fast"; \
	  $(CC_AFL) $(COMMON) $(SAN) $(INCLUDES) -c $< -o $@; \
	else :; fi

$(BUILD_DIR)/%.afl.driver.o: $(DRIVER_SRC) | $(BUILD_DIR)
	@if [ "$(HAVE_AFL)" = "yes" ]; then \
	  echo "[AFL++] Compiling driver $(DRIVER_SRC) with afl-clang-fast"; \
	  $(CC_AFL) $(COMMON) $(SAN) $(INCLUDES) -c $< -o $@; \
	else :; fi

$(BUILD_DIR)/%-afl: $(BUILD_DIR)/%.afl.harness.o $(BUILD_DIR)/%.afl.driver.o | $(BUILD_DIR)
	@if [ "$(HAVE_AFL)" = "yes" ]; then \
	  echo "[AFL++] Linking $@"; \
{{#if minimal}}
	  if [ -f "$(LIBPART)" ]; then \
	    $(CC_AFL) $(COMMON) $(SAN) $^ $(LIBPART) -o $@; \
	  else \
	    echo "[AFL++] No library found, building standalone demo"; \
	    $(CC_AFL) $(COMMON) $(SAN) $(INCLUDES) -DMINIMAL_STANDALONE src/$*.c $(DRIVER_SRC) -o $@; \
	  fi; \
{{else}}
	  if [ -f "../build/lib{{project_name}}-afl.a" ]; then \
	    $(CC_AFL) $(COMMON) $(SAN) $^ ../build/lib{{project_name}}-afl.a -o $@; \
	  else \
	    echo "[AFL++] Library not found, building with source files"; \
	    $(CC_AFL) $(COMMON) $(SAN) $(INCLUDES) src/$*.c $(DRIVER_SRC) ../src/lib.c -o $@; \
	  fi; \
{{/if}}
	else \
	  echo "⏭️  AFL++ skip (afl-clang-fast not found): $@"; \
	  echo "   Install AFL++ to enable AFL fuzzing"; \
	fi

# --- HonggFuzz target (harness + driver) ---
$(BUILD_DIR)/%.hfuzz.harness.o: src/%.c | $(BUILD_DIR)
	@if [ "$(HAVE_HFUZZ)" = "yes" ]; then \
	  echo "[HonggFuzz] Compiling harness $< with hfuzz-clang"; \
	  $(CC_HFUZZ) $(COMMON) $(SAN) $(INCLUDES) -c $< -o $@; \
	else :; fi

$(BUILD_DIR)/%.hfuzz.driver.o: $(DRIVER_SRC) | $(BUILD_DIR)
	@if [ "$(HAVE_HFUZZ)" = "yes" ]; then \
	  echo "[HonggFuzz] Compiling driver $(DRIVER_SRC) with hfuzz-clang"; \
	  $(CC_HFUZZ) $(COMMON) $(SAN) $(INCLUDES) -c $< -o $@; \
	else :; fi

$(BUILD_DIR)/%-honggfuzz: $(BUILD_DIR)/%.hfuzz.harness.o $(BUILD_DIR)/%.hfuzz.driver.o | $(BUILD_DIR)
	@if [ "$(HAVE_HFUZZ)" = "yes" ]; then \
	  echo "[HonggFuzz] Linking $@"; \
{{#if minimal}}
	  if [ -f "$(LIBPART)" ]; then \
	    $(CC_HFUZZ) $(COMMON) $(SAN) $^ $(LIBPART) -o $@; \
	  else \
	    echo "[HonggFuzz] No library found, building standalone demo"; \
	    $(CC_HFUZZ) $(COMMON) $(SAN) $(INCLUDES) -DMINIMAL_STANDALONE src/$*.c $(DRIVER_SRC) -o $@; \
	  fi; \
{{else}}
	  if [ -f "../build/lib{{project_name}}-honggfuzz.a" ]; then \
	    $(CC_HFUZZ) $(COMMON) $(SAN) $^ ../build/lib{{project_name}}-honggfuzz.a -o $@; \
	  else \
	    echo "[HonggFuzz] Library not found, building with source files"; \
	    $(CC_HFUZZ) $(COMMON) $(SAN) $(INCLUDES) src/$*.c $(DRIVER_SRC) ../src/lib.c -o $@; \
	  fi; \
{{/if}}
	else \
	  echo "⏭️  HonggFuzz skip (hfuzz-clang not found): $@"; \
	  echo "   Install HonggFuzz to enable HonggFuzz fuzzing"; \
	fi

# --- Standalone target (harness + driver) ---
$(BUILD_DIR)/%.plain.harness.o: src/%.c | $(BUILD_DIR)
	@echo "[Standalone] Compiling harness $< with $(CC_PLAIN)"
	$(CC_PLAIN) $(COMMON) $(INCLUDES) -c $< -o $@

$(BUILD_DIR)/%.plain.driver.o: $(DRIVER_SRC) | $(BUILD_DIR)
	@echo "[Standalone] Compiling driver $(DRIVER_SRC) with $(CC_PLAIN)"
	$(CC_PLAIN) $(COMMON) $(INCLUDES) -c $< -o $@

$(BUILD_DIR)/%-standalone: | check-library $(BUILD_DIR)
	@echo "[Standalone] Building $@"
{{#if minimal}}
	@echo "[Standalone] Building in minimal mode"
	$(CC_PLAIN) $(COMMON) $(INCLUDES) -DMINIMAL_STANDALONE src/$*.c $(DRIVER_SRC) -o $@
{{else}}
	@echo "[Standalone] Building with library sources to avoid sanitizer conflicts"
	$(CC_PLAIN) $(COMMON) $(INCLUDES) src/$*.c $(DRIVER_SRC) ../src/lib.c -o $@
{{/if}}

# --- Summary of what was built ---
summary:
	@echo
	@echo "===== Fuzz targets summary ====="
	@echo "Harness: $(HARNESS_NAME)"
	@if [ -f "$(BUILD_DIR)/$(HARNESS_NAME)-libfuzzer" ]; then \
	  echo "  ✓ libfuzzer : $(BUILD_DIR)/$(HARNESS_NAME)-libfuzzer"; \
	elif [ "$(HAVE_CLANG)" != "yes" ]; then \
	  echo "  ⏭️  libfuzzer : skipped (install clang: apt install clang / brew install llvm)"; \
	else \
	  echo "  ❌ libfuzzer : build failed"; \
	fi
	@if [ -f "$(BUILD_DIR)/$(HARNESS_NAME)-afl" ]; then \
	  echo "  ✓ afl       : $(BUILD_DIR)/$(HARNESS_NAME)-afl"; \
	elif [ "$(HAVE_AFL)" != "yes" ]; then \
	  echo "  ⏭️  afl       : skipped (install AFL++: apt install afl++ / brew install afl++)"; \
	else \
	  echo "  ❌ afl       : build failed"; \
	fi
	@if [ -f "$(BUILD_DIR)/$(HARNESS_NAME)-honggfuzz" ]; then \
	  echo "  ✓ honggfuzz : $(BUILD_DIR)/$(HARNESS_NAME)-honggfuzz"; \
	elif [ "$(HAVE_HFUZZ)" != "yes" ]; then \
	  echo "  ⏭️  honggfuzz : skipped (install HonggFuzz: github.com/google/honggfuzz)"; \
	else \
	  echo "  ❌ honggfuzz : build failed"; \
	fi
	@if [ -f "$(BUILD_DIR)/$(HARNESS_NAME)-standalone" ]; then \
	  echo "  ✓ standalone: $(BUILD_DIR)/$(HARNESS_NAME)-standalone"; \
	else \
	  echo "  ❌ standalone: build failed"; \
	fi
	@echo
	@echo "To run a fuzzer:"
	@echo "  libfuzzer : $(BUILD_DIR)/$(HARNESS_NAME)-libfuzzer corpus/"
	@echo "  afl       : afl-fuzz -i testsuite -o findings -- $(BUILD_DIR)/$(HARNESS_NAME)-afl"
	@echo "  honggfuzz : honggfuzz -i testsuite -W corpus -- $(BUILD_DIR)/$(HARNESS_NAME)-honggfuzz"
	@echo "  standalone: $(BUILD_DIR)/$(HARNESS_NAME)-standalone < test_input"

# --- Library verification ---
check-library:
{{#unless minimal}}
	@if [ "$(LIBRARY_SUFFIX)" != "all" ] && [ ! -f "$(LIBPART)" ]; then \
	  echo "Error: Instrumented library not found at $(LIBPART)"; \
	  echo "Run 'make fuzz-$(LIBRARY_SUFFIX)' in parent directory first."; \
	  echo ""; \
	  echo "Why? Fuzzing requires all code to be compiled with the same"; \
	  echo "sanitizer flags (e.g., AddressSanitizer) for proper bug detection."; \
	  exit 1; \
	fi
{{else}}
	@# Minimal mode: library is optional
{{/unless}}

# --- Clean build artifacts ---
clean:
	rm -rf $(BUILD_DIR)
	rm -rf corpus findings

# --- Help ---
help:
	@echo "Fuzzing Makefile for {{project_name}}"
	@echo ""
	@echo "Available targets:"
	@echo "  all         - Build all available fuzzers (default)"
	@echo "  libfuzzer   - Build only libFuzzer target"
	@echo "  afl         - Build only AFL++ target"
	@echo "  honggfuzz   - Build only HonggFuzz target"
	@echo "  standalone  - Build only standalone target"
	@echo "  clean       - Remove build artifacts"
	@echo "  help        - Show this help"
	@echo ""
	@echo "Detected tools:"
	@echo "  clang       : $(if $(HAVE_CLANG),yes,no)"
	@echo "  afl++       : $(if $(HAVE_AFL),yes,no)"
	@echo "  honggfuzz   : $(if $(HAVE_HFUZZ),yes,no)"
	@echo ""
{{#if minimal}}
	@echo "Mode: Minimal (standalone fuzz directory)"
	@echo "To integrate with your project, place your instrumented library at:"
	@echo "  $(LIBPART)"
{{else}}
	@echo "Mode: Full (integrated with parent project)"
	@echo "Library suffix: $(LIBRARY_SUFFIX)"
{{/if}}