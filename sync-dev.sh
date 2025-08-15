#!/bin/bash

# sync-dev.sh - Development deployment script
# Syncs the compiler and droe CLI to ~/.droelang/ for development/testing

set -e

echo "🔄 Syncing development files to ~/.droelang/"

# Ensure ~/.droelang exists
mkdir -p ~/.droelang

# Sync the compiler directory
if [ -d "compiler" ]; then
    echo "📂 Syncing compiler module..."
    rsync -av --delete compiler/ ~/.droelang/compiler/
    echo "✅ Compiler synced"
else
    echo "❌ Error: compiler/ directory not found"
    exit 1
fi

# Sync the droe CLI
if [ -f "droe" ]; then
    echo "🛠️  Syncing droe CLI..."
    cp droe ~/.droelang/droe
    chmod +x ~/.droelang/droe
    echo "✅ droe CLI synced"
else
    echo "❌ Error: droe file not found"
    exit 1
fi

# Sync additional runtime files if they exist
if [ -f "run.js" ]; then
    echo "🟨 Syncing Node.js runtime..."
    cp run.js ~/.droelang/run.js
    echo "✅ Node.js runtime synced"
fi

# Build and sync DroeVM
if [ -d "droevm" ]; then
    # Try to build DroeVM first
    if [ -f "build-droevm.sh" ]; then
        echo "🔨 Building DroeVM..."
        ./build-droevm.sh
    fi
    
    # Now sync the binary if it exists
    if [ -f "droevm/target/release/droevm" ]; then
        echo "🦀 Syncing DroeVM binary..."
        cp droevm/target/release/droevm ~/.droelang/droevm
        chmod +x ~/.droelang/droevm
        echo "✅ DroeVM binary synced"
    else
        echo "⚠️  DroeVM binary not found. Build may have failed."
    fi
fi

# Create bin directory and symlink for PATH usage
mkdir -p ~/.droelang/bin
if [ ! -e ~/.droelang/bin/droe ]; then
    ln -s ~/.droelang/droe ~/.droelang/bin/droe
    echo "🔗 Created symlink: ~/.droelang/bin/droe"
fi

echo ""
echo "✅ Development sync complete!"
echo ""
echo "📋 Synced components:"
echo "  • Compiler module: ~/.droelang/compiler/"
echo "  • droe CLI: ~/.droelang/droe"
echo "  • droe symlink: ~/.droelang/bin/droe"
if [ -f ~/.droelang/run.js ]; then
    echo "  • Node.js runtime: ~/.droelang/run.js"
fi
if [ -f ~/.droelang/droevm ]; then
    echo "  • DroeVM binary: ~/.droelang/droevm"
fi

# Create compiler bundle for install-from-curl flow
echo ""
echo "📦 Creating compiler bundle..."
if tar -czf compiler.tar.gz compiler/ droe run.js 2>/dev/null; then
    echo "✅ compiler.tar.gz updated ($(du -h compiler.tar.gz | cut -f1))"
else
    echo "⚠️  Warning: Could not create compiler.tar.gz"
fi

echo ""
echo "💡 Make sure ~/.droelang/bin is in your PATH:"
echo "   export PATH=\"\$HOME/.droelang/bin:\$PATH\""
echo ""
echo "🚀 You can now test with: droe --help"