"""Base code generator for Roelang compiler.

This module provides the abstract base class and common functionality
for all target-specific code generators.
"""

from abc import ABC, abstractmethod
from typing import List, Dict, Any, Optional
from .symbols import SymbolTable, VariableType
from .ast import ASTNode, Program


class CodeGenError(Exception):
    """Exception raised during code generation."""
    pass


class BaseCodeGenerator(ABC):
    """Abstract base class for all code generators."""
    
    def __init__(self):
        self.symbol_table = SymbolTable()
        self.output: List[str] = []
        self.indent_level = 0
        self.string_constants: Dict[str, int] = {}
        self.next_string_index = 0
        
        # Core libraries
        self.core_libs_enabled = True
        self.available_libs = set()
    
    @abstractmethod
    def generate(self, program: Program) -> str:
        """Generate code for the given AST program."""
        pass
    
    @abstractmethod
    def emit_expression(self, expr: ASTNode):
        """Emit code for an expression."""
        pass
    
    @abstractmethod
    def emit_statement(self, stmt: ASTNode):
        """Emit code for a statement."""
        pass
    
    def emit(self, code: str):
        """Emit a line of code with proper indentation."""
        indent = '  ' * self.indent_level
        self.output.append(f"{indent}{code}")
    
    def get_output(self) -> str:
        """Get the generated code as a string."""
        return '\\n'.join(self.output)
    
    def clear_output(self):
        """Clear the output buffer."""
        self.output.clear()
    
    def add_string_constant(self, value: str) -> int:
        """Add a string constant and return its index."""
        if value not in self.string_constants:
            self.string_constants[value] = self.next_string_index
            self.next_string_index += 1
        return self.string_constants[value]
    
    def enable_core_lib(self, lib_name: str):
        """Enable a core library."""
        self.available_libs.add(lib_name)
    
    def disable_core_lib(self, lib_name: str):
        """Disable a core library."""
        self.available_libs.discard(lib_name)
    
    def is_core_lib_enabled(self, lib_name: str) -> bool:
        """Check if a core library is enabled."""
        return lib_name in self.available_libs
    
    # Type system helpers
    def _is_numeric_type(self, var_type: VariableType) -> bool:
        """Check if a type is numeric."""
        return var_type in [VariableType.INT, VariableType.DECIMAL, VariableType.NUMBER]
    
    def _is_text_type(self, var_type: VariableType) -> bool:
        """Check if a type is text-based."""
        return var_type in [VariableType.STRING, VariableType.TEXT]
    
    def _is_boolean_type(self, var_type: VariableType) -> bool:
        """Check if a type is boolean."""
        return var_type in [VariableType.BOOLEAN, VariableType.FLAG, VariableType.YESNO]
    
    def _is_collection_type(self, var_type: VariableType) -> bool:
        """Check if a type is a collection."""
        return var_type in [VariableType.ARRAY, VariableType.LIST_OF, VariableType.GROUP_OF]
    
    def map_user_type_to_internal(self, user_type: str) -> VariableType:
        """Map user-facing type names to internal VariableType enum."""
        type_mapping = {
            # Modern types
            'int': VariableType.INT,
            'decimal': VariableType.DECIMAL,
            'text': VariableType.TEXT,
            'flag': VariableType.FLAG,
            'yesno': VariableType.YESNO,
            'date': VariableType.DATE,
            'list_of': VariableType.LIST_OF,
            'group_of': VariableType.GROUP_OF,
            'file': VariableType.FILE,
            
            # Legacy types
            'number': VariableType.NUMBER,
            'boolean': VariableType.BOOLEAN,
            'string': VariableType.STRING,
            'array': VariableType.ARRAY,
        }
        
        if user_type not in type_mapping:
            raise CodeGenError(f"Unknown type: {user_type}")
        
        return type_mapping[user_type]
    
    def infer_type(self, node: ASTNode) -> VariableType:
        """Infer the type of an AST node."""
        # This will be implemented by subclasses with target-specific logic
        # Base implementation provides common type inference
        from .ast import Literal, Identifier
        
        if isinstance(node, Literal):
            if isinstance(node.value, str):
                return VariableType.TEXT
            elif isinstance(node.value, bool):
                return VariableType.FLAG
            elif isinstance(node.value, int):
                return VariableType.INT
            elif isinstance(node.value, float):
                return VariableType.DECIMAL
        elif isinstance(node, Identifier):
            var = self.symbol_table.get_variable(node.name)
            if var:
                return var.type
        
        return VariableType.TEXT  # Default fallback
    
    def are_types_compatible(self, declared_type: VariableType, inferred_type: VariableType) -> bool:
        """Check if two types are compatible for assignment."""
        # Exact match
        if declared_type == inferred_type:
            return True
        
        # Numeric type compatibility
        if self._is_numeric_type(declared_type) and self._is_numeric_type(inferred_type):
            return True
        
        # Text type compatibility
        if self._is_text_type(declared_type) and self._is_text_type(inferred_type):
            return True
        
        # Boolean type compatibility
        if self._is_boolean_type(declared_type) and self._is_boolean_type(inferred_type):
            return True
        
        # Date-string compatibility
        if declared_type == VariableType.DATE and self._is_text_type(inferred_type):
            return True
        
        # Collection type compatibility
        if self._is_collection_type(declared_type) and self._is_collection_type(inferred_type):
            return True
        
        return False