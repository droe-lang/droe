"""Swift code generator for iOS apps."""

import os
from pathlib import Path
from typing import Dict, Any
from .base_generator import MobileGenerator
from ...ast import Program


class SwiftProjectGenerator:
    """Generate complete iOS projects from Roelang programs."""
    
    def __init__(self):
        self.base_generator = MobileGenerator()
    
    def generate_project(self, program: Program, output_dir: Path, mobile_config: Dict[str, Any]) -> Dict[str, str]:
        """Generate complete iOS project with configuration."""
        # Use mobile config for package name, app name, etc.
        app_name = mobile_config.get('appName', 'MyApp')
        
        # Update mobile config with app name for template rendering
        if 'appName' not in mobile_config:
            mobile_config['appName'] = app_name
        
        # Create the generator and use it
        swift_gen = SwiftGenerator()
        return swift_gen.generate(program, str(output_dir), mobile_config)


class SwiftGenerator(MobileGenerator):
    """Generate Swift/iOS code from Droe DSL."""
    
    def generate(self, program: Program, output_dir: str, mobile_config: Dict[str, Any] = None) -> Dict[str, str]:
        """Generate iOS project structure with Swift code."""
        output_path = Path(output_dir)
        
        # Extract UI components and API calls
        context = self.extract_ui_components(program)
        api_calls = self.extract_api_calls(program)
        context['api_calls'] = api_calls
        context['has_api'] = len(api_calls) > 0
        
        # Generated files mapping
        generated_files = {}
        
        # Create project structure
        self.create_ios_project_structure(output_path)
        
        # Generate API service if APIs are present
        if context['has_api']:
            api_service = self.generate_api_service(context)
            api_service_path = output_path / 'Network/ApiService.swift'
            generated_files[str(api_service_path)] = api_service
        
        # Generate ContentView.swift
        content_view = self.generate_content_view(context)
        content_view_path = output_path / 'ContentView.swift'
        generated_files[str(content_view_path)] = content_view
        
        # Generate App.swift
        app_file = self.generate_app_file(context)
        app_path = output_path / f'{context["app_name"]}App.swift'
        generated_files[str(app_path)] = app_file
        
        # Generate view files for layouts
        for layout in context['layouts']:
            layout_view = self.generate_layout_view(layout)
            layout_name = self.to_pascal_case(layout['name'])
            layout_path = output_path / f'Views/{layout_name}View.swift'
            generated_files[str(layout_path)] = layout_view
        
        # Generate form views
        for form in context['forms']:
            form_view = self.generate_form_view(form, context)
            form_name = self.to_pascal_case(form['name'])
            form_path = output_path / f'Views/{form_name}FormView.swift'
            generated_files[str(form_path)] = form_view
        
        # Generate models if needed
        if context.get('has_storage'):
            models = self.generate_models(context)
            models_path = output_path / 'Models/DataModels.swift'
            generated_files[str(models_path)] = models
        
        # Generate Info.plist
        info_plist = self.generate_info_plist(context)
        info_path = output_path / 'Info.plist'
        generated_files[str(info_path)] = info_plist
        
        # Generate project.pbxproj (simplified)
        project_file = self.generate_project_file(context)
        project_path = output_path / f'{context["app_name"]}.xcodeproj/project.pbxproj'
        generated_files[str(project_path)] = project_file
        
        # Write all files
        for file_path, content in generated_files.items():
            path = Path(file_path)
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content)
        
        return generated_files
    
    def create_ios_project_structure(self, output_path: Path):
        """Create iOS project directory structure."""
        dirs = [
            'Views',
            'Models',
            'Network',
            'Resources',
            'Assets.xcassets',
            'Assets.xcassets/AppIcon.appiconset',
            'Assets.xcassets/AccentColor.colorset',
            f'{output_path.name}.xcodeproj'
        ]
        
        for dir_path in dirs:
            (output_path / dir_path).mkdir(parents=True, exist_ok=True)
    
    def generate_content_view(self, context: Dict[str, Any]) -> str:
        """Generate main ContentView.swift file."""
        template = self.env.get_template('swift/content_view.swift.jinja2')
        return template.render(**context)
    
    def generate_app_file(self, context: Dict[str, Any]) -> str:
        """Generate App.swift file."""
        template = self.env.get_template('swift/app.swift.jinja2')
        return template.render(**context)
    
    def generate_layout_view(self, layout: Dict[str, Any]) -> str:
        """Generate SwiftUI view for a layout."""
        template = self.env.get_template('swift/layout_view.swift.jinja2')
        return template.render(layout=layout)
    
    def generate_form_view(self, form: Dict[str, Any], context: Dict[str, Any]) -> str:
        """Generate SwiftUI view for a form."""
        template = self.env.get_template('swift/form_view.swift.jinja2')
        return template.render(form=form, **context)
    
    def generate_models(self, context: Dict[str, Any]) -> str:
        """Generate data model files."""
        template = self.env.get_template('swift/models.swift.jinja2')
        return template.render(**context)
    
    def generate_info_plist(self, context: Dict[str, Any]) -> str:
        """Generate Info.plist file."""
        template = self.env.get_template('swift/info.plist.jinja2')
        return template.render(**context)
    
    def generate_project_file(self, context: Dict[str, Any]) -> str:
        """Generate Xcode project file."""
        template = self.env.get_template('swift/project.pbxproj.jinja2')
        return template.render(**context)
    
    def generate_api_service(self, context: Dict[str, Any]) -> str:
        """Generate Swift API service using URLSession."""
        template = self.env.get_template('swift/api_service.swift.jinja2')
        return template.render(**context)