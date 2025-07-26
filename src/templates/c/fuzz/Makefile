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

# Source files
FUZZ_SRC = src/$(TARGET).c
DRIVER_SRC = driver/main.c

# Integration Configuration - Choose your approach
# This template uses fuzzing library linking for consistent sanitizer instrumentation.
# The parent Makefile builds separate libraries with appropriate fuzzing flags.

# Option 1: Fuzzing Library Linking (current approach)
INCLUDES = -I../include
LIBPATH = -L..
# Note: Different fuzzers use different libraries with matching sanitizer flags
LIBS_STANDALONE = -lgps-fuzz
LIBS_AFL = -lgps-afl
LIBS_LIBFUZZER = -lgps-libfuzzer
LIBS_HONGGFUZZ = -lgps-honggfuzz

# Option 2: Direct Source Compilation (uncomment and modify)
# PROJECT_SOURCES = ../../src/parser.c ../../src/utils.c
# INCLUDES = -I../../include -I../../src

# Option 3: Object File Linking (uncomment and modify)  
# PROJECT_OBJS = ../../build/parser.o ../../build/utils.o
# INCLUDES = -I../../include

# Option 4: Hybrid (combine as needed)
# LIBS = -L../../lib -lcore
# PROJECT_SOURCES = ../../src/special.c  
# PROJECT_OBJS = ../../build/legacy.o

# Default target - builds standalone fuzzer
all: standalone

# Library check - ensures appropriate fuzzing library exists
check-library:
	@echo "Checking for fuzzing libraries..."
	@if [ ! -f ../libgps-fuzz.a ] && [ ! -f ../libgps-afl.a ] && [ ! -f ../libgps-libfuzzer.a ] && [ ! -f ../libgps-honggfuzz.a ]; then \
		echo "Warning: No fuzzing libraries found in parent directory."; \
		echo "Run 'make lib-<fuzzer>' in parent directory first (e.g., 'make lib-libfuzzer')."; \
		echo "This ensures consistent sanitizer instrumentation."; \
	fi

# Standalone fuzzer (no special fuzzer engine required)
standalone: $(STANDALONE_BIN)
$(STANDALONE_BIN): $(FUZZ_SRC) $(DRIVER_SRC)
	$(CC) $(CFLAGS) $(INCLUDES) -DFUZZER_TYPE_STANDALONE \
		$(DRIVER_SRC) $(FUZZ_SRC) $(PROJECT_SOURCES) $(PROJECT_OBJS) $(LIBPATH) $(LIBS_STANDALONE) -o $(STANDALONE_BIN)

# AFL fuzzer with persistent mode
afl: $(AFL_FUZZ_BIN)
$(AFL_FUZZ_BIN): $(FUZZ_SRC) $(DRIVER_SRC)
	afl-clang-fast $(CFLAGS) $(INCLUDES) -DFUZZER_TYPE_AFL \
		$(DRIVER_SRC) $(FUZZ_SRC) $(PROJECT_SOURCES) $(PROJECT_OBJS) $(LIBPATH) $(LIBS_AFL) -o $(AFL_FUZZ_BIN)

# libFuzzer
libfuzzer: $(LIBFUZZER_BIN)
$(LIBFUZZER_BIN): $(FUZZ_SRC)
	clang $(CFLAGS) $(INCLUDES) -fsanitize=fuzzer,address \
		-DFUZZER_TYPE_LIBFUZZER \
		$(FUZZ_SRC) $(PROJECT_SOURCES) $(PROJECT_OBJS) $(LIBPATH) $(LIBS_LIBFUZZER) -o $(LIBFUZZER_BIN)

# HonggFuzz
honggfuzz: $(HONGGFUZZ_BIN)
$(HONGGFUZZ_BIN): $(FUZZ_SRC) $(DRIVER_SRC)
	hfuzz-clang $(CFLAGS) $(INCLUDES) -DFUZZER_TYPE_HONGGFUZZ \
		$(DRIVER_SRC) $(FUZZ_SRC) $(PROJECT_SOURCES) $(PROJECT_OBJS) $(LIBPATH) $(LIBS_HONGGFUZZ) -o $(HONGGFUZZ_BIN)

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
	@echo ""
	@echo "Integration approaches (edit this Makefile):"
	@echo "  Fuzzing Library: Links against sanitizer-instrumented library (default)"
	@echo "  Sources:         Uncomment PROJECT_SOURCES, add your .c files"
	@echo "  Objects:         Uncomment PROJECT_OBJS, add your .o files"  
	@echo "  Hybrid:          Combine approaches as needed"
	@echo ""
	@echo "Note: Fuzzing libraries ensure consistent sanitizer instrumentation."
	@echo "See INTEGRATION.md for detailed instructions."

.PHONY: all standalone afl libfuzzer honggfuzz run-standalone run-afl run-libfuzzer run-honggfuzz test clean install-deps help