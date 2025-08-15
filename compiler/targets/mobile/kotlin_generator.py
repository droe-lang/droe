"""Kotlin code generator for Android apps."""

import os
from pathlib import Path
from typing import Dict, Any
from .base_generator import MobileGenerator
from ...ast import Program


class KotlinProjectGenerator:
    """Generate complete Android projects from Roelang programs."""
    
    def __init__(self):
        self.base_generator = MobileGenerator()
    
    def generate_project(self, program: Program, output_dir: Path, mobile_config: Dict[str, Any]) -> Dict[str, str]:
        """Generate complete Android project with configuration."""
        # Use mobile config for package name, app name, etc.
        package_name = mobile_config.get('package', 'com.example.app')
        app_name = mobile_config.get('appName', 'MyApp')
        
        # Update mobile config with package name for template rendering
        if 'package' not in mobile_config:
            mobile_config['package'] = package_name
        if 'appName' not in mobile_config:
            mobile_config['appName'] = app_name
        
        # Create the generator and use it
        kotlin_gen = KotlinGenerator()
        return kotlin_gen.generate(program, str(output_dir), mobile_config)


class KotlinGenerator(MobileGenerator):
    """Generate Kotlin/Android code from Droe DSL."""
    
    def generate(self, program: Program, output_dir: str, mobile_config: Dict[str, Any] = None) -> Dict[str, str]:
        """Generate Android project structure with Kotlin code."""
        output_path = Path(output_dir)
        
        # Extract UI components and API calls
        context = self.extract_ui_components(program)
        api_calls = self.extract_api_calls(program)
        context['api_calls'] = api_calls
        context['has_api'] = len(api_calls) > 0
        
        # Generated files mapping
        generated_files = {}
        
        # Create project structure
        self.create_android_project_structure(output_path)
        
        # Generate API-related files if APIs are present
        if context['has_api']:
            # Generate API service interface
            api_service = self.generate_api_service(context)
            api_service_path = output_path / 'app/src/main/java/com/example/myapp/network/ApiService.kt'
            generated_files[str(api_service_path)] = api_service
            
            # Generate network module for Hilt
            network_module = self.generate_network_module(context)
            network_module_path = output_path / 'app/src/main/java/com/example/myapp/di/NetworkModule.kt'
            generated_files[str(network_module_path)] = network_module
            
            # Generate repository
            repository = self.generate_repository(context)
            repository_path = output_path / 'app/src/main/java/com/example/myapp/repository/ApiRepository.kt'
            generated_files[str(repository_path)] = repository
        
        # Generate MainActivity.kt
        main_activity = self.generate_main_activity(context)
        main_activity_path = output_path / 'app/src/main/java/com/example/myapp/MainActivity.kt'
        generated_files[str(main_activity_path)] = main_activity
        
        # Generate layout XML files
        for layout in context['layouts']:
            layout_xml = self.generate_layout_xml(layout)
            layout_name = self.to_snake_case(layout['name'])
            layout_path = output_path / f'app/src/main/res/layout/{layout_name}.xml'
            generated_files[str(layout_path)] = layout_xml
        
        # Generate form activities
        for form in context['forms']:
            form_activity = self.generate_form_activity(form, context)
            form_name = self.to_pascal_case(form['name'])
            form_path = output_path / f'app/src/main/java/com/example/myapp/{form_name}Activity.kt'
            generated_files[str(form_path)] = form_activity
        
        # Generate AndroidManifest.xml
        manifest = self.generate_manifest(context)
        manifest_path = output_path / 'app/src/main/AndroidManifest.xml'
        generated_files[str(manifest_path)] = manifest
        
        # Generate build.gradle files
        app_gradle = self.generate_app_gradle(context)
        app_gradle_path = output_path / 'app/build.gradle'
        generated_files[str(app_gradle_path)] = app_gradle
        
        project_gradle = self.generate_project_gradle()
        project_gradle_path = output_path / 'build.gradle'
        generated_files[str(project_gradle_path)] = project_gradle
        
        # Write all files
        for file_path, content in generated_files.items():
            path = Path(file_path)
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content)
        
        return generated_files
    
    def create_android_project_structure(self, output_path: Path):
        """Create Android project directory structure."""
        dirs = [
            'app/src/main/java/com/example/myapp',
            'app/src/main/java/com/example/myapp/network',
            'app/src/main/java/com/example/myapp/di',
            'app/src/main/java/com/example/myapp/repository',
            'app/src/main/java/com/example/myapp/models',
            'app/src/main/res/layout',
            'app/src/main/res/values',
            'app/src/main/res/drawable',
            'app/src/main/res/mipmap-hdpi',
            'app/src/main/res/mipmap-mdpi',
            'app/src/main/res/mipmap-xhdpi',
            'app/src/main/res/mipmap-xxhdpi',
            'app/src/main/res/mipmap-xxxhdpi'
        ]
        
        for dir_path in dirs:
            (output_path / dir_path).mkdir(parents=True, exist_ok=True)
    
    def generate_main_activity(self, context: Dict[str, Any]) -> str:
        """Generate MainActivity.kt file."""
        template = self.env.get_template('kotlin/main_activity.kt.jinja2')
        return template.render(**context)
    
    def generate_layout_xml(self, layout: Dict[str, Any]) -> str:
        """Generate Android layout XML file."""
        template = self.env.get_template('kotlin/layout.xml.jinja2')
        return template.render(layout=layout)
    
    def generate_form_activity(self, form: Dict[str, Any], context: Dict[str, Any]) -> str:
        """Generate Kotlin activity for a form."""
        template = self.env.get_template('kotlin/form_activity.kt.jinja2')
        return template.render(form=form, **context)
    
    def generate_manifest(self, context: Dict[str, Any]) -> str:
        """Generate AndroidManifest.xml file."""
        template = self.env.get_template('kotlin/manifest.xml.jinja2')
        return template.render(**context)
    
    def generate_app_gradle(self, context: Dict[str, Any]) -> str:
        """Generate app-level build.gradle file."""
        template = self.env.get_template('kotlin/app_build.gradle.jinja2')
        return template.render(**context)
    
    def generate_project_gradle(self) -> str:
        """Generate project-level build.gradle file."""
        template = self.env.get_template('kotlin/project_build.gradle.jinja2')
        return template.render()
    
    def generate_api_service(self, context: Dict[str, Any]) -> str:
        """Generate Retrofit API service interface."""
        template = self.env.get_template('kotlin/api_service.kt.jinja2')
        return template.render(package_name='com.example.myapp', **context)
    
    def generate_network_module(self, context: Dict[str, Any]) -> str:
        """Generate Hilt network module."""
        template = self.env.get_template('kotlin/network_module.kt.jinja2')
        return template.render(package_name='com.example.myapp', **context)
    
    def generate_repository(self, context: Dict[str, Any]) -> str:
        """Generate API repository with business logic."""
        template = self.env.get_template('kotlin/repository.kt.jinja2')
        return template.render(package_name='com.example.myapp', **context)