"""Symbol table for variable management in Roe DSL."""

from typing import Dict, Any, Optional
from enum import Enum


class VariableType(Enum):
    """Supported variable types."""
    NUMBER = "number"
    STRING = "string"
    BOOLEAN = "boolean"
    ARRAY = "array"


class Variable:
    """Represents a variable in the symbol table."""
    
    def __init__(self, name: str, var_type: VariableType, value: Any = None, wasm_index: int = -1):
        self.name = name
        self.type = var_type
        self.value = value
        self.wasm_index = wasm_index  # Index in WASM local variables


class SymbolTable:
    """Manages variables and their types during compilation."""
    
    def __init__(self):
        self.variables: Dict[str, Variable] = {}
        self.next_local_index = 0
    
    def declare_variable(self, name: str, var_type: VariableType, value: Any = None) -> Variable:
        """Declare a new variable."""
        if name in self.variables:
            # Allow redeclaration (assignment)
            var = self.variables[name]
            var.type = var_type
            var.value = value
            return var
        
        var = Variable(name, var_type, value, self.next_local_index)
        self.variables[name] = var
        self.next_local_index += 1
        return var
    
    def get_variable(self, name: str) -> Optional[Variable]:
        """Get a variable by name."""
        return self.variables.get(name)
    
    def has_variable(self, name: str) -> bool:
        """Check if variable exists."""
        return name in self.variables
    
    def get_all_variables(self) -> Dict[str, Variable]:
        """Get all variables."""
        return self.variables.copy()
    
    def get_local_count(self) -> int:
        """Get the number of local variables needed."""
        return self.next_local_index