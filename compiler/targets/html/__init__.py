"""HTML target for Roelang compiler.

This module provides HTML/JavaScript-specific code generation functionality
for creating interactive web applications.
"""

from .codegen import HTMLCodeGenerator as HTMLGenerator

__all__ = ['HTMLGenerator']