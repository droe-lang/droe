"""Reverse code generator: Convert Puck JSON back to Droelang DSL."""

import json
from typing import Dict, Any, List, Optional
from ...codegen_base import BaseCodeGenerator, CodeGenError


class PuckToDSLConverter:
    """Converts Puck editor JSON format back to Droelang DSL."""
    
    def __init__(self):
        self.indent_level = 0
        self.output_lines = []
        self.component_counter = 0
        
    def convert(self, puck_json: Dict[str, Any], original_metadata: Optional[Dict[str, Any]] = None) -> str:
        """Convert Puck JSON back to DSL."""
        self.output_lines = []
        self.indent_level = 0
        self.component_counter = 0
        
        # Add original metadata (target directive, etc.)
        if original_metadata:
            target = original_metadata.get('target')
            if target:
                self.output_lines.append(f"@target {target}")
                self.output_lines.append("")
        
        # Add module wrapper
        self.output_lines.append("module VisualApp")
        self.indent_level += 1
        
        # Convert content
        content = puck_json.get('content', [])
        if content:
            self._convert_content(content)
        else:
            self._add_line("// Empty layout")
        
        # Close module
        self.indent_level -= 1
        self.output_lines.append("end module")
        
        return '\n'.join(self.output_lines)
    
    def _convert_content(self, content: List[Dict[str, Any]]):
        """Convert Puck content components to DSL."""
        layouts_found = False
        forms_found = False
        
        for component in content:
            component_type = component.get('type')
            
            if component_type == 'Section':
                if not layouts_found:
                    self._add_line("layout MainLayout")
                    self.indent_level += 1
                    layouts_found = True
                
                self._convert_section(component)
            
            elif component_type == 'Container':
                # Check if this looks like a form
                children = component.get('children', [])
                has_form_elements = any(
                    child.get('type') in ['TextInput', 'Textarea', 'Select', 'Button', 'Checkbox', 'Radio'] 
                    for child in children
                )
                
                if has_form_elements:
                    if not forms_found:
                        if layouts_found:
                            self.indent_level -= 1
                            self._add_line("end layout")
                            self._add_line("")
                        forms_found = True
                    
                    self._convert_form_container(component)
                else:
                    self._convert_container(component)
        
        # Close any open blocks
        if layouts_found:
            self.indent_level -= 1
            self._add_line("end layout")
    
    def _convert_section(self, section: Dict[str, Any]):
        """Convert Section component to DSL layout elements."""
        children = section.get('children', [])
        
        for child in children:
            child_type = child.get('type')
            
            if child_type == 'Row':
                self._add_line("row")
                self.indent_level += 1
                self._convert_row(child)
                self.indent_level -= 1
                self._add_line("end row")
            
            elif child_type == 'Column':
                self._add_line("column")
                self.indent_level += 1
                self._convert_column(child)
                self.indent_level -= 1
                self._add_line("end column")
            
            else:
                self._convert_component_to_dsl(child)
    
    def _convert_row(self, row: Dict[str, Any]):
        """Convert Row component."""
        children = row.get('children', [])
        for child in children:
            if child.get('type') == 'Column':
                self._add_line("column")
                self.indent_level += 1
                self._convert_column(child)
                self.indent_level -= 1
                self._add_line("end column")
            else:
                self._convert_component_to_dsl(child)
    
    def _convert_column(self, column: Dict[str, Any]):
        """Convert Column component."""
        children = column.get('children', [])
        for child in children:
            self._convert_component_to_dsl(child)
    
    def _convert_form_container(self, container: Dict[str, Any]):
        """Convert Container that represents a form."""
        form_name = f"Form{self.component_counter}"
        self.component_counter += 1
        
        self._add_line(f"form {form_name}")
        self.indent_level += 1
        
        children = container.get('children', [])
        for child in children:
            self._convert_form_component(child)
        
        self.indent_level -= 1
        self._add_line("end form")
    
    def _convert_container(self, container: Dict[str, Any]):
        """Convert regular Container component."""
        self._add_line("layout Container")
        self.indent_level += 1
        
        children = container.get('children', [])
        for child in children:
            self._convert_component_to_dsl(child)
        
        self.indent_level -= 1
        self._add_line("end layout")
    
    def _convert_form_component(self, component: Dict[str, Any]):
        """Convert form-specific components."""
        component_type = component.get('type')
        props = component.get('props', {})
        
        if component_type == 'TextInput':
            label = props.get('label', 'Input')
            placeholder = props.get('placeholder', '')
            required = props.get('required') == 'true'
            
            line = f'input "{label}"'
            if placeholder:
                line += f' placeholder="{placeholder}"'
            if required:
                line += ' required'
            self._add_line(line)
        
        elif component_type == 'Textarea':
            label = props.get('label', 'Message')
            rows = props.get('rows', 4)
            
            line = f'textarea "{label}" rows={rows}'
            self._add_line(line)
        
        elif component_type == 'Select':
            label = props.get('label', 'Select')
            options = props.get('options', [])
            
            line = f'dropdown "{label}"'
            if options:
                option_values = [opt.get('label', opt.get('value', '')) for opt in options]
                formatted_options = ', '.join(f'"{opt}"' for opt in option_values)
                line += f' options=[{formatted_options}]'
            self._add_line(line)
        
        elif component_type == 'Button':
            text = props.get('text', 'Button')
            variant = props.get('variant', 'default')
            
            if variant == 'default' and text.lower() in ['submit', 'send', 'save']:
                self._add_line(f'submit "{text}"')
            else:
                self._add_line(f'button "{text}"')
        
        elif component_type == 'Checkbox':
            label = props.get('label', 'Checkbox')
            self._add_line(f'checkbox "{label}"')
        
        elif component_type == 'Radio':
            label = props.get('label', 'Radio')
            self._add_line(f'radio "{label}"')
        
        else:
            # Fallback to general component conversion
            self._convert_component_to_dsl(component)
    
    def _convert_component_to_dsl(self, component: Dict[str, Any]):
        """Convert general components to DSL."""
        component_type = component.get('type')
        props = component.get('props', {})
        
        if component_type == 'Heading':
            text = props.get('text', 'Heading')
            level = props.get('level', 1)
            self._add_line(f'heading{level} "{text}"')
        
        elif component_type == 'Text':
            text = props.get('text', 'Text')
            self._add_line(f'text "{text}"')
        
        elif component_type == 'Image':
            src = props.get('src', 'image.jpg')
            alt = props.get('alt', 'Image')
            self._add_line(f'image "{src}" alt="{alt}"')
        
        elif component_type == 'Button':
            text = props.get('text', 'Button')
            self._add_line(f'button "{text}"')
        
        elif component_type == 'Spacer':
            height = props.get('height', '20px')
            self._add_line(f'spacer height={height}')
        
        elif component_type == 'Divider':
            style = props.get('style', 'solid')
            self._add_line(f'divider style={style}')
        
        else:
            # Unknown component - add as comment
            self._add_line(f'// Unknown component: {component_type}')
    
    def _add_line(self, content: str):
        """Add a line with proper indentation."""
        if content.strip():
            indent = "    " * self.indent_level
            self.output_lines.append(f"{indent}{content}")
        else:
            self.output_lines.append("")


def convert_puck_to_dsl(puck_json_str: str, original_metadata: Optional[Dict[str, Any]] = None) -> str:
    """Convert Puck JSON string back to DSL."""
    try:
        puck_data = json.loads(puck_json_str) if isinstance(puck_json_str, str) else puck_json_str
        converter = PuckToDSLConverter()
        return converter.convert(puck_data, original_metadata)
    except Exception as e:
        raise CodeGenError(f"Failed to convert Puck JSON to DSL: {e}")


# Example usage and test
if __name__ == "__main__":
    # Test the converter
    sample_puck_json = {
        "content": [
            {
                "type": "Section",
                "id": "section-1",
                "props": {"padding": 32, "background": "transparent"},
                "children": [
                    {
                        "type": "Row",
                        "id": "row-1",
                        "props": {"gap": 16},
                        "children": [
                            {
                                "type": "Column",
                                "id": "col-1",
                                "props": {"flex": "1"},
                                "children": [
                                    {
                                        "type": "Heading",
                                        "id": "heading-1",
                                        "props": {"text": "Welcome", "level": 1}
                                    },
                                    {
                                        "type": "Text",
                                        "id": "text-1",
                                        "props": {"text": "This is a sample layout"}
                                    }
                                ]
                            }
                        ]
                    }
                ]
            },
            {
                "type": "Container",
                "id": "form-container",
                "props": {"padding": 16},
                "children": [
                    {
                        "type": "TextInput",
                        "id": "input-1",
                        "props": {"label": "Name", "placeholder": "Enter your name", "required": "true"}
                    },
                    {
                        "type": "Textarea",
                        "id": "textarea-1",
                        "props": {"label": "Message", "rows": 4}
                    },
                    {
                        "type": "Button",
                        "id": "submit-btn",
                        "props": {"text": "Submit", "variant": "default"}
                    }
                ]
            }
        ],
        "root": {"props": {"title": "Sample Page"}}
    }
    
    metadata = {"target": "html"}
    
    result = convert_puck_to_dsl(sample_puck_json, metadata)
    print("Generated DSL:")
    print(result)