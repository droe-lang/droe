"""Structural elements parsing - modules, data, layouts, forms."""

import re
from typing import List, Optional
from ..ast import (
    ModuleDefinition, DataDefinition, DataField, 
    FragmentDefinition, FormDefinition, MetadataAnnotation,
    ActionDefinition, ScreenDefinition, FragmentReference,
    SlotComponent
)
from .ui_components import UIComponentParser
from .base import ParseError


class StructureParser(UIComponentParser):
    """Parser for structural elements like modules, data definitions, layouts, and forms."""
    
    def __init__(self, source: str):
        super().__init__(source)
        self.indent_size = None  # Store detected indentation size
    
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
            
            if not next_line:
                break
            
            if next_line.strip() == 'end module':
                break
            
            if next_line.strip().startswith('Export:'):
                self.consume_line()
                export_name = next_line.strip()[7:].strip()
                export_name = self.extract_string_literal(export_name) or export_name
                exports.append(export_name)
            else:
                self.consume_line()
        
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
            
            if not next_line:
                break
            
            if next_line.strip() == 'end data':
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
            else:
                self.consume_line()
        
        return DataDefinition(name, fields)
    
    
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
            
            if not next_line:
                break
            
            if next_line.strip() == 'end form':
                break
            
            self.consume_line()
            element_line = next_line.strip()
            
            # Skip Name line
            if element_line.startswith('Name:'):
                continue
            
            # Parse form element (UI component)
            element = self.parse_component(element_line)
            if element:
                elements.append(element)
        
        return FormDefinition(name, elements)  # elements are the children
    
    
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
    
    def parse_module_spec_syntax(self, line: str) -> ModuleDefinition:
        """Parse module with spec syntax: 'module ModuleName'"""
        # Extract module name
        parts = line.strip().split()
        if len(parts) < 2:
            raise ParseError(f"Invalid module definition: {line}")
        
        module_name = parts[1]
        body = []
        
        
        # Detect indentation level from the first indented line (store as instance variable)
        self.indent_size = None
        
        # Parse module body until 'end module' - ignore indentation, rely on explicit end markers
        while self.current_line < len(self.lines):
            next_line = self.peek_line()
            
            if next_line is None:
                break
                
            if next_line.strip() == 'end module':
                self.consume_line()  # consume 'end module'
                break
                
            # Skip empty lines
            if not next_line.strip():
                self.consume_line()
                continue
                
            # Parse any non-empty line that's not the end marker
            self.consume_line()
            content = next_line.strip()
            
            
            if content.startswith('data '):
                stmt = self.parse_data_spec_syntax(content)
                if stmt:
                    body.append(stmt)
            elif content.startswith('action '):
                stmt = self.parse_action_spec_syntax(content)
                if stmt:
                    body.append(stmt)
            elif content.startswith('fragment '):
                stmt = self.parse_fragment_spec_syntax(content)
                if stmt:
                    body.append(stmt)
            elif content.startswith('screen '):
                stmt = self.parse_screen_spec_syntax(content)
                if stmt:
                    body.append(stmt)
            else:
                # Parse as regular statement
                stmt = self.parse_statement(content)
                if stmt:
                    body.append(stmt)
        
        
        return ModuleDefinition(module_name, body)
    
    def parse_data_spec_syntax(self, line: str) -> DataDefinition:
        """Parse data with spec syntax: 'data DataName'"""
        # Extract data name
        parts = line.strip().split()
        if len(parts) < 2:
            raise ParseError(f"Invalid data definition: {line}")
        
        data_name = parts[1]
        fields = []
        
        # Parse data fields until 'end data'
        while self.current_line < len(self.lines):
            next_line = self.peek_line()
            
            if not next_line:
                break
                
            if next_line.strip() == 'end data':
                self.consume_line()  # consume 'end data'
                break
                
            # Parse field definitions with 'is' keyword
            if ' is ' in next_line:
                self.consume_line()
                field_line = next_line.strip()
                field = self.parse_data_field(field_line)
                if field:
                    fields.append(field)
            else:
                # Skip non-field lines
                self.consume_line()
        
        return DataDefinition(data_name, fields)
    
    def parse_data_field(self, line: str) -> Optional[DataField]:
        """Parse a data field with spec syntax: 'fieldName is type required unique key auto'"""
        if ' is ' not in line:
            return None
        
        parts = line.split(' is ', 1)
        if len(parts) != 2:
            return None
        
        field_name = parts[0].strip()
        type_and_annotations = parts[1].strip()
        
        # Split type and annotations
        type_parts = type_and_annotations.split()
        field_type = type_parts[0]  # First part is always the type
        
        # Remaining parts are annotations
        annotations = []
        for i in range(1, len(type_parts)):
            annotation = type_parts[i].lower()
            if annotation in ['required', 'unique', 'key', 'auto', 'optional']:
                annotations.append(annotation)
        
        return DataField(field_name, field_type, annotations)
    
    def parse_action_spec_syntax(self, line: str) -> Optional[ActionDefinition]:
        """Parse action with spec syntax: 'action actionName'"""
        # Extract action name and parameters
        parts = line.strip().split()
        if len(parts) < 2:
            raise ParseError(f"Invalid action definition: {line}")
        
        action_name = parts[1]
        body = []
        
        # Skip action body for now (parse until 'end action')
        while self.current_line < len(self.lines):
            next_line = self.peek_line()
            
            if not next_line:
                break
                
            if next_line.strip() == 'end action':
                self.consume_line()  # consume 'end action'
                break
                
            # Parse action body content
            self.consume_line()
            content = next_line.strip()
            stmt = self.parse_statement(content)
            if stmt:
                body.append(stmt)
        
        return ActionDefinition(action_name, body)
    
    def parse_fragment_spec_syntax(self, line: str) -> FragmentDefinition:
        """Parse fragment with spec syntax: 'fragment FragmentName'"""
        # Extract fragment name
        parts = line.strip().split()
        if len(parts) < 2:
            raise ParseError(f"Invalid fragment definition: {line}")
        
        fragment_name = parts[1]
        slots = []
        
        # Parse fragment body until 'end fragment'
        while self.current_line < len(self.lines):
            next_line = self.peek_line()
            
            if next_line is None:
                break
                
            if next_line.strip() == 'end fragment':
                self.consume_line()  # consume 'end fragment'
                break
                
            # Skip empty lines
            if not next_line.strip():
                self.consume_line()
                continue
                
            # Parse any non-empty line that's not the end marker
            self.consume_line()
            content = next_line.strip()
            
            # Parse slot definitions within the fragment
            if content.startswith('slot '):
                slot = self.parse_slot_definition(content)
                if slot:
                    slots.append(slot)
            else:
                # Parse as regular statement/component for default content
                stmt = self.parse_statement(content)
                if stmt:
                    # Add to first slot as default content if no slots defined yet
                    if not slots:
                        default_slot = SlotComponent("default", [stmt])
                        slots.append(default_slot)
                    else:
                        slots[-1].default_content.append(stmt)
        
        return FragmentDefinition(fragment_name, slots)
    
    def parse_slot_definition(self, line: str) -> SlotComponent:
        """Parse a slot definition within a fragment"""
        # Extract slot name and attributes
        match = re.match(r'slot\s+["\']([^"\']*)["\'](.*)' , line)
        if not match:
            # Try without quotes
            match = re.match(r'slot\s+(\w+)(.*)', line)
            
        if not match:
            raise ParseError(f"Invalid slot definition: {line}")
        
        slot_name = match.group(1)
        attributes_str = match.group(2).strip() if match.group(2) else ""
        
        # Parse classes and styles from attributes
        classes = []
        styles = None
        
        if 'classes ' in attributes_str:
            class_match = re.search(r'classes\s+["\']([^"\']*)["\']', attributes_str)
            if class_match:
                classes = [cls.strip() for cls in class_match.group(1).split(',')]
        
        if 'styles ' in attributes_str:
            style_match = re.search(r'styles\s+["\']([^"\']*)["\']', attributes_str)
            if style_match:
                styles = style_match.group(1)
        
        # Parse slot content if any
        default_content = []
        
        return SlotComponent(slot_name, default_content, [], classes, styles)
    
    
    def parse_screen_spec_syntax(self, line: str) -> ScreenDefinition:
        """Parse screen with spec syntax: 'screen ScreenName'"""
        # Extract screen name
        parts = line.strip().split()
        if len(parts) < 2:
            raise ParseError(f"Invalid screen definition: {line}")
        
        screen_name = parts[1]
        fragments = []
        
        # Parse screen body until 'end screen'
        end_marker = 'end screen'
        while self.current_line < len(self.lines):
            next_line = self.peek_line()
            
            if next_line is None:
                break
                
            if next_line.strip() == end_marker:
                self.consume_line()  # consume end marker
                break
                
            # Skip empty lines
            if not next_line.strip():
                self.consume_line()
                continue
                
            # Parse any non-empty line that's not the end marker
            self.consume_line()
            content = next_line.strip()
            
            # Parse fragment references
            if content.startswith('fragment '):
                fragment_ref = self.parse_fragment_reference(content)
                if fragment_ref:
                    fragments.append(fragment_ref)
            elif content.startswith('form '):
                stmt = self.parse_form_spec_syntax(content)
                if stmt:
                    # Wrap form in a default fragment reference
                    default_fragment = FragmentReference("form_fragment", {"default": [stmt]})
                    fragments.append(default_fragment)
            else:
                # Parse UI components or other statements
                component_stmt = self.parse_component(content)
                if component_stmt:
                    # Wrap component in a default fragment reference
                    default_fragment = FragmentReference("default_fragment", {"default": [component_stmt]})
                    fragments.append(default_fragment)
        
        return ScreenDefinition(screen_name, fragments)
    
    def parse_fragment_reference(self, line: str) -> FragmentReference:
        """Parse a fragment reference within a screen"""
        # Extract fragment name
        parts = line.strip().split()
        if len(parts) < 2:
            raise ParseError(f"Invalid fragment reference: {line}")
        
        fragment_name = parts[1]
        slot_contents = {}
        
        # Parse slot content assignments until 'end fragment'
        while self.current_line < len(self.lines):
            next_line = self.peek_line()
            
            if next_line is None:
                break
                
            if next_line.strip() == 'end fragment':
                self.consume_line()  # consume 'end fragment'
                break
                
            # Skip empty lines
            if not next_line.strip():
                self.consume_line()
                continue
                
            # Parse slot content assignments
            self.consume_line()
            content = next_line.strip()
            
            # Check if we're defining slot content
            if content.startswith('slot ') and ':' in content:
                # Parse slot content definition like 'slot "header":'
                slot_match = re.match(r'slot\s+["\'](.*?)["\']\s*:', content)
                if slot_match:
                    slot_name = slot_match.group(1)
                    slot_content = self.parse_slot_content_block()
                    slot_contents[slot_name] = slot_content
        
        return FragmentReference(fragment_name, slot_contents)
    
    def parse_slot_content_block(self) -> List:
        """Parse content for a specific slot until 'end slot'"""
        content = []
        
        while self.current_line < len(self.lines):
            next_line = self.peek_line()
            
            if next_line is None:
                break
                
            if next_line.strip() == 'end slot':
                self.consume_line()  # consume 'end slot'
                break
                
            # Skip empty lines
            if not next_line.strip():
                self.consume_line()
                continue
                
            # Parse content line
            self.consume_line()
            content_line = next_line.strip()
            
            
            # Parse UI components or other statements
            component_stmt = self.parse_component(content_line)
            if component_stmt:
                content.append(component_stmt)
            else:
                # Parse as regular statement
                stmt = self.parse_statement(content_line)
                if stmt:
                    content.append(stmt)
        
        return content
    
    def parse_form_spec_syntax(self, line: str) -> FormDefinition:
        """Parse form with spec syntax: 'form "FormName"'"""
        # Extract form name
        match = re.match(r'form\s+"([^"]+)"', line)
        if not match:
            match = re.match(r"form\s+'([^']+)'", line)
        
        if not match:
            # Try without quotes
            parts = line.strip().split()
            if len(parts) >= 2:
                form_name = parts[1]
            else:
                raise ParseError(f"Invalid form definition: {line}")
        else:
            form_name = match.group(1)
        
        children = []
        
        # Parse form body until 'end form'
        while self.current_line < len(self.lines):
            next_line = self.peek_line()
            
            if not next_line:
                break
                
            if next_line.strip() == 'end form':
                self.consume_line()  # consume 'end form'
                break
                
            # Skip empty lines
            if not next_line.strip():
                self.consume_line()
                continue
                
            # Parse form elements
            self.consume_line()
            content = next_line.strip()
            
            # Parse UI components within the form
            component_stmt = self.parse_component(content)
            if component_stmt:
                children.append(component_stmt)
            else:
                # Parse as regular statement
                stmt = self.parse_statement(content)
                if stmt:
                    children.append(stmt)
        
        return FormDefinition(form_name, children)