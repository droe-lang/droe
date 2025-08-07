"""Spring Boot template-based code generator."""

import os
from pathlib import Path
from typing import Dict, Any, List, Optional
from jinja2 import Environment, FileSystemLoader
from ...ast import (
    Program, ModuleDefinition, DataDefinition, DataField,
    ActionDefinitionWithParams, ServeStatement, AcceptStatement,
    RespondStatement, ParamsStatement, DatabaseStatement
)


class SpringBootGenerator:
    """Template-based Spring Boot code generator."""
    
    def __init__(self):
        # Setup Jinja2 environment
        template_dir = Path(__file__).parent / 'templates' / 'spring'
        self.env = Environment(
            loader=FileSystemLoader(str(template_dir)),
            trim_blocks=True,
            lstrip_blocks=True
        )
        
        # Register custom filters
        self.env.filters['capitalize'] = self.capitalize_filter
        self.env.filters['lower'] = lambda s: str(s).lower()
        self.env.filters['camelcase'] = self.to_camel_case
    
    def capitalize_filter(self, text: str) -> str:
        """Capitalize first letter of text."""
        if not text:
            return text
        return text[0].upper() + text[1:]
    
    def to_camel_case(self, text: str) -> str:
        """Convert text to camelCase."""
        words = text.replace('-', '_').split('_')
        return words[0].lower() + ''.join(w.capitalize() for w in words[1:])
    
    def generate_spring_boot_project(self, 
                                   program: Program,
                                   project_name: str,
                                   package_name: str = "com.example.app",
                                   database_config: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """Generate complete Spring Boot project structure."""
        
        # Extract entities and modules from program
        context = self._build_project_context(program, project_name, package_name, database_config)
        
        # Generate all files
        files = {}
        
        # Main application class
        files[f"src/main/java/{package_name.replace('.', '/')}/Application.java"] = \
            self._render_template('application.java.jinja2', context)
        
        # Generate entities, repositories, services, and controllers
        for entity in context['entities']:
            entity_context = {**context, **entity}
            
            # Entity class
            files[f"src/main/java/{package_name.replace('.', '/')}/entity/{entity['class_name']}.java"] = \
                self._render_template('entity.java.jinja2', entity_context)
            
            # Repository interface
            files[f"src/main/java/{package_name.replace('.', '/')}/repository/{entity['class_name']}Repository.java"] = \
                self._render_template('repository.java.jinja2', entity_context)
            
            # Service class
            files[f"src/main/java/{package_name.replace('.', '/')}/service/{entity['service_name']}.java"] = \
                self._render_template('service.java.jinja2', entity_context)
            
            # Controller class (if has REST endpoints)
            if entity.get('has_rest_endpoints', True):
                files[f"src/main/java/{package_name.replace('.', '/')}/controller/{entity['class_name']}Controller.java"] = \
                    self._render_template('controller.java.jinja2', entity_context)
        
        # Configuration files
        files['pom.xml'] = self._render_template('pom.xml.jinja2', context)
        files['src/main/resources/application.properties'] = \
            self._render_template('application.properties.jinja2', context)
        files['README.md'] = self._render_template('readme.md.jinja2', context)
        
        return {
            'files': files,
            'project_root': context['artifact_id'],
            'context': context
        }
    
    def _build_project_context(self, program: Program, project_name: str, package_name: str, database_config: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """Build template context from program AST."""
        
        # Merge database config with defaults
        db_config = {
            'type': 'h2',
            'host': 'localhost',
            'port': 5432,
            'name': f"{project_name.lower().replace('-', '_')}_db",
            'username': 'sa',
            'password': '',
            'ddl_auto': 'update',
            'show_sql': True
        }
        if database_config:
            db_config.update(database_config)
        
        # Base project context
        context = {
            'project_name': project_name,
            'app_name': project_name.replace('-', '').replace('_', '').capitalize(),
            'package_name': package_name,
            'artifact_id': project_name.lower().replace(' ', '-'),
            'group_id': '.'.join(package_name.split('.')[:-1]),
            'spring_boot_version': '3.1.5',
            'java_version': '17',
            'server_port': 8080,
            'database': db_config['type'],
            'db_host': db_config['host'],
            'db_port': db_config['port'],
            'db_name': db_config['name'],
            'db_username': db_config['username'],
            'db_password': db_config['password'],
            'ddl_auto': db_config['ddl_auto'],
            'show_sql': str(db_config['show_sql']).lower(),
            'entities': [],
            'modules': [],
            'generation_date': 'Generated by Roelang'
        }
        
        # Extract entities from data definitions and modules
        entities = []
        modules = []
        
        # Process modules
        print(f"DEBUG SpringGen: Processing program with {len(program.statements)} statements")
        for i, stmt in enumerate(program.statements):
            print(f"DEBUG SpringGen: Statement {i}: {type(stmt).__name__}")
            if isinstance(stmt, ModuleDefinition):
                print(f"DEBUG SpringGen: Found module: {stmt.name}")
                modules.append(self._process_module(stmt))
                
                # Extract data definitions from module
                for module_stmt in stmt.body:
                    print(f"DEBUG SpringGen: Module stmt: {type(module_stmt).__name__}")
                    if isinstance(module_stmt, DataDefinition):
                        print(f"DEBUG SpringGen: Found data def: {module_stmt.name}")
                        entity = self._process_data_definition(module_stmt, context)
                        entities.append(entity)
            
            elif isinstance(stmt, DataDefinition):
                print(f"DEBUG SpringGen: Found top-level data def: {stmt.name}")
                entity = self._process_data_definition(stmt, context)
                entities.append(entity)
        
        context['entities'] = entities
        context['modules'] = modules
        
        return context
    
    def _process_data_definition(self, data_def: DataDefinition, base_context: Dict[str, Any]) -> Dict[str, Any]:
        """Process a data definition into entity context."""
        
        # Convert fields
        fields = []
        has_name_field = False
        has_decimal_fields = False
        has_date_fields = False
        
        for field in data_def.fields:
            java_type = self._get_java_type(field.type)
            
            field_info = {
                'name': field.name,
                'java_type': java_type,
                'roe_type': field.type
            }
            fields.append(field_info)
            
            if field.name.lower() == 'name':
                has_name_field = True
            if java_type == 'BigDecimal':
                has_decimal_fields = True
            if 'Date' in java_type or 'Time' in java_type:
                has_date_fields = True
        
        service_name = f"{data_def.name}Service"
        
        return {
            'class_name': data_def.name,
            'entity_name': data_def.name,
            'service_name': service_name,
            'table_name': data_def.name.lower() + 's',
            'fields': fields,
            'has_name_field': has_name_field,
            'has_decimal_fields': has_decimal_fields,
            'has_date_fields': has_date_fields,
            'has_rest_endpoints': True,
            'create_fields': fields,  # Fields used in create operations
            'update_fields': fields,  # Fields used in update operations
            'create_with_params': True,
            'package_name': base_context['package_name']
        }
    
    def _process_module(self, module: ModuleDefinition) -> Dict[str, Any]:
        """Process a module definition."""
        
        # Extract serve statements (REST endpoints)
        serve_statements = []
        for stmt in module.body:
            if isinstance(stmt, ServeStatement):
                serve_statements.append(self._process_serve_statement(stmt))
        
        return {
            'name': module.name,
            'serve_statements': serve_statements,
            'has_rest_endpoints': len(serve_statements) > 0
        }
    
    def _process_serve_statement(self, serve: ServeStatement) -> Dict[str, Any]:
        """Process a serve statement into REST endpoint info."""
        
        return {
            'method': serve.method.upper(),
            'endpoint': serve.endpoint,
            'params': serve.params,
            'body': [stmt for stmt in serve.body]
        }
    
    def _get_java_type(self, roe_type: str) -> str:
        """Map Roelang types to Java types."""
        type_mapping = {
            'text': 'String',
            'number': 'Integer',
            'decimal': 'BigDecimal',
            'flag': 'Boolean',
            'date': 'LocalDate',
            'datetime': 'LocalDateTime',
            'time': 'LocalTime'
        }
        return type_mapping.get(roe_type, 'String')
    
    def _render_template(self, template_name: str, context: Dict[str, Any]) -> str:
        """Render a template with the given context."""
        template = self.env.get_template(template_name)
        return template.render(**context)