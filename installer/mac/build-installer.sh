#!/bin/bash

set -e  # Exit immediately if a command exits with a non-zero status

# Script should be run from installer/mac directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

echo "🧹 Cleaning old builds..."
rm -rf dist build

echo "📦 Packaging Droelang Installer with PyInstaller..."

# Go to project root to access files
cd ../..
PROJECT_ROOT=$(pwd)

# Build DroeVM if build script exists
if [ -f "$PROJECT_ROOT/build-droevm.sh" ] && [ -d "$PROJECT_ROOT/droevm" ]; then
    echo "🔨 Building DroeVM..."
    "$PROJECT_ROOT/build-droevm.sh"
fi

# Check if DroeVM binary exists and copy it locally to avoid path conflicts
if [ -f "$PROJECT_ROOT/droevm/target/release/droevm" ]; then
    echo "🦀 Found DroeVM binary, including in package..."
    # Copy to installer/mac directory with a different name to avoid conflict with droevm source directory
    cp "$PROJECT_ROOT/droevm/target/release/droevm" "$PROJECT_ROOT/installer/mac/droevm_binary"
    ROEVM_DATA="--add-binary $PROJECT_ROOT/installer/mac/droevm_binary:."
    CLEANUP_ROEVM=true
else
    echo "⚠️  DroeVM binary not found, skipping..."
    ROEVM_DATA=""
    CLEANUP_ROEVM=false
fi

# Run PyInstaller from project root with correct paths
pyinstaller installer/mac/installer_gui.py \
  --windowed \
  --name "Droelang Installer" \
  --add-data "$PROJECT_ROOT/droe:." \
  --add-data "$PROJECT_ROOT/compiler:compiler" \
  --add-data "$PROJECT_ROOT/run.js:." \
  --add-data "$PROJECT_ROOT/assets:assets" \
  $ROEVM_DATA \
  --icon="$PROJECT_ROOT/assets/icon.icns" \
  --distpath installer/mac/dist \
  --workpath installer/mac/build \
  --specpath installer/mac

# Clean up temporary droevm_binary if it was created
if [ "$CLEANUP_ROEVM" = true ]; then
    rm -f "$PROJECT_ROOT/installer/mac/droevm_binary"
fi

# Go back to installer/mac directory
cd installer/mac

echo "💿 Building DMG..."
dmgbuild -s dmg-settings.py "Droelang Installer" dist/DroelangInstaller.dmg

echo "✅ Done: DroelangInstaller.dmg created in installer/mac/dist/"
