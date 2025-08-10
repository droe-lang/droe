#!/bin/bash

set -e  # Exit immediately if a command exits with a non-zero status

# Script should be run from installer/mac directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

echo "🧹 Cleaning old builds..."
rm -rf dist build

echo "📦 Packaging Roelang Installer with PyInstaller..."

# Go to project root to access files
cd ../..
PROJECT_ROOT=$(pwd)

# Build RoeVM if build script exists
if [ -f "$PROJECT_ROOT/build-roevm.sh" ] && [ -d "$PROJECT_ROOT/roevm" ]; then
    echo "🔨 Building RoeVM..."
    "$PROJECT_ROOT/build-roevm.sh"
fi

# Check if RoeVM binary exists and copy it locally to avoid path conflicts
if [ -f "$PROJECT_ROOT/roevm/target/release/roevm" ]; then
    echo "🦀 Found RoeVM binary, including in package..."
    # Copy to installer/mac directory with a different name to avoid conflict with roevm source directory
    cp "$PROJECT_ROOT/roevm/target/release/roevm" "$PROJECT_ROOT/installer/mac/roevm_binary"
    ROEVM_DATA="--add-binary $PROJECT_ROOT/installer/mac/roevm_binary:."
    CLEANUP_ROEVM=true
else
    echo "⚠️  RoeVM binary not found, skipping..."
    ROEVM_DATA=""
    CLEANUP_ROEVM=false
fi

# Run PyInstaller from project root with correct paths
pyinstaller installer/mac/installer_gui.py \
  --windowed \
  --name "Roelang Installer" \
  --add-data "$PROJECT_ROOT/roe:." \
  --add-data "$PROJECT_ROOT/compiler:compiler" \
  --add-data "$PROJECT_ROOT/run.js:." \
  --add-data "$PROJECT_ROOT/assets:assets" \
  $ROEVM_DATA \
  --icon="$PROJECT_ROOT/assets/icon.icns" \
  --distpath installer/mac/dist \
  --workpath installer/mac/build \
  --specpath installer/mac

# Clean up temporary roevm_binary if it was created
if [ "$CLEANUP_ROEVM" = true ]; then
    rm -f "$PROJECT_ROOT/installer/mac/roevm_binary"
fi

# Go back to installer/mac directory
cd installer/mac

echo "💿 Building DMG..."
dmgbuild -s dmg-settings.py "Roelang Installer" dist/RoelangInstaller.dmg

echo "✅ Done: RoelangInstaller.dmg created in installer/mac/dist/"
