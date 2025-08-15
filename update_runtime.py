#!/usr/bin/env python3
"""Script to update the run.js runtime with new core library functions."""

import os
import sys
from pathlib import Path
from compiler.targets.wasm.runtime_builder import WASMRuntimeBuilder


def update_runtime():
    """Update the run.js file with new runtime functions."""
    # Path to run.js
    roelang_dir = Path.home() / ".droelang"
    runtime_path = roelang_dir / "run.js"
    
    # Create runtime builder
    builder = WASMRuntimeBuilder()
    
    # Enable all core libraries
    builder.enable_library('string_utils')
    builder.enable_library('math_utils')
    builder.enable_library('formatting')
    
    # Generate and write new runtime
    print(f"Updating runtime at: {runtime_path}")
    
    if builder.update_existing_runtime(str(runtime_path)):
        print("✅ Runtime updated successfully!")
        print("\n📚 Core libraries enabled:")
        print("  - String utilities (concatenation, substring, length)")
        print("  - Math utilities (abs, min, max, power, sqrt)")
        print("  - Formatting utilities (date, decimal, number formatting)")
    else:
        print("❌ Failed to update runtime")
        return False
    
    return True


if __name__ == "__main__":
    success = update_runtime()
    sys.exit(0 if success else 1)