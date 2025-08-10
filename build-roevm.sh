#!/bin/bash

# build-roevm.sh - Build the RoeVM binary
set -e

echo "🦀 Building RoeVM..."

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Rust/Cargo is not installed"
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

# Build RoeVM
cd roevm
cargo build --release

if [ -f "target/release/roevm" ]; then
    echo "✅ RoeVM built successfully: roevm/target/release/roevm"
    ls -lh target/release/roevm
else
    echo "❌ Error: RoeVM binary was not created"
    exit 1
fi

cd ..
echo "🚀 RoeVM build complete!"