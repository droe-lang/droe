#!/bin/bash

# sync-dev.sh - Development deployment script
# Syncs the unified droe binary to ~/.droelang/ for development/testing

set -e

# Ensure we're in the project root directory
cd "$(dirname "$0")/.."

echo "🔄 Syncing unified droe binary to ~/.droelang/"

# Ensure ~/.droelang exists
mkdir -p ~/.droelang

# Build the unified binary first
echo "🔨 Building unified droe binary..."
if cargo build --release; then
    echo "✅ Unified droe binary built successfully!"
    ls -lh target/release/droe
else
    echo "❌ Failed to build unified droe binary"
    exit 1
fi

# Sync the unified droe binary
if [ -f "target/release/droe" ]; then
    echo "🛠️  Syncing unified droe binary..."
    cp target/release/droe ~/.droelang/droe
    chmod +x ~/.droelang/droe
    echo "✅ Unified droe binary synced"
else
    echo "❌ Error: target/release/droe binary not found"
    exit 1
fi

# Create bin directory and symlink for PATH usage
mkdir -p ~/.droelang/bin
if [ -e ~/.droelang/bin/droe ]; then
    rm ~/.droelang/bin/droe  # Remove old symlink
fi
ln -s ~/.droelang/droe ~/.droelang/bin/droe
echo "🔗 Created symlink: ~/.droelang/bin/droe"

# Optionally create system-wide symlink to /usr/local/bin
if [ -w /usr/local/bin ] || [ "$(id -u)" = "0" ]; then
    # We have write access to /usr/local/bin (either writable or running as root)
    if [ -e /usr/local/bin/droe ]; then
        rm /usr/local/bin/droe  # Remove old symlink
    fi
    ln -s ~/.droelang/droe /usr/local/bin/droe
    echo "🌐 Created system-wide symlink: /usr/local/bin/droe"
    echo "✅ droe command is now available system-wide!"
else
    # Need sudo for /usr/local/bin
    echo ""
    echo "💡 For system-wide access, run:"
    echo "   sudo ln -sf ~/.droelang/droe /usr/local/bin/droe"
    echo "   (This will make 'droe' available without PATH modification)"
fi

# Clean up legacy files if they exist
echo "🧹 Cleaning up legacy files..."
if [ -d ~/.droelang/compiler ]; then
    rm -rf ~/.droelang/compiler
    echo "✅ Removed legacy Python compiler"
fi
if [ -f ~/.droelang/droevm ]; then
    rm ~/.droelang/droevm
    echo "✅ Removed separate DroeVM binary"
fi
if [ -f ~/.droelang/droe-llm ]; then
    rm ~/.droelang/droe-llm
    echo "✅ Removed separate DroeLLM binary"
fi
if [ -f ~/.droelang/run.js ]; then
    rm ~/.droelang/run.js
    echo "✅ Removed legacy Node.js runtime"
fi

echo ""
echo "✅ Development sync complete!"
echo ""
echo "📋 Unified binary includes all functionality:"
echo "  🛠️  CLI: droe init, compile, run, build, clean"
echo "  🚀 VM: droe vm run, validate, info"
echo "  🔌 LSP: droe lsp (for IDE integration)"
echo "  🤖 LLM: droe chat, llm-server (gRPC for VSCode)"
echo "  🏗️  Generate: droe generate spring|fastapi|fiber"
echo "  📦 Utils: droe lint, format, reverse"
echo ""
echo "📍 Binary location: ~/.droelang/droe (~$(du -h ~/.droelang/droe | cut -f1))"
echo "🔗 Local symlink: ~/.droelang/bin/droe"
if [ -L /usr/local/bin/droe ]; then
    echo "🌐 System symlink: /usr/local/bin/droe"
fi
echo ""
if [ -L /usr/local/bin/droe ]; then
    echo "✅ droe command is available system-wide!"
else
    echo "💡 Make sure ~/.droelang/bin is in your PATH:"
    echo "   export PATH=\"\$HOME/.droelang/bin:\$PATH\""
    echo ""
    echo "🌐 Or create system-wide symlink:"
    echo "   sudo ln -sf ~/.droelang/droe /usr/local/bin/droe"
fi
echo ""
echo "🚀 Test the unified binary:"
echo "   droe --help"
echo "   droe --version"
echo ""
echo "🤖 Start services:"
echo "   droe lsp --mode tcp --port 9257    # LSP server"
echo "   droe llm-server --port 50051       # gRPC LLM service"
echo "   droe daemon --port 9258 --llm      # Background daemon"
echo ""
echo "⚡ All functionality is now unified in one binary!"