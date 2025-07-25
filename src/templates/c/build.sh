#!/bin/bash

# Simple build script for GPS parser application

set -e

SRC_DIR="src"

# Default compiler settings
CC="${CC:-gcc}"
CFLAGS="-Wall -Wextra -g -O0 -fsanitize=address"
LDFLAGS="-fsanitize=address"

echo "🔨 Building GPS parser application..."
echo "Compiler: $CC"

# Build the main application
echo "Building GPS parser..."
$CC $CFLAGS \
    -I. \
    $SRC_DIR/gps_parser.c \
    $SRC_DIR/main.c \
    -o gps_parser \
    $LDFLAGS

echo "✅ Built gps_parser successfully!"

# Create test data if it doesn't exist
if [[ ! -d "test_data" ]]; then
    echo "📁 Creating test data..."
    mkdir -p test_data
    echo "GPGGA,123456.78,1234,N,5678,W,1,08,0.9,545.4,M,46.9,M,,*47" > test_data/valid.nmea
    echo "GPGGA,123456.78,1,N,0,W,1,08,0.9,545.4,M,46.9,M,,*47" > test_data/divide_by_zero.nmea
    echo "GPGGA,123456.78,2,N,-79927771,W,1,08,0.9,545.4,M,46.9,M,,*47" > test_data/integer_overflow.nmea
    echo "GPGGA,123456.78,3,N,-79927771,W,1,08,0.9,545.4,M,46.9,M,,*47" > test_data/oob_read.nmea
    echo "Invalid GPS data without proper format" > test_data/invalid.nmea
fi

echo ""
echo "🎯 Build complete! Next steps:"
echo ""
echo "  # Test the application:"
echo "  ./gps_parser test_data/valid.nmea"
echo ""
echo "  # Test vulnerability triggers:"
echo "  ./gps_parser test_data/divide_by_zero.nmea"
echo "  ./gps_parser test_data/integer_overflow.nmea"