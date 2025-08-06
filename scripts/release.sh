#!/bin/bash

# fuzz-init Release Script
# Usage: ./scripts/release.sh [version]
# Example: ./scripts/release.sh 1.0.0

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
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

# Check if version is provided
if [ $# -eq 0 ]; then
    print_error "Version number is required"
    echo "Usage: $0 <version>"
    echo "Example: $0 1.0.0"
    exit 1
fi

VERSION=$1

# Validate version format (simple check)
if [[ ! $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    print_error "Invalid version format. Use semantic versioning (e.g., 1.0.0)"
    exit 1
fi

print_status "Starting release process for version $VERSION"

# Check if we're on main branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "main" ]; then
    print_warning "You're not on the main branch. Current branch: $CURRENT_BRANCH"
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_error "Release cancelled"
        exit 1
    fi
fi

# Check if working directory is clean
if [ -n "$(git status --porcelain)" ]; then
    print_error "Working directory is not clean. Please commit or stash changes first."
    git status --short
    exit 1
fi

# Check if tag already exists
if git tag -l | grep -q "^v$VERSION$"; then
    print_error "Tag v$VERSION already exists"
    exit 1
fi

# Update version in Cargo.toml
print_status "Updating version in Cargo.toml"
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
else
    # Linux
    sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
fi

# Verify the change
if ! grep -q "^version = \"$VERSION\"" Cargo.toml; then
    print_error "Failed to update version in Cargo.toml"
    exit 1
fi

print_success "Updated Cargo.toml to version $VERSION"

# Build and test
print_status "Building and testing..."
cargo build --release
cargo test

print_success "Build and tests passed"

# Commit version change
print_status "Committing version change"
git add Cargo.toml
git commit -m "Bump version to $VERSION"

# Create and push tag
print_status "Creating tag v$VERSION"
git tag -a "v$VERSION" -m "Release version $VERSION"

print_status "Pushing changes and tag"
git push origin main
git push origin "v$VERSION"

print_success "Release v$VERSION has been created and pushed!"
print_status "GitHub Actions will now build and create the release automatically"
print_status "You can monitor the progress at: https://github.com/dbrumley/fuzz-init/actions"

# Optional: Open the releases page
if command -v open >/dev/null 2>&1; then
    print_status "Opening GitHub releases page..."
    open "https://github.com/dbrumley/fuzz-init/releases"
elif command -v xdg-open >/dev/null 2>&1; then
    print_status "Opening GitHub releases page..."
    xdg-open "https://github.com/dbrumley/fuzz-init/releases"
fi

print_success "Release process completed!" 