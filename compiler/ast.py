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
class ModuleDefinition(ASTNode):
    """Represents a module definition (module name ... end module)."""
    name: str
    body: List[ASTNode]


@dataclass
class DataDefinition(ASTNode):
    """Represents a data structure definition (data Name ... end data)."""
    name: str
    fields: List['DataField']


@dataclass
class DataField(ASTNode):
    """Represents a field in a data structure."""
    name: str
    type: str


@dataclass
class ActionDefinitionWithParams(ASTNode):
    """Represents an action definition with parameters and return type."""
    name: str
    parameters: List['ActionParameter']
    return_type: Optional[str]
    body: List[ASTNode]


@dataclass
class ActionParameter(ASTNode):
    """Represents a parameter in an action definition."""
    name: str
    type: str


@dataclass
class ActionInvocationWithArgs(ASTNode):
    """Represents an action invocation with arguments."""
    module_name: Optional[str]  # For module.action calls
    action_name: str
    arguments: List[ASTNode]


@dataclass
class StringInterpolation(ASTNode):
    """Represents a string with variable interpolation like 'Hello [name]'."""
    parts: List[ASTNode]  # Mix of Literal (for text) and Identifier (for variables)


@dataclass
class DataInstance(ASTNode):
    """Represents a data structure instance creation."""
    data_type: str  # The data type name (e.g., "User")
    field_values: List['FieldAssignment']  # Field assignments


@dataclass
class FieldAssignment(ASTNode):
    """Represents a field assignment in data instance creation."""
    field_name: str
    value: ASTNode


@dataclass
class Program(ASTNode):
    """Root node containing all statements in the program."""
    statements: List[ASTNode]