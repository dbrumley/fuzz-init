#!/bin/bash
set -e

echo "Building fuzz-init..."
cargo build --release

echo "Installing to /usr/local/bin (requires sudo)..."
sudo cp target/release/fuzz-init /usr/local/bin/

echo "✅ Installation complete!"
echo ""
echo "fuzz-init has been installed to /usr/local/bin/"
echo "Run 'fuzz-init --help' to get started."
echo ""
echo "Example usage:"
echo "  fuzz-init my-project --language c"
echo "  fuzz-init my-project --template github:user/repo"
echo "  fuzz-init my-project --language rust --minimal"