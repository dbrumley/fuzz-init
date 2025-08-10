SHELL := /bin/bash

# --- detect tools ---
HAVE_CLANG := $(shell which clang++ >/dev/null 2>&1 && echo yes)
HAVE_AFL   := $(shell which afl-clang-fast++ >/dev/null 2>&1 && echo yes)
HAVE_HFUZZ := $(shell which hfuzz-clang++ >/dev/null 2>&1 && echo yes)

CXX_CLANG := clang++
CXX_AFL   := afl-clang-fast++
CXX_HFUZZ := hfuzz-clang++
CXX_PLAIN := $(if $(HAVE_CLANG),clang++,g++)

# --- project layout / flags ---
BUILD_DIR  := build
DRIVER_SRC := driver/main.cpp
INCLUDES   := -I./include -I../include
COMMON     := -g -O1 -std=c++17 -Wall -Wextra -fno-omit-frame-pointer -fno-sanitize-recover=all
SAN        := -fsanitize=address,undefined

# auto-detect all harnesses
HARNESS_SRCS   := $(wildcard src/*.cpp)
HARNESS_NAMES  := $(basename $(notdir $(HARNESS_SRCS)))

# optional: auto-detect a static lib above us (skip fuzz/.git)
DETECTED_LIB := $(shell find .. -maxdepth 3 -name "lib*.a" ! -path "*/fuzz/*" ! -path "*/.git/*" 2>/dev/null | head -1)
ifneq ($(strip $(DETECTED_LIB)),)
  LIBPART := $(DETECTED_LIB)
else
  LIBPART :=
endif

# concrete targets per harness
LIBFUZZER_BINS := $(foreach h,$(HARNESS_NAMES),$(BUILD_DIR)/$(h)-libfuzzer)
AFL_BINS       := $(foreach h,$(HARNESS_NAMES),$(BUILD_DIR)/$(h)-afl)
HFUZZ_BINS     := $(foreach h,$(HARNESS_NAMES),$(BUILD_DIR)/$(h)-hongfuzz)
PLAIN_BINS     := $(foreach h,$(HARNESS_NAMES),$(BUILD_DIR)/$(h)-standalone)

.PHONY: all env-summary summary clean
all: env-summary $(LIBFUZZER_BINS) $(AFL_BINS) $(HFUZZ_BINS) $(PLAIN_BINS) summary

# ---------- one-time environment summary ----------
env-summary:
	@echo "=== Environment detection ==="
	@echo "  clang++    : $(if $(HAVE_CLANG),yes,no)"
	@echo "  afl-clang  : $(if $(HAVE_AFL),yes,no)"
	@echo "  hfuzz-clang: $(if $(HAVE_HFUZZ),yes,no)"
	@echo "  harnesses  : $(if $(HARNESS_NAMES),$(HARNESS_NAMES),(none found under src/))"
	@echo "  ext library: $(if $(LIBPART),$(LIBPART),(none))"
	@echo

$(BUILD_DIR):
	@mkdir -p $@

# ---------- libFuzzer (harness only) ----------
$(BUILD_DIR)/%-libfuzzer: src/%.cpp | $(BUILD_DIR)
	@if [ "$(HAVE_CLANG)" = "yes" ]; then \
	  echo "[libFuzzer] clang++ detected → building $@ with ASan+UBSan (+fuzzer)"; \
	  $(CXX_CLANG) $(COMMON) $(SAN) -fsanitize=fuzzer $(INCLUDES) $< $(LIBPART) -o $@; \
	else \
	  echo "⏭️  libFuzzer skip (clang++ not found): $@"; \
	fi

# ---------- AFL++ (driver + harness) ----------
$(BUILD_DIR)/%.afl.harness.o: src/%.cpp | $(BUILD_DIR)
	@if [ "$(HAVE_AFL)" = "yes" ]; then \
	  echo "[AFL++] compiling harness $< with afl-clang-fast++ + ASan+UBSan"; \
	  $(CXX_AFL) $(COMMON) $(SAN) $(INCLUDES) -c $< -o $@; \
	else :; fi

$(BUILD_DIR)/%.afl.driver.o: $(DRIVER_SRC) | $(BUILD_DIR)
	@if [ "$(HAVE_AFL)" = "yes" ]; then \
	  echo "[AFL++] compiling driver $(DRIVER_SRC) with afl-clang-fast++ + ASan+UBSan"; \
	  $(CXX_AFL) $(COMMON) $(SAN) $(INCLUDES) -c $< -o $@; \
	else :; fi

$(BUILD_DIR)/%-afl: $(BUILD_DIR)/%.afl.harness.o $(BUILD_DIR)/%.afl.driver.o | $(BUILD_DIR)
	@if [ "$(HAVE_AFL)" = "yes" ]; then \
	  echo "[AFL++] linking $@"; \
	  $(CXX_AFL) $(COMMON) $(SAN) $^ $(LIBPART) -o $@; \
	else \
	  echo "⏭️  AFL++ skip (afl-clang-fast++ not found): $@"; \
	fi

# ---------- hongfuzz (driver + harness) ----------
$(BUILD_DIR)/%.hfuzz.harness.o: src/%.cpp | $(BUILD_DIR)
	@if [ "$(HAVE_HFUZZ)" = "yes" ]; then \
	  echo "[hongfuzz] compiling harness $< with hfuzz-clang++ + ASan+UBSan"; \
	  $(CXX_HFUZZ) $(COMMON) $(SAN) $(INCLUDES) -c $< -o $@; \
	else :; fi

$(BUILD_DIR)/%.hfuzz.driver.o: $(DRIVER_SRC) | $(BUILD_DIR)
	@if [ "$(HAVE_HFUZZ)" = "yes" ]; then \
	  echo "[hongfuzz] compiling driver $(DRIVER_SRC) with hfuzz-clang++ + ASan+UBSan"; \
	  $(CXX_HFUZZ) $(COMMON) $(SAN) $(INCLUDES) -c $< -o $@; \
	else :; fi

$(BUILD_DIR)/%-hongfuzz: $(BUILD_DIR)/%.hfuzz.harness.o $(BUILD_DIR)/%.hfuzz.driver.o | $(BUILD_DIR)
	@if [ "$(HAVE_HFUZZ)" = "yes" ]; then \
	  echo "[hongfuzz] linking $@"; \
	  $(CXX_HFUZZ) $(COMMON) $(SAN) $^ $(LIBPART) -o $@; \
	else \
	  echo "⏭️  hongfuzz skip (hfuzz-clang++ not found): $@"; \
	fi

# ---------- plain (driver + harness) ----------
$(BUILD_DIR)/%.plain.harness.o: src/%.cpp | $(BUILD_DIR)
	@echo "[plain] compiling harness $< with $(CXX_PLAIN)"
	$(CXX_PLAIN) $(COMMON) $(INCLUDES) -c $< -o $@

$(BUILD_DIR)/%.plain.driver.o: $(DRIVER_SRC) | $(BUILD_DIR)
	@echo "[plain] compiling driver $(DRIVER_SRC) with $(CXX_PLAIN)"
	$(CXX_PLAIN) $(COMMON) $(INCLUDES) -c $< -o $@

$(BUILD_DIR)/%-standalone: $(BUILD_DIR)/%.plain.harness.o $(BUILD_DIR)/%.plain.driver.o | $(BUILD_DIR)
	@echo "[plain] linking $@"
	$(CXX_PLAIN) $(COMMON) $^ $(LIBPART) -o $@

# ---------- final recap ----------
summary:
summary:
	@echo
	@echo "===== Fuzz targets summary ====="
	@for h in $(HARNESS_NAMES); do \
	  echo "Harness: $$h"; \
	  if [ -f "$(BUILD_DIR)/$$h-libfuzzer" ]; then \
	    echo "  - libfuzzer : built: $(BUILD_DIR)/$$h-libfuzzer"; \
	  elif [ "$(HAVE_CLANG)" != "yes" ]; then \
	    echo "  - libfuzzer : skipped: install clang++ (apt: clang, brew: llvm) and rerun make"; \
	  else \
	    echo "  - libfuzzer : skipped: see build log (does your harness define extern \"C\" LLVMFuzzerTestOneInput?)"; \
	  fi; \
	  if [ -f "$(BUILD_DIR)/$$h-afl" ]; then \
	    echo "  - afl       : built: $(BUILD_DIR)/$$h-afl"; \
	  elif [ "$(HAVE_AFL)" != "yes" ]; then \
	    echo "  - afl       : skipped: install afl++ (apt: afl++, brew: afl-fuzz) and rerun make"; \
	  else \
	    echo "  - afl       : skipped: see build log"; \
	  fi; \
	  if [ -f "$(BUILD_DIR)/$$h-hongfuzz" ]; then \
	    echo "  - hongfuzz  : built: $(BUILD_DIR)/$$h-hongfuzz"; \
	  elif [ "$(HAVE_HFUZZ)" != "yes" ]; then \
	    echo "  - hongfuzz  : skipped: install hongfuzz (apt: honggfuzz, brew: honggfuzz) and rerun make"; \
	  else \
	    echo "  - hongfuzz  : skipped: see build log"; \
	  fi; \
	  if [ -f "$(BUILD_DIR)/$$h-standalone" ]; then \
	    echo "  - standalone: built: $(BUILD_DIR)/$$h-standalone"; \
	  else \
	    echo "  - standalone: skipped: (unexpected) see build log"; \
	  fi; \
	  echo; \
	done
	@echo "Tips:"
	@echo "  - Dry run: 'make -n'"
	@echo "  - Limit harnesses: make HARNESS_SRCS=\"src/foo.cpp src/bar.cpp\""

clean:
	rm -rf $(BUILD_DIR)
