"""Puck editor JSON format code generator for Droelang DSL."""

import json
from typing import Dict, Any, List, Optional, Union
from ...ast import (
    ASTNode, Program, DisplayStatement, IfStatement,
    Literal, Identifier, BinaryOp, PropertyAccess,
    Assignment, ArrayLiteral, WhileLoop, ForEachLoop,
    TaskAction, ActionDefinition, ReturnStatement,
    ModuleDefinition, DataDefinition, DataField,
    LayoutDefinition, FormDefinition, TitleComponent,
    InputComponent, TextareaComponent, DropdownComponent,
    CheckboxComponent, RadioComponent, ButtonComponent,
    ImageComponent, VideoComponent, AudioComponent,
    StringInterpolation, DataInstance, FieldAssignment
)
from ...codegen_base import BaseCodeGenerator, CodeGenError


class PuckCodeGenerator(BaseCodeGenerator):
    """Generates Puck editor JSON format from Droelang DSL."""
    
    def __init__(self):
        super().__init__()
        self.component_counter = 0
        self.data_models: Dict[str, DataDefinition] = {}
        self.actions: Dict[str, ActionDefinition] = {}
        self.layouts: List[LayoutDefinition] = []
        self.forms: List[FormDefinition] = []
        
    def generate(self, program: Program) -> str:
        """Generate Puck JSON from AST."""
        self.clear_output()
        self.component_counter = 0
        self.data_models.clear()
        self.actions.clear()
        self.layouts = []
        self.forms = []
        
        # First pass: collect definitions
        self._collect_definitions(program)
        
        # Generate Puck data structure
        puck_data = self._generate_puck_data()
        
        # Return as JSON string
        return json.dumps(puck_data, indent=2)
    
    def _collect_definitions(self, program: Program):
        """Collect all definitions from the program."""
        for stmt in program.statements:
            if isinstance(stmt, ModuleDefinition):
                # Process module contents
                for module_stmt in stmt.body:
                    self._process_statement(module_stmt)
            else:
                self._process_statement(stmt)
    
    def _process_statement(self, stmt: ASTNode):
        """Process individual statements to collect definitions."""
        if isinstance(stmt, DataDefinition):
            self.data_models[stmt.name] = stmt
        elif isinstance(stmt, ActionDefinition):
            self.actions[stmt.name] = stmt
        elif isinstance(stmt, LayoutDefinition):
            self.layouts.append(stmt)
        elif isinstance(stmt, FormDefinition):
            self.forms.append(stmt)
    
    def _generate_puck_data(self) -> Dict[str, Any]:
        """Generate the Puck editor data structure."""
        content = []
        root_props = {"title": "Droelang Page"}
        
        # Convert layouts to Puck components
        for layout in self.layouts:
            content.extend(self._convert_layout(layout))
        
        # Convert forms to Puck components
        for form in self.forms:
            content.extend(self._convert_form(form))
        
        # If no layouts or forms, create default content from other components
        if not content and (self.data_models or self.actions):
            content = self._create_default_layout()
        
        return {
            "content": content,
            "root": {"props": root_props}
        }
    
    def _convert_layout(self, layout: LayoutDefinition) -> List[Dict[str, Any]]:
        """Convert a layout definition to Puck components."""
        components = []
        
        # Create a Section component for the layout
        section = {
            "type": "Section",
            "id": self._generate_id(),
            "props": {
                "padding": 32,
                "background": "transparent"
            },
            "children": []
        }
        
        # Process layout children
        for child in layout.children:
            if isinstance(child, str) and child == "row":
                row_component = self._create_row_component()
                section["children"].append(row_component)
            elif isinstance(child, str) and child == "column":
                column_component = self._create_column_component()
                if section["children"] and section["children"][-1]["type"] == "Row":
                    # Add column to the last row
                    section["children"][-1]["children"].append(column_component)
                else:
                    # Create a new row for the column
                    row = self._create_row_component()
                    row["children"].append(column_component)
                    section["children"].append(row)
            elif hasattr(child, '__class__'):
                # Convert specific component types
                component = self._convert_component(child)
                if component:
                    self._add_component_to_layout(section, component)
        
        components.append(section)
        return components
    
    def _convert_form(self, form: FormDefinition) -> List[Dict[str, Any]]:
        """Convert a form definition to Puck components."""
        components = []
        
        # Create a Container component for the form
        container = {
            "type": "Container",
            "id": self._generate_id(),
            "props": {
                "padding": 16,
                "background": "white",
                "border": "light"
            },
            "children": []
        }
        
        # Add form title if exists
        if hasattr(form, 'title') and form.title:
            title = {
                "type": "Heading",
                "id": self._generate_id(),
                "props": {
                    "text": form.title,
                    "level": 2,
                    "align": "left"
                }
            }
            container["children"].append(title)
        
        # Process form fields
        for field in form.fields:
            field_component = self._convert_form_field(field)
            if field_component:
                container["children"].append(field_component)
        
        # Add submit button if form has action
        if hasattr(form, 'action') and form.action:
            submit_button = {
                "type": "Button",
                "id": self._generate_id(),
                "props": {
                    "text": "Submit",
                    "variant": "default",
                    "size": "default",
                    "fullWidth": "false"
                }
            }
            container["children"].append(submit_button)
        
        components.append(container)
        return components
    
    def _convert_component(self, component: ASTNode) -> Optional[Dict[str, Any]]:
        """Convert individual DSL components to Puck components."""
        if isinstance(component, TitleComponent):
            return {
                "type": "Heading",
                "id": self._generate_id(),
                "props": {
                    "text": component.text if isinstance(component.text, str) else str(component.text),
                    "level": component.level if hasattr(component, 'level') else 1,
                    "align": "left"
                }
            }
        elif isinstance(component, InputComponent):
            return {
                "type": "TextInput",
                "id": self._generate_id(),
                "props": {
                    "label": component.label if hasattr(component, 'label') else "Input",
                    "placeholder": component.placeholder if hasattr(component, 'placeholder') else "",
                    "required": "true" if hasattr(component, 'required') and component.required else "false",
                    "fullWidth": "true"
                }
            }
        elif isinstance(component, TextareaComponent):
            return {
                "type": "Textarea",
                "id": self._generate_id(),
                "props": {
                    "label": component.label if hasattr(component, 'label') else "Message",
                    "placeholder": component.placeholder if hasattr(component, 'placeholder') else "",
                    "rows": component.rows if hasattr(component, 'rows') else 4,
                    "required": "false"
                }
            }
        elif isinstance(component, ButtonComponent):
            return {
                "type": "Button",
                "id": self._generate_id(),
                "props": {
                    "text": component.text if hasattr(component, 'text') else "Button",
                    "variant": "default",
                    "size": "default",
                    "fullWidth": "false"
                }
            }
        elif isinstance(component, ImageComponent):
            return {
                "type": "Image",
                "id": self._generate_id(),
                "props": {
                    "src": component.src if hasattr(component, 'src') else "https://placehold.co/400x200",
                    "alt": component.alt if hasattr(component, 'alt') else "Image",
                    "width": "auto",
                    "rounded": "md"
                }
            }
        elif isinstance(component, DropdownComponent):
            options = []
            if hasattr(component, 'options'):
                options = [{"label": opt, "value": opt} for opt in component.options]
            return {
                "type": "Select",
                "id": self._generate_id(),
                "props": {
                    "label": component.label if hasattr(component, 'label') else "Select",
                    "options": options,
                    "required": "false"
                }
            }
        elif isinstance(component, CheckboxComponent):
            return {
                "type": "Checkbox",
                "id": self._generate_id(),
                "props": {
                    "label": component.label if hasattr(component, 'label') else "Options",
                    "name": "checkbox-group",
                    "options": [{"label": "Option 1", "value": "option1"}],
                    "required": "false"
                }
            }
        elif isinstance(component, RadioComponent):
            return {
                "type": "Radio",
                "id": self._generate_id(),
                "props": {
                    "label": component.label if hasattr(component, 'label') else "Choose",
                    "name": "radio-group",
                    "options": [{"label": "Option 1", "value": "option1"}],
                    "required": "false"
                }
            }
        elif isinstance(component, DisplayStatement):
            # Convert display statements to Text components
            text_value = self._extract_text_value(component.message)
            return {
                "type": "Text",
                "id": self._generate_id(),
                "props": {
                    "text": text_value,
                    "size": "base",
                    "align": "left"
                }
            }
        return None
    
    def _convert_form_field(self, field: ASTNode) -> Optional[Dict[str, Any]]:
        """Convert form field to Puck component."""
        # Reuse component conversion logic
        return self._convert_component(field)
    
    def _create_default_layout(self) -> List[Dict[str, Any]]:
        """Create a default layout when no explicit layouts are defined."""
        section = {
            "type": "Section",
            "id": self._generate_id(),
            "props": {
                "padding": 32,
                "background": "transparent"
            },
            "children": []
        }
        
        # Add a heading
        heading = {
            "type": "Heading",
            "id": self._generate_id(),
            "props": {
                "text": "Droelang Application",
                "level": 1,
                "align": "center"
            }
        }
        section["children"].append(heading)
        
        # Add data model info if available
        if self.data_models:
            text = {
                "type": "Text",
                "id": self._generate_id(),
                "props": {
                    "text": f"Data Models: {', '.join(self.data_models.keys())}",
                    "size": "base",
                    "align": "left"
                }
            }
            section["children"].append(text)
        
        # Add action info if available
        if self.actions:
            text = {
                "type": "Text",
                "id": self._generate_id(),
                "props": {
                    "text": f"Actions: {', '.join(self.actions.keys())}",
                    "size": "base",
                    "align": "left"
                }
            }
            section["children"].append(text)
        
        return [section]
    
    def _create_row_component(self) -> Dict[str, Any]:
        """Create a Row component."""
        return {
            "type": "Row",
            "id": self._generate_id(),
            "props": {
                "gap": 16,
                "alignItems": "stretch",
                "justifyContent": "flex-start",
                "wrap": "wrap"
            },
            "children": []
        }
    
    def _create_column_component(self) -> Dict[str, Any]:
        """Create a Column component."""
        return {
            "type": "Column",
            "id": self._generate_id(),
            "props": {
                "flex": "1",
                "gap": 8,
                "alignItems": "stretch",
                "minWidth": 120
            },
            "children": []
        }
    
    def _add_component_to_layout(self, layout: Dict[str, Any], component: Dict[str, Any]):
        """Add a component to the appropriate place in the layout."""
        if layout["children"]:
            last_child = layout["children"][-1]
            if last_child["type"] in ["Row", "Column", "Container"]:
                last_child["children"].append(component)
            else:
                layout["children"].append(component)
        else:
            layout["children"].append(component)
    
    def _extract_text_value(self, node: ASTNode) -> str:
        """Extract text value from various node types."""
        if isinstance(node, Literal):
            return str(node.value)
        elif isinstance(node, StringInterpolation):
            # Simplified string interpolation handling
            return f"{{dynamic text}}"
        elif isinstance(node, Identifier):
            return f"{{{{node.name}}}}"
        else:
            return str(node)
    
    def _generate_id(self) -> str:
        """Generate a unique component ID."""
        self.component_counter += 1
        return f"component-{self.component_counter}"