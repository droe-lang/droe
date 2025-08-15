"""Modular parser package for Droe DSL."""

from .core import Parser, ParseError


def parse(source: str):
    """Parse source code into AST (backward compatibility)."""
    parser = Parser(source)
    return parser.parse()


__all__ = ['Parser', 'ParseError', 'parse']