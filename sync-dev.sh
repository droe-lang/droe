#!/bin/bash

# Development sync script for Roelang
# Syncs files to ~/.roelang without needing to build and install DMG

set -e

echo "🔄 Syncing Roelang development files to ~/.roelang..."

# Create directories
mkdir -p ~/.roelang/bin
mkdir -p ~/.roelang/compiler

# Copy files
echo "📁 Copying roe CLI..."
cp roe ~/.roelang/bin/roe
chmod +x ~/.roelang/bin/roe

echo "📁 Copying compiler module..."
# Copy the entire compiler module with all its components
cp -r compiler/* ~/.roelang/compiler/
# Also keep the standalone for backward compatibility
cp compiler_standalone.py ~/.roelang/compiler.py

echo "📁 Copying runtime..."
cp run.js ~/.roelang/run.js

# Check if PATH is already configured
SHELL_RC=""
if [[ -f ~/.zshrc ]]; then
    SHELL_RC=~/.zshrc
elif [[ -f ~/.bashrc ]]; then
    SHELL_RC=~/.bashrc
else
    SHELL_RC=~/.zshrc
fi

PATH_LINE='export PATH="$HOME/.roelang/bin:$PATH"'

if [[ -f "$SHELL_RC" ]]; then
    if ! grep -q "$PATH_LINE" "$SHELL_RC"; then
        echo "🔧 Adding ~/.roelang/bin to PATH in $SHELL_RC"
        echo "" >> "$SHELL_RC"
        echo "$PATH_LINE" >> "$SHELL_RC"
        echo "⚠️  Please restart your terminal or run: source $SHELL_RC"
    else
        echo "✅ PATH already configured in $SHELL_RC"
    fi
else
    echo "📝 Creating $SHELL_RC with PATH configuration"
    echo "$PATH_LINE" > "$SHELL_RC"
    echo "⚠️  Please restart your terminal or run: source $SHELL_RC"
fi

echo "✅ Roelang development files synced successfully!"
echo "💡 You can now use 'roe' commands from anywhere"