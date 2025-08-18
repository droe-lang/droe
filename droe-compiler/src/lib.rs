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