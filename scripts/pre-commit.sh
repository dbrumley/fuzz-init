#!/bin/bash

# Pre-commit hook for fuzz-init
# This script runs before each commit to ensure code quality

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_status "Running pre-commit checks..."

# Check if we have Python and PyYAML
if ! python3 -c "import yaml" 2>/dev/null; then
    print_warning "PyYAML not installed. Skipping YAML validation."
    YAML_CHECK=false
else
    YAML_CHECK=true
fi

# 1. Validate YAML files
if [ "$YAML_CHECK" = true ]; then
    print_status "Validating YAML files..."
    if python3 scripts/validate-yaml.py; then
        print_success "All YAML files are valid"
    else
        print_error "YAML validation failed"
        exit 1
    fi
fi

# 2. Check Rust code formatting
print_status "Checking Rust code formatting..."
if cargo fmt --check; then
    print_success "Rust code is properly formatted"
else
    print_error "Rust code formatting check failed"
    print_status "Run 'cargo fmt' to fix formatting"
    exit 1
fi

# 3. Check for warnings in debug build
print_status "Checking for warnings..."
if cargo build 2>&1 | grep -E "(warning|error)" | grep -v "unused imports" | grep -v "unused variables" | grep -v "dead_code"; then
    print_warning "Found warnings in build (excluding common ones)"
else
    print_success "No significant warnings found"
fi

# 4. Run basic tests
print_status "Running basic tests..."
if cargo test; then
    print_success "Basic tests passed"
else
    print_error "Basic tests failed"
    exit 1
fi

# 5. Check if Cargo.toml version is reasonable
print_status "Checking Cargo.toml version..."
VERSION=$(grep '^version = ' Cargo.toml | cut -d'"' -f2)
if [[ $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    print_success "Version format is valid: $VERSION"
else
    print_warning "Version format may be invalid: $VERSION"
fi

# 6. Check for large files that shouldn't be committed
print_status "Checking for large files..."
LARGE_FILES=$(find . -type f -size +10M -not -path "./.git/*" -not -path "./target/*" -not -path "./scratch/*" | head -5)
if [ -n "$LARGE_FILES" ]; then
    print_warning "Found large files (>10MB):"
    echo "$LARGE_FILES"
    print_warning "Consider adding these to .gitignore"
fi

print_success "Pre-commit checks completed successfully!"
print_status "Ready to commit!" 