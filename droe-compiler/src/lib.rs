//! Droe DSL Compiler Library
//! 
//! A fast, efficient compiler for the Droe domain-specific language.

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod expressions;
pub mod type_checker;
pub mod codegen;
pub mod diagnostics;
pub mod adapters;
pub mod compiler_factory;
pub mod module_resolver;
pub mod symbols;
pub mod codegen_base;


pub use ast::{Node, Program, ParseError, ParseResult};
pub use lexer::Lexer;
pub use parser::Parser;
pub use type_checker::TypeChecker;
pub use diagnostics::Diagnostic;
pub use codegen::{CodeGenerator, JavaScriptGenerator, WebAssemblyGenerator, BytecodeGenerator, GoGenerator, PythonGenerator, RustGenerator, JavaCodeGenerator, HTMLGenerator, PuckCodeGenerator};
pub use adapters::{FrameworkAdapter, AdapterOptions, AdapterOutput, FiberAdapter, SpringAdapter, FastAPIAdapter, FastifyAdapter};
pub use compiler_factory::{CompilerFactory, CompilerTarget, Framework, CompilerOptions, CompilationResult, compile_to_target, COMPILER_FACTORY};
pub use module_resolver::{ModuleResolver, ModuleResolutionError};
pub use symbols::{SymbolTable, VariableType, VariableValue, Variable};
pub use codegen_base::{BaseCodeGenerator, CodeGenContext, CodeGenError, CoreLibraries, TypeSystemHelpers};

/// Main compiler interface
pub struct Compiler {
    parser: Parser,
    type_checker: TypeChecker,
    module_resolver: ModuleResolver,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
            type_checker: TypeChecker::new(),
            module_resolver: ModuleResolver::new("."),
        }
    }
    
    /// Parse source code into an AST
    pub fn parse(&self, source: &str) -> ParseResult<Program> {
        self.parser.parse(source)
    }
    
    /// Lint source code and return diagnostics
    pub fn lint(&self, source: &str) -> Vec<Diagnostic> {
        match self.parse(source) {
            Ok(ast) => self.type_checker.check(&ast),
            Err(parse_error) => vec![Diagnostic::from_parse_error(parse_error)],
        }
    }
    
    /// Compile source with module resolution
    pub fn compile_with_modules(
        &mut self,
        source: &str,
        source_file_path: &str,
        target: &str,
        framework: Option<&str>,
    ) -> Result<CompilationResult, String> {
        // Parse the source
        let program = self.parse(source).map_err(|e| format!("Parse error: {:?}", e))?;
        
        // Resolve includes
        let resolved_program = self
            .module_resolver
            .resolve_includes(program, std::path::Path::new(source_file_path), false)
            .map_err(|e| format!("Module resolution error: {}", e))?;
        
        // Use compiler factory for target generation
        compile_to_target(&resolved_program, target, framework)
    }

    /// Format source code
    pub fn format(&self, source: &str) -> Result<String, String> {
        match self.parse(source) {
            Ok(ast) => Ok(self.format_ast(&ast)),
            Err(e) => Err(format!("Parse error: {}", e.message)),
        }
    }
    
    fn format_ast(&self, _ast: &Program) -> String {
        // TODO: Implement formatter
        String::new()
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

// Public API functions for the unified binary

/// Lint a file and optionally fix issues
pub fn lint_file(path: &std::path::Path, fix: bool) -> Result<(), String> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let compiler = Compiler::new();
    let diagnostics = compiler.lint(&source);
    
    if !diagnostics.is_empty() {
        for diagnostic in &diagnostics {
            eprintln!("{}:{}:{}: {}", 
                path.display(),
                diagnostic.line,
                diagnostic.character,
                diagnostic.message
            );
        }
        
        if fix {
            // TODO: Implement auto-fix functionality
            println!("Auto-fix not yet implemented");
        }
        
        return Err(format!("Found {} issues", diagnostics.len()));
    }
    
    Ok(())
}

/// Format a file and optionally just check formatting
pub fn format_file(path: &std::path::Path, check: bool) -> Result<(), String> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let compiler = Compiler::new();
    let formatted = compiler.format(&source)?;
    
    if check {
        if source != formatted {
            return Err("File is not properly formatted".to_string());
        }
    } else {
        std::fs::write(path, formatted)
            .map_err(|e| format!("Failed to write file: {}", e))?;
    }
    
    Ok(())
}

/// Reverse compile Puck JSON to DSL
pub fn reverse_compile_puck(input: &std::path::Path, output: &std::path::Path) -> Result<(), String> {
    let _json_content = std::fs::read_to_string(input)
        .map_err(|e| format!("Failed to read JSON file: {}", e))?;
    
    // TODO: Implement puck reverse codegen
    let dsl_code = format!("// Reverse compiled from {}\nDisplay \"Hello from reverse compilation!\"", input.display());
    
    std::fs::write(output, dsl_code)
        .map_err(|e| format!("Failed to write DSL file: {}", e))?;
    
    Ok(())
}

/// Generate framework-specific projects
pub fn generate_spring_project(output_dir: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(output_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;
    
    // TODO: Implement Spring project generation
    let project_files = vec![
        ("src/main/java/Application.java".to_string(), "// Spring Boot application\npublic class Application {}".to_string()),
        ("pom.xml".to_string(), "<!-- Maven POM -->\n<project></project>".to_string()),
    ];
    
    for (file_path, content) in project_files {
        let full_path = output_dir.join(file_path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }
        std::fs::write(full_path, content)
            .map_err(|e| format!("Failed to write file: {}", e))?;
    }
    
    Ok(())
}

pub fn generate_fastapi_project(output_dir: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(output_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;
    
    // TODO: Implement FastAPI project generation
    let project_files = vec![
        ("main.py".to_string(), "# FastAPI application\nfrom fastapi import FastAPI\napp = FastAPI()".to_string()),
        ("requirements.txt".to_string(), "fastapi\nuvicorn".to_string()),
    ];
    
    for (file_path, content) in project_files {
        let full_path = output_dir.join(file_path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }
        std::fs::write(full_path, content)
            .map_err(|e| format!("Failed to write file: {}", e))?;
    }
    
    Ok(())
}

pub fn generate_fiber_project(output_dir: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(output_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;
    
    // TODO: Implement Fiber project generation
    let project_files = vec![
        ("main.go".to_string(), "// Go Fiber application\npackage main\n\nfunc main() {}".to_string()),
        ("go.mod".to_string(), "module fiber-app\n\ngo 1.21".to_string()),
    ];
    
    for (file_path, content) in project_files {
        let full_path = output_dir.join(file_path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }
        std::fs::write(full_path, content)
            .map_err(|e| format!("Failed to write file: {}", e))?;
    }
    
    Ok(())
}

pub fn generate_android_project(output_dir: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(output_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;
    
    // TODO: Implement Android project generation
    let main_activity = r#"package com.example.droe

import android.app.Activity
import android.os.Bundle

class MainActivity : Activity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // Generated by Droe compiler
    }
}
"#;
    
    std::fs::write(output_dir.join("MainActivity.kt"), main_activity)
        .map_err(|e| format!("Failed to write file: {}", e))?;
    
    Ok(())
}

pub fn generate_ios_project(output_dir: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(output_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;
    
    // TODO: Implement iOS project generation
    let content_view = r#"import SwiftUI

struct ContentView: View {
    var body: some View {
        Text("Generated by Droe compiler")
            .padding()
    }
}

#Preview {
    ContentView()
}
"#;
    
    std::fs::write(output_dir.join("ContentView.swift"), content_view)
        .map_err(|e| format!("Failed to write file: {}", e))?;
    
    Ok(())
}

/// Error type for validation
#[derive(Debug)]
pub struct ValidationError {
    pub line: usize,
    pub column: usize,
    pub length: usize,
    pub message: String,
}

/// Parse and validate source code
pub fn parse_and_validate(source: &str) -> Result<Program, Vec<ValidationError>> {
    let compiler = Compiler::new();
    
    match compiler.parse(source) {
        Ok(program) => {
            let diagnostics = compiler.lint(source);
            if diagnostics.is_empty() {
                Ok(program)
            } else {
                let errors = diagnostics.into_iter().map(|d| ValidationError {
                    line: d.line,
                    column: d.character,
                    length: 1, // TODO: Calculate proper length
                    message: d.message,
                }).collect();
                Err(errors)
            }
        }
        Err(parse_error) => {
            Err(vec![ValidationError {
                line: parse_error.line,
                column: parse_error.column,
                length: 1,
                message: parse_error.message,
            }])
        }
    }
}

/// Format source code
pub fn format_code(source: &str) -> Result<String, anyhow::Error> {
    let compiler = Compiler::new();
    compiler.format(source)
        .map_err(|e| anyhow::anyhow!("Format error: {}", e))
}