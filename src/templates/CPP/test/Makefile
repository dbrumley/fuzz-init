{{#unless minimal}}
# Test Makefile for {{project_name}}
# Builds and runs unit tests for the library

CXX ?= c++  
CXXFLAGS = -g -O0 -Wall -Wextra -std=c++11
INCLUDES = -I../include
LIBPATH = -L../build
LIBS = -l{{project_name}}

# Build directories
BUILD_DIR = ../build
TEST_BUILD_DIR = $(BUILD_DIR)/test
LIBRARY_PATH = $(BUILD_DIR)/lib{{project_name}}.a

# Source files
TEST_SOURCES = test_lib.cpp
TEST_TARGET = $(TEST_BUILD_DIR)/test_{{project_name}}

# Default target
all: test

# Create test build directory
$(TEST_BUILD_DIR):
	@mkdir -p $(TEST_BUILD_DIR)

# Build the test executable
$(TEST_TARGET): $(TEST_SOURCES) | check-library $(TEST_BUILD_DIR)
	@echo "Building test suite..."
	$(CXX) $(CXXFLAGS) $(INCLUDES) $(TEST_SOURCES) $(LIBPATH) $(LIBS) -o $@

# Run the tests
test: $(TEST_TARGET)
	@echo "Running test suite..."
	$(TEST_TARGET)

# Check that the library exists
check-library:
	@if [ ! -f $(LIBRARY_PATH) ]; then \
		echo "Error: Library not found at $(LIBRARY_PATH)"; \
		echo "Run 'make lib' in parent directory first."; \
		exit 1; \
	fi
	@echo "✓ Library found: $(LIBRARY_PATH)"

# Clean test artifacts
clean:
	rm -f $(TEST_TARGET)
	rm -rf $(TEST_BUILD_DIR)

# Help target
help:
	@echo "Test Makefile for {{project_name}}"
	@echo ""
	@echo "Available targets:"
	@echo "  all          - Build and run tests (default)"
	@echo "  test         - Build and run tests"
	@echo "  clean        - Remove test build artifacts"
	@echo "  check-library - Verify main library exists"
	@echo "  help         - Show this help"
	@echo ""
	@echo "Dependencies:"
	@echo "  Requires lib{{project_name}}.a to be built first"
	@echo "  Run 'make lib' in parent directory before testing"

.PHONY: all test clean check-library help
{{else}}
# Minimal mode - no test Makefile generated
# For minimal mode, integrate tests with your existing build system
all:
	@echo "Minimal mode: Tests not included in minimal template"
	@echo "For testing guidance, see full mode template or INTEGRATION.md"

.PHONY: all
{{/unless}}