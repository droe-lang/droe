#!/bin/bash

# sync-dev.sh - Development deployment script
# Syncs the compiler and ddroe CLI to ~/.ddroelang/ for development/testing

set -e

echo "🔄 Syncing development files to ~/.ddroelang/"

# Ensure ~/.ddroelang exists
mkdir -p ~/.ddroelang

# Sync the compiler directory
if [ -d "compiler" ]; then
    echo "📂 Syncing compiler module..."
    rsync -av --delete compiler/ ~/.ddroelang/compiler/
    echo "✅ Compiler synced"
else
    echo "❌ Error: compiler/ directory not found"
    exit 1
fi

# Sync the droe CLI
if [ -f "droe" ]; then
    echo "🛠️  Syncing droe CLI..."
    cp droe ~/.ddroelang/droe
    chmod +x ~/.ddroelang/droe
    echo "✅ droe CLI synced"
else
    echo "❌ Error: droe file not found"
    exit 1
fi

# Sync additional runtime files if they exist
if [ -f "run.js" ]; then
    echo "🟨 Syncing Node.js runtime..."
    cp run.js ~/.ddroelang/run.js
    echo "✅ Node.js runtime synced"
fi

# Build and sync DroeVM
if [ -d "ddroevm" ]; then
    # Try to build DroeVM first
    if [ -f "build-ddroevm.sh" ]; then
        echo "🔨 Building DroeVM..."
        ./build-ddroevm.sh
    fi
    
    # Now sync the binary if it exists
    if [ -f "ddroevm/target/release/ddroevm" ]; then
        echo "🦀 Syncing DroeVM binary..."
        cp ddroevm/target/release/ddroevm ~/.ddroelang/ddroevm
        chmod +x ~/.ddroelang/ddroevm
        echo "✅ DroeVM binary synced"
    else
        echo "⚠️  DroeVM binary not found. Build may have failed."
    fi
fi

# Create bin directory and symlink for PATH usage
mkdir -p ~/.ddroelang/bin
if [ ! -e ~/.ddroelang/bin/droe ]; then
    ln -s ~/.ddroelang/droe ~/.ddroelang/bin/droe
    echo "🔗 Created symlink: ~/.ddroelang/bin/droe"
fi

echo ""
echo "✅ Development sync complete!"
echo ""
echo "📋 Synced components:"
echo "  • Compiler module: ~/.ddroelang/compiler/"
echo "  • droe CLI: ~/.ddroelang/droe"
echo "  • droe symlink: ~/.ddroelang/bin/droe"
if [ -f ~/.ddroelang/run.js ]; then
    echo "  • Node.js runtime: ~/.ddroelang/run.js"
fi
if [ -f ~/.ddroelang/ddroevm ]; then
    echo "  • DroeVM binary: ~/.ddroelang/ddroevm"
fi

echo ""
echo "💡 Make sure ~/.ddroelang/bin is in your PATH:"
echo "   export PATH=\"\$HOME/.ddroelang/bin:\$PATH\""
echo ""
echo "🚀 You can now test with: droe --help"