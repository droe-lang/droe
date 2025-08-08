"""Rust code generator for Rust target with Axum and database support."""

import json
import os
from typing import Dict, List, Any, Optional
from pathlib import Path
from jinja2 import Environment, FileSystemLoader
from ...codegen_base import BaseCodeGenerator
from ...ast import *


class RustCodeGenerator(BaseCodeGenerator):
    """Generates Rust code with Axum for HTTP server and configurable database support."""
    
    def __init__(self, source_file_path: str = None, is_main_file: bool = False, 
                 framework: str = "axum", package: Optional[str] = None, 
                 database: Optional[Dict[str, Any]] = None):
        super().__init__()
        self.source_file_path = source_file_path
        self.is_main_file = is_main_file
        self.framework = framework
        self.package = package or "roelang_app"
        self.database = database or {}
        
        # Track discovered features
        self.has_serve_endpoints = False
        self.has_database_ops = False
        self.data_structures = {}
        self.serve_endpoints = []
        self.modules = {}
        
        # Database configuration
        self.db_type = self.database.get('type', 'postgres')  # Default to postgres
        self.db_url = self.database.get('url', '')
        
        # Setup Jinja2 environment
        template_dir = os.path.join(os.path.dirname(__file__), 'templates', 'axum')
        self.jinja_env = Environment(loader=FileSystemLoader(template_dir))
        self._setup_jinja_filters()
    
    def _setup_jinja_filters(self):
        """Setup custom Jinja2 filters."""
        self.jinja_env.filters['snake_case'] = self._to_snake_case
        self.jinja_env.filters['rust_type'] = self._roe_type_to_rust_filter
        
    def _roe_type_to_rust_filter(self, roe_type: str) -> str:
        """Jinja2 filter version of type conversion."""
        return self._roe_type_to_rust(roe_type, [])
        
    def generate(self, node: ASTNode) -> Dict[str, Any]:
        """Generate complete Rust project with Axum and database support."""
        if isinstance(node, Program):
            # Analyze the program to understand what features are used
            self._analyze_program(node)
            
            # Generate project structure
            files = {}
            
            # Prepare template context
            context = self._get_template_context()
            
            # Generate Cargo.toml
            files['Cargo.toml'] = self._generate_from_template('cargo.toml.jinja2', context)
            
            # Generate main.rs with Axum server
            files['src/main.rs'] = self._generate_from_template('main.rs.jinja2', context)
            
            # Generate models if data structures exist
            if self.data_structures:
                files['src/models.rs'] = self._generate_from_template('models.rs.jinja2', context)
                
            # Generate handlers for serve endpoints
            if self.serve_endpoints:
                files['src/handlers.rs'] = self._generate_from_template('handlers.rs.jinja2', context)
                
            # Generate database module if database operations exist
            if self.has_database_ops:
                files['src/db.rs'] = self._generate_from_template('db.rs.jinja2', context)
                
            # Generate lib.rs to tie modules together
            files['src/lib.rs'] = self._generate_lib_rs()
            
            return {
                'files': files,
                'project_root': self.package.replace('.', '_').replace('-', '_')
            }
        
        return {'files': {}, 'project_root': self.package}
    
    def _analyze_program(self, program: Program):
        """Analyze the program to discover features and structures."""
        for stmt in program.statements:
            self._analyze_statement(stmt)
            
    def _analyze_statement(self, stmt: ASTNode):
        """Recursively analyze statements to discover features."""
        if isinstance(stmt, DataDefinition):
            self.data_structures[stmt.name] = stmt
            self.has_database_ops = True  # Assume data structures will be persisted
            
        elif isinstance(stmt, ServeStatement):
            self.has_serve_endpoints = True
            self.serve_endpoints.append(stmt)
            # Check serve body for database operations
            for body_stmt in stmt.body:
                self._analyze_statement(body_stmt)
            
        elif isinstance(stmt, DatabaseStatement):
            self.has_database_ops = True
            
        elif isinstance(stmt, ModuleDefinition):
            self.modules[stmt.name] = stmt
            # Recursively analyze module body
            for module_stmt in stmt.body:
                self._analyze_statement(module_stmt)
                
        elif isinstance(stmt, IncludeStatement):
            # Track included modules
            pass
    
    def _get_template_context(self) -> Dict[str, Any]:
        """Prepare template context with all necessary data."""
        # Process serve endpoints to add handler names
        processed_endpoints = []
        for endpoint in self.serve_endpoints:
            endpoint_data = {
                'method': endpoint.method,
                'endpoint': endpoint.endpoint,
                'handler_name': self._get_handler_name(endpoint),
                'params': getattr(endpoint, 'params', []),
                'accept_type': getattr(endpoint, 'accept_type', None),
                'response_action': getattr(endpoint, 'response_action', None)
            }
            processed_endpoints.append(endpoint_data)
        
        return {
            'package_name': self.package.replace('.', '_').replace('-', '_'),
            'has_serve_endpoints': self.has_serve_endpoints,
            'has_database_ops': self.has_database_ops,
            'data_structures': self.data_structures,
            'serve_endpoints': processed_endpoints,
            'modules': self.modules,
            'db_type': self.db_type,
            'default_db_url': self._get_default_db_url(),
            'has_uuid_fields': any(self._has_auto_id(ds) for ds in self.data_structures.values()),
            'has_datetime_fields': any(self._has_datetime(ds) for ds in self.data_structures.values()),
        }
    
    def _generate_from_template(self, template_name: str, context: Dict[str, Any]) -> str:
        """Generate content from Jinja2 template."""
        template = self.jinja_env.get_template(template_name)
        return template.render(**context)
    
    def _generate_lib_rs(self) -> str:
        """Generate lib.rs to export modules."""
        content = 'pub mod models;\npub mod handlers;\n'
        
        if self.has_database_ops:
            content += 'pub mod db;\n'
            
        content += '\npub use models::*;\n'
        return content
    
    def _get_handler_name(self, endpoint: ServeStatement) -> str:
        """Generate a handler function name from endpoint."""
        method = endpoint.method.lower()
        path_parts = endpoint.endpoint.strip('/').split('/')
        
        # Filter out parameter parts
        path_parts = [p for p in path_parts if not p.startswith(':')]
        
        if path_parts:
            return f'{method}_{"_".join(path_parts)}'
        else:
            return f'{method}_root'
            
    def _roe_type_to_rust(self, roe_type: str, annotations: List[str]) -> str:
        """Convert Roe type to Rust type."""
        type_map = {
            'text': 'String',
            'int': 'i32',
            'decimal': 'f64',
            'flag': 'bool',
            'date': 'DateTime<Utc>',
            'datetime': 'DateTime<Utc>',
        }
        
        # Handle auto-generated IDs
        if 'key' in annotations and 'auto' in annotations:
            return 'Uuid'
            
        return type_map.get(roe_type, 'String')
    
    def _to_snake_case(self, name: str) -> str:
        """Convert CamelCase to snake_case."""
        result = []
        for i, char in enumerate(name):
            if char.isupper() and i > 0:
                result.append('_')
            result.append(char.lower())
        return ''.join(result)
    
    def _has_auto_id(self, data_def: DataDefinition) -> bool:
        """Check if data definition has auto-generated ID field."""
        for field in data_def.fields:
            if 'key' in field.annotations and 'auto' in field.annotations:
                return True
        return False
    
    def _has_datetime(self, data_def: DataDefinition) -> bool:
        """Check if data definition has datetime fields."""
        for field in data_def.fields:
            if field.type in ['date', 'datetime']:
                return True
        return False
    
    def _get_default_db_url(self) -> str:
        """Get default database URL based on database type."""
        urls = {
            'postgres': 'postgresql://localhost/roelang_db',
            'mysql': 'mysql://localhost/roelang_db',
            'sqlite': 'sqlite:roelang.db',
        }
        return urls.get(self.db_type, 'postgresql://localhost/roelang_db')
    
    def emit_statement(self, stmt: ASTNode) -> str:
        """Emit Rust code for a statement (required by base class)."""
        # This is handled by the generate method for project generation
        return ""
    
    def emit_expression(self, expr: ASTNode) -> str:
        """Emit Rust code for an expression (required by base class)."""
        # This is handled by the generate method for project generation
        return ""