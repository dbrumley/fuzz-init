.PHONY: build release install uninstall test clean

# Build debug version
build:
	cargo build

# Build optimized release version
release:
	cargo build --release

# Install to /usr/local/bin (requires sudo)
install: release
	sudo cp target/release/fuzz-init /usr/local/bin/
	@echo "fuzz-init installed to /usr/local/bin/"
	@echo "Run 'fuzz-init --help' to get started."

# Remove from /usr/local/bin
uninstall:
	sudo rm -f /usr/local/bin/fuzz-init
	@echo "fuzz-init removed from /usr/local/bin/"

# Run tests
test:
	cargo test

# Run clippy linter
clippy:
	cargo clippy

# Format code
fmt:
	cargo fmt

# Clean build artifacts
clean:
	cargo clean

# Show help
help:
	@echo "Available targets:"
	@echo "  build      - Build debug version"
	@echo "  release    - Build optimized release version"
	@echo "  install    - Install to /usr/local/bin (requires sudo)"
	@echo "  uninstall  - Remove from /usr/local/bin"
	@echo "  test       - Run tests"
	@echo "  clippy     - Run clippy linter"
	@echo "  fmt        - Format code"
	@echo "  clean      - Clean build artifacts"
	@echo "  help       - Show this help"