#!/bin/bash
# =============================================================================
# Publish Hope OS to PyPI
# =============================================================================

set -e

echo "╦ ╦╔═╗╔═╗╔═╗  ╔═╗╔═╗"
echo "╠═╣║ ║╠═╝║╣   ║ ║╚═╗"
echo "╩ ╩╚═╝╩  ╚═╝  ╚═╝╚═╝"
echo ""
echo "Publishing to PyPI..."
echo ""

# Check maturin
if ! command -v maturin &> /dev/null; then
    echo "Installing maturin..."
    pip install maturin
fi

# Build wheels for multiple Python versions
echo "Building wheels..."
maturin build --release --features python

# Run Python tests
echo "Running Python tests..."
pip install -e . --features python
pytest tests/

# Confirm
echo ""
read -p "Ready to publish to PyPI? (y/n) " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    maturin publish --features python
    echo ""
    echo "Published to PyPI!"
    echo "https://pypi.org/project/hope-os/"
else
    echo "Cancelled."
fi
