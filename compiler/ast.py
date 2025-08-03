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
class Assignment(ASTNode):
    """Represents a variable assignment (set x to value)."""
    variable: str
    value: ASTNode


@dataclass
class ArrayLiteral(ASTNode):
    """Represents an array literal like ["a", "b", "c"]."""
    elements: List[ASTNode]


@dataclass
class WhileLoop(ASTNode):
    """Represents a while loop."""
    condition: ASTNode
    body: List[ASTNode]


@dataclass
class ForEachLoop(ASTNode):
    """Represents a for each loop."""
    variable: str
    iterable: ASTNode
    body: List[ASTNode]


@dataclass
class ArithmeticOp(ASTNode):
    """Represents arithmetic operations (+, -, *, /)."""
    left: ASTNode
    operator: str
    right: ASTNode


@dataclass
class TaskAction(ASTNode):
    """Represents a task action definition (task name ... end)."""
    name: str
    body: List[ASTNode]


@dataclass
class TaskInvocation(ASTNode):
    """Represents a task invocation (run task_name)."""
    task_name: str


@dataclass
class ActionDefinition(ASTNode):
    """Represents an action definition (action name ... end action)."""
    name: str
    body: List[ASTNode]


@dataclass
class ReturnStatement(ASTNode):
    """Represents a return statement (respond with, answer is, output, give)."""
    expression: ASTNode
    return_type: str  # 'respond_with', 'answer_is', 'output', 'give'


@dataclass
class ActionInvocation(ASTNode):
    """Represents an action invocation that returns a value."""
    action_name: str


@dataclass
class Program(ASTNode):
    """Root node containing all statements in the program."""
    statements: List[ASTNode]