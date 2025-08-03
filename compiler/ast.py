"""AST node definitions for Roe DSL compiler."""

from dataclasses import dataclass
from typing import Any, List, Optional, Union


@dataclass
class ASTNode:
    """Base class for all AST nodes."""
    pass


@dataclass
class Literal(ASTNode):
    """Represents a literal value (string, number, boolean)."""
    value: Union[str, int, float, bool]
    type: str  # 'string', 'number', 'boolean'


@dataclass
class Identifier(ASTNode):
    """Represents an identifier (variable name)."""
    name: str


@dataclass
class BinaryOp(ASTNode):
    """Represents a binary operation (e.g., >, <, ==, +, -)."""
    left: ASTNode
    operator: str
    right: ASTNode


@dataclass
class DisplayStatement(ASTNode):
    """Represents a display statement."""
    expression: ASTNode


@dataclass
class IfStatement(ASTNode):
    """Represents an if-then statement."""
    condition: ASTNode
    then_body: List[ASTNode]
    else_body: Optional[List[ASTNode]] = None


@dataclass
class PropertyAccess(ASTNode):
    """Represents property access (e.g., user.age)."""
    object: ASTNode
    property: str


@dataclass
class Program(ASTNode):
    """Root node containing all statements in the program."""
    statements: List[ASTNode]