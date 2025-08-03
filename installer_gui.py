#!/usr/bin/env python3

import tkinter as tk
from tkinter import messagebox
import os
import sys
import shutil
import platform
from pathlib import Path

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

def install():
    try:
        # Create necessary folders
        ROELANG_DIR.mkdir(parents=True, exist_ok=True)
        BIN_DIR.mkdir(parents=True, exist_ok=True)

        # Copy binaries
        shutil.copy(BASE_DIR / "roe", BIN_DIR / "roe")
        shutil.copy(BASE_DIR / "compiler_standalone.py", ROELANG_DIR / "compiler.py")
        shutil.copy(BASE_DIR / "run.js", ROELANG_DIR / "run.js")

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

        messagebox.showinfo("Roelang Installer", "✅ Roelang installed successfully!\nRestart your terminal to use 'roe'.")
    except Exception as e:
        messagebox.showerror("Installation Failed", str(e))

# GUI Setup
root = tk.Tk()
root.title("Roelang Installer")
root.geometry("350x200")
root.resizable(False, False)

label = tk.Label(root, text="Install Roelang DSL to your system", font=("Arial", 12))
label.pack(pady=20)

install_btn = tk.Button(root, text="Install Roelang", font=("Arial", 11), width=20, command=install)
install_btn.pack(pady=10)

quit_btn = tk.Button(root, text="Exit", font=("Arial", 10), command=root.destroy)
quit_btn.pack(pady=5)

root.mainloop()
