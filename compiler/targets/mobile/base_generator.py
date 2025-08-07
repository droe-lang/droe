"""Base mobile code generator with shared functionality."""

import os
from pathlib import Path
from typing import Dict, Any, List
from jinja2 import Environment, FileSystemLoader, Template
from ...ast import (
    Program, ASTNode, LayoutDefinition, FormDefinition,
    TitleComponent, InputComponent, ButtonComponent,
    TextareaComponent, DropdownComponent, CheckboxComponent,
    RadioComponent, ToggleComponent, ImageComponent,
    VideoComponent, AudioComponent, ApiCallStatement, 
    ApiHeader, DataDefinition
)


class MobileGenerator:
    """Base class for mobile code generation."""
    
    def __init__(self):
        # Setup Jinja2 environment
        template_dir = Path(__file__).parent / 'templates'
        self.env = Environment(
            loader=FileSystemLoader(str(template_dir)),
            trim_blocks=True,
            lstrip_blocks=True
        )
        
        # Register custom filters
        self.env.filters['camelcase'] = self.to_camel_case
        self.env.filters['pascalcase'] = self.to_pascal_case
        self.env.filters['snake_case'] = self.to_snake_case
        self.env.filters['snakecase'] = self.to_snake_case
    
    def to_camel_case(self, text: str) -> str:
        """Convert text to camelCase."""
        words = text.replace('-', '_').split('_')
        return words[0].lower() + ''.join(w.capitalize() for w in words[1:])
    
    def to_pascal_case(self, text: str) -> str:
        """Convert text to PascalCase."""
        words = text.replace('-', '_').split('_')
        return ''.join(w.capitalize() for w in words)
    
    def to_snake_case(self, text: str) -> str:
        """Convert text to snake_case."""
        return text.replace('-', '_').lower()
    
    def extract_ui_components(self, program: Program) -> Dict[str, Any]:
        """Extract UI components from the AST."""
        context = {
            'app_name': 'MyApp',
            'package_name': 'com.example.myapp',
            'layouts': [],
            'forms': [],
            'components': [],
            'has_camera': False,
            'has_location': False,
            'has_notifications': False,
            'has_storage': False,
            'has_sensors': False,
            'has_contacts': False,
            'permissions': set()
        }
        
        for stmt in program.statements:
            if isinstance(stmt, LayoutDefinition):
                layout = self.process_layout(stmt)
                context['layouts'].append(layout)
                self.update_permissions(context, layout['components'])
            
            elif isinstance(stmt, FormDefinition):
                form = self.process_form(stmt)
                context['forms'].append(form)
                self.update_permissions(context, form['elements'])
            
            # Process individual components
            elif isinstance(stmt, (TitleComponent, InputComponent, ButtonComponent,
                                  TextareaComponent, DropdownComponent, CheckboxComponent,
                                  RadioComponent, ToggleComponent, ImageComponent,
                                  VideoComponent, AudioComponent)):
                component = self.process_component(stmt)
                context['components'].append(component)
                self.update_permissions(context, [component])
        
        # Convert permissions set to list
        context['permissions'] = list(context['permissions'])
        
        return context
    
    def process_layout(self, layout: LayoutDefinition) -> Dict[str, Any]:
        """Process a layout definition."""
        return {
            'name': layout.name,
            'components': [self.process_component(c) for c in layout.children]
        }
    
    def process_form(self, form: FormDefinition) -> Dict[str, Any]:
        """Process a form definition."""
        return {
            'name': form.name,
            'elements': [self.process_component(e) for e in form.children]
        }
    
    def process_component(self, component: ASTNode) -> Dict[str, Any]:
        """Process an individual UI component."""
        comp_data = {
            'type': component.__class__.__name__.replace('Component', '').lower()
        }
        
        # Extract component-specific properties
        if isinstance(component, TitleComponent):
            comp_data['text'] = component.text
            comp_data['level'] = getattr(component, 'level', 1)
        
        elif isinstance(component, InputComponent):
            # Extract placeholder from attributes
            placeholder = ""
            for attr in component.attributes:
                if attr.name == 'placeholder':
                    placeholder = attr.value
                    break
            comp_data['placeholder'] = placeholder
            comp_data['input_type'] = component.input_type
            comp_data['id'] = self.get_component_id(component)
        
        elif isinstance(component, ButtonComponent):
            comp_data['text'] = component.text
            comp_data['action'] = self.get_action(component)
            
            # Check for mobile-specific components
            for attr in component.attributes:
                if attr.name == 'mobile_component':
                    comp_data['mobile_type'] = attr.value
                    comp_data['type'] = 'mobile_' + attr.value
        
        elif isinstance(component, TextareaComponent):
            # Extract placeholder and rows from attributes
            placeholder = ""
            rows = 4
            for attr in component.attributes:
                if attr.name == 'placeholder':
                    placeholder = attr.value
                elif attr.name == 'rows':
                    try:
                        rows = int(attr.value)
                    except ValueError:
                        rows = 4
            comp_data['placeholder'] = placeholder
            comp_data['rows'] = rows
            comp_data['id'] = self.get_component_id(component)
        
        elif isinstance(component, DropdownComponent):
            # Extract options as strings
            options = []
            for option in component.options:
                if hasattr(option, 'value'):
                    options.append(option.value)
                else:
                    options.append(str(option))
            comp_data['options'] = options
            
            # Extract default from attributes
            default_value = None
            for attr in component.attributes:
                if attr.name == 'default':
                    default_value = attr.value
                    break
            comp_data['default'] = default_value
            comp_data['id'] = self.get_component_id(component)
        
        elif isinstance(component, CheckboxComponent):
            # Extract options from attributes
            options = []
            for attr in component.attributes:
                if attr.name == 'options':
                    options = attr.value.split(',')
                    break
            
            if not options and component.text:
                options = [component.text]
            
            comp_data['options'] = options
            comp_data['id'] = self.get_component_id(component)
        
        elif isinstance(component, RadioComponent):
            # Extract name and options from attributes
            name = ""
            options = []
            default_value = None
            
            for attr in component.attributes:
                if attr.name == 'name':
                    name = attr.value
                elif attr.name == 'options':
                    options = attr.value.split(',')
                elif attr.name == 'default':
                    default_value = attr.value
            
            comp_data['name'] = name
            comp_data['options'] = options
            comp_data['default'] = default_value
            comp_data['id'] = self.get_component_id(component)
        
        elif isinstance(component, ToggleComponent):
            # Extract label from attributes
            label = ""
            for attr in component.attributes:
                if attr.name == 'label':
                    label = attr.value
                    break
            comp_data['label'] = label
            comp_data['id'] = self.get_component_id(component)
        
        elif isinstance(component, ImageComponent):
            comp_data['src'] = component.src
            comp_data['alt'] = component.alt
        
        elif isinstance(component, VideoComponent):
            comp_data['src'] = component.src
            comp_data['controls'] = component.controls
            comp_data['autoplay'] = component.autoplay
        
        elif isinstance(component, AudioComponent):
            comp_data['src'] = component.src
            comp_data['controls'] = component.controls
            comp_data['autoplay'] = component.autoplay
        
        return comp_data
    
    def extract_api_calls(self, program: Program) -> List[Dict[str, Any]]:
        """Extract API calls from the AST."""
        api_calls = []
        
        def find_api_calls_in_statements(statements):
            for stmt in statements:
                if isinstance(stmt, ApiCallStatement):
                    api_call = self.process_api_call(stmt)
                    api_calls.append(api_call)
                
                # Check inside action definitions
                elif hasattr(stmt, 'body') and stmt.body:
                    find_api_calls_in_statements(stmt.body)
        
        find_api_calls_in_statements(program.statements)
        
        return api_calls
    
    def process_api_call(self, api_call: ApiCallStatement) -> Dict[str, Any]:
        """Process an API call statement."""
        # Generate function name from verb and endpoint
        endpoint_parts = api_call.endpoint.strip('/').split('/')
        function_name = api_call.verb.lower()
        if endpoint_parts and endpoint_parts[0]:
            function_name += ''.join(part.capitalize() for part in endpoint_parts)
        
        # Process headers
        headers = []
        for header in api_call.headers:
            headers.append({
                'name': header.name,
                'value': header.value
            })
        
        # Determine payload and response types
        payload_type = api_call.payload if api_call.payload else None
        response_type = api_call.response_variable if api_call.response_variable else "ApiResponse"
        
        # Convert payload to proper type name if it exists
        if payload_type:
            payload_type = self.to_pascal_case(payload_type)
        
        return {
            'verb': api_call.verb,
            'endpoint': api_call.endpoint,
            'method': api_call.method,
            'function_name': function_name,
            'payload': api_call.payload,
            'payload_type': payload_type,
            'response_variable': api_call.response_variable,
            'response_type': self.to_pascal_case(response_type) if response_type else "ApiResponse",
            'headers': headers
        }
    
    def get_component_id(self, component: ASTNode) -> str:
        """Get component ID from attributes or generate one."""
        for attr in getattr(component, 'attributes', []):
            if attr.name == 'id':
                return attr.value
        
        # Generate ID based on component type
        comp_type = component.__class__.__name__.replace('Component', '').lower()
        return f"{comp_type}_field"
    
    def get_action(self, component: ButtonComponent) -> str:
        """Get action from button attributes."""
        for attr in component.attributes:
            if hasattr(attr, '__class__') and attr.__class__.__name__ == 'ActionAttribute':
                return attr.action_name
            elif attr.name == 'action':
                return attr.value
        return ""
    
    def update_permissions(self, context: Dict[str, Any], components: List[Dict[str, Any]]):
        """Update required permissions based on components."""
        for comp in components:
            comp_type = comp.get('type', '')
            
            if comp_type == 'mobile_camera':
                context['has_camera'] = True
                context['permissions'].add('camera')
            
            elif comp_type == 'mobile_location':
                context['has_location'] = True
                context['permissions'].add('location')
            
            elif comp_type == 'mobile_notification':
                context['has_notifications'] = True
                context['permissions'].add('notifications')
            
            elif comp_type == 'mobile_storage':
                context['has_storage'] = True
                context['permissions'].add('storage')
            
            elif comp_type == 'mobile_sensor':
                context['has_sensors'] = True
                context['permissions'].add('sensors')
            
            elif comp_type == 'mobile_contact':
                context['has_contacts'] = True
                context['permissions'].add('contacts')
            
            elif comp_type in ['image', 'video', 'audio']:
                context['permissions'].add('media')