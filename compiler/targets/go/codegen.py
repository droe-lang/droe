"""Go code generator for Roelang compiler."""

import os
from jinja2 import Environment, FileSystemLoader
from typing import List, Dict, Any
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


class GoCodeGenerator(BaseCodeGenerator):
    """Generates Go code from Roelang AST."""
    
    def __init__(self, framework=None, database_config=None, package=None):
        super().__init__()
        self.framework = framework
        self.database_config = database_config or {}
        self.package = package
        self.imports = set()
        self.type_definitions = []
        self.function_definitions = []
        self.main_code = []
        self.variables = {}  # Track variable types for Go typing
        
        # Setup Jinja2 environment for templates
        template_dir = os.path.join(os.path.dirname(__file__), 'templates', self.framework or 'fiber')
        self.jinja_env = Environment(loader=FileSystemLoader(template_dir))
        
        # Collect data for template context
        self.data_definitions = []
        self.serve_actions = []
    
    def generate(self, program: Program) -> str:
        """Generate Go code from AST."""
        if self.framework == 'fiber':
            return self._generate_fiber_project(program)
        
        # Fallback to original Go generation for non-framework mode
        self.clear_output()
        self.imports.clear()
        self.type_definitions.clear()
        self.function_definitions.clear()
        self.main_code.clear()
        self.variables.clear()
        
        # Add core imports
        self.imports.add("fmt")
        self.imports.add("strconv")
        self.imports.add("time")
        
        if self.is_core_lib_enabled('math_utils'):
            self.imports.add("math")
        
        if self.is_core_lib_enabled('string_utils'):
            self.imports.add("strings")
        
        # Process all statements
        for stmt in program.statements:
            self.emit_statement(stmt)
        
        # Generate final Go code
        return self._build_go_file()
    
    def _generate_fiber_project(self, program: Program) -> str:
        """Generate Fiber web framework project."""
        # Parse AST to extract data definitions and serve actions
        self._parse_ast_for_fiber(program)
        
        # Prepare template context
        context = {
            'package_name': self.package or 'go_fiber_app',  # Use configured package name
            'data_definitions': self.data_definitions,
            'actions': self.serve_actions,
            'database': self.database_config
        }
        
        # Generate project files
        project_files = {}
        
        # Go module file
        template = self.jinja_env.get_template('go.mod.jinja2')
        project_files['go.mod'] = template.render(context)
        
        # Main server file
        template = self.jinja_env.get_template('main.go.jinja2')
        project_files['main.go'] = template.render(context)
        
        # Models file
        template = self.jinja_env.get_template('models.go.jinja2')
        project_files['models.go'] = template.render(context)
        
        # Database file
        template = self.jinja_env.get_template('database.go.jinja2')
        project_files['database.go'] = template.render(context)
        
        # Routes file
        template = self.jinja_env.get_template('routes.go.jinja2')
        project_files['routes.go'] = template.render(context)
        
        # Handlers file
        template = self.jinja_env.get_template('handlers.go.jinja2')
        project_files['handlers.go'] = template.render(context)
        
        # Environment file
        template = self.jinja_env.get_template('.env.jinja2')
        project_files['.env'] = template.render(context)
        
        # Return in the format expected by the target factory
        return {
            'files': project_files,
            'project_root': context['package_name'],
            'language': 'go',
            'framework': self.framework
        }
    
    def _parse_ast_for_fiber(self, program: Program):
        """Parse AST to extract data definitions and serve actions."""
        for stmt in program.statements:
            if isinstance(stmt, DataDefinition):
                # Convert DataDefinition to template-friendly format
                data_def = {
                    'name': stmt.name,
                    'fields': []
                }
                
                for field in stmt.fields:
                    field_info = {
                        'name': field.name,
                        'type': str(field.type).lower(),
                        'required': 'required' in getattr(field, 'annotations', [])
                    }
                    data_def['fields'].append(field_info)
                
                self.data_definitions.append(data_def)
            
            elif isinstance(stmt, ServeStatement):
                # Convert ServeStatement to template-friendly format
                # Generate action name from method and endpoint
                action_name = self._generate_action_name(stmt.method, stmt.endpoint)
                action = {
                    'name': action_name,
                    'method': stmt.method.upper(),
                    'path': stmt.endpoint
                }
                self.serve_actions.append(action)
    
    def _generate_action_name(self, method: str, endpoint: str) -> str:
        """Generate action name from HTTP method and endpoint."""
        # Convert /api/users -> GetUsers, /api/users/:id -> GetUser
        path_parts = [part for part in endpoint.strip('/').split('/') if not part.startswith(':')]
        if path_parts:
            resource = path_parts[-1].title()  # 'users' -> 'Users'
            if method.lower() == 'get' and ':id' in endpoint:
                # GET /api/users/:id -> GetUser (singular)
                resource = resource.rstrip('s') if resource.endswith('s') else resource
                return f"Get{resource}"
            elif method.lower() == 'get':
                # GET /api/users -> GetUsers (plural)
                return f"Get{resource}"
            elif method.lower() == 'post':
                # POST /api/users -> CreateUser
                resource = resource.rstrip('s') if resource.endswith('s') else resource
                return f"Create{resource}"
            elif method.lower() == 'put':
                # PUT /api/users/:id -> UpdateUser
                resource = resource.rstrip('s') if resource.endswith('s') else resource
                return f"Update{resource}"
            elif method.lower() == 'delete':
                # DELETE /api/users/:id -> DeleteUser
                resource = resource.rstrip('s') if resource.endswith('s') else resource
                return f"Delete{resource}"
        
        # Fallback
        return f"{method.title()}Handler"
    
    def _build_go_file(self) -> str:
        """Build the complete Go file."""
        lines = []
        
        # Package declaration
        lines.append("package main")
        lines.append("")
        
        # Add imports
        if self.imports:
            lines.append("import (")
            for imp in sorted(self.imports):
                lines.append(f'\\t"{imp}"')
            lines.append(")")
            lines.append("")
        
        # No separate runtime library needed - using inline code generation
        
        # Add type definitions
        for type_def in self.type_definitions:
            lines.extend(type_def)
            lines.append("")
        
        # Add function definitions
        for func_def in self.function_definitions:
            lines.extend(func_def)
            lines.append("")
        
        # Add main function
        lines.append("func main() {")
        for line in self.main_code:
            lines.append(f"\\t{line}")
        lines.append("}")
        
        return "\n".join(lines)
    
    
    def _get_go_type(self, var_type: VariableType) -> str:
        """Get Go type for Roelang type."""
        type_map = {
            VariableType.INT: "int",
            VariableType.NUMBER: "int", 
            VariableType.DECIMAL: "float64",
            VariableType.TEXT: "string",
            VariableType.STRING: "string",
            VariableType.FLAG: "bool",
            VariableType.YESNO: "bool",
            VariableType.BOOLEAN: "bool",
            VariableType.DATE: "string",  # Store as ISO string
            VariableType.LIST_OF: "[]interface{}",
            VariableType.GROUP_OF: "[]interface{}",
            VariableType.ARRAY: "[]interface{}",
        }
        return type_map.get(var_type, "interface{}")
    
    def emit_statement(self, stmt: ASTNode):
        """Emit code for a statement."""
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
            self.main_code.append(f"// TODO: Implement {type(stmt).__name__}")
    
    def emit_expression(self, expr: ASTNode) -> str:
        """Emit code for an expression and return the expression string."""
        if isinstance(expr, Literal):
            if isinstance(expr.value, str):
                return f'"{expr.value}"'
            elif isinstance(expr.value, bool):
                return "true" if expr.value else "false"
            else:
                return str(expr.value)
        elif isinstance(expr, Identifier):
            return expr.name
        elif isinstance(expr, BinaryOp):
            left = self.emit_expression(expr.left)
            right = self.emit_expression(expr.right)
            
            # Handle Go-specific operators
            if expr.operator == "==":
                return f"({left} == {right})"
            elif expr.operator == "!=":
                return f"({left} != {right})"
            else:
                return f"({left} {expr.operator} {right})"
        elif isinstance(expr, ArithmeticOp):
            left = self.emit_expression(expr.left)
            right = self.emit_expression(expr.right)
            return f"({left} {expr.operator} {right})"
        elif isinstance(expr, ArrayLiteral):
            elements = [self.emit_expression(elem) for elem in expr.elements]
            return f"[]interface{{{{{', '.join(elements)}}}}}"
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
            return f"/* TODO: {type(expr).__name__} */"
    
    def emit_display_statement(self, stmt: DisplayStatement):
        """Emit display statement with native formatting."""
        expr_str = self.emit_expression(stmt.expression)
        expr_type = self.infer_type(stmt.expression)
        
        # Handle boolean formatting inline
        if expr_type == VariableType.BOOLEAN or expr_type == VariableType.FLAG or expr_type == VariableType.YESNO:
            self.main_code.append(f'if {expr_str} {{ fmt.Println("true") }} else {{ fmt.Println("false") }}')
        # Handle slice formatting inline  
        elif expr_type in [VariableType.LIST_OF, VariableType.GROUP_OF, VariableType.ARRAY]:
            self.main_code.append(f'fmt.Print("["); for i, item := range {expr_str} {{ if i > 0 {{ fmt.Print(", ") }}; fmt.Print(item) }}; fmt.Println("]")')
        else:
            self.main_code.append(f"fmt.Println({expr_str})")
    
    def emit_assignment(self, stmt: Assignment):
        """Emit assignment statement."""
        value_str = self.emit_expression(stmt.value)
        
        # Handle variable declaration vs assignment
        if stmt.variable not in self.variables:
            # First assignment - use Go's type inference
            self.main_code.append(f"{stmt.variable} := {value_str}")
            # Try to infer type for tracking
            inferred_type = self.infer_type(stmt.value)
            go_type = self._get_go_type(inferred_type)
            self.variables[stmt.variable] = go_type
        else:
            # Reassignment
            self.main_code.append(f"{stmt.variable} = {value_str}")
    
    def emit_if_statement(self, stmt: IfStatement):
        """Emit if statement."""
        condition_str = self.emit_expression(stmt.condition)
        self.main_code.append(f"if {condition_str} {{")
        
        # Emit then block (simplified)
        if stmt.then_body:
            for then_stmt in stmt.then_body:
                if isinstance(then_stmt, DisplayStatement):
                    expr_str = self.emit_expression(then_stmt.expression)
                    self.main_code.append(f"\\tDisplay({expr_str})")
        
        if stmt.else_body:
            self.main_code.append("} else {")
            for else_stmt in stmt.else_body:
                if isinstance(else_stmt, DisplayStatement):
                    expr_str = self.emit_expression(else_stmt.expression)
                    self.main_code.append(f"\\tDisplay({expr_str})")
        
        self.main_code.append("}")
    
    def emit_while_loop(self, stmt: WhileLoop):
        """Emit while loop."""
        condition_str = self.emit_expression(stmt.condition)
        self.main_code.append(f"for {condition_str} {{")
        self.main_code.append("\\t// Loop body")
        self.main_code.append("}")
    
    def emit_foreach_loop(self, stmt: ForEachLoop):
        """Emit for-each loop."""
        collection_str = self.emit_expression(stmt.collection)
        self.main_code.append(f"for _, {stmt.variable} := range {collection_str} {{")
        self.main_code.append("\\t// Loop body")
        self.main_code.append("}")
    
    def emit_action_definition(self, stmt: ActionDefinition):
        """Emit action definition as Go function."""
        func_lines = [f"func {stmt.name}() interface{{}} {{"]
        
        if stmt.body:
            for body_stmt in stmt.body:
                if isinstance(body_stmt, ReturnStatement):
                    return_expr = self.emit_expression(body_stmt.expression)
                    func_lines.append(f"\\treturn {return_expr}")
                else:
                    func_lines.append("\\t// Function body")
        else:
            func_lines.append("\\treturn nil")
        
        func_lines.append("}")
        self.function_definitions.append(func_lines)
    
    def emit_module_definition(self, stmt: ModuleDefinition):
        """Emit module definition as Go struct."""
        type_lines = [f"type {stmt.name} struct {{", "}"]
        self.type_definitions.append(type_lines)
        
        # Add methods for the struct
        if stmt.body:
            for body_stmt in stmt.body:
                if isinstance(body_stmt, ActionDefinition):
                    method_lines = [
                        f"func ({stmt.name[0].lower()}) {stmt.name}) {body_stmt.name}() interface{{}} {{",
                        "\\t// Method body",
                        "\\treturn nil",
                        "}"
                    ]
                    self.function_definitions.append(method_lines)
    
    def emit_string_interpolation(self, expr: StringInterpolation) -> str:
        """Emit string interpolation using fmt.Sprintf."""
        format_parts = []
        args = []
        
        for part in expr.parts:
            if isinstance(part, str):
                format_parts.append(part)
            else:
                format_parts.append("%v")
                args.append(self.emit_expression(part))
        
        format_string = ''.join(format_parts)
        if args:
            args_str = ', '.join(args)
            return f'fmt.Sprintf("{format_string}", {args_str})'
        else:
            return f'"{format_string}"'
    
    def _inline_date_formatting(self, expr_str: str, pattern: str) -> str:
        """Generate inline date formatting code."""
        if pattern == "MM/dd/yyyy":
            return f'func() string {{ t, _ := time.Parse("2006-01-02", {expr_str}); return t.Format("01/02/2006") }}()'
        elif pattern == "dd/MM/yyyy":
            return f'func() string {{ t, _ := time.Parse("2006-01-02", {expr_str}); return t.Format("02/01/2006") }}()'
        elif pattern == "MMM dd, yyyy":
            return f'func() string {{ t, _ := time.Parse("2006-01-02", {expr_str}); return t.Format("Jan 02, 2006") }}()'
        elif pattern == "long":
            return f'func() string {{ t, _ := time.Parse("2006-01-02", {expr_str}); return t.Format("Monday, January 02, 2006") }}()'
        elif pattern == "short":
            return f'func() string {{ t, _ := time.Parse("2006-01-02", {expr_str}); return t.Format("01/02/06") }}()'
        elif pattern == "iso":
            return f'func() string {{ t, _ := time.Parse("2006-01-02", {expr_str}); return t.Format("2006-01-02") }}()'
        return expr_str
    
    def _inline_decimal_formatting(self, expr_str: str, pattern: str) -> str:
        """Generate inline decimal formatting code."""
        if pattern == "0.00":
            return f'fmt.Sprintf("%.2f", {expr_str})'
        elif pattern == "$0.00":
            return f'fmt.Sprintf("$%.2f", {expr_str})'
        elif pattern == "percent":
            return f'fmt.Sprintf("%.2f%%", {expr_str})'
        return f'fmt.Sprintf("%.2f", {expr_str})'
    
    def _inline_number_formatting(self, expr_str: str, pattern: str) -> str:
        """Generate inline number formatting code."""
        if pattern == "0000":
            return f'fmt.Sprintf("%04d", {expr_str})'
        elif pattern == "hex":
            return f'fmt.Sprintf("0x%X", {expr_str})'
        elif pattern == "oct":
            return f'fmt.Sprintf("0o%o", {expr_str})'
        elif pattern == "bin":
            return f'fmt.Sprintf("0b%b", {expr_str})'
        return f'fmt.Sprintf("%d", {expr_str})'
    
    def emit_api_call(self, stmt: ApiCallStatement):
        """Emit API call statement with framework or native HTTP."""
        if self.framework == "plain":
            self._emit_native_http_call(stmt)
        else:
            # Fiber framework - not implemented for client calls yet
            self.main_code.append(f"// TODO: Implement Fiber client call for {stmt.method} {stmt.url}")
    
    def emit_database_statement(self, stmt: DatabaseStatement):
        """Emit database statement with framework or native database access."""
        if self.framework == "plain":
            self._emit_native_database_operation(stmt)
        else:
            # Fiber framework with GORM - not implemented yet
            self.main_code.append(f"// TODO: Implement GORM operation for {stmt.operation}")
    
    def emit_serve_statement(self, stmt: ServeStatement):
        """Emit serve statement with framework or native HTTP server."""
        if self.framework == "plain":
            self._emit_native_http_server(stmt)
        else:
            # Fiber framework - handled by template generation
            self.main_code.append(f"// Serve {stmt.method} {stmt.endpoint} handled by Fiber")
    
    def _emit_native_http_call(self, stmt: ApiCallStatement):
        """Generate native Go HTTP client call using net/http."""
        # Add required imports
        self.imports.add("net/http")
        self.imports.add("io")
        self.imports.add("encoding/json")
        self.imports.add("bytes")
        self.imports.add("fmt")
        
        # Generate HTTP request code
        self.main_code.append(f"// HTTP {stmt.method.upper()} request to {stmt.url}")
        
        if stmt.body:
            body_str = self.emit_expression(stmt.body)
            self.main_code.append(f"requestData, err := json.Marshal({body_str})")
            self.main_code.append("if err != nil {")
            self.main_code.append("\tlog.Fatal(err)")
            self.main_code.append("}")
            self.main_code.append(f'req, err := http.NewRequest("{stmt.method.upper()}", "{stmt.url}", bytes.NewBuffer(requestData))')
            self.main_code.append("req.Header.Set(\"Content-Type\", \"application/json\")")
        else:
            self.main_code.append(f'req, err := http.NewRequest("{stmt.method.upper()}", "{stmt.url}", nil)')
        
        self.main_code.append("if err != nil {")
        self.main_code.append("\tlog.Fatal(err)")
        self.main_code.append("}")
        
        # Make the request
        self.main_code.append("client := &http.Client{}")
        self.main_code.append("resp, err := client.Do(req)")
        self.main_code.append("if err != nil {")
        self.main_code.append("\tlog.Fatal(err)")
        self.main_code.append("}")
        self.main_code.append("defer resp.Body.Close()")
        
        # Handle response
        self.main_code.append("body, err := io.ReadAll(resp.Body)")
        self.main_code.append("if err != nil {")
        self.main_code.append("\tlog.Fatal(err)")
        self.main_code.append("}")
        
        if hasattr(stmt, 'response_var') and stmt.response_var:
            self.main_code.append(f"var {stmt.response_var} interface{{}}")
            self.main_code.append(f"json.Unmarshal(body, &{stmt.response_var})")
        else:
            self.main_code.append('fmt.Printf("Response Status: %s\\n", resp.Status)')
            self.main_code.append('fmt.Printf("Response Body: %s\\n", string(body))')
    
    def _emit_native_database_operation(self, stmt: DatabaseStatement):
        """Generate native Go database operation using database/sql."""
        # Add required imports
        self.imports.add("database/sql")
        self.imports.add("fmt")
        self.imports.add("log")
        # Add driver import based on database type
        if hasattr(self, 'database') and self.database.get('type') == 'postgres':
            self.imports.add('_ "github.com/lib/pq"')
        elif hasattr(self, 'database') and self.database.get('type') == 'mysql':
            self.imports.add('_ "github.com/go-sql-driver/mysql"')
        else:
            self.imports.add('_ "github.com/mattn/go-sqlite3"')
        
        # Database connection
        if hasattr(self, 'database') and self.database.get('url'):
            db_url = self.database['url']
        else:
            db_url = "roelang.db"
        
        self.main_code.append(f"// Database operation: {stmt.operation}")
        self.main_code.append(f'db, err := sql.Open("sqlite3", "{db_url}")')
        self.main_code.append("if err != nil {")
        self.main_code.append("\tlog.Fatal(err)")
        self.main_code.append("}")
        self.main_code.append("defer db.Close()")
        
        if stmt.operation == 'CREATE':
            # Create table operation
            table_name = getattr(stmt, 'table', 'data')
            self.main_code.append(f'_, err = db.Exec("CREATE TABLE IF NOT EXISTS {table_name} (id INTEGER PRIMARY KEY)")')
        elif stmt.operation == 'INSERT':
            # Insert operation
            table_name = getattr(stmt, 'table', 'data')
            self.main_code.append(f'_, err = db.Exec("INSERT INTO {table_name} DEFAULT VALUES")')
        elif stmt.operation == 'SELECT':
            # Select operation
            table_name = getattr(stmt, 'table', 'data')
            if hasattr(stmt, 'where') and stmt.where:
                where_str = self.emit_expression(stmt.where)
                self.main_code.append(f'rows, err := db.Query("SELECT * FROM {table_name} WHERE {where_str}")')
            else:
                self.main_code.append(f'rows, err := db.Query("SELECT * FROM {table_name}")')
            
            self.main_code.append("if err != nil {")
            self.main_code.append("\tlog.Fatal(err)")
            self.main_code.append("}")
            self.main_code.append("defer rows.Close()")
            
            if hasattr(stmt, 'result_var') and stmt.result_var:
                self.main_code.append(f"var {stmt.result_var} []interface{{}}")
                self.main_code.append("for rows.Next() {")
                self.main_code.append("\tvar row interface{}")
                self.main_code.append("\trows.Scan(&row)")
                self.main_code.append(f"\t{stmt.result_var} = append({stmt.result_var}, row)")
                self.main_code.append("}")
            else:
                self.main_code.append("for rows.Next() {")
                self.main_code.append("\tvar row interface{}")
                self.main_code.append("\trows.Scan(&row)")
                self.main_code.append("\tfmt.Println(row)")
                self.main_code.append("}")
            return  # Skip the general error check since we handled it
        elif stmt.operation == 'UPDATE':
            # Update operation
            table_name = getattr(stmt, 'table', 'data')
            self.main_code.append(f'_, err = db.Exec("UPDATE {table_name} SET data = data")')
        elif stmt.operation == 'DELETE':
            # Delete operation
            table_name = getattr(stmt, 'table', 'data')
            if hasattr(stmt, 'where') and stmt.where:
                where_str = self.emit_expression(stmt.where)
                self.main_code.append(f'_, err = db.Exec("DELETE FROM {table_name} WHERE {where_str}")')
            else:
                self.main_code.append(f'_, err = db.Exec("DELETE FROM {table_name}")')
        
        self.main_code.append("if err != nil {")
        self.main_code.append("\tlog.Fatal(err)")
        self.main_code.append("}")
    
    def _emit_native_http_server(self, stmt: ServeStatement):
        """Generate native Go HTTP server using net/http."""
        # Add required imports
        self.imports.add("net/http")
        self.imports.add("fmt")
        self.imports.add("log")
        self.imports.add("encoding/json")
        self.imports.add("io")
        
        # Generate handler function
        handler_name = f"{stmt.method.lower()}{stmt.endpoint.replace('/', '_').replace(':', '')}_handler"
        
        handler_lines = [
            f"func {handler_name}(w http.ResponseWriter, r *http.Request) {{",
            "\t// Set content type",
            "\tw.Header().Set(\"Content-Type\", \"application/json\")",
            "\t",
            f"\t// Check method",
            f"\tif r.Method != \"{stmt.method.upper()}\" {{",
            "\t\thttp.Error(w, \"Method not allowed\", http.StatusMethodNotAllowed)",
            "\t\treturn",
            "\t}",
            "\t",
            "\t// Read request body",
            "\tbody, err := io.ReadAll(r.Body)",
            "\tif err != nil {",
            "\t\thttp.Error(w, \"Error reading body\", http.StatusBadRequest)",
            "\t\treturn",
            "\t}",
            "\t",
            "\t// Process request (placeholder)",
            "\tvar requestData interface{}",
            "\tif len(body) > 0 {",
            "\t\tjson.Unmarshal(body, &requestData)",
            "\t}",
            "\t",
            "\t// Generate response",
            '\tresponse := map[string]interface{}{',
            '\t\t"message": "Hello from Roelang server",',
            f'\t\t"method": "{stmt.method.upper()}",',
            f'\t\t"path": "{stmt.endpoint}",',
            "\t\t\"data\": requestData,",
            "\t}",
            "\t",
            "\t// Send JSON response",
            "\tjsonResponse, _ := json.Marshal(response)",
            "\tw.Write(jsonResponse)",
            "}"
        ]
        
        self.function_definitions.append(handler_lines)
        
        # Add server startup code
        port = getattr(stmt, 'port', 8080)
        self.main_code.append(f"// Start HTTP server for {stmt.method.upper()} {stmt.endpoint}")
        self.main_code.append(f'http.HandleFunc("{stmt.endpoint}", {handler_name})')
        self.main_code.append(f'fmt.Println("Server starting on http://localhost:{port}{stmt.endpoint}")')
        self.main_code.append(f'log.Fatal(http.ListenAndServe(":{port}", nil))')