#!/bin/bash

# Simple build script for the example app

set -e

SRC_DIR="src"

# Default compiler settings
CC="${CC:-gcc}"
CFLAGS="-Wall -Wextra -g -O0 "


echo "🔨 Building hello_fuzz application..."
echo "Compiler: $CC"

$CC $CFLAGS \
    -I. \
    $SRC_DIR/main.c \
    $SRC_DIR/lib.c \
    -o hello_fuzz \
    $LDFLAGS

echo "✅ Built hello_fuzz successfully!"



echo ""
echo "🎯 Build complete! Next steps:"
echo ""
echo "  # Test the application:"
echo "  ./hello_fuzz test_data/valid.txt"
echo ""
echo "  # Test vulnerability triggers:"
echo "  ./hello_fuzz test_data/oob_write.txt"
