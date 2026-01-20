#!/bin/bash
# =============================================================================
# Publish Hope OS to crates.io
# =============================================================================

set -e

echo "╦ ╦╔═╗╔═╗╔═╗  ╔═╗╔═╗"
echo "╠═╣║ ║╠═╝║╣   ║ ║╚═╗"
echo "╩ ╩╚═╝╩  ╚═╝  ╚═╝╚═╝"
echo ""
echo "Publishing to crates.io..."
echo ""

# Check if logged in
if ! cargo login --help > /dev/null 2>&1; then
    echo "Error: cargo not found"
    exit 1
fi

# Run tests first
echo "Running tests..."
cargo test --release

# Check clippy
echo "Running clippy..."
cargo clippy --all-targets -- -D warnings

# Format check
echo "Checking format..."
cargo fmt -- --check

# Dry run
echo "Dry run..."
cargo publish --dry-run

# Confirm
echo ""
read -p "Ready to publish to crates.io? (y/n) " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    cargo publish
    echo ""
    echo "Published to crates.io!"
    echo "https://crates.io/crates/hope-os"
else
    echo "Cancelled."
fi
