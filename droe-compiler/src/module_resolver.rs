//! Module resolution and loading for Droe DSL includes
//! 
//! This module handles resolving include statements, loading modules from files,
//! detecting circular imports, and merging module statements into the main program.

use crate::ast::{Program, Node, IncludeStatement, ActionDefinitionWithParams};
use crate::parser::Parser;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::fs;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModuleResolutionError {
    #[error("Module file not found: {path}")]
    FileNotFound { path: String },
    
    #[error("Failed to read module {module}: {source}")]
    IoError { module: String, source: std::io::Error },
    
    #[error("Failed to parse module {module}: {error}")]
    ParseError { module: String, error: String },
    
    #[error("Circular import detected: {module}")]
    CircularImport { module: String },
    
    #[error("Module not loaded: {module}")]
    ModuleNotLoaded { module: String },
}

/// Handles module resolution and loading for Include statements
pub struct ModuleResolver {
    /// Base path for resolving relative module paths
    base_path: PathBuf,
    /// Cache of loaded modules: module_name -> parsed AST
    loaded_modules: HashMap<String, Program>,
    /// Module file paths: module_name -> file_path
    module_paths: HashMap<String, PathBuf>,
    /// Track circular imports
    loading_stack: HashSet<String>,
    /// Parser instance for parsing module files
    parser: Parser,
}

impl ModuleResolver {
    /// Create a new module resolver with the given base path
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
            loaded_modules: HashMap::new(),
            module_paths: HashMap::new(),
            loading_stack: HashSet::new(),
            parser: Parser::new(),
        }
    }

    /// Resolve all include statements in a program and load referenced modules
    ///
    /// # Arguments
    /// * `program` - The main program AST
    /// * `current_file_path` - Path to the current file being compiled
    /// * `preserve_base_path` - If true, don't change the base path (for recursive calls)
    ///
    /// # Returns
    /// Program with resolved includes and loaded modules
    ///
    /// # Errors
    /// Returns `ModuleResolutionError` if module resolution fails
    pub fn resolve_includes(
        &mut self,
        program: Program,
        current_file_path: &Path,
        preserve_base_path: bool,
    ) -> Result<Program, ModuleResolutionError> {
        // Set base path relative to current file (only for top-level calls)
        if !preserve_base_path {
            if let Some(current_dir) = current_file_path.parent() {
                self.base_path = current_dir.to_path_buf();
            }
        }

        // Find all include statements and collect them
        let includes: Vec<IncludeStatement> = program
            .statements
            .iter()
            .filter_map(|stmt| match stmt {
                Node::IncludeStatement(include) => Some(include.clone()),
                _ => None,
            })
            .collect();

        if includes.is_empty() {
            return Ok(program); // No includes to resolve
        }

        // Load all included modules
        let mut loaded_modules = Vec::new();
        for include_stmt in &includes {
            let module_program = self.load_module(&include_stmt.module_name, &include_stmt.file_path)?;
            loaded_modules.push(module_program);
        }

        // Create new program with includes resolved
        // Remove include statements from main program
        let main_statements: Vec<Node> = program
            .statements
            .into_iter()
            .filter(|stmt| !matches!(stmt, Node::IncludeStatement(_)))
            .collect();

        // Merge all module statements into the main program
        let mut all_statements = Vec::new();

        // First add all the loaded module statements
        for module_program in loaded_modules {
            all_statements.extend(module_program.statements);
        }

        // Then add the main program statements
        all_statements.extend(main_statements);

        let resolved_program = Program {
            statements: all_statements,
            included_modules: Some(includes),
            metadata: program.metadata,
            line_number: program.line_number,
        };

        Ok(resolved_program)
    }

    /// Load a module from file
    ///
    /// # Arguments
    /// * `module_name` - Name of the module
    /// * `file_path` - Relative path to the .droe file
    ///
    /// # Returns
    /// Parsed program AST for the module
    ///
    /// # Errors
    /// Returns `ModuleResolutionError` if loading fails
    fn load_module(&mut self, module_name: &str, file_path: &str) -> Result<Program, ModuleResolutionError> {
        // Check for circular imports
        if self.loading_stack.contains(module_name) {
            return Err(ModuleResolutionError::CircularImport {
                module: module_name.to_string(),
            });
        }

        // Return cached module if already loaded
        if let Some(cached_program) = self.loaded_modules.get(module_name) {
            return Ok(cached_program.clone());
        }

        // Construct full path
        let full_path = self.base_path.join(file_path);

        if !full_path.exists() {
            return Err(ModuleResolutionError::FileNotFound {
                path: full_path.to_string_lossy().to_string(),
            });
        }

        // Read and parse module
        self.loading_stack.insert(module_name.to_string());

        let result = (|| {
            // Read file
            let source_code = fs::read_to_string(&full_path).map_err(|e| {
                ModuleResolutionError::IoError {
                    module: module_name.to_string(),
                    source: e,
                }
            })?;

            // Parse the module
            let module_program = self.parser.parse(&source_code).map_err(|e| {
                ModuleResolutionError::ParseError {
                    module: module_name.to_string(),
                    error: format!("{:?}", e),
                }
            })?;

            // Recursively resolve includes in the module (preserve base path)
            let resolved_program = self.resolve_includes(module_program, &full_path, true)?;

            // Cache the loaded module
            self.loaded_modules
                .insert(module_name.to_string(), resolved_program.clone());
            self.module_paths.insert(module_name.to_string(), full_path);

            Ok(resolved_program)
        })();

        self.loading_stack.remove(module_name);
        result
    }

    /// Get all actions defined in a module
    ///
    /// # Arguments
    /// * `module_name` - Name of the module
    ///
    /// # Returns
    /// Vector of action definitions in the module
    ///
    /// # Errors
    /// Returns `ModuleResolutionError` if module not loaded
    pub fn get_module_actions(
        &self,
        module_name: &str,
    ) -> Result<Vec<ActionDefinitionWithParams>, ModuleResolutionError> {
        let module_program = self
            .loaded_modules
            .get(module_name)
            .ok_or_else(|| ModuleResolutionError::ModuleNotLoaded {
                module: module_name.to_string(),
            })?;

        let mut actions = Vec::new();

        // Look for actions in module definitions and top-level
        for stmt in &module_program.statements {
            match stmt {
                Node::ModuleDefinition(module_def) => {
                    // Actions inside modules
                    for module_stmt in &module_def.body {
                        if let Node::ActionDefinitionWithParams(action) = module_stmt {
                            actions.push(action.clone());
                        }
                    }
                }
                Node::ActionDefinitionWithParams(action) => {
                    // Top-level actions
                    actions.push(action.clone());
                }
                _ => {}
            }
        }

        Ok(actions)
    }

    /// Find a specific action in a module
    ///
    /// # Arguments
    /// * `module_name` - Name of the module
    /// * `action_name` - Name of the action
    ///
    /// # Returns
    /// Action definition if found, None otherwise
    pub fn find_action(
        &self,
        module_name: &str,
        action_name: &str,
    ) -> Option<ActionDefinitionWithParams> {
        match self.get_module_actions(module_name) {
            Ok(actions) => actions
                .into_iter()
                .find(|action| action.name == action_name),
            Err(_) => None,
        }
    }

    /// Get all loaded modules
    pub fn get_loaded_modules(&self) -> HashMap<String, Program> {
        self.loaded_modules.clone()
    }

    /// Clear the module cache
    pub fn clear_cache(&mut self) {
        self.loaded_modules.clear();
        self.module_paths.clear();
        self.loading_stack.clear();
    }

    /// Get the file path for a loaded module
    pub fn get_module_path(&self, module_name: &str) -> Option<&PathBuf> {
        self.module_paths.get(module_name)
    }

    /// Check if a module is loaded
    pub fn is_module_loaded(&self, module_name: &str) -> bool {
        self.loaded_modules.contains_key(module_name)
    }

    /// Get all loaded module names
    pub fn get_loaded_module_names(&self) -> Vec<String> {
        self.loaded_modules.keys().cloned().collect()
    }
}

impl Default for ModuleResolver {
    fn default() -> Self {
        Self::new(".")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{DataField, DataDefinition};
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_program() -> Program {
        Program {
            statements: vec![
                Node::DataDefinition(DataDefinition {
                    name: "User".to_string(),
                    fields: vec![DataField {
                        name: "name".to_string(),
                        field_type: "text".to_string(),
                        annotations: vec![],
                        line_number: Some(1),
                    }],
                    storage_type: Some("database".to_string()),
                    line_number: Some(1),
                }),
                Node::IncludeStatement(IncludeStatement {
                    module_name: "utils".to_string(),
                    file_path: "utils.droe".to_string(),
                    line_number: Some(2),
                }),
            ],
            metadata: vec![],
            included_modules: None,
            line_number: None,
        }
    }

    #[test]
    fn test_no_includes() {
        let mut resolver = ModuleResolver::new(".");
        let program = Program {
            statements: vec![Node::DataDefinition(DataDefinition {
                name: "Test".to_string(),
                fields: vec![],
                storage_type: None,
                line_number: Some(1),
            })],
            metadata: vec![],
            included_modules: None,
            line_number: None,
        };

        let result = resolver
            .resolve_includes(program.clone(), Path::new("test.droe"), false)
            .unwrap();
        assert_eq!(result.statements.len(), 1);
    }

    #[test]
    fn test_module_loading() {
        let temp_dir = TempDir::new().unwrap();
        let utils_path = temp_dir.path().join("utils.droe");

        // Create a simple module file
        let mut file = fs::File::create(&utils_path).unwrap();
        writeln!(file, "data Helper {{ name text }}").unwrap();

        let mut resolver = ModuleResolver::new(temp_dir.path());
        
        // This would need a proper parser implementation to work fully
        // For now we test the structure
        assert!(!resolver.is_module_loaded("utils"));
        assert_eq!(resolver.get_loaded_module_names().len(), 0);
    }

    #[test]
    fn test_circular_import_detection() {
        let mut resolver = ModuleResolver::new(".");
        resolver.loading_stack.insert("test".to_string());

        let result = resolver.load_module("test", "test.droe");
        assert!(matches!(
            result,
            Err(ModuleResolutionError::CircularImport { .. })
        ));
    }
}