#!/bin/bash

# sync-dev.sh - Development deployment script
# Syncs the compiler and roe CLI to ~/.roelang/ for development/testing

set -e

echo "🔄 Syncing development files to ~/.roelang/"

# Ensure ~/.roelang exists
mkdir -p ~/.roelang

# Sync the compiler directory
if [ -d "compiler" ]; then
    echo "📂 Syncing compiler module..."
    rsync -av --delete compiler/ ~/.roelang/compiler/
    echo "✅ Compiler synced"
else
    echo "❌ Error: compiler/ directory not found"
    exit 1
fi

# Sync the roe CLI
if [ -f "roe" ]; then
    echo "🛠️  Syncing roe CLI..."
    cp roe ~/.roelang/roe
    chmod +x ~/.roelang/roe
    echo "✅ roe CLI synced"
else
    echo "❌ Error: roe file not found"
    exit 1
fi

# Sync additional runtime files if they exist
if [ -f "run.js" ]; then
    echo "🟨 Syncing Node.js runtime..."
    cp run.js ~/.roelang/run.js
    echo "✅ Node.js runtime synced"
fi

# Build and sync RoeVM
if [ -d "roevm" ]; then
    # Try to build RoeVM first
    if [ -f "build-roevm.sh" ]; then
        echo "🔨 Building RoeVM..."
        ./build-roevm.sh
    fi
    
    # Now sync the binary if it exists
    if [ -f "roevm/target/release/roevm" ]; then
        echo "🦀 Syncing RoeVM binary..."
        cp roevm/target/release/roevm ~/.roelang/roevm
        chmod +x ~/.roelang/roevm
        echo "✅ RoeVM binary synced"
    else
        echo "⚠️  RoeVM binary not found. Build may have failed."
    fi
fi

# Create bin directory and symlink for PATH usage
mkdir -p ~/.roelang/bin
if [ ! -e ~/.roelang/bin/roe ]; then
    ln -s ~/.roelang/roe ~/.roelang/bin/roe
    echo "🔗 Created symlink: ~/.roelang/bin/roe"
fi

echo ""
echo "✅ Development sync complete!"
echo ""
echo "📋 Synced components:"
echo "  • Compiler module: ~/.roelang/compiler/"
echo "  • roe CLI: ~/.roelang/roe"
echo "  • roe symlink: ~/.roelang/bin/roe"
if [ -f ~/.roelang/run.js ]; then
    echo "  • Node.js runtime: ~/.roelang/run.js"
fi
if [ -f ~/.roelang/roevm ]; then
    echo "  • RoeVM binary: ~/.roelang/roevm"
fi

echo ""
echo "💡 Make sure ~/.roelang/bin is in your PATH:"
echo "   export PATH=\"\$HOME/.roelang/bin:\$PATH\""
echo ""
echo "🚀 You can now test with: roe --help"