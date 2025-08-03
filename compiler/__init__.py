"""Roe DSL Compiler Package."""

from .compiler import compile, compile_file, CompilerError
from .parser import parse, ParseError
from .codegen_wat import generate_wat, CodeGenError
from .ast import *

__all__ = [
    'compile', 
    'compile_file', 
    'CompilerError',
    'parse', 
    'ParseError',
    'generate_wat', 
    'CodeGenError',
]