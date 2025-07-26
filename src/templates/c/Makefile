CC ?= cc
AR ?= ar

# Source files
SRC_DIR = src
INC_DIR = include
LIB_SOURCES = $(SRC_DIR)/gps.c
MAIN_SRC = $(SRC_DIR)/main.c
CFLAGS = -g -O0 -I$(INC_DIR)

# Fuzzing-specific flags (for sanitized builds)
FUZZ_CFLAGS = -g -O1 -I$(INC_DIR) -fsanitize=address,fuzzer-no-link
LIBFUZZER_CFLAGS = -g -O1 -I$(INC_DIR) -fsanitize=address,fuzzer-no-link
AFL_CFLAGS = -g -O1 -I$(INC_DIR) -fsanitize=address
HONGGFUZZ_CFLAGS = -g -O1 -I$(INC_DIR) -fsanitize=address

# Targets
TARGET = gps
LIBRARY = libgps.a
FUZZ_LIBRARY = libgps-fuzz.a
LIBFUZZER_LIBRARY = libgps-libfuzzer.a
AFL_LIBRARY = libgps-afl.a
HONGGFUZZ_LIBRARY = libgps-honggfuzz.a

.PHONY: all clean test lib lib-fuzz lib-libfuzzer lib-afl lib-honggfuzz
.PHONY: fuzz fuzz-afl fuzz-libfuzzer fuzz-honggfuzz fuzz-standalone fuzz-test fuzz-clean fuzz-help

all: $(LIBRARY) $(TARGET)

# Build the GPS parser library
lib: $(LIBRARY)

$(LIBRARY): $(LIB_SOURCES:.c=.o)
	@echo "Creating library $(LIBRARY)..."
	$(AR) rcs $@ $^

# Build fuzzing libraries with sanitizer instrumentation
lib-fuzz: $(FUZZ_LIBRARY)

$(FUZZ_LIBRARY): $(LIB_SOURCES:.c=-fuzz.o)
	@echo "Creating fuzzing library $(FUZZ_LIBRARY)..."
	$(AR) rcs $@ $^

lib-libfuzzer: $(LIBFUZZER_LIBRARY)

$(LIBFUZZER_LIBRARY): $(LIB_SOURCES:.c=-libfuzzer.o)
	@echo "Creating libFuzzer library $(LIBFUZZER_LIBRARY)..."
	$(AR) rcs $@ $^

lib-afl: $(AFL_LIBRARY)

$(AFL_LIBRARY): $(LIB_SOURCES:.c=-afl.o)
	@echo "Creating AFL library $(AFL_LIBRARY)..."
	$(AR) rcs $@ $^

lib-honggfuzz: $(HONGGFUZZ_LIBRARY)

$(HONGGFUZZ_LIBRARY): $(LIB_SOURCES:.c=-honggfuzz.o)
	@echo "Creating HonggFuzz library $(HONGGFUZZ_LIBRARY)..."
	$(AR) rcs $@ $^

# Build object files
$(SRC_DIR)/%.o: $(SRC_DIR)/%.c
	@echo "Compiling $<..."
	$(CC) $(CFLAGS) -c -o $@ $<

# Build fuzzing object files with different sanitizer configurations
$(SRC_DIR)/%-fuzz.o: $(SRC_DIR)/%.c
	@echo "Compiling $< for fuzzing..."
	$(CC) $(FUZZ_CFLAGS) -c -o $@ $<

$(SRC_DIR)/%-libfuzzer.o: $(SRC_DIR)/%.c
	@echo "Compiling $< for libFuzzer..."
	clang $(LIBFUZZER_CFLAGS) -c -o $@ $<

$(SRC_DIR)/%-afl.o: $(SRC_DIR)/%.c
	@echo "Compiling $< for AFL..."
	afl-clang-fast $(AFL_CFLAGS) -c -o $@ $< 2>/dev/null || clang $(AFL_CFLAGS) -c -o $@ $<

$(SRC_DIR)/%-honggfuzz.o: $(SRC_DIR)/%.c
	@echo "Compiling $< for HonggFuzz..."
	hfuzz-clang $(HONGGFUZZ_CFLAGS) -c -o $@ $< 2>/dev/null || clang $(HONGGFUZZ_CFLAGS) -c -o $@ $<

# Build the main GPS parser application
$(TARGET): $(MAIN_SRC) $(LIBRARY)
	@echo "Building $(TARGET)..."
	$(CC) $(CFLAGS) -o $@ $(MAIN_SRC) -L. -lgps


# Test the main application
test: $(TARGET) 
	@echo "Testing main application with sample data..."
	@echo "=== Testing valid GPS data ==="
	./$(TARGET) test_data/valid.nmea
	@echo ""
	@echo "=== Testing divide by zero trigger ==="
	./$(TARGET) test_data/divide_by_zero.nmea
	@echo ""
	@echo "=== Testing integer overflow trigger ==="
	./$(TARGET) test_data/integer_overflow.nmea

# Fuzzing targets - delegate to fuzz/Makefile
{{#if (eq integration "make")}}
# Your selected fuzzer: {{default_fuzzer}}
fuzz: fuzz-{{default_fuzzer}}

{{#if (eq default_fuzzer "standalone")}}
fuzz-{{default_fuzzer}}: lib-fuzz
	$(MAKE) -C fuzz {{default_fuzzer}}
{{else}}
fuzz-{{default_fuzzer}}: lib-{{default_fuzzer}}
	$(MAKE) -C fuzz {{default_fuzzer}}
{{/if}}

{{#if (eq default_fuzzer "standalone")}}
# Additional fuzzer options (install engines as needed)
fuzz-afl: lib-afl
	$(MAKE) -C fuzz afl

fuzz-libfuzzer: lib-libfuzzer
	$(MAKE) -C fuzz libfuzzer

fuzz-honggfuzz: lib-honggfuzz
	$(MAKE) -C fuzz honggfuzz
{{/if}}

{{#if (eq default_fuzzer "afl")}}
# Additional fuzzer options
fuzz-standalone: lib-fuzz
	$(MAKE) -C fuzz standalone

fuzz-libfuzzer: lib-libfuzzer
	$(MAKE) -C fuzz libfuzzer

fuzz-honggfuzz: lib-honggfuzz
	$(MAKE) -C fuzz honggfuzz
{{/if}}

{{#if (eq default_fuzzer "libfuzzer")}}
# Additional fuzzer options
fuzz-standalone: lib-fuzz
	$(MAKE) -C fuzz standalone

fuzz-afl: lib-afl
	$(MAKE) -C fuzz afl

fuzz-honggfuzz: lib-honggfuzz
	$(MAKE) -C fuzz honggfuzz
{{/if}}

{{#if (eq default_fuzzer "honggfuzz")}}
# Additional fuzzer options
fuzz-standalone: lib-fuzz
	$(MAKE) -C fuzz standalone

fuzz-afl: lib-afl
	$(MAKE) -C fuzz afl

fuzz-libfuzzer: lib-libfuzzer
	$(MAKE) -C fuzz libfuzzer
{{/if}}

fuzz-test:
	$(MAKE) -C fuzz test

fuzz-clean:
	$(MAKE) -C fuzz clean

fuzz-help:
	$(MAKE) -C fuzz help
{{/if}}

{{#if (eq integration "standalone")}}
# For standalone integration, use build.sh directly
fuzz:
	cd fuzz && ./build.sh

fuzz-test:
	cd fuzz && ./build.sh && echo "Test input" | ./{{target_name}}-standalone

fuzz-clean:
	cd fuzz && rm -f {{target_name}}-*
{{/if}}

clean: fuzz-clean
	@echo "Cleaning build artifacts..."
	rm -f $(TARGET) $(LIBRARY) $(FUZZ_LIBRARY) $(LIBFUZZER_LIBRARY) $(AFL_LIBRARY) $(HONGGFUZZ_LIBRARY)
	rm -f $(SRC_DIR)/*.o $(SRC_DIR)/*-fuzz.o $(SRC_DIR)/*-libfuzzer.o $(SRC_DIR)/*-afl.o $(SRC_DIR)/*-honggfuzz.o
	rm -rf $(TARGET).dSYM

help:
	@echo "Available targets:"
	@echo "  all           - Build library and GPS parser application (default)"
	@echo "  lib           - Build only the GPS parser library ($(LIBRARY))"
	@echo "  $(TARGET)     - Build the GPS parser application"  
	@echo "  test          - Test the application with sample data"
	@echo "  clean         - Remove all build artifacts"
	@echo "  help          - Show this help message"
	@echo ""
	@echo "Fuzzing targets:"
{{#if (eq integration "make")}}
	@echo "  fuzz          - Build your selected fuzzer ({{default_fuzzer}})"
	@echo "  fuzz-{{default_fuzzer}} - Build {{default_fuzzer}} fuzzer"
	@echo "  fuzz-test     - Run fuzzer smoke test"
	@echo "  fuzz-clean    - Clean fuzzer artifacts"
	@echo "  fuzz-help     - Show detailed fuzzing help"
{{#if (eq default_fuzzer "standalone")}}
	@echo "  fuzz-afl      - Build AFL fuzzer (install AFL++ first)"
	@echo "  fuzz-libfuzzer - Build libFuzzer (install clang first)"
	@echo "  fuzz-honggfuzz - Build HonggFuzz (install honggfuzz first)"
{{/if}}
{{#if (eq default_fuzzer "afl")}}
	@echo "  fuzz-standalone - Build standalone fuzzer"
	@echo "  fuzz-libfuzzer - Build libFuzzer"
	@echo "  fuzz-honggfuzz - Build HonggFuzz"
{{/if}}
{{#if (eq default_fuzzer "libfuzzer")}}
	@echo "  fuzz-standalone - Build standalone fuzzer"
	@echo "  fuzz-afl      - Build AFL fuzzer"
	@echo "  fuzz-honggfuzz - Build HonggFuzz"
{{/if}}
{{#if (eq default_fuzzer "honggfuzz")}}
	@echo "  fuzz-standalone - Build standalone fuzzer"
	@echo "  fuzz-afl      - Build AFL fuzzer"
	@echo "  fuzz-libfuzzer - Build libFuzzer"
{{/if}}
{{/if}}
{{#if (eq integration "standalone")}}
	@echo "  fuzz          - Build fuzzer using build.sh"
	@echo "  fuzz-test     - Build and test fuzzer"
	@echo "  fuzz-clean    - Clean fuzzer artifacts"
{{/if}}