#!/bin/bash

# x402 Rust Release Script
# This script helps prepare and publish the x402 Rust crate

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

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Please run this script from the rust project root directory"
    exit 1
fi

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    print_error "Cargo is not installed. Please install Rust first."
    exit 1
fi

print_status "Starting x402 Rust release process..."

# Step 1: Run all checks
print_status "Running pre-release checks..."

print_status "Running tests..."
cargo test --all-features
print_success "All tests passed!"

print_status "Running clippy..."
cargo clippy --all-features -- -D warnings
print_success "Clippy passed!"

print_status "Running format check..."
cargo fmt --all -- --check
print_success "Format check passed!"

# Step 2: Check if we're logged into crates.io
print_status "Checking crates.io authentication..."
if ! cargo login --help &> /dev/null; then
    print_warning "Not logged into crates.io. You'll need to run 'cargo login' first."
    print_warning "Get your token from: https://crates.io/settings/tokens"
fi

# Step 3: Show current version
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | cut -d'"' -f2)
print_status "Current version: $CURRENT_VERSION"

# Step 4: Ask for confirmation
echo
print_warning "This will publish version $CURRENT_VERSION to crates.io"
read -p "Do you want to continue? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    print_status "Release cancelled."
    exit 0
fi

# Step 5: Publish
print_status "Publishing to crates.io..."
if cargo publish; then
    print_success "Successfully published x402 v$CURRENT_VERSION to crates.io!"
    print_success "Crate URL: https://crates.io/crates/x402"
    print_success "Documentation: https://docs.rs/x402"
else
    print_error "Failed to publish to crates.io"
    exit 1
fi

# Step 6: Create git tag
print_status "Creating git tag..."
if git tag "v$CURRENT_VERSION"; then
    print_success "Created git tag v$CURRENT_VERSION"
    print_warning "Don't forget to push the tag: git push origin v$CURRENT_VERSION"
else
    print_warning "Failed to create git tag (tag might already exist)"
fi

print_success "Release process completed!"
print_status "Next steps:"
echo "  1. Push the git tag: git push origin v$CURRENT_VERSION"
echo "  2. Create a GitHub release"
echo "  3. Update any dependent projects"
echo "  4. Monitor for issues and feedback"
