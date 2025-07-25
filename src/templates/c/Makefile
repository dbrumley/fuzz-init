CC = gcc
CFLAGS = -Wall -Wextra -g -O0 -fsanitize=address
LDFLAGS = -fsanitize=address

# Source files
SRC_DIR = src
SOURCES = $(SRC_DIR)/gps_parser.c
MAIN_SRC = $(SRC_DIR)/main.c

# Target
TARGET = gps_parser

.PHONY: all clean test

all: $(TARGET)

# Build the main GPS parser application
$(TARGET): $(SOURCES) $(MAIN_SRC)
	@echo "Building GPS parser application..."
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS)

# Create sample test data
test-data:
	@echo "Creating sample GPS data files..."
	@mkdir -p test_data
	@echo "GPGGA,123456.78,1234,N,5678,W,1,08,0.9,545.4,M,46.9,M,,*47" > test_data/valid.nmea
	@echo "GPGGA,123456.78,1,N,0,W,1,08,0.9,545.4,M,46.9,M,,*47" > test_data/divide_by_zero.nmea
	@echo "GPGGA,123456.78,2,N,-79927771,W,1,08,0.9,545.4,M,46.9,M,,*47" > test_data/integer_overflow.nmea
	@echo "GPGGA,123456.78,3,N,-79927771,W,1,08,0.9,545.4,M,46.9,M,,*47" > test_data/oob_read.nmea
	@echo "Invalid GPS data without commas" > test_data/invalid.nmea

# Test the main application
test: $(TARGET) test-data
	@echo "Testing main application with sample data..."
	@echo "=== Testing valid GPS data ==="
	./$(TARGET) test_data/valid.nmea
	@echo ""
	@echo "=== Testing divide by zero trigger ==="
	./$(TARGET) test_data/divide_by_zero.nmea
	@echo ""
	@echo "=== Testing integer overflow trigger ==="
	./$(TARGET) test_data/integer_overflow.nmea

clean:
	@echo "Cleaning build artifacts..."
	rm -f $(TARGET)
	rm -rf test_data

help:
	@echo "Available targets:"
	@echo "  all           - Build the GPS parser application (default)"
	@echo "  $(TARGET)     - Build the GPS parser application"  
	@echo "  test-data     - Generate sample GPS test files"
	@echo "  test          - Test the application with sample data"
	@echo "  clean         - Remove all build artifacts"
	@echo "  help          - Show this help message"