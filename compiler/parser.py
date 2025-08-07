"""
Roe DSL Parser - Modular parser for the Roe Domain-Specific Language.

This module provides a backward-compatible interface to the new modular parser system.
The actual parsing logic is split across multiple modules in the parser/ package:
- base.py: Base parser functionality and utilities
- expressions.py: Expression parsing (literals, operators, interpolation)  
- ui_components.py: UI component parsing (shared between web and mobile)
- statements.py: Statement parsing (if, loops, assignments, etc.)
- structures.py: Structural elements (modules, data, layouts, forms)
- core.py: Main parser that orchestrates all modules
"""

# Import from the new modular structure for backward compatibility
from .parser import Parser, ParseError

# Re-export for backward compatibility
__all__ = ['Parser', 'ParseError']