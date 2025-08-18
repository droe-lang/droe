//! Code generation module for Droe DSL
//! 
//! This module provides various code generation targets.
//! Each target is implemented in a separate submodule.

use crate::ast::Program;

/// Base trait for all code generators
pub trait CodeGenerator {
    fn generate(&self, program: &Program) -> Result<String, String>;
}

pub mod webassembly;
pub mod wasm_runtime;
pub mod bytecode;
pub mod go;
pub mod java;
pub mod python;
pub mod rust;
pub mod html;
pub mod mobile;
pub mod puck;
pub mod utils;

// #[cfg(test)]
// mod python_test;
// #[cfg(test)]
// mod rust_test;

// Re-export generators for easy access
pub use webassembly::WebAssemblyGenerator;
pub use wasm_runtime::WasmRuntimeBuilder;
pub use bytecode::BytecodeGenerator;
pub use go::GoGenerator;
pub use java::JavaCodeGenerator;
pub use python::PythonGenerator;
pub use rust::RustGenerator;
pub use html::{HTMLGenerator, JavaScriptGenerator};
pub use mobile::MobileGenerator;
pub use puck::PuckCodeGenerator;
pub use utils::{CoreUtilities, MathUtils, StringUtils, FormattingUtils};