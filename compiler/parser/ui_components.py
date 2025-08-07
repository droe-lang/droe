"""UI components parsing - fixed to match AST definitions."""

import re
from typing import List, Optional, Dict, Any
from ..ast import (
    ASTNode, TitleComponent, InputComponent, TextareaComponent, 
    DropdownComponent, ToggleComponent, CheckboxComponent, RadioComponent,
    ButtonComponent, ImageComponent, VideoComponent, AudioComponent,
    AssetInclude, AttributeDefinition, ValidationAttribute, 
    BindingAttribute, ActionAttribute, Literal
)
from .base import BaseParser, ParseError


class UIComponentParser(BaseParser):
    """Parser for UI components that work across web and mobile - fixed version."""
    
    # Platform-specific component mappings
    MOBILE_COMPONENTS = {
        'Camera': 'camera',
        'Location': 'location', 
        'Notification': 'notification',
        'Storage': 'storage',
        'Sensor': 'sensor',
        'Contact': 'contact'
    }
    
    def parse_component(self, line: str) -> Optional[ASTNode]:
        """Parse any UI component from a line."""
        line = line.strip()
        
        # Standard components
        if line.startswith('Title:'):
            return self.parse_title_component(line)
        elif line.startswith('Input:'):
            return self.parse_input_component(line)
        elif line.startswith('Textarea:'):
            return self.parse_textarea_component(line)
        elif line.startswith('Dropdown:'):
            return self.parse_dropdown_component(line)
        elif line.startswith('Toggle:'):
            return self.parse_toggle_component(line)
        elif line.startswith('Checkbox:'):
            return self.parse_checkbox_component(line)
        elif line.startswith('Radio:'):
            return self.parse_radio_component(line)
        elif line.startswith('Button:'):
            return self.parse_button_component(line)
        elif line.startswith('Image:'):
            return self.parse_image_component(line)
        elif line.startswith('Video:'):
            return self.parse_video_component(line)
        elif line.startswith('Audio:'):
            return self.parse_audio_component(line)
        elif line.startswith('Asset:'):
            return self.parse_asset_include(line)
        
        # Mobile-specific components
        for component_name in self.MOBILE_COMPONENTS:
            if line.startswith(f'{component_name}:'):
                return self.parse_mobile_component(line, component_name)
        
        return None
    
    def parse_mobile_component(self, line: str, component_type: str) -> ASTNode:
        """Parse mobile-specific components like Camera, Location, etc."""
        content = line[len(component_type) + 1:].strip()
        attributes = []
        
        # Parse attributes if present
        if '[' in content and ']' in content:
            attr_start = content.index('[')
            attr_end = content.index(']')
            attr_text = content[attr_start + 1:attr_end]
            attributes = self.parse_attributes(attr_text)
            content = content[:attr_start].strip()
        
        # Add a special attribute to identify this as a mobile component
        attributes.append(AttributeDefinition('mobile_component', component_type.lower()))
        
        # Extract text from content
        text = self.extract_string_literal(content) or content or f"Access {component_type}"
        
        # Extract action from attributes
        action = None
        for attr in attributes:
            if hasattr(attr, '__class__') and attr.__class__.__name__ == 'ActionAttribute':
                action = attr.action_name
                break
        
        # Create a button component that will be transformed by mobile generators
        return ButtonComponent(text=text, action=action, attributes=attributes)
    
    def parse_title_component(self, line: str) -> TitleComponent:
        """Parse Title component."""
        content = line[6:].strip()  # Remove "Title:"
        
        # Check for attributes
        attributes = []
        if '[' in content and ']' in content:
            attr_start = content.index('[')
            attr_end = content.index(']')
            attr_text = content[attr_start + 1:attr_end]
            attributes = self.parse_attributes(attr_text)
            content = content[:attr_start].strip()
        
        # Remove quotes if present
        text = self.extract_string_literal(content) or content
        
        return TitleComponent(text=text, attributes=attributes)
    
    def parse_input_component(self, line: str) -> InputComponent:
        """Parse Input component with attributes - fixed to match AST."""
        content = line[6:].strip()  # Remove "Input:"
        
        # Parse component parts
        input_type = "text"
        binding = None
        attributes = []
        element_id = None
        
        # Check for attributes in square brackets
        if '[' in content and ']' in content:
            attr_start = content.index('[')
            attr_end = content.index(']')
            attr_text = content[attr_start + 1:attr_end]
            attributes = self.parse_attributes(attr_text)
            content = content[:attr_start].strip()
        
        # Parse placeholder and add it as an attribute
        placeholder = self.extract_string_literal(content) or content
        if placeholder:
            attributes.insert(0, AttributeDefinition('placeholder', placeholder))
        
        # Extract specific values from attributes
        for attr in attributes:
            if attr.name == 'type':
                input_type = attr.value
            elif attr.name == 'id':
                element_id = attr.value
            elif hasattr(attr, '__class__') and attr.__class__.__name__ == 'BindingAttribute':
                binding = attr.binding_target
        
        return InputComponent(
            input_type=input_type,
            binding=binding,
            attributes=attributes,
            element_id=element_id
        )
    
    def parse_textarea_component(self, line: str) -> TextareaComponent:
        """Parse Textarea component - fixed to match AST."""
        content = line[9:].strip()  # Remove "Textarea:"
        
        binding = None
        attributes = []
        element_id = None
        
        if '[' in content and ']' in content:
            attr_start = content.index('[')
            attr_end = content.index(']')
            attr_text = content[attr_start + 1:attr_end]
            attributes = self.parse_attributes(attr_text)
            content = content[:attr_start].strip()
        
        # Parse placeholder and add as attribute
        placeholder = self.extract_string_literal(content) or content
        if placeholder:
            attributes.insert(0, AttributeDefinition('placeholder', placeholder))
        
        # Also add rows attribute with default value
        attributes.append(AttributeDefinition('rows', '4'))
        
        # Extract specific values
        for attr in attributes:
            if attr.name == 'id':
                element_id = attr.value
            elif hasattr(attr, '__class__') and attr.__class__.__name__ == 'BindingAttribute':
                binding = attr.binding_target
        
        return TextareaComponent(
            binding=binding,
            attributes=attributes,
            element_id=element_id
        )
    
    def parse_dropdown_component(self, line: str) -> DropdownComponent:
        """Parse Dropdown component - fixed to match AST."""
        content = line[9:].strip()  # Remove "Dropdown:"
        
        options = []
        binding = None
        attributes = []
        element_id = None
        
        if '[' in content and ']' in content:
            # Find the last set of brackets (attributes)
            attr_start = content.rfind('[')
            attr_end = content.rfind(']')
            
            # Check if these are attributes (contain '=' or known attribute names)
            potential_attrs = content[attr_start + 1:attr_end]
            if '=' in potential_attrs or any(x in potential_attrs for x in ['bind:', 'validate:', 'action:']):
                attr_text = potential_attrs
                attributes = self.parse_attributes(attr_text)
                content = content[:attr_start].strip()
        
        # Parse options array and default value
        default_value = None
        if content.startswith('[') and ']' in content:
            options_end = content.index(']')
            options_str = content[1:options_end]
            
            for opt in options_str.split(','):
                opt = opt.strip()
                opt_val = self.extract_string_literal(opt) or opt
                if opt_val:
                    options.append(Literal(opt_val, 'string'))
            
            # Check for default value after options
            remaining = content[options_end + 1:].strip()
            if remaining.startswith('default:'):
                default_val = remaining[8:].strip()
                default_val = self.extract_string_literal(default_val) or default_val
                if default_val:
                    attributes.append(AttributeDefinition('default', default_val))
        
        # Extract specific values
        for attr in attributes:
            if attr.name == 'id':
                element_id = attr.value
            elif hasattr(attr, '__class__') and attr.__class__.__name__ == 'BindingAttribute':
                binding = attr.binding_target
        
        return DropdownComponent(
            options=options,
            binding=binding,
            attributes=attributes,
            element_id=element_id
        )
    
    def parse_toggle_component(self, line: str) -> ToggleComponent:
        """Parse Toggle component - fixed to match AST."""
        content = line[7:].strip()  # Remove "Toggle:"
        
        binding = None
        attributes = []
        element_id = None
        
        if '[' in content and ']' in content:
            attr_start = content.index('[')
            attr_end = content.index(']')
            attr_text = content[attr_start + 1:attr_end]
            attributes = self.parse_attributes(attr_text)
            content = content[:attr_start].strip()
        
        # Parse label and add as attribute
        label = self.extract_string_literal(content) or content
        if label:
            attributes.insert(0, AttributeDefinition('label', label))
        
        # Extract specific values
        for attr in attributes:
            if attr.name == 'id':
                element_id = attr.value
            elif hasattr(attr, '__class__') and attr.__class__.__name__ == 'BindingAttribute':
                binding = attr.binding_target
        
        return ToggleComponent(
            binding=binding,
            attributes=attributes,
            element_id=element_id
        )
    
    def parse_checkbox_component(self, line: str) -> CheckboxComponent:
        """Parse Checkbox component - fixed to match AST."""
        content = line[9:].strip()  # Remove "Checkbox:"
        
        text = None
        binding = None
        attributes = []
        element_id = None
        
        if '[' in content and ']' in content:
            # Find the last set of brackets
            attr_start = content.rfind('[')
            attr_end = content.rfind(']')
            
            potential_attrs = content[attr_start + 1:attr_end]
            if '=' in potential_attrs or any(x in potential_attrs for x in ['bind:', 'validate:', 'action:']):
                attr_text = potential_attrs
                attributes = self.parse_attributes(attr_text)
                content = content[:attr_start].strip()
        
        # Parse options array as text
        if content.startswith('[') and ']' in content:
            options_end = content.index(']')
            options_str = content[1:options_end]
            
            # Store options as attribute for generators to use
            options = []
            for opt in options_str.split(','):
                opt = opt.strip()
                opt = self.extract_string_literal(opt) or opt
                if opt:
                    options.append(opt)
            
            if options:
                # Use first option as text, store all as attribute
                text = options[0] if len(options) == 1 else None
                attributes.insert(0, AttributeDefinition('options', ','.join(options)))
        else:
            # Single checkbox with text
            text = self.extract_string_literal(content) or content
        
        # Extract specific values
        for attr in attributes:
            if attr.name == 'id':
                element_id = attr.value
            elif hasattr(attr, '__class__') and attr.__class__.__name__ == 'BindingAttribute':
                binding = attr.binding_target
        
        return CheckboxComponent(
            text=text,
            binding=binding,
            attributes=attributes,
            element_id=element_id
        )
    
    def parse_radio_component(self, line: str) -> RadioComponent:
        """Parse Radio component - fixed to match AST."""
        content = line[6:].strip()  # Remove "Radio:"
        
        text = None
        value = None
        binding = None
        attributes = []
        element_id = None
        
        # Parse name/text
        if content.startswith('"') or content.startswith("'"):
            quote_char = content[0]
            end_quote = content.index(quote_char, 1)
            text = content[1:end_quote]
            content = content[end_quote + 1:].strip()
        
        if '[' in content and ']' in content:
            # Find the last set of brackets
            attr_start = content.rfind('[')
            attr_end = content.rfind(']')
            
            potential_attrs = content[attr_start + 1:attr_end]
            if '=' in potential_attrs or any(x in potential_attrs for x in ['bind:', 'validate:', 'action:']):
                attr_text = potential_attrs
                attributes = self.parse_attributes(attr_text)
                content = content[:attr_start].strip()
        
        # Parse options
        if content.startswith('[') and ']' in content:
            options_end = content.index(']')
            options_str = content[1:options_end]
            
            options = []
            for opt in options_str.split(','):
                opt = opt.strip()
                opt = self.extract_string_literal(opt) or opt
                if opt:
                    options.append(opt)
            
            # Store options and name as attributes
            if options:
                attributes.insert(0, AttributeDefinition('options', ','.join(options)))
            if text:
                attributes.insert(0, AttributeDefinition('name', text))
            
            # Check for default value
            remaining = content[options_end + 1:].strip()
            if remaining.startswith('default:'):
                default_value = remaining[8:].strip()
                default_value = self.extract_string_literal(default_value) or default_value
                if default_value:
                    attributes.append(AttributeDefinition('default', default_value))
                    value = default_value
        
        # Extract specific values
        for attr in attributes:
            if attr.name == 'id':
                element_id = attr.value
            elif hasattr(attr, '__class__') and attr.__class__.__name__ == 'BindingAttribute':
                binding = attr.binding_target
        
        return RadioComponent(
            text=text,
            value=value,
            binding=binding,
            attributes=attributes,
            element_id=element_id
        )
    
    def parse_button_component(self, line: str) -> ButtonComponent:
        """Parse Button component - fixed to match AST."""
        content = line[7:].strip()  # Remove "Button:"
        
        text = ""
        action = None
        attributes = []
        
        if '[' in content and ']' in content:
            attr_start = content.index('[')
            attr_end = content.index(']')
            attr_text = content[attr_start + 1:attr_end]
            attributes = self.parse_attributes(attr_text)
            content = content[:attr_start].strip()
        
        text = self.extract_string_literal(content) or content
        
        # Extract action from attributes
        for attr in attributes:
            if hasattr(attr, '__class__') and attr.__class__.__name__ == 'ActionAttribute':
                action = attr.action_name
                break
        
        return ButtonComponent(text=text, action=action, attributes=attributes)
    
    def parse_image_component(self, line: str) -> ImageComponent:
        """Parse Image component - fixed to match AST."""
        content = line[6:].strip()  # Remove "Image:"
        
        src = ""
        alt = None
        attributes = []
        
        # Parse src
        if content.startswith('"') or content.startswith("'"):
            quote_char = content[0]
            end_quote = content.index(quote_char, 1)
            src = content[1:end_quote]
            content = content[end_quote + 1:].strip()
        
        # Parse alt text
        if content and (content.startswith('"') or content.startswith("'")):
            quote_char = content[0]
            end_quote = content.index(quote_char, 1)
            alt = content[1:end_quote]
            content = content[end_quote + 1:].strip()
        
        # Parse attributes
        if '[' in content and ']' in content:
            attr_start = content.index('[')
            attr_end = content.index(']')
            attr_text = content[attr_start + 1:attr_end]
            attributes = self.parse_attributes(attr_text)
        
        return ImageComponent(src=src, alt=alt, attributes=attributes)
    
    def parse_video_component(self, line: str) -> VideoComponent:
        """Parse Video component - fixed to match AST."""
        content = line[6:].strip()  # Remove "Video:"
        
        src = ""
        controls = True
        autoplay = False
        loop = False
        muted = False
        attributes = []
        
        # Parse src
        if content.startswith('"') or content.startswith("'"):
            quote_char = content[0]
            end_quote = content.index(quote_char, 1)
            src = content[1:end_quote]
            content = content[end_quote + 1:].strip()
        
        # Parse attributes
        if '[' in content and ']' in content:
            attr_start = content.index('[')
            attr_end = content.index(']')
            attr_text = content[attr_start + 1:attr_end]
            attributes = self.parse_attributes(attr_text)
            
            # Extract video-specific attributes
            for attr in attributes:
                if attr.name == 'controls':
                    controls = attr.value.lower() != 'false'
                elif attr.name == 'autoplay':
                    autoplay = attr.value.lower() == 'true'
                elif attr.name == 'loop':
                    loop = attr.value.lower() == 'true'
                elif attr.name == 'muted':
                    muted = attr.value.lower() == 'true'
        
        return VideoComponent(
            src=src,
            controls=controls,
            autoplay=autoplay,
            loop=loop,
            muted=muted,
            attributes=attributes
        )
    
    def parse_audio_component(self, line: str) -> AudioComponent:
        """Parse Audio component - fixed to match AST."""
        content = line[6:].strip()  # Remove "Audio:"
        
        src = ""
        controls = True
        autoplay = False
        loop = False
        attributes = []
        
        # Parse src
        if content.startswith('"') or content.startswith("'"):
            quote_char = content[0]
            end_quote = content.index(quote_char, 1)
            src = content[1:end_quote]
            content = content[end_quote + 1:].strip()
        
        # Parse attributes
        if '[' in content and ']' in content:
            attr_start = content.index('[')
            attr_end = content.index(']')
            attr_text = content[attr_start + 1:attr_end]
            attributes = self.parse_attributes(attr_text)
            
            # Extract audio-specific attributes
            for attr in attributes:
                if attr.name == 'controls':
                    controls = attr.value.lower() != 'false'
                elif attr.name == 'autoplay':
                    autoplay = attr.value.lower() == 'true'
                elif attr.name == 'loop':
                    loop = attr.value.lower() == 'true'
        
        return AudioComponent(
            src=src,
            controls=controls,
            autoplay=autoplay,
            loop=loop,
            attributes=attributes
        )
    
    def parse_asset_include(self, line: str) -> AssetInclude:
        """Parse Asset include statement."""
        content = line[6:].strip()  # Remove "Asset:"
        
        # Parse asset type and path
        parts = content.split(maxsplit=1)
        if len(parts) != 2:
            raise ParseError(f"Invalid Asset statement: {line}")
        
        asset_type = parts[0]
        path = parts[1].strip()
        
        # Remove quotes from path
        path = self.extract_string_literal(path) or path
        
        return AssetInclude(asset_path=path, asset_type=asset_type)
    
    def parse_attributes(self, attr_text: str) -> List[AttributeDefinition]:
        """Parse component attributes from bracket notation."""
        attributes = []
        
        # Split by comma, but respect nested structures
        parts = []
        current = ""
        depth = 0
        in_string = False
        string_char = None
        
        for char in attr_text:
            if not in_string and (char == '"' or char == "'"):
                in_string = True
                string_char = char
            elif in_string and char == string_char:
                in_string = False
                string_char = None
            elif not in_string:
                if char in '([{':
                    depth += 1
                elif char in ')]}':
                    depth -= 1
                elif char == ',' and depth == 0:
                    parts.append(current.strip())
                    current = ""
                    continue
            
            current += char
        
        if current.strip():
            parts.append(current.strip())
        
        # Parse each attribute
        for part in parts:
            part = part.strip()
            
            # Validation attributes
            if part.startswith('validate:'):
                validate_content = part[9:].strip()
                attributes.append(ValidationAttribute(validate_content))
            
            # Binding attributes
            elif part.startswith('bind:'):
                bind_target = part[5:].strip()
                attributes.append(BindingAttribute(bind_target))
            
            # Action attributes
            elif part.startswith('action:'):
                action_content = part[7:].strip()
                attributes.append(ActionAttribute(action_content))
            
            # Regular key=value attributes
            elif '=' in part:
                key, value = part.split('=', 1)
                key = key.strip()
                value = value.strip()
                # Remove quotes from value if present
                value = self.extract_string_literal(value) or value
                attributes.append(AttributeDefinition(key, value))
            
            # Simple flag attributes
            else:
                attributes.append(AttributeDefinition(part, 'true'))
        
        return attributes