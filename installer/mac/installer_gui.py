#!/usr/bin/env python3

import tkinter as tk
from tkinter import messagebox
import os
import sys
import shutil
import platform
import subprocess
from pathlib import Path
from datetime import datetime

# Version information
ROE_VERSION = "1.0.0"
INSTALLER_VERSION = "1.0.0"
BUILD_DATE = datetime.now().strftime("%Y-%m-%d")

# Determine resource path based on bundled (.app) or dev environment
if getattr(sys, 'frozen', False):
    BASE_DIR = Path(sys._MEIPASS)
else:
    BASE_DIR = Path(__file__).parent

# Setup paths
HOME = Path.home()
ROELANG_DIR = HOME / ".roelang"
BIN_DIR = ROELANG_DIR / "bin"

# Determine shell config file
if platform.system() != "Windows":
    if (HOME / ".zshrc").exists():
        SHELL_RC = HOME / ".zshrc"
    elif (HOME / ".bashrc").exists():
        SHELL_RC = HOME / ".bashrc"
    else:
        SHELL_RC = HOME / ".zshrc"
else:
    SHELL_RC = None

def register_file_association():
    """Register .roe file association on macOS with icon"""
    if platform.system() != "Darwin":
        return True
    
    try:
        # Copy icon to a permanent location first
        icon_source = BASE_DIR / "assets" / "icon.icns"
        icon_dest = ROELANG_DIR / "icon.icns"
        
        if not icon_source.exists():
            print("Warning: icon.icns not found in assets")
            return False
            
        shutil.copy(icon_source, icon_dest)
        
        # Create Roe.app in Applications directory (renamed from RoeHandler)
        app_bundle = Path("/Applications/Roe.app")
        # Remove old RoeHandler.app if it exists
        old_app = Path("/Applications/RoeHandler.app")
        if old_app.exists():
            shutil.rmtree(old_app)
        if app_bundle.exists():
            shutil.rmtree(app_bundle)
        
        # Create a simple app bundle structure for the file association
        contents_dir = app_bundle / "Contents"
        macos_dir = contents_dir / "MacOS"
        resources_dir = contents_dir / "Resources"
        
        # Create directories
        macos_dir.mkdir(parents=True, exist_ok=True)
        resources_dir.mkdir(parents=True, exist_ok=True)
        
        # Copy icon to Resources - this is the key step that was missing
        shutil.copy(icon_dest, resources_dir / "icon.icns")
        
        # Create Info.plist for the app bundle
        info_plist = contents_dir / "Info.plist"
        plist_content = f'''<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>roe-launcher</string>
    <key>CFBundleIdentifier</key>
    <string>com.roelang.app</string>
    <key>CFBundleName</key>
    <string>Roe</string>
    <key>CFBundleVersion</key>
    <string>{ROE_VERSION}</string>
    <key>CFBundleShortVersionString</key>
    <string>{ROE_VERSION}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleIconFile</key>
    <string>icon.icns</string>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeExtensions</key>
            <array>
                <string>roe</string>
            </array>
            <key>CFBundleTypeName</key>
            <string>Roelang Source File</string>
            <key>CFBundleTypeRole</key>
            <string>Editor</string>
            <key>CFBundleTypeIconFile</key>
            <string>icon.icns</string>
            <key>LSHandlerRank</key>
            <string>Owner</string>
            <key>LSTypeIsPackage</key>
            <false/>
        </dict>
    </array>
    <key>UTExportedTypeDeclarations</key>
    <array>
        <dict>
            <key>UTTypeConformsTo</key>
            <array>
                <string>public.source-code</string>
                <string>public.plain-text</string>
            </array>
            <key>UTTypeDescription</key>
            <string>Roelang Source File</string>
            <key>UTTypeIdentifier</key>
            <string>com.roelang.source</string>
            <key>UTTypeTagSpecification</key>
            <dict>
                <key>public.filename-extension</key>
                <array>
                    <string>roe</string>
                </array>
            </dict>
            <key>UTTypeIconFile</key>
            <string>icon.icns</string>
        </dict>
    </array>
</dict>
</plist>'''
        
        info_plist.write_text(plist_content)
        
        # Create a launcher script with GUI popup
        handler_script = macos_dir / "roe-launcher"
        handler_content = f'''#!/bin/bash

# Roe Language Launcher
# Handles both .roe file opening and Spotlight launches

if [ "$1" != "" ]; then
    # A .roe file was passed - open it in the default text editor
    open -t "$1"
else
    # Launched from Spotlight or Finder - show info dialog (no System Events needed)
    result=$(osascript -e 'display dialog "Roe Language v{ROE_VERSION}\\nBuild Date: {BUILD_DATE}\\n\\nA unified programming system that compiles high-level app definitions to native, web, mobile, and server runtimes.\\n\\nTo use Roe:\\n• Open Terminal\\n• Type: roe --help\\n\\nTo create a new project:\\n• roe init my-project\\n\\nTo compile a .roe file:\\n• roe compile file.roe\\n\\nDocumentation:\\nhttps://github.com/roelang/docs" buttons {{"Open Terminal", "OK"}} default button "OK" with title "Roe Language" with icon note' 2>/dev/null)
    
    # Check if user clicked "Open Terminal"
    if [[ "$result" == *"Open Terminal"* ]]; then
        osascript -e 'tell application "Terminal" to activate' -e 'tell application "Terminal" to do script "echo \"Welcome to Roe v{ROE_VERSION}\"; echo \"Type: roe --help to get started\"; echo \"\""' 2>/dev/null
    fi
fi
'''
        handler_script.write_text(handler_content)
        os.chmod(handler_script, 0o755)
        
        # Register the app bundle with Launch Services
        subprocess.run(["/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister", "-f", str(app_bundle)], check=True)
        
        # Force rebuild Launch Services database (user domain only - no sudo needed)
        subprocess.run(["/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister", "-kill", "-r", "-domain", "user"], check=False)
        
        return True
        
    except Exception as e:
        print(f"Warning: Could not register file association: {e}")
        return False

def install():
    try:
        # Create necessary folders
        ROELANG_DIR.mkdir(parents=True, exist_ok=True)
        BIN_DIR.mkdir(parents=True, exist_ok=True)

        # Copy binaries
        shutil.copy(BASE_DIR / "roe", BIN_DIR / "roe")
        
        # Copy compiler module directory
        compiler_src = BASE_DIR / "compiler"
        compiler_dst = ROELANG_DIR / "compiler"
        if compiler_src.exists():
            if compiler_dst.exists():
                shutil.rmtree(compiler_dst)
            shutil.copytree(compiler_src, compiler_dst)
        
        shutil.copy(BASE_DIR / "run.js", ROELANG_DIR / "run.js")
        
        # Copy RoeVM if available (PyInstaller bundles it as roevm_binary to avoid conflict with directory)
        roevm_src = BASE_DIR / "roevm_binary"
        if roevm_src.exists() and roevm_src.is_file():
            shutil.copy(roevm_src, ROELANG_DIR / "roevm")
            os.chmod(ROELANG_DIR / "roevm", 0o755)

        # Make CLI executable (Unix only)
        if platform.system() != "Windows":
            os.chmod(BIN_DIR / "roe", 0o755)

        # Add to PATH
        path_line = 'export PATH="$HOME/.roelang/bin:$PATH"'
        if SHELL_RC:
            if not SHELL_RC.exists():
                SHELL_RC.write_text(path_line + "\n")
            else:
                with SHELL_RC.open("r+") as f:
                    contents = f.read()
                    if path_line not in contents:
                        f.write("\n" + path_line + "\n")

        # Register file association
        file_association_success = register_file_association()
        
        success_msg = f"✅ Roelang v{ROE_VERSION} installed successfully!\nRestart your terminal to use 'roe'."
        if file_association_success:
            success_msg += "\n\n.roe files are now associated with the Roe app.\nYou can find 'Roe' in Spotlight."
        
        messagebox.showinfo("Roelang Installer", success_msg)
    except Exception as e:
        messagebox.showerror("Installation Failed", str(e))

# GUI Setup
root = tk.Tk()
root.title(f"Roelang Installer v{INSTALLER_VERSION}")
root.geometry("350x250")
root.resizable(False, False)

label = tk.Label(root, text="Install Roelang DSL to your system", font=("Arial", 12))
label.pack(pady=20)

version_label = tk.Label(root, text=f"Roe Language v{ROE_VERSION}\nInstaller v{INSTALLER_VERSION}", font=("Arial", 10), fg="gray")
version_label.pack(pady=5)

install_btn = tk.Button(root, text="Install Roelang", font=("Arial", 11), width=20, command=install)
install_btn.pack(pady=10)

quit_btn = tk.Button(root, text="Exit", font=("Arial", 10), command=root.destroy)
quit_btn.pack(pady=5)

root.mainloop()
