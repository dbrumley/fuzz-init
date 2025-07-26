#!/bin/bash
set -e

echo "Building fuzz-init..."
cargo build

echo "Generating test project..."
rm -rf test-fixed
./target/debug/fuzz-init test-fixed --language c --integration make --fuzzer libfuzzer

echo "Testing the build..."
cd test-fixed
make lib-libfuzzer
make fuzz-libfuzzer

echo "Success! The templates work correctly."