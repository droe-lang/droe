"""UI components parsing - fixed to match AST definitions."""

import re
from typing import List, Optional, Dict, Any
from ..ast import (
    ASTNode, TitleComponent, InputComponent, TextareaComponent, 
    DropdownComponent, ToggleComponent, CheckboxComponent, RadioComponent,
    ButtonComponent, ImageComponent, VideoComponent, AudioComponent,
    AssetInclude, AttributeDefinition, ValidationAttribute, 
    BindingAttribute, ActionAttribute, Literal, SlotComponent
)
from .base import BaseParser, ParseError


class UIComponentParser(BaseParser):
    """Parser for UI components that work across web and mobile - fixed version."""
    
    # Platform-specific component mappings
    MOBILE_COMPONENTS = {
        'camera': 'camera',
        'location': 'location', 
        'notification': 'notification',
        'storage': 'storage',
        'sensor': 'sensor',
        'contact': 'contact'
    }
    
    def _parse_tokens(self, content: str) -> List[str]:
        """Parse content into tokens, respecting quoted strings."""
        tokens = []
        current = ""
        in_quotes = False
        quote_char = None
        
        for char in content:
            if not in_quotes and (char == '"' or char == "'"):
                in_quotes = True
                quote_char = char
                current += char
            elif in_quotes and char == quote_char:
                in_quotes = False
                quote_char = None
                current += char
            elif not in_quotes and char == ' ':
                if current.strip():
                    tokens.append(current.strip())
                current = ""
            else:
                current += char
        
        if current.strip():
            tokens.append(current.strip())
        
        return tokens
    
    def _unquote(self, text: str) -> str:
        """Remove quotes from a string if present."""
        if (text.startswith('"') and text.endswith('"')) or \
           (text.startswith("'") and text.endswith("'")):
            return text[1:-1]
        return text
    
    def parse_component(self, line: str) -> Optional[ASTNode]:
        """Parse any UI component from a line."""
        line = line.strip()
        
        # New syntax (lowercase without colons) - preferred
        if line.startswith('title '):
            return self.parse_title_spec_syntax(line)
        elif line.startswith('text '):
            return self.parse_text_spec_syntax(line)
        elif line.startswith('input '):
            return self.parse_input_spec_syntax(line)
        elif line.startswith('textarea '):
            return self.parse_textarea_spec_syntax(line)
        elif line.startswith('dropdown '):
            return self.parse_dropdown_spec_syntax(line)
        elif line.startswith('toggle '):
            return self.parse_toggle_spec_syntax(line)
        elif line.startswith('checkbox '):
            return self.parse_checkbox_spec_syntax(line)
        elif line.startswith('radio '):
            return self.parse_radio_spec_syntax(line)
        elif line.startswith('button '):
            return self.parse_button_spec_syntax(line)
        elif line.startswith('image '):
            return self.parse_image_spec_syntax(line)
        elif line.startswith('video '):
            return self.parse_video_spec_syntax(line)
        elif line.startswith('audio '):
            return self.parse_audio_spec_syntax(line)
        elif line.startswith('slot '):
            return self.parse_slot_spec_syntax(line)
        
        
        return None
    
    def parse_title_spec_syntax(self, line: str) -> TitleComponent:
        """Parse title with new spec syntax: 'title "text" class "class-name"'"""
        # Extract text and attributes
        content = line[6:].strip()  # Remove "title "
        
        # Parse the title text (first quoted string)
        if content.startswith('"'):
            text_end = content.find('"', 1)
            if text_end != -1:
                text = content[1:text_end]
                remaining = content[text_end + 1:].strip()
            else:
                text = content[1:]  # Rest of line if no closing quote
                remaining = ""
        else:
            # Find first space or end of line for unquoted text
            space_pos = content.find(' ')
            if space_pos != -1:
                text = content[:space_pos]
                remaining = content[space_pos:].strip()
            else:
                text = content
                remaining = ""
        
        # Parse CSS classes and other attributes from remaining content
        css_classes = []
        attributes = []
        
        if 'classes ' in remaining:
            # Extract classes attribute
            class_start = remaining.find('classes ') + 8
            class_content = remaining[class_start:].strip()
            if class_content.startswith('"'):
                class_end = class_content.find('"', 1)
                if class_end != -1:
                    class_value = class_content[1:class_end]
                    css_classes = [cls.strip() for cls in class_value.split(',')]
        
        # Parse styles if present
        styles = None
        if 'styles ' in remaining:
            style_start = remaining.find('styles ') + 7
            style_content = remaining[style_start:].strip()
            if style_content.startswith('"'):
                style_end = style_content.find('"', 1)
                if style_end != -1:
                    styles = style_content[1:style_end]
        
        # Create TitleComponent with text, CSS classes, and styles
        title_comp = TitleComponent(text)
        title_comp.classes = css_classes
        title_comp.styles = styles
        return title_comp
    
    def parse_text_spec_syntax(self, line: str) -> 'TextComponent':
        """Parse text with new spec syntax: 'text "content" class "class-name"'"""
        # Extract text and attributes
        content = line[5:].strip()  # Remove "text "
        
        # Parse the text content (first quoted string)
        if content.startswith('"'):
            text_end = content.find('"', 1)
            if text_end != -1:
                text = content[1:text_end]
                remaining = content[text_end + 1:].strip()
            else:
                text = content[1:]  # Rest of line if no closing quote
                remaining = ""
        else:
            # Find first space or end of line for unquoted text
            space_pos = content.find(' ')
            if space_pos != -1:
                text = content[:space_pos]
                remaining = content[space_pos:].strip()
            else:
                text = content
                remaining = ""
        
        # Parse CSS classes and other attributes from remaining content
        css_classes = []
        attributes = []
        
        if 'classes ' in remaining:
            # Extract classes attribute
            class_start = remaining.find('classes ') + 8
            class_content = remaining[class_start:].strip()
            if class_content.startswith('"'):
                class_end = class_content.find('"', 1)
                if class_end != -1:
                    class_value = class_content[1:class_end]
                    css_classes = [cls.strip() for cls in class_value.split(',')]
        
        # Parse styles if present
        styles = None
        if 'styles ' in remaining:
            style_start = remaining.find('styles ') + 7
            style_content = remaining[style_start:].strip()
            if style_content.startswith('"'):
                style_end = style_content.find('"', 1)
                if style_end != -1:
                    styles = style_content[1:style_end]
        
        # Import TextComponent here to avoid circular imports
        from ..ast import TextComponent
        
        # Create TextComponent with text, CSS classes, and styles
        text_comp = TextComponent(text)
        text_comp.classes = css_classes
        text_comp.styles = styles
        return text_comp
    
    def parse_input_spec_syntax(self, line: str) -> Optional[InputComponent]:
        """Parse input with new spec syntax: 'input id name type placeholder bind validate class'"""
        content = line[6:].strip()  # Remove "input "
        
        # Parse components
        input_type = "text"
        binding = None
        attributes = []
        element_id = None
        placeholder = None
        
        # Split content into tokens while respecting quotes
        tokens = self._parse_tokens(content)
        
        i = 0
        while i < len(tokens):
            token = tokens[i]
            
            if token == "id" and i + 1 < len(tokens):
                element_id = tokens[i + 1]
                i += 2
            elif token == "type" and i + 1 < len(tokens):
                input_type = tokens[i + 1]
                i += 2
            elif token == "placeholder" and i + 1 < len(tokens):
                placeholder = self._unquote(tokens[i + 1])
                i += 2
            elif token == "bind" and i + 1 < len(tokens):
                binding = tokens[i + 1]
                attributes.append(BindingAttribute(binding))
                i += 2
            elif token == "validate" and i + 1 < len(tokens):
                validate_value = tokens[i + 1]
                attributes.append(ValidationAttribute(validate_value))
                i += 2
            elif token == "class" and i + 1 < len(tokens):
                class_value = self._unquote(tokens[i + 1])
                attributes.append(AttributeDefinition('class', class_value))
                i += 2
            else:
                i += 1
        
        # Add placeholder as attribute if specified
        if placeholder:
            attributes.insert(0, AttributeDefinition('placeholder', placeholder))
        
        return InputComponent(
            input_type=input_type,
            binding=binding,
            attributes=attributes,
            element_id=element_id
        )
    
    def parse_textarea_spec_syntax(self, line: str) -> Optional[TextareaComponent]:
        """Parse textarea with new spec syntax: 'textarea "label" placeholder="..." rows=4 class="..." id="..."'"""
        content = line[9:].strip()  # Remove "textarea "
        
        binding = None
        attributes = []
        element_id = None
        placeholder = None
        label = None
        rows = None
        
        # Parse tokens
        tokens = self._parse_tokens(content)
        
        # First token could be the label if it's quoted
        i = 0
        if tokens and (tokens[0].startswith('"') or tokens[0].startswith("'")):
            label = self._unquote(tokens[0])
            i = 1
        
        while i < len(tokens):
            token = tokens[i]
            
            if token == "id" and i + 1 < len(tokens):
                element_id = tokens[i + 1]
                i += 2
            elif token == "placeholder" and i + 1 < len(tokens):
                placeholder = self._unquote(tokens[i + 1])
                i += 2
            elif token.startswith("placeholder="):
                placeholder = self._unquote(token[12:])  # Remove "placeholder="
                i += 1
            elif token == "bind" and i + 1 < len(tokens):
                binding = tokens[i + 1]
                attributes.append(BindingAttribute(binding))
                i += 2
            elif token == "rows" and i + 1 < len(tokens):
                rows = int(tokens[i + 1])
                i += 2
            elif token.startswith("rows="):
                rows = int(token[5:])  # Remove "rows="
                i += 1
            elif token == "class" and i + 1 < len(tokens):
                class_value = self._unquote(tokens[i + 1])
                attributes.append(AttributeDefinition('class', class_value))
                i += 2
            elif token.startswith("class="):
                class_value = self._unquote(token[6:])  # Remove "class="
                attributes.append(AttributeDefinition('class', class_value))
                i += 1
            else:
                i += 1
        
        return TextareaComponent(
            label=label,
            placeholder=placeholder,
            rows=rows or 4,
            binding=binding,
            attributes=attributes,
            element_id=element_id
        )
    
    def parse_dropdown_spec_syntax(self, line: str) -> Optional[DropdownComponent]:
        """Parse dropdown with new spec syntax: 'dropdown "label" options=[...] id name bind class'"""
        content = line[9:].strip()  # Remove "dropdown "
        
        options = []
        binding = None
        attributes = []
        element_id = None
        label = None
        
        # Parse tokens
        tokens = self._parse_tokens(content)
        
        # First token could be the label if it's quoted
        i = 0
        if tokens and (tokens[0].startswith('"') or tokens[0].startswith("'")):
            label = self._unquote(tokens[0])
            i = 1
        
        while i < len(tokens):
            token = tokens[i]
            
            if token == "id" and i + 1 < len(tokens):
                element_id = tokens[i + 1]
                i += 2
            elif token == "bind" and i + 1 < len(tokens):
                binding = tokens[i + 1]
                attributes.append(BindingAttribute(binding))
                i += 2
            elif token == "class" and i + 1 < len(tokens):
                class_value = self._unquote(tokens[i + 1])
                attributes.append(AttributeDefinition('class', class_value))
                i += 2
            elif token == "default" and i + 1 < len(tokens):
                default_value = self._unquote(tokens[i + 1])
                attributes.append(AttributeDefinition('default', default_value))
                i += 2
            elif token.startswith("options="):
                # Parse options=[...] syntax - need to collect all tokens for the array
                options_str = token[8:]  # Remove "options="
                
                # Collect remaining tokens that are part of the options array
                while i + 1 < len(tokens) and not options_str.endswith(']'):
                    i += 1
                    options_str += ' ' + tokens[i]
                
                if options_str.startswith('[') and options_str.endswith(']'):
                    options_str = options_str[1:-1]  # Remove brackets
                    # Split by comma and clean up quotes
                    option_items = [item.strip().strip('"\'').rstrip(',') for item in options_str.split(',')]
                    options = [opt for opt in option_items if opt]
                i += 1
            else:
                i += 1
        
        # Note: Options will be parsed from subsequent "option" lines in a multi-line context
        # For now, return the dropdown with empty options that can be populated later
        return DropdownComponent(
            label=label,
            options=options,
            binding=binding,
            attributes=attributes,
            element_id=element_id
        )
    
    def parse_toggle_spec_syntax(self, line: str) -> Optional[ToggleComponent]:
        """Parse toggle with new spec syntax: 'toggle id "label" bind default class'"""
        content = line[7:].strip()  # Remove "toggle "
        
        binding = None
        attributes = []
        element_id = None
        label = None
        
        # Parse tokens
        tokens = self._parse_tokens(content)
        
        i = 0
        while i < len(tokens):
            token = tokens[i]
            
            if token == "id" and i + 1 < len(tokens):
                element_id = tokens[i + 1]
                i += 2
            elif token == "bind" and i + 1 < len(tokens):
                binding = tokens[i + 1]
                attributes.append(BindingAttribute(binding))
                i += 2
            elif token == "default" and i + 1 < len(tokens):
                default_value = tokens[i + 1]
                attributes.append(AttributeDefinition('default', default_value))
                i += 2
            elif token == "class" and i + 1 < len(tokens):
                class_value = self._unquote(tokens[i + 1])
                attributes.append(AttributeDefinition('class', class_value))
                i += 2
            elif (token.startswith('"') or token.startswith("'")) and not label:
                # First quoted string is the label
                label = self._unquote(token)
                i += 1
            else:
                i += 1
        
        # Add label as attribute if specified
        if label:
            attributes.insert(0, AttributeDefinition('label', label))
        
        return ToggleComponent(
            binding=binding,
            attributes=attributes,
            element_id=element_id
        )
    
    def parse_checkbox_spec_syntax(self, line: str) -> Optional[CheckboxComponent]:
        """Parse checkbox with new spec syntax: 'checkbox id "text" bind class'"""
        content = line[9:].strip()  # Remove "checkbox "
        
        text = None
        binding = None
        attributes = []
        element_id = None
        
        # Parse tokens
        tokens = self._parse_tokens(content)
        
        i = 0
        while i < len(tokens):
            token = tokens[i]
            
            if token == "id" and i + 1 < len(tokens):
                element_id = tokens[i + 1]
                i += 2
            elif token == "bind" and i + 1 < len(tokens):
                binding = tokens[i + 1]
                attributes.append(BindingAttribute(binding))
                i += 2
            elif token == "class" and i + 1 < len(tokens):
                class_value = self._unquote(tokens[i + 1])
                attributes.append(AttributeDefinition('class', class_value))
                i += 2
            elif (token.startswith('"') or token.startswith("'")) and not text:
                # First quoted string is the text
                text = self._unquote(token)
                i += 1
            else:
                i += 1
        
        return CheckboxComponent(
            text=text,
            binding=binding,
            attributes=attributes,
            element_id=element_id
        )
    
    def parse_radio_spec_syntax(self, line: str) -> Optional[RadioComponent]:
        """Parse radio with new spec syntax: 'radio id group "groupname" "text" bind default class'"""
        content = line[6:].strip()  # Remove "radio "
        
        text = None
        value = None
        binding = None
        attributes = []
        element_id = None
        group_name = None
        
        # Parse tokens
        tokens = self._parse_tokens(content)
        
        i = 0
        while i < len(tokens):
            token = tokens[i]
            
            if token == "id" and i + 1 < len(tokens):
                element_id = tokens[i + 1]
                i += 2
            elif token == "group" and i + 1 < len(tokens):
                group_name = self._unquote(tokens[i + 1])
                attributes.append(AttributeDefinition('name', group_name))
                i += 2
            elif token == "bind" and i + 1 < len(tokens):
                binding = tokens[i + 1]
                attributes.append(BindingAttribute(binding))
                i += 2
            elif token == "default" and i + 1 < len(tokens):
                default_value = self._unquote(tokens[i + 1])
                attributes.append(AttributeDefinition('default', default_value))
                value = default_value
                i += 2
            elif token == "class" and i + 1 < len(tokens):
                class_value = self._unquote(tokens[i + 1])
                attributes.append(AttributeDefinition('class', class_value))
                i += 2
            elif (token.startswith('"') or token.startswith("'")) and not text:
                # First quoted string is the text
                text = self._unquote(token)
                i += 1
            else:
                i += 1
        
        return RadioComponent(
            text=text,
            value=value,
            binding=binding,
            attributes=attributes,
            element_id=element_id
        )
    
    def parse_button_spec_syntax(self, line: str) -> Optional[ButtonComponent]:
        """Parse button with new spec syntax: 'button "text" action actionName class "class-name"'"""
        content = line[7:].strip()  # Remove "button "
        
        text = ""
        action = None
        attributes = []
        
        # Parse the button text (first quoted string or first token)
        tokens = self._parse_tokens(content)
        
        if tokens and (tokens[0].startswith('"') or tokens[0].startswith("'")):
            text = self._unquote(tokens[0])
            tokens = tokens[1:]
        elif tokens:
            text = tokens[0]
            tokens = tokens[1:]
        
        # Parse remaining tokens
        i = 0
        while i < len(tokens):
            token = tokens[i]
            
            if token == "action" and i + 1 < len(tokens):
                action = tokens[i + 1]
                attributes.append(ActionAttribute(action))
                i += 2
            elif token == "type" and i + 1 < len(tokens):
                button_type = tokens[i + 1]
                attributes.append(AttributeDefinition('type', button_type))
                # Handle mobile component types
                if button_type in ['camera', 'location', 'notification', 'storage', 'sensor', 'contact']:
                    attributes.append(AttributeDefinition('mobile_component', button_type))
                i += 2
            elif token == "class" and i + 1 < len(tokens):
                class_value = self._unquote(tokens[i + 1])
                attributes.append(AttributeDefinition('class', class_value))
                i += 2
            elif token == "id" and i + 1 < len(tokens):
                element_id = tokens[i + 1]
                attributes.append(AttributeDefinition('id', element_id))
                i += 2
            else:
                i += 1
        
        return ButtonComponent(text=text, action=action, attributes=attributes)
    
    def parse_image_spec_syntax(self, line: str) -> Optional[ImageComponent]:
        """Parse image with new spec syntax: 'image source "path" alt "text" class "class-name"'"""
        content = line[6:].strip()  # Remove "image "
        
        src = ""
        alt = None
        attributes = []
        
        # Parse tokens
        tokens = self._parse_tokens(content)
        
        i = 0
        while i < len(tokens):
            token = tokens[i]
            
            if token == "source" and i + 1 < len(tokens):
                src = self._unquote(tokens[i + 1])
                i += 2
            elif token == "alt" and i + 1 < len(tokens):
                alt = self._unquote(tokens[i + 1])
                i += 2
            elif token == "class" and i + 1 < len(tokens):
                class_value = self._unquote(tokens[i + 1])
                attributes.append(AttributeDefinition('class', class_value))
                i += 2
            elif token == "id" and i + 1 < len(tokens):
                element_id = tokens[i + 1]
                attributes.append(AttributeDefinition('id', element_id))
                i += 2
            else:
                i += 1
        
        return ImageComponent(src=src, alt=alt, attributes=attributes)
    
    def parse_video_spec_syntax(self, line: str) -> Optional[VideoComponent]:
        """Parse video with new spec syntax: 'video source "path" controls autoplay loop muted class "class-name"'"""
        content = line[6:].strip()  # Remove "video "
        
        src = ""
        controls = True
        autoplay = False
        loop = False
        muted = False
        attributes = []
        
        # Parse tokens
        tokens = self._parse_tokens(content)
        
        i = 0
        while i < len(tokens):
            token = tokens[i]
            
            if token == "source" and i + 1 < len(tokens):
                src = self._unquote(tokens[i + 1])
                i += 2
            elif token == "controls":
                controls = True
                attributes.append(AttributeDefinition('controls', 'true'))
                i += 1
            elif token == "autoplay":
                autoplay = True
                attributes.append(AttributeDefinition('autoplay', 'true'))
                i += 1
            elif token == "loop":
                loop = True
                attributes.append(AttributeDefinition('loop', 'true'))
                i += 1
            elif token == "muted":
                muted = True
                attributes.append(AttributeDefinition('muted', 'true'))
                i += 1
            elif token == "class" and i + 1 < len(tokens):
                class_value = self._unquote(tokens[i + 1])
                attributes.append(AttributeDefinition('class', class_value))
                i += 2
            elif token == "id" and i + 1 < len(tokens):
                element_id = tokens[i + 1]
                attributes.append(AttributeDefinition('id', element_id))
                i += 2
            else:
                i += 1
        
        return VideoComponent(
            src=src,
            controls=controls,
            autoplay=autoplay,
            loop=loop,
            muted=muted,
            attributes=attributes
        )
    
    def parse_audio_spec_syntax(self, line: str) -> Optional[AudioComponent]:
        """Parse audio with new spec syntax: 'audio source "path" controls autoplay loop class "class-name"'"""
        content = line[6:].strip()  # Remove "audio "
        
        src = ""
        controls = True
        autoplay = False
        loop = False
        attributes = []
        
        # Parse tokens
        tokens = self._parse_tokens(content)
        
        i = 0
        while i < len(tokens):
            token = tokens[i]
            
            if token == "source" and i + 1 < len(tokens):
                src = self._unquote(tokens[i + 1])
                i += 2
            elif token == "controls":
                controls = True
                attributes.append(AttributeDefinition('controls', 'true'))
                i += 1
            elif token == "autoplay":
                autoplay = True
                attributes.append(AttributeDefinition('autoplay', 'true'))
                i += 1
            elif token == "loop":
                loop = True
                attributes.append(AttributeDefinition('loop', 'true'))
                i += 1
            elif token == "class" and i + 1 < len(tokens):
                class_value = self._unquote(tokens[i + 1])
                attributes.append(AttributeDefinition('class', class_value))
                i += 2
            elif token == "id" and i + 1 < len(tokens):
                element_id = tokens[i + 1]
                attributes.append(AttributeDefinition('id', element_id))
                i += 2
            else:
                i += 1
        
        return AudioComponent(
            src=src,
            controls=controls,
            autoplay=autoplay,
            loop=loop,
            attributes=attributes
        )
    
    def parse_slot_spec_syntax(self, line: str) -> Optional[SlotComponent]:
        """Parse slot with spec syntax: 'slot "name"'"""
        content = line[5:].strip()  # Remove "slot "
        
        # Parse the slot name (first quoted string)
        slot_name = "content"  # Default slot name
        attributes = []
        css_classes = []
        
        # Parse tokens
        tokens = self._parse_tokens(content)
        
        i = 0
        while i < len(tokens):
            token = tokens[i]
            
            # First token should be the slot name
            if i == 0:
                slot_name = self._unquote(token)
                i += 1
            elif token == "class" and i + 1 < len(tokens):
                class_value = self._unquote(tokens[i + 1])
                css_classes.append(class_value)
                i += 2
            elif token == "id" and i + 1 < len(tokens):
                element_id = self._unquote(tokens[i + 1])
                attributes.append(AttributeDefinition('id', element_id))
                i += 2
            else:
                i += 1
        
        return SlotComponent(name=slot_name, attributes=attributes, css_classes=css_classes)
    
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