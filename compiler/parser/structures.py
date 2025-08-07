"""Structural elements parsing - modules, data, layouts, forms."""

import re
from typing import List, Optional
from ..ast import (
    ModuleDefinition, DataDefinition, DataField, 
    LayoutDefinition, FormDefinition, MetadataAnnotation
)
from .ui_components import UIComponentParser
from .base import ParseError


class StructureParser(UIComponentParser):
    """Parser for structural elements like modules, data definitions, layouts, and forms."""
    
    def parse_module_definition(self) -> ModuleDefinition:
        """Parse Module definition."""
        name_line = self.consume_line()
        if not name_line or not name_line.startswith('    Name:'):
            raise ParseError("Module missing Name")
        
        name = name_line.strip()[5:].strip()
        name = self.extract_string_literal(name) or name
        
        # Parse exports
        exports = []
        while self.current_line < len(self.lines):
            next_line = self.peek_line()
            
            if not next_line or not next_line.startswith('    '):
                break
            
            if next_line.strip().startswith('Export:'):
                self.consume_line()
                export_name = next_line.strip()[7:].strip()
                export_name = self.extract_string_literal(export_name) or export_name
                exports.append(export_name)
        
        return ModuleDefinition(name, exports)
    
    def parse_data_definition(self) -> DataDefinition:
        """Parse Data definition."""
        name_line = self.consume_line()
        if not name_line or not name_line.startswith('    Name:'):
            raise ParseError("Data missing Name")
        
        name = name_line.strip()[5:].strip()
        name = self.extract_string_literal(name) or name
        
        # Parse fields
        fields = []
        while self.current_line < len(self.lines):
            next_line = self.peek_line()
            
            if not next_line or not next_line.startswith('    '):
                break
            
            if next_line.strip().startswith('Field:'):
                self.consume_line()
                field_spec = next_line.strip()[6:].strip()
                
                # Parse field specification (name: type)
                if ':' in field_spec:
                    field_name, field_type = field_spec.split(':', 1)
                    field_name = field_name.strip()
                    field_type = field_type.strip()
                    fields.append(DataField(field_name, field_type))
        
        return DataDefinition(name, fields)
    
    def parse_layout_definition(self) -> LayoutDefinition:
        """Parse Layout definition."""
        name_line = self.consume_line()
        if not name_line or not name_line.startswith('    Name:'):
            raise ParseError("Layout missing Name")
        
        name = name_line.strip()[5:].strip()
        name = self.extract_string_literal(name) or name
        
        # Parse components
        components = []
        while self.current_line < len(self.lines):
            next_line = self.peek_line()
            
            if not next_line or not next_line.startswith('    '):
                break
            
            self.consume_line()
            component_line = next_line[4:].strip()
            
            # Skip Name line
            if component_line.startswith('Name:'):
                continue
            
            # Parse UI component
            component = self.parse_component(component_line)
            if component:
                components.append(component)
        
        return LayoutDefinition(name, 'column', components)
    
    def parse_form_definition(self) -> FormDefinition:
        """Parse Form definition."""
        name_line = self.consume_line()
        if not name_line or not name_line.startswith('    Name:'):
            raise ParseError("Form missing Name")
        
        name = name_line.strip()[5:].strip()
        name = self.extract_string_literal(name) or name
        
        # Parse form elements
        elements = []
        while self.current_line < len(self.lines):
            next_line = self.peek_line()
            
            if not next_line or not next_line.startswith('    '):
                break
            
            self.consume_line()
            element_line = next_line[4:].strip()
            
            # Skip Name line
            if element_line.startswith('Name:'):
                continue
            
            # Parse form element (UI component)
            element = self.parse_component(element_line)
            if element:
                elements.append(element)
        
        return FormDefinition(name, elements)  # elements are the children
    
    def parse_inline_layout(self, line: str) -> LayoutDefinition:
        """Parse inline layout definition like Layout "name": [components]."""
        match = re.match(r'Layout\s+"([^"]+)"\s*:\s*\[(.*)\]', line, re.DOTALL)
        if not match:
            match = re.match(r"Layout\s+'([^']+)'\s*:\s*\[(.*)\]", line, re.DOTALL)
        
        if not match:
            raise ParseError(f"Invalid inline Layout: {line}")
        
        name = match.group(1)
        components_str = match.group(2).strip()
        
        # Parse components
        components = []
        if components_str:
            # Split by commas at the top level (not inside brackets)
            component_parts = []
            current = ""
            depth = 0
            in_string = False
            string_char = None
            
            for char in components_str:
                if not in_string and (char == '"' or char == "'"):
                    in_string = True
                    string_char = char
                elif in_string and char == string_char:
                    in_string = False
                    string_char = None
                elif not in_string:
                    if char in '[({':
                        depth += 1
                    elif char in '])}':
                        depth -= 1
                    elif char == ',' and depth == 0:
                        component_parts.append(current.strip())
                        current = ""
                        continue
                
                current += char
            
            if current.strip():
                component_parts.append(current.strip())
            
            # Parse each component
            for comp_str in component_parts:
                component = self.parse_component(comp_str)
                if component:
                    components.append(component)
        
        return LayoutDefinition(name, 'column', components)
    
    def parse_metadata(self, line: str) -> Optional[MetadataAnnotation]:
        """Parse metadata annotation like @metadata(key="value") or @target html."""
        if not line.startswith('@'):
            return None
        
        # Match @metadata(key="value", key2="value2") - complex syntax
        match = re.match(r'@(\w+)\((.*)\)', line)
        if match:
            annotation_type = match.group(1)
            params_str = match.group(2).strip()
            
            # Parse parameters
            params = {}
            if params_str:
                # Simple key="value" parsing
                param_matches = re.findall(r'(\w+)\s*=\s*"([^"]*)"', params_str)
                for key, value in param_matches:
                    params[key] = value
                
                # Also check for single quotes
                param_matches = re.findall(r"(\w+)\s*=\s*'([^']*)'", params_str)
                for key, value in param_matches:
                    params[key] = value
            
            return MetadataAnnotation(key=annotation_type, value=params)
        
        # Match simple @key value - legacy syntax like @target html
        match = re.match(r'@(\w+)(?:\s+(.+))?', line)
        if match:
            key = match.group(1)
            value = match.group(2).strip() if match.group(2) else ""
            return MetadataAnnotation(key=key, value=value)
        
        return None