//! Runtime updater module for WebAssembly JavaScript runtime generation
//! 
//! This module provides functionality to generate and update the run.js runtime file
//! with the latest core library functions from the Rust utilities.

use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::fs;
use droe_compiler::codegen::utils::CoreUtilities;

pub struct RuntimeUpdater {
    core_utils: CoreUtilities,
}

impl RuntimeUpdater {
    pub fn new() -> Self {
        Self {
            core_utils: CoreUtilities::new(),
        }
    }

    /// Update the runtime at the specified path
    pub fn update_runtime(&self, runtime_path: &Path, force: bool) -> Result<bool> {
        // Check if runtime exists and force flag
        if runtime_path.exists() && !force {
            println!("ℹ️  Runtime already exists at: {}", runtime_path.display());
            println!("   Use --force to overwrite existing runtime");
            return Ok(false);
        }

        // Ensure parent directory exists
        if let Some(parent) = runtime_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        // Generate new runtime
        let runtime_content = self.generate_runtime_js()?;

        // Write to file
        fs::write(runtime_path, runtime_content)
            .with_context(|| format!("Failed to write runtime to: {}", runtime_path.display()))?;

        Ok(true)
    }

    /// Generate the complete JavaScript runtime
    pub fn generate_runtime_js(&self) -> Result<String> {
        let mut runtime_lines = Vec::new();

        // Add header
        runtime_lines.extend([
            "const fs = require(\"fs\");".to_string(),
            "const path = process.argv[2];".to_string(),
            "".to_string(),
            "(async () => {".to_string(),
            "  const wasmBuffer = fs.readFileSync(path);".to_string(),
            "".to_string(),
            "  const importObject = {".to_string(),
            "    env: {".to_string(),
        ]);

        // Add basic runtime functions
        runtime_lines.extend(self.generate_basic_print_functions());

        // Add core library functions
        runtime_lines.extend(self.generate_core_library_functions());

        // Add footer
        runtime_lines.extend([
            "    },".to_string(),
            "  };".to_string(),
            "".to_string(),
            "  const { instance } = await WebAssembly.instantiate(wasmBuffer, importObject);".to_string(),
            "".to_string(),
            "  instance.exports.main();".to_string(),
            "})();".to_string(),
            "".to_string(),
        ]);

        Ok(runtime_lines.join("\n"))
    }

    fn generate_basic_print_functions(&self) -> Vec<String> {
        vec![
            "      // Basic print functions".to_string(),
            "      print: (offset, length) => {".to_string(),
            "        const memory = instance.exports.memory;".to_string(),
            "        const bytes = new Uint8Array(memory.buffer, offset, length);".to_string(),
            "        const text = new TextDecoder(\"utf-8\").decode(bytes);".to_string(),
            "        console.log(text);".to_string(),
            "      },".to_string(),
            "      print_i32: (value) => {".to_string(),
            "        console.log(value);".to_string(),
            "      },".to_string(),
            "      print_string_from_offset: (offset) => {".to_string(),
            "        const memory = instance.exports.memory;".to_string(),
            "        const bytes = new Uint8Array(memory.buffer);".to_string(),
            "        let length = 0;".to_string(),
            "        while (bytes[offset + length] !== 0 && offset + length < bytes.length) {".to_string(),
            "          length++;".to_string(),
            "        }".to_string(),
            "        const stringBytes = new Uint8Array(memory.buffer, offset, length);".to_string(),
            "        const text = new TextDecoder(\"utf-8\").decode(stringBytes);".to_string(),
            "        console.log(text);".to_string(),
            "      },".to_string(),
            "".to_string(),
            "      // Core library functions".to_string(),
        ]
    }

    fn generate_core_library_functions(&self) -> Vec<String> {
        let js_functions = self.core_utils.get_all_js_runtime_functions();
        let mut lines = Vec::new();
        
        for (func_name, func_code) in js_functions {
            lines.push(format!("      {}: {},", func_name, func_code));
        }
        
        lines
    }

    /// Get the default runtime path (~/.droelang/run.js)
    pub fn get_default_runtime_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        
        Ok(home_dir.join(".droelang").join("run.js"))
    }

    /// Get a summary of available core library functions
    pub fn get_function_summary(&self) -> Vec<String> {
        self.core_utils.get_function_summary()
    }
}

impl Default for RuntimeUpdater {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_generation() {
        let updater = RuntimeUpdater::new();
        let runtime_content = updater.generate_runtime_js().unwrap();
        
        // Check that runtime contains expected elements
        assert!(runtime_content.contains("WebAssembly.instantiate"));
        assert!(runtime_content.contains("instance.exports.main()"));
        assert!(runtime_content.contains("print:"));
        assert!(runtime_content.contains("math_abs_i32:"));
        assert!(runtime_content.contains("string_concat:"));
        assert!(runtime_content.contains("format_date:"));
    }

    #[test]
    fn test_runtime_update() {
        let updater = RuntimeUpdater::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().join("run.js");

        // Test successful update (file doesn't exist yet)
        let result = updater.update_runtime(&temp_path, false).unwrap();
        assert!(result);
        assert!(temp_path.exists());

        // Test without force flag (should not update since file exists)
        let result = updater.update_runtime(&temp_path, false).unwrap();
        assert!(!result);

        // Test with force flag (should update)
        let result = updater.update_runtime(&temp_path, true).unwrap();
        assert!(result);
    }

    #[test]
    fn test_function_summary() {
        let updater = RuntimeUpdater::new();
        let summary = updater.get_function_summary();
        
        assert!(!summary.is_empty());
        assert!(summary.iter().any(|line| line.contains("Math Functions")));
        assert!(summary.iter().any(|line| line.contains("String Functions")));
        assert!(summary.iter().any(|line| line.contains("Formatting Functions")));
    }
}