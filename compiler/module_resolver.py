"""Module resolution and loading for Droe DSL includes."""

import os
from typing import Dict, List, Optional, Set
from .parser import parse, ParseError
from .ast import Program, IncludeStatement, ModuleDefinition, ActionDefinitionWithParams


class ModuleResolutionError(Exception):
    """Raised when module resolution fails."""
    pass


class ModuleResolver:
    """Handles module resolution and loading for Include statements."""
    
    def __init__(self, base_path: str = "."):
        self.base_path = os.path.abspath(base_path)
        self.loaded_modules: Dict[str, Program] = {}  # module_name -> parsed AST
        self.module_paths: Dict[str, str] = {}        # module_name -> file_path
        self.loading_stack: Set[str] = set()          # Track circular imports
    
    def resolve_includes(self, program: Program, current_file_path: str, preserve_base_path: bool = False) -> Program:
        """
        Resolve all include statements in a program and load referenced modules.
        
        Args:
            program: The main program AST
            current_file_path: Path to the current file being compiled
            preserve_base_path: If True, don't change the base path (for recursive calls)
            
        Returns:
            Program with resolved includes and loaded modules
            
        Raises:
            ModuleResolutionError: If module resolution fails
        """
        # Set base path relative to current file (only for top-level calls)
        if not preserve_base_path:
            current_dir = os.path.dirname(os.path.abspath(current_file_path))
            self.base_path = current_dir
        
        # Find all include statements
        includes = [stmt for stmt in program.statements if isinstance(stmt, IncludeStatement)]
        
        if not includes:
            return program  # No includes to resolve
        
        # Load all included modules
        loaded_modules = []
        for include_stmt in includes:
            module_program = self._load_module(include_stmt.module_name, include_stmt.file_path)
            loaded_modules.append(module_program)
        
        # Create new program with includes resolved
        # Remove include statements from main program
        main_statements = [stmt for stmt in program.statements if not isinstance(stmt, IncludeStatement)]
        
        # Merge all module statements into the main program
        all_statements = []
        
        # First add all the loaded module statements
        for module_program in loaded_modules:
            all_statements.extend(module_program.statements)
        
        # Then add the main program statements
        all_statements.extend(main_statements)
        
        resolved_program = Program(
            statements=all_statements,
            included_modules=includes
        )
        
        return resolved_program
    
    def _load_module(self, module_name: str, file_path: str) -> Program:
        """
        Load a module from file.
        
        Args:
            module_name: Name of the module
            file_path: Relative path to the .droe file
            
        Returns:
            Parsed program AST for the module
            
        Raises:
            ModuleResolutionError: If loading fails
        """
        # Check for circular imports
        if module_name in self.loading_stack:
            raise ModuleResolutionError(f"Circular import detected: {module_name}")
        
        # Return cached module if already loaded
        if module_name in self.loaded_modules:
            return self.loaded_modules[module_name]
        
        # Construct full path
        full_path = os.path.join(self.base_path, file_path)
        
        if not os.path.exists(full_path):
            raise ModuleResolutionError(f"Module file not found: {full_path}")
        
        # Read and parse module
        try:
            self.loading_stack.add(module_name)
            
            with open(full_path, 'r') as f:
                source_code = f.read()
            
            # Parse the module
            module_program = parse(source_code)
            
            # Recursively resolve includes in the module (preserve base path)
            module_program = self.resolve_includes(module_program, full_path, preserve_base_path=True)
            
            # Cache the loaded module
            self.loaded_modules[module_name] = module_program
            self.module_paths[module_name] = full_path
            
            return module_program
            
        except (IOError, OSError) as e:
            raise ModuleResolutionError(f"Failed to read module {module_name}: {str(e)}")
        except ParseError as e:
            raise ModuleResolutionError(f"Failed to parse module {module_name}: {str(e)}")
        finally:
            self.loading_stack.discard(module_name)
    
    def get_module_actions(self, module_name: str) -> List[ActionDefinitionWithParams]:
        """
        Get all actions defined in a module.
        
        Args:
            module_name: Name of the module
            
        Returns:
            List of action definitions in the module
            
        Raises:
            ModuleResolutionError: If module not loaded
        """
        if module_name not in self.loaded_modules:
            raise ModuleResolutionError(f"Module not loaded: {module_name}")
        
        module_program = self.loaded_modules[module_name]
        actions = []
        
        # Look for actions in module definitions and top-level
        for stmt in module_program.statements:
            if isinstance(stmt, ModuleDefinition):
                # Actions inside modules
                for module_stmt in stmt.body:
                    if isinstance(module_stmt, ActionDefinitionWithParams):
                        actions.append(module_stmt)
            elif isinstance(stmt, ActionDefinitionWithParams):
                # Top-level actions
                actions.append(stmt)
        
        return actions
    
    def find_action(self, module_name: str, action_name: str) -> Optional[ActionDefinitionWithParams]:
        """
        Find a specific action in a module.
        
        Args:
            module_name: Name of the module
            action_name: Name of the action
            
        Returns:
            Action definition if found, None otherwise
        """
        try:
            actions = self.get_module_actions(module_name)
            for action in actions:
                if action.name == action_name:
                    return action
            return None
        except ModuleResolutionError:
            return None
    
    def get_loaded_modules(self) -> Dict[str, Program]:
        """Get all loaded modules."""
        return self.loaded_modules.copy()
    
    def clear_cache(self):
        """Clear the module cache."""
        self.loaded_modules.clear()
        self.module_paths.clear()
        self.loading_stack.clear()