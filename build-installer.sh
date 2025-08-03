#!/bin/bash

set -e  # Exit immediately if a command exits with a non-zero status

echo "🧹 Cleaning old builds..."
rm -rf dist build RoelangInstaller.dmg

echo "📦 Packaging Roelang Installer with PyInstaller..."
pyinstaller installer_gui.py \
  --windowed \
  --name "Roelang Installer" \
  --add-data "roe:." \
  --add-data "compiler.py:." \
  --add-data "run.js:." \
  --icon=assets/icon.icns

echo "💿 Building DMG..."
dmgbuild -s dmg-settings.py "Roelang Installer" RoelangInstaller.dmg

echo "✅ Done: RoelangInstaller.dmg created."
