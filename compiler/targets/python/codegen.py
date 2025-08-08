"""Python code generator for Roelang compiler with FastAPI and SQLAlchemy support."""

import os
from typing import List, Dict, Any, Optional
from jinja2 import Environment, FileSystemLoader
from ...ast import (
    ASTNode, Program, DisplayStatement, IfStatement,
    Literal, Identifier, BinaryOp, PropertyAccess,
    Assignment, ArrayLiteral, WhileLoop, ForEachLoop, ArithmeticOp,
    TaskAction, TaskInvocation, ActionDefinition, ReturnStatement, ActionInvocation,
    ModuleDefinition, DataDefinition, DataField, ActionDefinitionWithParams,
    ActionParameter, ActionInvocationWithArgs, StringInterpolation,
    DataInstance, FieldAssignment, FormatExpression, ServeStatement, DatabaseStatement, ApiCallStatement
)
from ...symbols import SymbolTable, VariableType
from ...codegen_base import BaseCodeGenerator, CodeGenError


class PythonCodeGenerator(BaseCodeGenerator):
    """Generates Python code with FastAPI for HTTP server and SQLAlchemy for database support."""
    
    def __init__(self, source_file_path: str = None, is_main_file: bool = False, 
                 framework: str = "fastapi", package: Optional[str] = None, 
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
        
        # Legacy support for direct code generation
        self.imports = set()
        self.class_definitions = []
        self.function_definitions = []
        self.main_code = []
        
        # Setup Jinja2 environment for FastAPI templates
        if framework == "fastapi":
            template_dir = os.path.join(os.path.dirname(__file__), 'templates', 'fastapi')
            self.jinja_env = Environment(loader=FileSystemLoader(template_dir))
            self._setup_jinja_filters()
    
    def _setup_jinja_filters(self):
        """Setup custom Jinja2 filters."""
        self.jinja_env.filters['snake_case'] = self._to_snake_case
        self.jinja_env.filters['python_type'] = self._roe_type_to_python_filter
        
    def _roe_type_to_python_filter(self, roe_type: str) -> str:
        """Jinja2 filter version of type conversion."""
        return self._roe_type_to_python(roe_type, [])
    
    def generate(self, node: ASTNode) -> Dict[str, Any]:
        """Generate Python code from AST."""
        if isinstance(node, Program):
            # For FastAPI framework, generate project structure
            if self.framework == "fastapi":
                return self._generate_fastapi_project(node)
            else:
                # Legacy single-file generation
                return self._generate_legacy_python(node)
        
        return {'files': {}, 'project_root': self.package}
    
    def _generate_fastapi_project(self, program: Program) -> Dict[str, Any]:
        """Generate complete FastAPI project structure."""
        # Analyze the program to understand what features are used
        self._analyze_program(program)
        
        # Generate project structure
        files = {}
        
        # Prepare template context
        context = self._get_template_context()
        
        # Generate requirements.txt
        files['requirements.txt'] = self._generate_from_template('requirements.txt.jinja2', context)
        
        # Generate main.py
        files[f'{self.package}/main.py'] = self._generate_from_template('main.py.jinja2', context)
        
        # Generate models if data structures exist
        if self.data_structures:
            files[f'{self.package}/models.py'] = self._generate_from_template('models.py.jinja2', context)
            
        # Generate database module if database operations exist
        if self.has_database_ops:
            files[f'{self.package}/database.py'] = self._generate_from_template('database.py.jinja2', context)
            
        # Generate routers for endpoints
        if self.serve_endpoints or self.data_structures:
            files[f'{self.package}/routers.py'] = self._generate_from_template('routers.py.jinja2', context)
            
        # Generate __init__.py
        files[f'{self.package}/__init__.py'] = self._generate_from_template('__init__.py.jinja2', context)
        
        return {
            'files': files,
            'project_root': self.package
        }
    
    def _generate_legacy_python(self, program: Program) -> str:
        """Generate legacy single-file Python code."""
        self.clear_output()
        self.imports.clear()
        self.class_definitions.clear()
        self.function_definitions.clear()
        self.main_code.clear()
        
        # Add core imports
        self.imports.add("from typing import List, Dict, Any, Union")
        self.imports.add("from datetime import datetime, date")
        self.imports.add("from decimal import Decimal")
        self.imports.add("import sys")
        
        # Enable core libraries
        if self.is_core_lib_enabled('math_utils'):
            self.imports.add("import math")
        
        if self.is_core_lib_enabled('formatting'):
            self.imports.add("import locale")
        
        # Process all statements
        for stmt in program.statements:
            self.emit_statement(stmt)
        
        # Generate final Python code
        return {'files': {'main.py': self._build_python_file()}, 'project_root': 'main'}
    
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
                
    def _get_template_context(self) -> Dict[str, Any]:
        """Prepare template context with all necessary data."""
        # Process serve endpoints to add handler and router names
        processed_endpoints = []
        for endpoint in self.serve_endpoints:
            endpoint_data = {
                'method': endpoint.method,
                'endpoint': endpoint.endpoint,
                'handler_name': self._get_handler_name(endpoint),
                'router_name': self._get_router_name(endpoint),
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
        }
    
    def _generate_from_template(self, template_name: str, context: Dict[str, Any]) -> str:
        """Generate content from Jinja2 template."""
        template = self.jinja_env.get_template(template_name)
        return template.render(**context)
    
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
    
    def _get_router_name(self, endpoint: ServeStatement) -> str:
        """Generate a router name from endpoint."""
        path_parts = endpoint.endpoint.strip('/').split('/')
        path_parts = [p for p in path_parts if not p.startswith(':')]
        
        if path_parts:
            return f'{"_".join(path_parts)}_router'
        else:
            return 'root_router'
            
    def _roe_type_to_python(self, roe_type: str, annotations: List[str]) -> str:
        """Convert Roe type to Python type."""
        type_map = {
            'text': 'str',
            'int': 'int',
            'decimal': 'float',
            'flag': 'bool',
            'date': 'datetime',
            'datetime': 'datetime',
        }
        
        # Handle auto-generated IDs
        if 'key' in annotations and 'auto' in annotations:
            return 'UUID'
            
        return type_map.get(roe_type, 'str')
    
    def _to_snake_case(self, name: str) -> str:
        """Convert CamelCase to snake_case."""
        result = []
        for i, char in enumerate(name):
            if char.isupper() and i > 0:
                result.append('_')
            result.append(char.lower())
        return ''.join(result)
    
    def _get_default_db_url(self) -> str:
        """Get default database URL based on database type."""
        urls = {
            'postgres': 'postgresql://localhost/roelang_db',
            'mysql': 'mysql://localhost/roelang_db',
            'sqlite': 'sqlite:///./roelang.db',
        }
        return urls.get(self.db_type, 'postgresql://localhost/roelang_db')
    
    # Legacy methods for backward compatibility
    def _build_python_file(self) -> str:
        """Build the complete Python file."""
        lines = []
        
        # Add imports
        lines.extend(sorted(self.imports))
        lines.append("")
        
        # Add class definitions
        for class_def in self.class_definitions:
            lines.extend(class_def)
            lines.append("")
        
        # Add function definitions  
        for func_def in self.function_definitions:
            lines.extend(func_def)
            lines.append("")
        
        # Add main code
        lines.append("def main():")
        if self.main_code:
            for line in self.main_code:
                lines.append(f"    {line}")
        else:
            lines.append("    pass")
        
        lines.append("")
        lines.append("if __name__ == '__main__':")
        lines.append("    main()")
        
        return "\n".join(lines)
    
    def emit_statement(self, stmt: ASTNode):
        """Emit code for a statement (legacy compatibility)."""
        if isinstance(stmt, DisplayStatement):
            self.emit_display_statement(stmt)
        elif isinstance(stmt, Assignment):
            self.emit_assignment(stmt)
        elif isinstance(stmt, IfStatement):
            self.emit_if_statement(stmt)
        elif isinstance(stmt, WhileLoop):
            self.emit_while_loop(stmt)
        elif isinstance(stmt, ForEachLoop):
            self.emit_foreach_loop(stmt)
        elif isinstance(stmt, ActionDefinition):
            self.emit_action_definition(stmt)
        elif isinstance(stmt, ApiCallStatement):
            self.emit_api_call(stmt)
        elif isinstance(stmt, DatabaseStatement):
            self.emit_database_statement(stmt)
        elif isinstance(stmt, ServeStatement):
            self.emit_serve_statement(stmt)
        elif isinstance(stmt, ModuleDefinition):
            self.emit_module_definition(stmt)
        else:
            # Add as comment for unsupported statements
            self.main_code.append(f"# TODO: Implement {type(stmt).__name__}")
    
    def emit_expression(self, expr: ASTNode) -> str:
        """Emit code for an expression and return the expression string (legacy compatibility)."""
        if isinstance(expr, Literal):
            if isinstance(expr.value, str):
                return f'"{expr.value}"'
            elif isinstance(expr.value, bool):
                return str(expr.value)
            else:
                return str(expr.value)
        elif isinstance(expr, Identifier):
            return expr.name
        elif isinstance(expr, BinaryOp):
            left = self.emit_expression(expr.left)
            right = self.emit_expression(expr.right)
            return f"({left} {expr.operator} {right})"
        elif isinstance(expr, ArithmeticOp):
            left = self.emit_expression(expr.left)
            right = self.emit_expression(expr.right)
            return f"({left} {expr.operator} {right})"
        elif isinstance(expr, ArrayLiteral):
            elements = [self.emit_expression(elem) for elem in expr.elements]
            return f"[{', '.join(elements)}]"
        elif isinstance(expr, StringInterpolation):
            return self.emit_string_interpolation(expr)
        elif isinstance(expr, FormatExpression):
            expr_str = self.emit_expression(expr.expression)
            pattern = f'"{expr.format_pattern}"'
            expr_type = self.infer_type(expr.expression)
            
            if expr_type == VariableType.DATE:
                return self._inline_date_formatting(expr_str, expr.format_pattern)
            elif expr_type == VariableType.DECIMAL:
                return self._inline_decimal_formatting(expr_str, expr.format_pattern)
            elif self._is_numeric_type(expr_type):
                return self._inline_number_formatting(expr_str, expr.format_pattern)
            else:
                return expr_str
        else:
            return f"# TODO: {type(expr).__name__}"
    
    def emit_display_statement(self, stmt: DisplayStatement):
        """Emit display statement with native formatting."""
        expr_str = self.emit_expression(stmt.expression)
        expr_type = self.infer_type(stmt.expression)
        
        # Handle boolean formatting inline
        if expr_type == VariableType.BOOLEAN or expr_type == VariableType.FLAG or expr_type == VariableType.YESNO:
            self.main_code.append(f"print('true' if {expr_str} else 'false')")
        # Handle list formatting inline  
        elif expr_type in [VariableType.LIST_OF, VariableType.GROUP_OF, VariableType.ARRAY]:
            self.main_code.append(f"print('[' + ', '.join(str(item) for item in {expr_str}) + ']')")
        else:
            self.main_code.append(f"print({expr_str})")
    
    def emit_assignment(self, stmt: Assignment):
        """Emit assignment statement."""
        value_str = self.emit_expression(stmt.value)
        
        # Simple assignment without explicit type conversion for now
        # Type inference could be added here based on the value
        self.main_code.append(f"{stmt.variable} = {value_str}")
        
        # Try to infer and track variable type
        inferred_type = self.infer_type(stmt.value)
        self.symbol_table.declare_variable(stmt.variable, inferred_type)
    
    def emit_if_statement(self, stmt: IfStatement):
        """Emit if statement."""
        condition_str = self.emit_expression(stmt.condition)
        self.main_code.append(f"if {condition_str}:")
        
        # Emit then block
        if stmt.then_body:
            for then_stmt in stmt.then_body:
                # Process nested statements (this is simplified)
                if isinstance(then_stmt, DisplayStatement):
                    expr_str = self.emit_expression(then_stmt.expression)
                    self.main_code.append(f"    print({expr_str})")
        else:
            self.main_code.append("    pass")
        
        # Emit else block if present
        if stmt.else_body:
            self.main_code.append("else:")
            for else_stmt in stmt.else_body:
                if isinstance(else_stmt, DisplayStatement):
                    expr_str = self.emit_expression(else_stmt.expression)
                    self.main_code.append(f"    print({expr_str})")
    
    def emit_while_loop(self, stmt: WhileLoop):
        """Emit while loop."""
        condition_str = self.emit_expression(stmt.condition)
        self.main_code.append(f"while {condition_str}:")
        
        if stmt.body:
            for body_stmt in stmt.body:
                # Simplified - would need proper indentation handling
                self.main_code.append("    # Loop body")
        else:
            self.main_code.append("    pass")
    
    def emit_foreach_loop(self, stmt: ForEachLoop):
        """Emit for-each loop."""
        collection_str = self.emit_expression(stmt.collection)
        self.main_code.append(f"for {stmt.variable} in {collection_str}:")
        self.main_code.append("    # Loop body")
    
    def emit_action_definition(self, stmt: ActionDefinition):
        """Emit action definition as Python function."""
        func_lines = [f"def {stmt.name}():"]
        
        if stmt.body:
            for body_stmt in stmt.body:
                if isinstance(body_stmt, ReturnStatement):
                    return_expr = self.emit_expression(body_stmt.expression)
                    func_lines.append(f"    return {return_expr}")
                else:
                    func_lines.append("    # Function body")
        else:
            func_lines.append("    pass")
        
        self.function_definitions.append(func_lines)
    
    def emit_module_definition(self, stmt: ModuleDefinition):
        """Emit module definition as Python class."""
        class_lines = [f"class {stmt.name}:"]
        
        if stmt.body:
            for body_stmt in stmt.body:
                if isinstance(body_stmt, ActionDefinition):
                    class_lines.append("    @staticmethod")
                    class_lines.append(f"    def {body_stmt.name}():")
                    class_lines.append("        # Method body")
                    class_lines.append("")
        else:
            class_lines.append("    pass")
        
        self.class_definitions.append(class_lines)
    
    def emit_string_interpolation(self, expr: StringInterpolation) -> str:
        """Emit string interpolation as f-string."""
        parts = []
        for part in expr.parts:
            if isinstance(part, str):
                parts.append(part)
            else:
                part_expr = self.emit_expression(part)
                parts.append(f"{{{part_expr}}}")
        
        interpolated = "".join(parts)
        return f'f"{interpolated}"'
    
    def _inline_date_formatting(self, expr_str: str, pattern: str) -> str:
        """Generate inline date formatting code."""
        if pattern == "MM/dd/yyyy":
            return f"datetime.fromisoformat({expr_str}).strftime('%m/%d/%Y') if isinstance({expr_str}, str) else {expr_str}.strftime('%m/%d/%Y')"
        elif pattern == "dd/MM/yyyy":
            return f"datetime.fromisoformat({expr_str}).strftime('%d/%m/%Y') if isinstance({expr_str}, str) else {expr_str}.strftime('%d/%m/%Y')"
        elif pattern == "MMM dd, yyyy":
            return f"datetime.fromisoformat({expr_str}).strftime('%b %d, %Y') if isinstance({expr_str}, str) else {expr_str}.strftime('%b %d, %Y')"
        elif pattern == "long":
            return f"datetime.fromisoformat({expr_str}).strftime('%A, %B %d, %Y') if isinstance({expr_str}, str) else {expr_str}.strftime('%A, %B %d, %Y')"
        elif pattern == "short":
            return f"datetime.fromisoformat({expr_str}).strftime('%m/%d/%y') if isinstance({expr_str}, str) else {expr_str}.strftime('%m/%d/%y')"
        elif pattern == "iso":
            return f"datetime.fromisoformat({expr_str}).strftime('%Y-%m-%d') if isinstance({expr_str}, str) else {expr_str}.strftime('%Y-%m-%d')"
        return expr_str
    
    def _inline_decimal_formatting(self, expr_str: str, pattern: str) -> str:
        """Generate inline decimal formatting code."""
        if pattern == "0.00":
            return f"f'{{{expr_str}:.2f}}'"
        elif pattern == "#,##0.00":
            return f"f'{{{expr_str}:,.2f}}'"
        elif pattern == "$0.00":
            return f"f'${{{expr_str}:.2f}}'"
        elif pattern == "percent":
            return f"f'{{{expr_str}:.2f}}%'"
        return f"str({expr_str})"
    
    def _inline_number_formatting(self, expr_str: str, pattern: str) -> str:
        """Generate inline number formatting code."""
        if pattern == "#,##0":
            return f"f'{{{expr_str}:,}}'"
        elif pattern == "0000":
            return f"f'{{{expr_str}:04d}}'"
        elif pattern == "hex":
            return f"f'0x{{{expr_str}:X}}'"
        elif pattern == "oct":
            return f"f'0o{{{expr_str}:o}}'"
        elif pattern == "bin":
            return f"f'0b{{{expr_str}:b}}'"
        return f"str({expr_str})"
    
    def emit_api_call(self, stmt: ApiCallStatement):
        """Emit API call statement with framework or native HTTP."""
        if self.framework == "plain":
            self._emit_native_http_call(stmt)
        else:
            # FastAPI framework - not implemented for client calls yet
            self.main_code.append(f"# TODO: Implement FastAPI client call for {stmt.method} {stmt.url}")
    
    def emit_database_statement(self, stmt: DatabaseStatement):
        """Emit database statement with framework or native database access."""
        if self.framework == "plain":
            self._emit_native_database_operation(stmt)
        else:
            # FastAPI framework with SQLAlchemy - not implemented yet
            self.main_code.append(f"# TODO: Implement SQLAlchemy operation for {stmt.operation}")
    
    def emit_serve_statement(self, stmt: ServeStatement):
        """Emit serve statement with framework or native HTTP server."""
        if self.framework == "plain":
            self._emit_native_http_server(stmt)
        else:
            # FastAPI framework - handled by template generation
            self.main_code.append(f"# Serve {stmt.method} {stmt.endpoint} handled by FastAPI")
    
    def _emit_native_http_call(self, stmt: ApiCallStatement):
        """Generate native Python HTTP client call using http.client."""
        # Add imports
        if stmt.url.startswith('https://'):
            self.imports.add("import http.client")
            self.imports.add("import ssl")
        else:
            self.imports.add("import http.client")
        self.imports.add("import json")
        self.imports.add("from urllib.parse import urlparse")
        
        # Parse URL components
        self.main_code.append(f"# HTTP {stmt.method.upper()} request to {stmt.url}")
        self.main_code.append(f"url_parts = urlparse('{stmt.url}')")
        
        # Create connection
        if stmt.url.startswith('https://'):
            self.main_code.append("conn = http.client.HTTPSConnection(url_parts.netloc)")
        else:
            self.main_code.append("conn = http.client.HTTPConnection(url_parts.netloc)")
        
        # Prepare headers and body
        headers = {'Content-Type': 'application/json'} if stmt.body else {}
        self.main_code.append(f"headers = {headers}")
        
        if stmt.body:
            body_str = self.emit_expression(stmt.body)
            self.main_code.append(f"body = json.dumps({body_str})")
        else:
            self.main_code.append("body = None")
        
        # Make request
        self.main_code.append(f"conn.request('{stmt.method.upper()}', url_parts.path or '/', body, headers)")
        self.main_code.append("response = conn.getresponse()")
        self.main_code.append("response_data = response.read().decode('utf-8')")
        self.main_code.append("conn.close()")
        
        # Handle response
        if hasattr(stmt, 'response_var') and stmt.response_var:
            self.main_code.append(f"{stmt.response_var} = json.loads(response_data) if response_data else None")
        else:
            self.main_code.append("print(f'Response: {response.status} {response.reason}')")
            self.main_code.append("if response_data: print(response_data)")
    
    def _emit_native_database_operation(self, stmt: DatabaseStatement):
        """Generate native SQLite database operation."""
        self.imports.add("import sqlite3")
        self.imports.add("import os")
        
        # Database connection
        db_file = self.database.get('url', 'roelang.db').replace('sqlite:///', '')
        self.main_code.append(f"# Database operation: {stmt.operation}")
        self.main_code.append(f"conn = sqlite3.connect('{db_file}')")
        self.main_code.append("cursor = conn.cursor()")
        
        if stmt.operation == 'CREATE':
            # Create table operation
            table_name = getattr(stmt, 'table', 'data')
            self.main_code.append(f"cursor.execute('CREATE TABLE IF NOT EXISTS {table_name} (id INTEGER PRIMARY KEY)')")
        elif stmt.operation == 'INSERT':
            # Insert operation
            table_name = getattr(stmt, 'table', 'data')
            if hasattr(stmt, 'data') and stmt.data:
                data_str = self.emit_expression(stmt.data)
                self.main_code.append(f"cursor.execute('INSERT INTO {table_name} DEFAULT VALUES')")
            else:
                self.main_code.append(f"cursor.execute('INSERT INTO {table_name} DEFAULT VALUES')")
        elif stmt.operation == 'SELECT':
            # Select operation
            table_name = getattr(stmt, 'table', 'data')
            if hasattr(stmt, 'where') and stmt.where:
                where_str = self.emit_expression(stmt.where)
                self.main_code.append(f"cursor.execute('SELECT * FROM {table_name} WHERE {where_str}')")
            else:
                self.main_code.append(f"cursor.execute('SELECT * FROM {table_name}')")
            self.main_code.append("results = cursor.fetchall()")
            if hasattr(stmt, 'result_var') and stmt.result_var:
                self.main_code.append(f"{stmt.result_var} = results")
            else:
                self.main_code.append("for row in results: print(row)")
        elif stmt.operation == 'UPDATE':
            # Update operation
            table_name = getattr(stmt, 'table', 'data')
            self.main_code.append(f"cursor.execute('UPDATE {table_name} SET data = data')")
        elif stmt.operation == 'DELETE':
            # Delete operation
            table_name = getattr(stmt, 'table', 'data')
            if hasattr(stmt, 'where') and stmt.where:
                where_str = self.emit_expression(stmt.where)
                self.main_code.append(f"cursor.execute('DELETE FROM {table_name} WHERE {where_str}')")
            else:
                self.main_code.append(f"cursor.execute('DELETE FROM {table_name}')")
        
        self.main_code.append("conn.commit()")
        self.main_code.append("conn.close()")
    
    def _emit_native_http_server(self, stmt: ServeStatement):
        """Generate native Python HTTP server using http.server."""
        self.imports.add("import http.server")
        self.imports.add("import socketserver")
        self.imports.add("import json")
        self.imports.add("from urllib.parse import parse_qs, urlparse")
        
        # Create handler class
        handler_name = f"{stmt.method.capitalize()}{stmt.endpoint.replace('/', '_').replace(':', '')}Handler"
        
        handler_lines = [
            f"class {handler_name}(http.server.BaseHTTPRequestHandler):",
            f"    def do_{stmt.method.upper()}(self):",
            "        # Parse request path and query",
            "        parsed_path = urlparse(self.path)",
            "        query_params = parse_qs(parsed_path.query)",
            "        ",
            "        # Set response headers",
            "        self.send_response(200)",
            "        self.send_header('Content-Type', 'application/json')",
            "        self.end_headers()",
            "        ",
            "        # Process request body if present",
            "        if hasattr(self, 'headers') and 'Content-Length' in self.headers:",
            "            content_length = int(self.headers['Content-Length'])",
            "            request_body = self.rfile.read(content_length).decode('utf-8')",
            "            try:",
            "                request_data = json.loads(request_body) if request_body else {}",
            "            except json.JSONDecodeError:",
            "                request_data = {}",
            "        else:",
            "            request_data = {}",
            "        ",
            "        # Generate response",
            "        response_data = {'message': 'Hello from Roelang server', 'method': self.command, 'path': self.path}",
            "        self.wfile.write(json.dumps(response_data).encode('utf-8'))"
        ]
        
        self.class_definitions.append(handler_lines)
        
        # Add server startup code
        port = getattr(stmt, 'port', 8080)
        self.main_code.append(f"# Start HTTP server for {stmt.method.upper()} {stmt.endpoint}")
        self.main_code.append(f"PORT = {port}")
        self.main_code.append(f"with socketserver.TCPServer(('', PORT), {handler_name}) as httpd:")
        self.main_code.append(f"    print(f'Server running on http://localhost:{port}{stmt.endpoint}')")
        self.main_code.append("    httpd.serve_forever()")