//! Compiler factory for target and framework routing
//! 
//! This module provides a factory system for routing compilation to the correct
//! target language generators and framework adapters, similar to the Python implementation.

use crate::ast::Program;
use crate::codegen::{CodeGenerator, JavaScriptGenerator, WebAssemblyGenerator, BytecodeGenerator, GoGenerator, PythonGenerator, RustGenerator, JavaCodeGenerator, HTMLGenerator, MobileGenerator, PuckCodeGenerator};
use crate::adapters::{FrameworkAdapter, AdapterOptions, AdapterOutput, SpringAdapter, FastAPIAdapter, FiberAdapter, FastifyAdapter, IOSAdapter, AndroidAdapter};
use std::collections::HashMap;

/// Supported compilation targets
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CompilerTarget {
    JavaScript,
    WebAssembly, 
    Bytecode,
    Go,
    Python,
    Rust,
    Java,
    HTML,
    Mobile,
    Puck,
}

impl CompilerTarget {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "javascript" | "js" => Ok(Self::JavaScript),
            "webassembly" | "wasm" => Ok(Self::WebAssembly),
            "bytecode" | "bc" => Ok(Self::Bytecode),
            "go" => Ok(Self::Go),
            "python" | "py" => Ok(Self::Python),
            "rust" | "rs" => Ok(Self::Rust),
            "java" => Ok(Self::Java),
            "html" | "web" => Ok(Self::HTML),
            "mobile" | "android" | "ios" => Ok(Self::Mobile),
            "puck" | "puck-json" => Ok(Self::Puck),
            _ => Err(format!("Unsupported target: {}", s)),
        }
    }

    pub fn file_extension(&self) -> &str {
        match self {
            Self::JavaScript => ".js",
            Self::WebAssembly => ".wasm",
            Self::Bytecode => ".bc",
            Self::Go => ".go", 
            Self::Python => ".py",
            Self::Rust => ".rs",
            Self::Java => ".java",
            Self::HTML => ".html",
            Self::Mobile => "", // Mobile generates project structures, not single files
            Self::Puck => ".json",
        }
    }
}

/// Supported frameworks per target
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Framework {
    // Language-agnostic
    Plain,
    
    // Python frameworks
    FastAPI,
    Django,
    
    // Java frameworks  
    Spring,
    
    // Go frameworks
    Fiber,
    Gin,
    
    // Rust frameworks
    Axum,
    
    // Node.js frameworks
    Express,
    Fastify,
    
    // Mobile frameworks
    SwiftUI,
    Kotlin,
}

impl Framework {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "plain" | "none" => Ok(Self::Plain),
            "fastapi" => Ok(Self::FastAPI),
            "django" => Ok(Self::Django),
            "spring" | "springboot" => Ok(Self::Spring),
            "fiber" => Ok(Self::Fiber),
            "gin" => Ok(Self::Gin),
            "axum" => Ok(Self::Axum),
            "express" => Ok(Self::Express),
            "fastify" => Ok(Self::Fastify),
            "swiftui" | "ios" => Ok(Self::SwiftUI),
            "kotlin" | "android" => Ok(Self::Kotlin),
            _ => Err(format!("Unsupported framework: {}", s)),
        }
    }

    pub fn is_valid_for_target(&self, target: &CompilerTarget) -> bool {
        match (self, target) {
            (Framework::Plain, _) => true,
            (Framework::FastAPI | Framework::Django, CompilerTarget::Python) => true,
            (Framework::Spring, CompilerTarget::Java) => true,
            (Framework::Fiber | Framework::Gin, CompilerTarget::Go) => true,
            (Framework::Axum, CompilerTarget::Rust) => true,
            (Framework::Express | Framework::Fastify, CompilerTarget::JavaScript) => true,
            (Framework::Express | Framework::Fastify, CompilerTarget::HTML) => true,
            (Framework::SwiftUI | Framework::Kotlin, CompilerTarget::Mobile) => true,
            _ => false,
        }
    }
}

/// Compilation configuration
#[derive(Debug, Clone)]
pub struct CompilerOptions {
    pub target: CompilerTarget,
    pub framework: Framework,
    pub package_name: Option<String>,
    pub database_type: Option<String>,
    pub source_file_path: Option<String>,
    pub is_main_file: bool,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            target: CompilerTarget::JavaScript,
            framework: Framework::Plain,
            package_name: None,
            database_type: None,
            source_file_path: None,
            is_main_file: false,
        }
    }
}

/// Result of compilation - either simple code or complex project
#[derive(Debug)]
pub enum CompilationResult {
    /// Simple single-file output (from core generators)
    Code(String),
    /// Complex multi-file project (from framework adapters)
    Project(AdapterOutput),
}

/// Main compiler factory
pub struct CompilerFactory {
    /// Available targets and their core generators
    core_generators: HashMap<CompilerTarget, fn() -> Box<dyn CodeGenerator>>,
    /// Available framework adapters
    framework_adapters: HashMap<(CompilerTarget, Framework), fn() -> Box<dyn FrameworkAdapter>>,
}

impl CompilerFactory {
    pub fn new() -> Self {
        let mut factory = Self {
            core_generators: HashMap::new(),
            framework_adapters: HashMap::new(),
        };
        
        factory.register_core_generators();
        factory.register_framework_adapters();
        factory
    }

    fn register_core_generators(&mut self) {
        self.core_generators.insert(CompilerTarget::JavaScript, || Box::new(JavaScriptGenerator::new()));
        self.core_generators.insert(CompilerTarget::WebAssembly, || Box::new(WebAssemblyGenerator::new()));
        self.core_generators.insert(CompilerTarget::Bytecode, || Box::new(BytecodeGenerator::new()));
        self.core_generators.insert(CompilerTarget::Go, || Box::new(GoGenerator::new()));
        self.core_generators.insert(CompilerTarget::Python, || Box::new(PythonGenerator::new()));
        self.core_generators.insert(CompilerTarget::Rust, || {
            Box::new(RustGenerator::new(None, false, None, None, None).expect("Failed to create RustGenerator"))
        });
        self.core_generators.insert(CompilerTarget::Java, || Box::new(JavaCodeGenerator::new()));
        self.core_generators.insert(CompilerTarget::HTML, || Box::new(HTMLGenerator::new()));
        self.core_generators.insert(CompilerTarget::Mobile, || {
            Box::new(MobileGenerator::new().expect("Failed to create MobileGenerator"))
        });
        self.core_generators.insert(CompilerTarget::Puck, || Box::new(PuckCodeGenerator::new()));
    }

    fn register_framework_adapters(&mut self) {
        // Python frameworks
        self.framework_adapters.insert((CompilerTarget::Python, Framework::FastAPI), || Box::new(FastAPIAdapter::new()));
        
        // Java frameworks
        self.framework_adapters.insert((CompilerTarget::Java, Framework::Spring), || Box::new(SpringAdapter::new()));
        
        // Go frameworks
        self.framework_adapters.insert((CompilerTarget::Go, Framework::Fiber), || {
            Box::new(FiberAdapter::new().expect("Failed to create FiberAdapter"))
        });
        
        // Node.js frameworks
        self.framework_adapters.insert((CompilerTarget::JavaScript, Framework::Fastify), || {
            Box::new(FastifyAdapter::new().expect("Failed to create FastifyAdapter"))
        });
        
        // Mobile frameworks
        self.framework_adapters.insert((CompilerTarget::Mobile, Framework::SwiftUI), || {
            Box::new(IOSAdapter::new().expect("Failed to create IOSAdapter"))
        });
        self.framework_adapters.insert((CompilerTarget::Mobile, Framework::Kotlin), || {
            Box::new(AndroidAdapter::new().expect("Failed to create AndroidAdapter"))
        });
    }

    /// Get available targets
    pub fn get_available_targets(&self) -> Vec<CompilerTarget> {
        self.core_generators.keys().cloned().collect()
    }

    /// Get available frameworks for a target
    pub fn get_available_frameworks(&self, target: &CompilerTarget) -> Vec<Framework> {
        let mut frameworks = vec![Framework::Plain]; // Plain always available
        
        frameworks.extend(
            self.framework_adapters.keys()
                .filter(|(t, _)| t == target)
                .map(|(_, f)| f.clone())
        );
        
        frameworks
    }

    /// Compile program with proper routing
    pub fn compile(&self, program: &Program, options: CompilerOptions) -> Result<CompilationResult, String> {
        // Validate framework for target
        if !options.framework.is_valid_for_target(&options.target) {
            return Err(format!(
                "Framework {:?} is not compatible with target {:?}",
                options.framework, options.target
            ));
        }

        // Route to framework adapter or core generator
        match options.framework {
            Framework::Plain => {
                // Use core language generator
                self.compile_with_core_generator(program, &options)
            }
            _ => {
                // Use framework adapter
                self.compile_with_framework_adapter(program, &options)
            }
        }
    }

    fn compile_with_core_generator(&self, program: &Program, options: &CompilerOptions) -> Result<CompilationResult, String> {
        let generator_factory = self.core_generators.get(&options.target)
            .ok_or_else(|| format!("No core generator for target {:?}", options.target))?;
        
        let generator = generator_factory();
        let code = generator.generate(program)?;
        
        Ok(CompilationResult::Code(code))
    }

    fn compile_with_framework_adapter(&self, program: &Program, options: &CompilerOptions) -> Result<CompilationResult, String> {
        let adapter_factory = self.framework_adapters.get(&(options.target.clone(), options.framework.clone()))
            .ok_or_else(|| format!("No adapter for {:?} + {:?}", options.target, options.framework))?;
        
        let adapter = adapter_factory();
        
        // Convert compiler options to adapter options
        let adapter_options = AdapterOptions {
            package_name: options.package_name.clone(),
            database_type: options.database_type.clone(),
            ..Default::default()
        };
        
        let output = adapter.generate(program, adapter_options)?;
        
        Ok(CompilationResult::Project(output))
    }

    /// Convenience method for simple compilation
    pub fn compile_to_string(&self, program: &Program, target: &str, framework: Option<&str>) -> Result<String, String> {
        let target = CompilerTarget::from_str(target)?;
        let framework = framework.map(Framework::from_str).unwrap_or(Ok(Framework::Plain))?;
        
        let options = CompilerOptions {
            target,
            framework,
            ..Default::default()
        };

        match self.compile(program, options)? {
            CompilationResult::Code(code) => Ok(code),
            CompilationResult::Project(project) => {
                // For project results, return summary or main file
                if let Some((_, main_content)) = project.files.iter().find(|(path, _)| {
                    path.contains("main.") || path.contains("Application.") || path.ends_with("__init__.py")
                }) {
                    Ok(main_content.clone())
                } else {
                    Ok(format!("Generated project with {} files", project.files.len()))
                }
            }
        }
    }
}

impl Default for CompilerFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Global factory instance
lazy_static::lazy_static! {
    pub static ref COMPILER_FACTORY: CompilerFactory = CompilerFactory::new();
}

/// Convenience function for compilation
pub fn compile_to_target(program: &Program, target: &str, framework: Option<&str>) -> Result<CompilationResult, String> {
    let target = CompilerTarget::from_str(target)?;
    let framework = framework.map(Framework::from_str).unwrap_or(Ok(Framework::Plain))?;
    
    let options = CompilerOptions {
        target,
        framework,
        ..Default::default()
    };

    COMPILER_FACTORY.compile(program, options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Node, DataDefinition, DataField};

    fn create_test_program() -> Program {
        Program {
            statements: vec![
                Node::DataDefinition(DataDefinition {
                    name: "User".to_string(),
                    fields: vec![
                        DataField {
                            name: "name".to_string(),
                            field_type: "text".to_string(),
                            annotations: vec![],
                            line_number: Some(1),
                        },
                    ],
                    storage_type: Some("database".to_string()),
                    line_number: Some(1),
                }),
            ],
            metadata: vec![],
            included_modules: None,
            line_number: None,
        }
    }

    #[test]
    fn test_target_routing() {
        let factory = CompilerFactory::new();
        let program = create_test_program();

        // Test core generator routing
        let options = CompilerOptions {
            target: CompilerTarget::Python,
            framework: Framework::Plain,
            ..Default::default()
        };

        let result = factory.compile(&program, options);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), CompilationResult::Code(_)));
    }

    #[test]
    fn test_framework_routing() {
        let factory = CompilerFactory::new();
        let program = create_test_program();

        // Test framework adapter routing
        let options = CompilerOptions {
            target: CompilerTarget::Python,
            framework: Framework::FastAPI,
            ..Default::default()
        };

        let result = factory.compile(&program, options);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), CompilationResult::Project(_)));
    }

    #[test]
    fn test_invalid_framework_for_target() {
        let factory = CompilerFactory::new();
        let program = create_test_program();

        // Test invalid combination
        let options = CompilerOptions {
            target: CompilerTarget::Python,
            framework: Framework::Spring, // Spring is for Java, not Python
            ..Default::default()
        };

        let result = factory.compile(&program, options);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not compatible"));
    }
}