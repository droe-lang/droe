"""Puck editor JSON format code generator for Droelang DSL."""

import json
import uuid
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
            "root": {"props": root_props},
            "content": content,
            "zones": {}
        }
    
    def _convert_layout(self, layout: LayoutDefinition) -> List[Dict[str, Any]]:
        """Convert a layout definition to Puck components."""
        components = []
        
        # Create direct components without nesting - flatten everything
        def extract_components(layout_obj):
            extracted = []
            for child in layout_obj.children:
                if isinstance(child, LayoutDefinition):
                    # Recursively extract from nested layouts
                    extracted.extend(extract_components(child))
                elif hasattr(child, '__class__'):
                    # Convert leaf components
                    component = self._convert_component(child)
                    if component:
                        extracted.append(component)
            return extracted
        
        # Extract all components and return flat list
        return extract_components(layout)
    
    def _convert_form(self, form: FormDefinition) -> List[Dict[str, Any]]:
        """Convert a form definition to Puck components."""
        components = []
        
        # Create a Container component for the form
        container = {
            "type": "Container",
            "id": self._generate_id("Container"),
            "props": {
                "padding": 16,
                "background": "white",
                "border": "light",
                "items": []
            }
        }
        
        # Add form title if exists
        if hasattr(form, 'title') and form.title:
            title = {
                "type": "Heading",
                "props": {
                    "id": self._generate_id("Heading"),
                    "text": form.title,
                    "level": 2,
                    "align": "left"
                }
            }
            container["props"]["items"].append(title)
        
        # Process form fields
        for field in form.fields:
            field_component = self._convert_form_field(field)
            if field_component:
                container["props"]["items"].append(field_component)
        
        # Add submit button if form has action
        if hasattr(form, 'action') and form.action:
            submit_button = {
                "type": "Button",
                "props": {
                    "id": self._generate_id("Button"),
                    "text": "Submit",
                    "variant": "default",
                    "size": "default",
                    "fullWidth": "false"
                }
            }
            container["props"]["items"].append(submit_button)
        
        components.append(container)
        return components
    
    def _convert_component(self, component: ASTNode) -> Optional[Dict[str, Any]]:
        """Convert individual DSL components to Puck components."""
        if isinstance(component, TitleComponent):
            props = {
                "text": component.text if isinstance(component.text, str) else str(component.text),
                "level": component.level if hasattr(component, 'level') else 1,
                "align": "left",
                "id": self._generate_id("Heading")  # ID goes inside props
            }
            
            # Add CSS classes to props for round-trip preservation
            if hasattr(component, 'css_classes') and component.css_classes:
                props["cssClasses"] = component.css_classes
                props["className"] = " ".join(component.css_classes)
            
            return {
                "type": "Heading",
                "props": props
            }
        elif isinstance(component, InputComponent):
            return {
                "type": "TextInput",
                "props": {
                    "label": component.label if hasattr(component, 'label') else "Input",
                    "placeholder": component.placeholder if hasattr(component, 'placeholder') else "",
                    "required": "true" if hasattr(component, 'required') and component.required else "false",
                    "fullWidth": "true",
                    "id": self._generate_id("TextInput")
                }
            }
        elif isinstance(component, TextareaComponent):
            return {
                "type": "Textarea",
                "props": {
                    "id": self._generate_id("Textarea"),
                    "label": component.label if hasattr(component, 'label') else "Message",
                    "placeholder": component.placeholder if hasattr(component, 'placeholder') else "",
                    "rows": component.rows if hasattr(component, 'rows') else 4,
                    "required": "false"
                }
            }
        elif isinstance(component, ButtonComponent):
            return {
                "type": "Button",
                "props": {
                    "id": self._generate_id("Button"),
                    "text": component.text if hasattr(component, 'text') else "Button",
                    "variant": "default",
                    "size": "default",
                    "fullWidth": "false"
                }
            }
        elif isinstance(component, ImageComponent):
            return {
                "type": "Image",
                "props": {
                    "id": self._generate_id("Image"),
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
                "props": {
                    "id": self._generate_id("Select"),
                    "label": component.label if hasattr(component, 'label') else "Select",
                    "options": options,
                    "required": "false"
                }
            }
        elif isinstance(component, CheckboxComponent):
            return {
                "type": "Checkbox",
                "props": {
                    "id": self._generate_id("Checkbox"),
                    "label": component.label if hasattr(component, 'label') else "Options",
                    "name": "checkbox-group",
                    "options": [{"label": "Option 1", "value": "option1"}],
                    "required": "false"
                }
            }
        elif isinstance(component, RadioComponent):
            return {
                "type": "Radio",
                "props": {
                    "id": self._generate_id("Radio"),
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
                "props": {
                    "id": self._generate_id("Text"),
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
            "id": self._generate_id("Section"),
            "props": {
                "padding": 32,
                "background": "transparent",
                "items": []
            }
        }
        
        # Add a heading
        heading = {
            "type": "Heading",
            "id": self._generate_id("Heading"),
            "props": {
                "text": "Droelang Application",
                "level": 1,
                "align": "center"
            }
        }
        section["props"]["items"].append(heading)
        
        # Add data model info if available
        if self.data_models:
            text = {
                "type": "Text",
                "props": {
                    "id": self._generate_id("Text"),
                    "text": f"Data Models: {', '.join(self.data_models.keys())}",
                    "size": "base",
                    "align": "left"
                }
            }
            section["props"]["items"].append(text)
        
        # Add action info if available
        if self.actions:
            text = {
                "type": "Text",
                "props": {
                    "id": self._generate_id("Text"),
                    "text": f"Actions: {', '.join(self.actions.keys())}",
                    "size": "base",
                    "align": "left"
                }
            }
            section["props"]["items"].append(text)
        
        return [section]
    
    def _create_row_component(self, layout: Optional[LayoutDefinition] = None) -> Dict[str, Any]:
        """Create a Row component."""
        props = {
            "gap": 16,
            "alignItems": "stretch",
            "justifyContent": "flex-start",
            "wrap": "wrap",
            "items": []
        }
        
        # Add CSS classes to props for round-trip preservation
        if layout and hasattr(layout, 'css_classes') and layout.css_classes:
            props["cssClasses"] = layout.css_classes
            props["className"] = " ".join(layout.css_classes)
        
        return {
            "type": "Row",
            "id": self._generate_id("Row"),
            "props": props
        }
    
    def _create_column_component(self, layout: Optional[LayoutDefinition] = None) -> Dict[str, Any]:
        """Create a Column component."""
        props = {
            "flex": "1",
            "gap": 8,
            "alignItems": "stretch",
            "minWidth": 120,
            "items": []
        }
        
        # Add CSS classes to props for round-trip preservation
        if layout and hasattr(layout, 'css_classes') and layout.css_classes:
            props["cssClasses"] = layout.css_classes
            props["className"] = " ".join(layout.css_classes)
        
        return {
            "type": "Column",
            "id": self._generate_id("Column"),
            "props": props
        }
    
    def _add_component_to_layout(self, layout: Dict[str, Any], component: Dict[str, Any]):
        """Add a component to the appropriate place in the layout."""
        if layout["props"]["items"]:
            last_child = layout["props"]["items"][-1]
            if last_child["type"] in ["Row", "Column", "Container"]:
                last_child["props"]["items"].append(component)
            else:
                layout["props"]["items"].append(component)
        else:
            layout["props"]["items"].append(component)
    
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
    
    def _generate_id(self, component_type: str = "Component") -> str:
        """Generate a unique component ID using UUID in Puck format."""
        return f"{component_type}-{uuid.uuid4()}"
    
    def emit_expression(self, expr: ASTNode):
        """Emit code for an expression.
        
        Note: This method is required by BaseCodeGenerator but not used
        in Puck JSON generation since we build the structure directly.
        """
        # Not used in Puck generation - we handle expressions in _generate_puck_data
        pass
    
    def emit_statement(self, stmt: ASTNode):
        """Emit code for a statement.
        
        Note: This method is required by BaseCodeGenerator but not used
        in Puck JSON generation since we build the structure directly.
        """
        # Not used in Puck generation - we handle statements in _collect_definitions
        pass