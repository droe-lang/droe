#!/bin/bash

# build-droevm.sh - Build the DroeVM binary
set -e

echo "🦀 Building DroeVM..."

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Rust/Cargo is not installed"
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

# Build DroeVM
cd droevm
cargo build --release

if [ -f "target/release/droevm" ]; then
    echo "✅ DroeVM built successfully: droevm/target/release/droevm"
    ls -lh target/release/droevm
else
    echo "❌ Error: DroeVM binary was not created"
    exit 1
fi

cd ..
echo "🚀 DroeVM build complete!"