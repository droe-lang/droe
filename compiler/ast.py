"""AST node definitions for Roe DSL compiler."""

from dataclasses import dataclass, field
from typing import Any, List, Optional, Union


@dataclass
class ASTNode:
    """Base class for all AST nodes."""
    line_number: Optional[int] = field(default=None, init=False)


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
    """Represents a task action definition (task name with params ... end)."""
    name: str
    parameters: List['ActionParameter'] = field(default_factory=list)  # Reuse ActionParameter for consistency
    body: List[ASTNode] = field(default_factory=list)


@dataclass
class TaskInvocation(ASTNode):
    """Represents a task invocation (run task_name with args)."""
    task_name: str
    arguments: List[ASTNode] = field(default_factory=list)  # Arguments for parameterized tasks


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
    storage_type: Optional[str] = None  # 'short_store' (sessionStorage) or 'long_store' (localStorage)


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
class IncludeStatement(ASTNode):
    """Represents an include statement (include ModuleName.roe)."""
    module_name: str  # The module name without .roe extension
    file_path: str    # The full file path (ModuleName.roe)


@dataclass
class AssetInclude(ASTNode):
    """Represents an asset include statement (include assets/style.css)."""
    asset_path: str  # Path to asset file
    asset_type: str  # 'css', 'js', 'font', etc.


@dataclass
class FormatExpression(ASTNode):
    """Represents a format expression (format variable as "pattern")."""
    expression: ASTNode  # The expression to format
    format_pattern: str  # The format pattern string


@dataclass
class MetadataAnnotation(ASTNode):
    """Represents a metadata annotation like @target web or @name user_form."""
    key: str  # The annotation key (target, name, description)
    value: str  # The annotation value


@dataclass
class LayoutDefinition(ASTNode):
    """Represents a layout definition (layout name ... end layout)."""
    name: str
    layout_type: str  # 'column', 'row', 'grid', 'stack', 'overlay'
    children: List[ASTNode]
    attributes: List['AttributeDefinition'] = field(default_factory=list)
    css_classes: List[str] = field(default_factory=list)
    style: Optional[str] = None  # Inline style attribute


@dataclass
class FormDefinition(ASTNode):
    """Represents a form definition (form name ... end form)."""
    name: str
    children: List[ASTNode]
    attributes: List['AttributeDefinition'] = field(default_factory=list)
    css_classes: List[str] = field(default_factory=list)


@dataclass
class TitleComponent(ASTNode):
    """Represents a title component."""
    text: str
    attributes: List['AttributeDefinition'] = field(default_factory=list)
    css_classes: List[str] = field(default_factory=list)
    component_type: str = field(default="title", init=False)


@dataclass
class InputComponent(ASTNode):
    """Represents an input component."""
    input_type: str = "text"  # 'text', 'password', 'email', etc.
    binding: Optional[str] = None  # Data binding target
    attributes: List['AttributeDefinition'] = field(default_factory=list)
    element_id: Optional[str] = None  # Element ID for form handling
    css_classes: List[str] = field(default_factory=list)
    component_type: str = field(default="input", init=False)


@dataclass
class TextareaComponent(ASTNode):
    """Represents a textarea component."""
    binding: Optional[str] = None
    attributes: List['AttributeDefinition'] = field(default_factory=list)
    css_classes: List[str] = field(default_factory=list)
    element_id: Optional[str] = None  # Element ID for form handling
    component_type: str = field(default="textarea", init=False)


@dataclass
class DropdownComponent(ASTNode):
    """Represents a dropdown/select component."""
    options: List[ASTNode] = field(default_factory=list)
    binding: Optional[str] = None
    attributes: List['AttributeDefinition'] = field(default_factory=list)
    element_id: Optional[str] = None  # Element ID for form handling
    css_classes: List[str] = field(default_factory=list)
    component_type: str = field(default="dropdown", init=False)


@dataclass
class ToggleComponent(ASTNode):
    """Represents a toggle/switch component."""
    binding: Optional[str] = None
    attributes: List['AttributeDefinition'] = field(default_factory=list)
    css_classes: List[str] = field(default_factory=list)
    element_id: Optional[str] = None  # Element ID for form handling
    component_type: str = field(default="toggle", init=False)


@dataclass
class CheckboxComponent(ASTNode):
    """Represents a checkbox component."""
    text: Optional[str] = None
    binding: Optional[str] = None
    attributes: List['AttributeDefinition'] = field(default_factory=list)
    element_id: Optional[str] = None  # Element ID for form handling
    css_classes: List[str] = field(default_factory=list)
    component_type: str = field(default="checkbox", init=False)


@dataclass
class RadioComponent(ASTNode):
    """Represents a radio button component."""
    text: Optional[str] = None
    value: Optional[str] = None
    binding: Optional[str] = None
    attributes: List['AttributeDefinition'] = field(default_factory=list)
    element_id: Optional[str] = None  # Element ID for form handling
    css_classes: List[str] = field(default_factory=list)
    component_type: str = field(default="radio", init=False)


@dataclass
class ButtonComponent(ASTNode):
    """Represents a button component."""
    text: str
    action: Optional[str] = None  # Action to run on click
    attributes: List['AttributeDefinition'] = field(default_factory=list)
    css_classes: List[str] = field(default_factory=list)
    component_type: str = field(default="button", init=False)


@dataclass
class ImageComponent(ASTNode):
    """Represents an image component."""
    src: str  # Image source path
    alt: Optional[str] = None  # Alt text
    attributes: List['AttributeDefinition'] = field(default_factory=list)
    css_classes: List[str] = field(default_factory=list)
    component_type: str = field(default="image", init=False)


@dataclass
class VideoComponent(ASTNode):
    """Represents a video component."""
    src: str  # Video source path
    controls: bool = True
    autoplay: bool = False
    loop: bool = False
    muted: bool = False
    attributes: List['AttributeDefinition'] = field(default_factory=list)
    css_classes: List[str] = field(default_factory=list)
    component_type: str = field(default="video", init=False)


@dataclass
class AudioComponent(ASTNode):
    """Represents an audio component."""
    src: str  # Audio source path
    controls: bool = True
    autoplay: bool = False
    loop: bool = False
    attributes: List['AttributeDefinition'] = field(default_factory=list)
    css_classes: List[str] = field(default_factory=list)
    component_type: str = field(default="audio", init=False)


@dataclass
class AttributeDefinition(ASTNode):
    """Represents an attribute definition (validate required, bind LoginForm.email, etc.)."""
    name: str
    value: Optional[ASTNode] = None  # Can be a literal or expression


@dataclass
class ValidationAttribute(ASTNode):
    """Represents a validation attribute."""
    validation_type: str  # 'required', 'email', 'numeric', etc.
    name: str = field(default="validate", init=False)


@dataclass
class BindingAttribute(ASTNode):
    """Represents a data binding attribute."""
    binding_target: str  # e.g., 'LoginForm.email'
    name: str = field(default="bind", init=False)


@dataclass
class ActionAttribute(ASTNode):
    """Represents an action attribute (run action_name)."""
    action_name: str
    name: str = field(default="run", init=False)


@dataclass
class Program(ASTNode):
    """Root node containing all statements in the program."""
    statements: List[ASTNode]
    metadata: List[MetadataAnnotation] = field(default_factory=list)  # Metadata annotations
    included_modules: List['IncludeStatement'] = None  # Track included modules