# Makefile for fuzzing {{project_name}}
# This Makefile integrates with your existing build system

# Compiler and flags
CC ?= clang
CFLAGS ?= -g -O2 -Wall -Wextra
TARGET = {{target_name}}

# Fuzzing specific flags and binaries
AFL_FUZZ_BIN = $(TARGET)-afl
LIBFUZZER_BIN = $(TARGET)-libfuzzer
HONGGFUZZ_BIN = $(TARGET)-honggfuzz
STANDALONE_BIN = $(TARGET)-standalone

# Source files - adjust these paths to match your project structure
SRC_DIR = ../src
FUZZ_SRC = $(SRC_DIR)/$(TARGET).c
DRIVER_SRC = driver/main.c

# Include directories - adjust to match your project
INCLUDES = -I$(SRC_DIR) -I../include

# Default target - builds standalone fuzzer
all: standalone

# Standalone fuzzer (no special fuzzer engine required)
standalone: $(STANDALONE_BIN)
$(STANDALONE_BIN): $(FUZZ_SRC) $(DRIVER_SRC)
	$(CC) $(CFLAGS) $(INCLUDES) -DFUZZER_TYPE_STANDALONE \
		$(DRIVER_SRC) $(FUZZ_SRC) -o $(STANDALONE_BIN)

# AFL fuzzer with persistent mode
afl: $(AFL_FUZZ_BIN)
$(AFL_FUZZ_BIN): $(FUZZ_SRC) $(DRIVER_SRC)
	afl-clang-fast $(CFLAGS) $(INCLUDES) -DFUZZER_TYPE_AFL \
		$(DRIVER_SRC) $(FUZZ_SRC) -o $(AFL_FUZZ_BIN)

# libFuzzer
libfuzzer: $(LIBFUZZER_BIN)
$(LIBFUZZER_BIN): $(FUZZ_SRC) $(DRIVER_SRC)
	clang $(CFLAGS) $(INCLUDES) -fsanitize=fuzzer,address \
		-DFUZZER_TYPE_LIBFUZZER \
		$(DRIVER_SRC) $(FUZZ_SRC) -o $(LIBFUZZER_BIN)

# HonggFuzz
honggfuzz: $(HONGGFUZZ_BIN)
$(HONGGFUZZ_BIN): $(FUZZ_SRC) $(DRIVER_SRC)
	hfuzz-clang $(CFLAGS) $(INCLUDES) -DFUZZER_TYPE_HONGGFUZZ \
		$(DRIVER_SRC) $(FUZZ_SRC) -o $(HONGGFUZZ_BIN)

# Run targets
run-standalone: $(STANDALONE_BIN)
	./$(STANDALONE_BIN) testcases/

run-afl: $(AFL_FUZZ_BIN)
	mkdir -p findings
	afl-fuzz -i testcases -o findings -- ./$(AFL_FUZZ_BIN)

run-libfuzzer: $(LIBFUZZER_BIN)
	mkdir -p corpus
	./$(LIBFUZZER_BIN) corpus/ -dict=dictionaries/$(TARGET).dict

run-honggfuzz: $(HONGGFUZZ_BIN)
	mkdir -p corpus
	honggfuzz -i testcases/ -W corpus/ -- ./$(HONGGFUZZ_BIN)

# Test target - builds and runs basic smoke test
test: standalone
	@echo "Running smoke test..."
	@echo "FUZZ" | ./$(STANDALONE_BIN)
	@echo "Smoke test passed!"

# Clean up build artifacts
clean:
	rm -f $(AFL_FUZZ_BIN) $(LIBFUZZER_BIN) $(HONGGFUZZ_BIN) $(STANDALONE_BIN)
	rm -rf findings corpus

# Install fuzzer engines (requires root/sudo)
install-deps:
	@echo "Installing fuzzing dependencies..."
	@echo "Note: This requires root privileges and may vary by system"
	@if command -v apt-get >/dev/null 2>&1; then \
		sudo apt-get update && sudo apt-get install -y afl++ honggfuzz; \
	elif command -v brew >/dev/null 2>&1; then \
		brew install afl-fuzz honggfuzz; \
	else \
		echo "Please install AFL++ and HonggFuzz manually for your system"; \
	fi

# Help target
help:
	@echo "Available targets:"
	@echo "  all          - Build standalone fuzzer (default)"
	@echo "  standalone   - Build standalone fuzzer"
	@echo "  afl          - Build AFL fuzzer"
	@echo "  libfuzzer    - Build libFuzzer"
	@echo "  honggfuzz    - Build HonggFuzz"
	@echo "  run-*        - Build and run specific fuzzer"
	@echo "  test         - Build and run smoke test"
	@echo "  clean        - Remove build artifacts"
	@echo "  install-deps - Install fuzzing dependencies"
	@echo "  help         - Show this help"

.PHONY: all standalone afl libfuzzer honggfuzz run-standalone run-afl run-libfuzzer run-honggfuzz test clean install-deps help