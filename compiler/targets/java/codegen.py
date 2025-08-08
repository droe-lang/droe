"""Java code generator for Roelang compiler."""

from typing import List, Dict, Any, Optional
from pathlib import Path
from ...ast import (
    ASTNode, Program, DisplayStatement, IfStatement,
    Literal, Identifier, BinaryOp, PropertyAccess,
    Assignment, ArrayLiteral, WhileLoop, ForEachLoop, ArithmeticOp,
    TaskAction, TaskInvocation, ActionDefinition, ReturnStatement, ActionInvocation,
    ModuleDefinition, DataDefinition, DataField, ActionDefinitionWithParams,
    ActionParameter, ActionInvocationWithArgs, StringInterpolation,
    DataInstance, FieldAssignment, FormatExpression, ServeStatement, AcceptStatement,
    RespondStatement, ParamsStatement, DatabaseStatement, ApiCallStatement
)
from ...symbols import SymbolTable, VariableType
from ...codegen_base import BaseCodeGenerator, CodeGenError


class JavaCodeGenerator(BaseCodeGenerator):
    """Generates Java code from Roelang AST."""
    
    def __init__(self, source_file_path: Optional[str] = None, is_main_file: bool = False, framework: str = None, package: Optional[str] = None, database: Optional[Dict[str, Any]] = None):
        super().__init__()
        self.source_file_path = source_file_path
        self.is_main_file = is_main_file
        self.framework = framework or "plain"  # Support for different frameworks
        self.package = package  # Custom package name
        self.database = database or {}  # Database configuration
        self.class_name = "RoelangProgram"  # Default
        self.imports = set()
        self.fields = []  # Instance variables
        self.constructor_code = []  # Procedural code goes here
        self.methods = []  # Action definitions become methods
        self.module_classes = []  # Module definitions become separate classes
        self.has_modules = False
        self.spring_boot_config = {}  # Spring Boot specific configuration
        self.jpa_entities = []  # JPA entity classes
        self.api_controllers = []  # REST controllers
        self.services = []  # Service classes
        self.repositories = []  # Repository interfaces
        
        # Determine class name from file or detect modules
        if source_file_path:
            file_name = Path(source_file_path).stem
            # Convert to PascalCase for Java class naming
            self.class_name = self._to_pascal_case(file_name)
    
    def _to_pascal_case(self, name: str) -> str:
        """Convert name to PascalCase for Java class naming."""
        # Handle snake_case, kebab-case, or camelCase
        parts = name.replace('-', '_').replace(' ', '_').split('_')
        class_name = ''.join(word.capitalize() for word in parts if word)
        
        # Java class names cannot start with numbers - strip leading digits
        if class_name:
            class_name = class_name.lstrip('0123456789')
        
        # Ensure we have a valid class name
        if not class_name or not class_name[0].isalpha():
            class_name = 'RoelangProgram'
            
        return class_name
    
    def _get_java_type(self, var_type: VariableType) -> str:
        """Get Java type for Roelang type."""
        type_map = {
            VariableType.INT: "int",
            VariableType.NUMBER: "int", 
            VariableType.DECIMAL: "double",
            VariableType.TEXT: "String",
            VariableType.STRING: "String",
            VariableType.FLAG: "boolean",
            VariableType.YESNO: "boolean",
            VariableType.BOOLEAN: "boolean",
            VariableType.DATE: "String",  # Store as ISO string, could use LocalDate
            VariableType.LIST_OF: "List<Object>",
            VariableType.GROUP_OF: "List<Object>",
            VariableType.ARRAY: "List<Object>",
            VariableType.FILE: "String",
        }
        
        java_type = type_map.get(var_type, "Object")
        
        # Add java.util.* import for List types
        if java_type.startswith("List<"):
            self.imports.add("java.util.*")
        
        return java_type
    
    def _get_java_type_from_declared(self, declared_type: str) -> str:
        """Get Java type from declared compound type like 'list_of_int'."""
        # Handle compound collection types
        if declared_type.startswith('list_of_'):
            element_type = declared_type[8:]  # Remove 'list_of_' prefix
            java_element_type = self._get_java_element_type(element_type)
            # Add java.util.* import and use short form
            self.imports.add("java.util.*")
            return f"List<{java_element_type}>"
        elif declared_type.startswith('group_of_'):
            element_type = declared_type[9:]  # Remove 'group_of_' prefix  
            java_element_type = self._get_java_element_type(element_type)
            # Add java.util.* import and use short form
            self.imports.add("java.util.*")
            return f"List<{java_element_type}>"
        else:
            # Fall back to regular type mapping
            internal_type = self.map_user_type_to_internal(declared_type)
            return self._get_java_type(internal_type)
    
    def _get_java_element_type(self, element_type: str) -> str:
        """Get Java wrapper type for collection elements."""
        element_map = {
            'int': 'Integer',
            'decimal': 'Double', 
            'text': 'String',
            'flag': 'Boolean',
            'yesno': 'Boolean',
            'boolean': 'Boolean',
            'date': 'String',
            'number': 'Integer',
            'string': 'String',
        }
        return element_map.get(element_type, 'Object')
    
    def generate(self, program: Program) -> str:
        """Generate Java code from AST."""
        self.clear_output()
        self.imports.clear()
        self.fields.clear()
        self.constructor_code.clear()
        self.methods.clear()
        self.module_classes.clear()
        self.jpa_entities.clear()
        self.api_controllers.clear()
        self.services.clear()
        self.repositories.clear()
        self.has_modules = False
        
        # Add core imports
        self.imports.add("java.util.*")
        self.imports.add("java.time.*")
        self.imports.add("java.time.format.*")
        self.imports.add("java.text.*")
        
        # Setup framework-specific imports
        if self.framework == "spring":
            self._setup_spring_boot_imports()
        
        # First pass: Check for modules and extract them
        modules_found = []
        data_definitions = []
        serve_modules = []
        non_module_statements = []
        
        for stmt in program.statements:
            if isinstance(stmt, ModuleDefinition):
                modules_found.append(stmt)
                self.has_modules = True
                # Check if module has serve statements (API module)
                if any(isinstance(s, ServeStatement) for s in stmt.body):
                    serve_modules.append(stmt)
                # Check if module has data definitions
                for body_stmt in stmt.body:
                    if isinstance(body_stmt, DataDefinition):
                        data_definitions.append(body_stmt)
            elif isinstance(stmt, DataDefinition):
                data_definitions.append(stmt)
            else:
                non_module_statements.append(stmt)
        
        # Generate Spring Boot components if framework is spring
        if self.framework == "spring":
            from .spring_generator import SpringBootGenerator
            spring_gen = SpringBootGenerator()
            project_name = self.class_name.lower().replace("application", "")
            
            # Use provided package or generate default
            package_name = self.package or f"com.example.{project_name}"
            
            # Generate Spring Boot project using templates
            project_result = spring_gen.generate_spring_boot_project(
                program=program, 
                project_name=project_name,
                package_name=package_name,
                database_config=self.database
            )
            # Return the complete result dictionary for file creation
            return project_result
        
        # Traditional approach for plain Java
        if modules_found:
            # Process all modules as separate classes
            for module in modules_found:
                self._process_module(module, is_main=False)
        
        # Process non-module statements (procedural code)
        for stmt in non_module_statements:
            self.emit_statement(stmt)
        
        # Generate final Java code
        return self._build_java_file()
    
    def _process_module(self, module: ModuleDefinition, is_main: bool = False):
        """Process a module definition."""
        if is_main:
            # Main module becomes the main class
            for stmt in module.body:
                if isinstance(stmt, ActionDefinition):
                    self._add_method_from_action(stmt)
                elif isinstance(stmt, ActionDefinitionWithParams):
                    self._add_method_from_parameterized_action(stmt)
                else:
                    # Other statements in main module go to constructor
                    self.emit_statement(stmt)
        else:
            # Other modules become separate classes
            module_class = self._generate_module_class(module)
            self.module_classes.append(module_class)
    
    def _generate_module_class(self, module: ModuleDefinition) -> List[str]:
        """Generate a separate class for a module."""
        lines = []
        lines.append(f"class {module.name} {{")
        
        # Add methods from module actions
        for stmt in module.body:
            if isinstance(stmt, ActionDefinition):
                method_lines = self._generate_method_from_action(stmt, is_static=True)
                for line in method_lines:
                    lines.append(f"    {line}")
            elif isinstance(stmt, ActionDefinitionWithParams):
                method_lines = self._generate_method_from_parameterized_action(stmt, is_static=True)
                for line in method_lines:
                    lines.append(f"    {line}")
        
        lines.append("}")
        return lines
    
    def _add_method_from_action(self, action: ActionDefinition):
        """Add method from action to main class methods."""
        method_lines = self._generate_method_from_action(action, is_static=False)
        self.methods.extend(method_lines)
    
    def _add_method_from_parameterized_action(self, action: ActionDefinitionWithParams):
        """Add method from parameterized action to main class methods."""
        method_lines = self._generate_method_from_parameterized_action(action, is_static=False)
        self.methods.extend(method_lines)
    
    def _generate_method_from_action(self, action: ActionDefinition, is_static: bool = False) -> List[str]:
        """Generate method lines from action definition."""
        lines = []
        static_modifier = "static " if is_static else ""
        
        # Determine return type (simplified - could be enhanced with proper type inference)
        return_type = "Object"  # Default
        if action.body:
            for stmt in action.body:
                if isinstance(stmt, ReturnStatement):
                    inferred_type = self.infer_type(stmt.expression)
                    return_type = self._get_java_type(inferred_type)
                    break
        
        lines.append(f"public {static_modifier}{return_type} {action.name}() {{")
        
        # Add method body
        if action.body:
            for stmt in action.body:
                if isinstance(stmt, ReturnStatement):
                    expr_str = self.emit_expression(stmt.expression)
                    lines.append(f"    return {expr_str};")
                else:
                    # Process other statements in method
                    lines.append("    // Method body statement")
        else:
            lines.append("    return null;")
        
        lines.append("}")
        return lines
    
    def _generate_method_from_parameterized_action(self, action: ActionDefinitionWithParams, is_static: bool = False) -> List[str]:
        """Generate method lines from parameterized action definition."""
        lines = []
        static_modifier = "static " if is_static else ""
        
        # Build parameter list
        params = []
        if action.parameters:
            for param in action.parameters:
                param_type = self._get_java_type(self.map_user_type_to_internal(param.type))
                params.append(f"{param_type} {param.name}")
        
        param_list = ", ".join(params)
        
        # Determine return type
        return_type = "Object"
        if action.return_type:
            return_type = self._get_java_type(self.map_user_type_to_internal(action.return_type))
        
        lines.append(f"public {static_modifier}{return_type} {action.name}({param_list}) {{")
        
        # Add method body
        if action.body:
            for stmt in action.body:
                if isinstance(stmt, ReturnStatement):
                    expr_str = self.emit_expression(stmt.expression)
                    lines.append(f"    return {expr_str};")
                else:
                    lines.append("    // Method body statement")
        else:
            if return_type == "void":
                pass  # No return needed
            else:
                lines.append("    return null;")
        
        lines.append("}")
        return lines
    
    def _build_java_file(self) -> str:
        """Build the complete Java file."""
        lines = []
        
        # Add imports (including runtime)
        standard_imports = sorted(self.imports)
        if standard_imports:
            for imp in standard_imports:
                lines.append(f"import {imp};")
        
        # No runtime import needed - using inline code generation
        lines.append("")
        
        # Add module classes (if any)
        for module_class in self.module_classes:
            lines.extend(module_class)
            lines.append("")
        
        # Main class - only public if it matches the filename
        class_modifier = "public " if self.source_file_path and Path(self.source_file_path).stem.lower() == self.class_name.lower() else ""
        lines.append(f"{class_modifier}class {self.class_name} {{")
        
        # Add instance fields
        for field in self.fields:
            lines.append(f"    {field}")
        if self.fields:
            lines.append("")
        
        # Add constructor (with procedural code)
        if self.constructor_code:
            lines.append(f"    public {self.class_name}() {{")
            for line in self.constructor_code:
                lines.append(f"        {line}")
            lines.append("    }")
            lines.append("")
        
        # Add methods (from actions)
        for method_line in self.methods:
            lines.append(f"    {method_line}")
        if self.methods:
            lines.append("")
        
        # Add main method for standalone execution
        # Always add main method unless this is clearly a library/module file
        should_add_main = self.is_main_file or not self.has_modules
        if should_add_main:
            lines.extend([
                "    public static void main(String[] args) {",
                f"        new {self.class_name}();",
                "    }",
                ""
            ])
        
        lines.append("}")
        
        return "\n".join(lines)
    
    
    def emit_statement(self, stmt: ASTNode):
        """Emit code for a statement (goes to constructor for procedural code)."""
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
            # Actions become methods, don't add to constructor
            self._add_method_from_action(stmt)
        elif isinstance(stmt, ActionDefinitionWithParams):
            # Parameterized actions become methods
            self._add_method_from_parameterized_action(stmt)
        elif isinstance(stmt, ApiCallStatement):
            self.emit_api_call(stmt)
        elif isinstance(stmt, DatabaseStatement):
            self.emit_database_statement(stmt)
        elif isinstance(stmt, ModuleDefinition):
            # Modules are handled separately
            pass
        else:
            self.constructor_code.append(f"// TODO: Implement {type(stmt).__name__}")
    
    def emit_expression(self, expr: ASTNode) -> str:
        """Emit code for an expression."""
        if isinstance(expr, Literal):
            if isinstance(expr.value, str):
                # Escape Java string literals
                escaped = expr.value.replace('\\\\', '\\\\\\\\').replace('"', '\\\\"').replace('\\n', '\\\\n')
                return f'"{escaped}"'
            elif isinstance(expr.value, bool):
                return "true" if expr.value else "false"
            else:
                return str(expr.value)
        elif isinstance(expr, Identifier):
            return expr.name
        elif isinstance(expr, BinaryOp):
            left = self.emit_expression(expr.left)
            right = self.emit_expression(expr.right)
            
            # Handle Java-specific operators
            if expr.operator == "==":
                # Use .equals() for strings, == for primitives
                return f"Objects.equals({left}, {right})"
            elif expr.operator == "!=":
                return f"!Objects.equals({left}, {right})"
            else:
                return f"({left} {expr.operator} {right})"
        elif isinstance(expr, ArithmeticOp):
            left = self.emit_expression(expr.left)
            right = self.emit_expression(expr.right)
            return f"({left} {expr.operator} {right})"
        elif isinstance(expr, ArrayLiteral):
            elements = [self.emit_expression(elem) for elem in expr.elements]
            return f"Arrays.asList({', '.join(elements)})"
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
        elif isinstance(expr, ActionInvocation):
            if expr.module_name:
                return f"{expr.module_name}.{expr.action_name}()"
            else:
                return f"{expr.action_name}()"
        elif isinstance(expr, ActionInvocationWithArgs):
            args = [self.emit_expression(arg) for arg in expr.arguments]
            args_str = ", ".join(args)
            if expr.module_name:
                return f"{expr.module_name}.{expr.action_name}({args_str})"
            else:
                return f"{expr.action_name}({args_str})"
        else:
            return f"/* TODO: {type(expr).__name__} */"
    
    def emit_display_statement(self, stmt: DisplayStatement):
        """Emit display statement with native formatting (to constructor)."""
        expr_str = self.emit_expression(stmt.expression)
        expr_type = self.infer_type(stmt.expression)
        
        # Handle boolean formatting inline
        if expr_type == VariableType.BOOLEAN or expr_type == VariableType.FLAG or expr_type == VariableType.YESNO:
            self.constructor_code.append(f'System.out.println(({expr_str}) ? "true" : "false");')
        # Handle list formatting inline  
        elif expr_type in [VariableType.LIST_OF, VariableType.GROUP_OF, VariableType.ARRAY]:
            self.constructor_code.append(f'System.out.println("[" + String.join(", ", {expr_str}.stream().map(Object::toString).toArray(String[]::new)) + "]");')
            self.imports.add("java.util.stream.*")
        else:
            self.constructor_code.append(f"System.out.println({expr_str});")
    
    def emit_assignment(self, stmt: Assignment):
        """Emit assignment statement (to constructor)."""
        value_str = self.emit_expression(stmt.value)
        
        # Use declared type if available, otherwise infer from value
        if hasattr(stmt, 'declared_var_type') and stmt.declared_var_type:
            # Use the declared compound type for proper Java generic typing
            java_type = self._get_java_type_from_declared(stmt.declared_var_type)
            declared_internal_type = self.map_user_type_to_internal(stmt.declared_var_type)
        else:
            # Fall back to type inference
            inferred_type = self.infer_type(stmt.value)
            java_type = self._get_java_type(inferred_type)
            declared_internal_type = inferred_type
        
        # Check if this is the first assignment to this variable
        field_exists = any(f"private {java_type} {stmt.variable};" in field for field in self.fields)
        if not field_exists:
            self.fields.append(f"private {java_type} {stmt.variable};")
        
        # Add assignment to constructor
        self.constructor_code.append(f"this.{stmt.variable} = {value_str};")
        
        # Track in symbol table
        self.symbol_table.declare_variable(stmt.variable, declared_internal_type)
    
    def emit_if_statement(self, stmt: IfStatement):
        """Emit if statement (to constructor)."""
        condition_str = self.emit_expression(stmt.condition)
        self.constructor_code.append(f"if ({condition_str}) {{")
        
        # Simplified - would need proper statement handling
        if stmt.then_body:
            for then_stmt in stmt.then_body:
                if isinstance(then_stmt, DisplayStatement):
                    expr_str = self.emit_expression(then_stmt.expression)
                    self.constructor_code.append(f"    System.out.println({expr_str});")
        
        if stmt.else_body:
            self.constructor_code.append("} else {")
            for else_stmt in stmt.else_body:
                if isinstance(else_stmt, DisplayStatement):
                    expr_str = self.emit_expression(else_stmt.expression)
                    self.constructor_code.append(f"    System.out.println({expr_str});")
        
        self.constructor_code.append("}")
    
    def emit_while_loop(self, stmt: WhileLoop):
        """Emit while loop (to constructor)."""
        condition_str = self.emit_expression(stmt.condition)
        self.constructor_code.append(f"while ({condition_str}) {{")
        self.constructor_code.append("    // Loop body")
        self.constructor_code.append("}")
    
    def emit_foreach_loop(self, stmt: ForEachLoop):
        """Emit for-each loop (to constructor)."""
        collection_str = self.emit_expression(stmt.collection)
        self.constructor_code.append(f"for (Object {stmt.variable} : (List<Object>) {collection_str}) {{")
        self.constructor_code.append("    // Loop body")
        self.constructor_code.append("}")
    
    def emit_string_interpolation(self, expr: StringInterpolation) -> str:
        """Emit string interpolation using String.format."""
        format_parts = []
        args = []
        
        for part in expr.parts:
            if isinstance(part, str):
                format_parts.append(part)
            else:
                format_parts.append("%s")
                args.append(self.emit_expression(part))
        
        format_string = ''.join(format_parts)
        if args:
            args_str = ', '.join(args)
            return f'String.format("{format_string}", {args_str})'
        else:
            return f'"{format_string}"'
    
    def _inline_date_formatting(self, expr_str: str, pattern: str) -> str:
        """Generate inline date formatting code."""
        if pattern == "MM/dd/yyyy":
            return f'LocalDate.parse({expr_str}).format(DateTimeFormatter.ofPattern("MM/dd/yyyy"))'
        elif pattern == "dd/MM/yyyy":
            return f'LocalDate.parse({expr_str}).format(DateTimeFormatter.ofPattern("dd/MM/yyyy"))'
        elif pattern == "MMM dd, yyyy":
            return f'LocalDate.parse({expr_str}).format(DateTimeFormatter.ofPattern("MMM dd, yyyy"))'
        elif pattern == "long":
            return f'LocalDate.parse({expr_str}).format(DateTimeFormatter.ofPattern("EEEE, MMMM dd, yyyy"))'
        elif pattern == "short":
            return f'LocalDate.parse({expr_str}).format(DateTimeFormatter.ofPattern("MM/dd/yy"))'
        elif pattern == "iso":
            return f'LocalDate.parse({expr_str}).format(DateTimeFormatter.ISO_LOCAL_DATE)'
        return expr_str
    
    def _inline_decimal_formatting(self, expr_str: str, pattern: str) -> str:
        """Generate inline decimal formatting code."""
        if pattern == "0.00":
            return f'new DecimalFormat("0.00").format({expr_str})'
        elif pattern == "#,##0.00":
            return f'new DecimalFormat("#,##0.00").format({expr_str})'
        elif pattern == "$0.00":
            return f'new DecimalFormat("$0.00").format({expr_str})'
        elif pattern == "percent":
            return f'new DecimalFormat("0.00%").format({expr_str})'
        return f'String.valueOf({expr_str})'
    
    def _inline_number_formatting(self, expr_str: str, pattern: str) -> str:
        """Generate inline number formatting code."""
        if pattern == "#,##0":
            return f'NumberFormat.getNumberInstance().format({expr_str})'
        elif pattern == "0000":
            return f'String.format("%04d", {expr_str})'
        elif pattern == "hex":
            return f'"0x" + Integer.toHexString({expr_str}).toUpperCase()'
        elif pattern == "oct":
            return f'"0o" + Integer.toOctalString({expr_str})'
        elif pattern == "bin":
            return f'"0b" + Integer.toBinaryString({expr_str})'
        return f'String.valueOf({expr_str})'
    
    def _setup_spring_boot_imports(self):
        """Add Spring Boot specific imports."""
        if self.framework == "spring":
            self.imports.update([
                "org.springframework.boot.SpringApplication",
                "org.springframework.boot.autoconfigure.SpringBootApplication",
                "org.springframework.web.bind.annotation.*",
                "org.springframework.stereotype.Service",
                "org.springframework.stereotype.Repository",
                "org.springframework.data.jpa.repository.JpaRepository",
                "jakarta.persistence.*",
                "org.springframework.beans.factory.annotation.Autowired",
                "org.springframework.http.ResponseEntity",
                "org.springframework.http.HttpStatus",
                "java.util.Optional"
            ])
    
    def _generate_spring_boot_application(self) -> List[str]:
        """Generate the main Spring Boot application class."""
        lines = []
        lines.append("@SpringBootApplication")
        lines.append(f"public class {self.class_name}Application {{")
        lines.append("")
        lines.append("    public static void main(String[] args) {")
        lines.append(f"        SpringApplication.run({self.class_name}Application.class, args);")
        lines.append("    }")
        lines.append("}")
        return lines
    
    def _generate_jpa_entity(self, data_def: DataDefinition) -> List[str]:
        """Generate a JPA entity class from a data definition."""
        lines = []
        lines.append("@Entity")
        lines.append("@Table(name = \"" + data_def.name.lower() + "s\")")
        lines.append(f"public class {data_def.name} {{")
        lines.append("")
        
        # Generate fields
        id_field_added = False
        for field in data_def.fields:
            if field.name.lower() == "id":
                lines.append("    @Id")
                lines.append("    @GeneratedValue(strategy = GenerationType.IDENTITY)")
                id_field_added = True
            
            java_type = self._get_java_type_from_declared(field.type)
            lines.append(f"    @Column(name = \"{field.name}\")")
            lines.append(f"    private {java_type} {field.name};")
            lines.append("")
        
        # Add default ID field if not present
        if not id_field_added:
            lines.insert(-len(data_def.fields) * 3, "    @Id")
            lines.insert(-len(data_def.fields) * 3 + 1, "    @GeneratedValue(strategy = GenerationType.IDENTITY)")
            lines.insert(-len(data_def.fields) * 3 + 2, "    @Column(name = \"id\")")
            lines.insert(-len(data_def.fields) * 3 + 3, "    private Long id;")
            lines.insert(-len(data_def.fields) * 3 + 4, "")
        
        # Generate getters and setters
        if not id_field_added:
            lines.extend(self._generate_getter_setter("Long", "id"))
            
        for field in data_def.fields:
            java_type = self._get_java_type_from_declared(field.type)
            lines.extend(self._generate_getter_setter(java_type, field.name))
        
        lines.append("}")
        return lines
    
    def _generate_getter_setter(self, java_type: str, field_name: str) -> List[str]:
        """Generate getter and setter methods for a field."""
        lines = []
        capitalized_name = field_name.capitalize()
        
        # Getter
        lines.append(f"    public {java_type} get{capitalized_name}() {{")
        lines.append(f"        return {field_name};")
        lines.append("    }")
        lines.append("")
        
        # Setter
        lines.append(f"    public void set{capitalized_name}({java_type} {field_name}) {{")
        lines.append(f"        this.{field_name} = {field_name};")
        lines.append("    }")
        lines.append("")
        
        return lines
    
    def _generate_repository(self, entity_name: str) -> List[str]:
        """Generate a JPA repository interface."""
        lines = []
        lines.append("@Repository")
        lines.append(f"public interface {entity_name}Repository extends JpaRepository<{entity_name}, Long> {{")
        lines.append("    // Custom query methods can be added here")
        lines.append("}")
        return lines
    
    def _generate_service(self, module: ModuleDefinition) -> List[str]:
        """Generate a service class from a module definition."""
        lines = []
        lines.append("@Service")
        lines.append(f"public class {module.name} {{")
        lines.append("")
        
        # Add repository injections based on data types referenced
        repositories = set()
        for stmt in module.body:
            if isinstance(stmt, ActionDefinitionWithParams):
                # Scan action body for database operations
                for body_stmt in stmt.body:
                    if isinstance(body_stmt, DatabaseStatement):
                        repositories.add(body_stmt.entity_name)
        
        for entity in repositories:
            lines.append("    @Autowired")
            lines.append(f"    private {entity}Repository {entity.lower()}Repository;")
            lines.append("")
        
        # Generate service methods from actions
        for stmt in module.body:
            if isinstance(stmt, ActionDefinitionWithParams):
                lines.extend(self._generate_service_method(stmt))
        
        lines.append("}")
        return lines
    
    def _generate_service_method(self, action: ActionDefinitionWithParams) -> List[str]:
        """Generate a service method from an action definition."""
        lines = []
        
        # Build parameter list
        params = []
        if action.parameters:
            for param in action.parameters:
                param_type = self._get_java_type(self.map_user_type_to_internal(param.type))
                params.append(f"{param_type} {param.name}")
        
        param_list = ", ".join(params)
        
        # Determine return type
        return_type = "Object"
        if action.return_type:
            return_type = self._get_java_type(self.map_user_type_to_internal(action.return_type))
        
        lines.append(f"    public {return_type} {action.name}({param_list}) {{")
        
        # Generate method body from database operations
        for stmt in action.body:
            if isinstance(stmt, DatabaseStatement):
                lines.extend(self._generate_database_operation(stmt, 2))
            elif isinstance(stmt, ReturnStatement):
                if hasattr(stmt.expression, 'name'):
                    lines.append(f"        return {stmt.expression.name};")
                else:
                    expr_str = self.emit_expression(stmt.expression)
                    lines.append(f"        return {expr_str};")
        
        if return_type != "void" and not any("return" in line for line in lines[-5:]):
            lines.append("        return null;")
        
        lines.append("    }")
        lines.append("")
        return lines
    
    def _generate_database_operation(self, stmt: DatabaseStatement, indent: int = 0) -> List[str]:
        """Generate database operation code."""
        lines = []
        indent_str = "    " * indent
        
        if stmt.operation == "find":
            if stmt.conditions:
                # For now, simple findById operation
                lines.append(f"{indent_str}Optional<{stmt.entity_name}> result = {stmt.entity_name.lower()}Repository.findById({stmt.conditions[0].name});")
                lines.append(f"{indent_str}{stmt.entity_name} {stmt.entity_name.lower()} = result.orElse(null);")
            else:
                lines.append(f"{indent_str}List<{stmt.entity_name}> results = {stmt.entity_name.lower()}Repository.findAll();")
        
        elif stmt.operation == "create":
            lines.append(f"{indent_str}{stmt.entity_name} {stmt.entity_name.lower()} = new {stmt.entity_name}();")
            for field in stmt.fields:
                if hasattr(field, 'field_name') and hasattr(field, 'value'):
                    lines.append(f"{indent_str}{stmt.entity_name.lower()}.set{field.field_name.capitalize()}({field.value.name});")
            lines.append(f"{indent_str}{stmt.entity_name.lower()} = {stmt.entity_name.lower()}Repository.save({stmt.entity_name.lower()});")
        
        elif stmt.operation == "update":
            if stmt.conditions:
                lines.append(f"{indent_str}Optional<{stmt.entity_name}> result = {stmt.entity_name.lower()}Repository.findById({stmt.conditions[0].name});")
                lines.append(f"{indent_str}if (result.isPresent()) {{")
                lines.append(f"{indent_str}    {stmt.entity_name} {stmt.entity_name.lower()} = result.get();")
                for field in stmt.fields:
                    if hasattr(field, 'field_name') and hasattr(field, 'value'):
                        lines.append(f"{indent_str}    {stmt.entity_name.lower()}.set{field.field_name.capitalize()}({field.value.name});")
                lines.append(f"{indent_str}    {stmt.entity_name.lower()} = {stmt.entity_name.lower()}Repository.save({stmt.entity_name.lower()});")
                lines.append(f"{indent_str}}}")
        
        return lines
    
    def emit_api_call(self, stmt: ApiCallStatement):
        """Generate native Java HTTP client call."""
        if self.framework == "plain":
            self._emit_native_http_call(stmt)
        else:
            # Framework mode uses different approach
            self.constructor_code.append(f"// TODO: Framework HTTP call for {stmt.verb} {stmt.endpoint}")
    
    def _emit_native_http_call(self, stmt: ApiCallStatement):
        """Generate native Java HTTP client call using HttpClient."""
        self.imports.add("java.net.http.*")
        self.imports.add("java.net.URI")
        self.imports.add("java.time.Duration")
        self.imports.add("java.io.IOException")
        self.imports.add("java.util.concurrent.CompletableFuture")
        
        # Generate HTTP client code
        endpoint_url = stmt.endpoint
        method = stmt.method.upper()
        
        lines = []
        lines.append("try {")
        lines.append("    HttpClient client = HttpClient.newHttpClient();")
        lines.append(f"    HttpRequest.Builder requestBuilder = HttpRequest.newBuilder()")
        lines.append(f"        .uri(URI.create(\"{endpoint_url}\"))")
        lines.append(f"        .timeout(Duration.ofSeconds(30))")
        
        # Add headers
        for header in stmt.headers:
            lines.append(f"        .header(\"{header.name}\", \"{header.value}\")")
        
        # Add method and body
        if method in ["POST", "PUT", "PATCH"] and stmt.payload:
            lines.append(f"        .{method.lower()}(HttpRequest.BodyPublishers.ofString({stmt.payload}));")
        else:
            lines.append(f"        .{method.lower()}(HttpRequest.BodyPublishers.noBody());")
        
        lines.append("    HttpRequest request = requestBuilder.build();")
        lines.append("    HttpResponse<String> response = client.send(request,")
        lines.append("        HttpResponse.BodyHandlers.ofString());")
        
        # Store response if variable specified
        if stmt.response_variable:
            lines.append(f"    String {stmt.response_variable} = response.body();")
            lines.append(f"    int {stmt.response_variable}Status = response.statusCode();")
        
        lines.append("} catch (IOException | InterruptedException e) {")
        lines.append("    System.err.println(\"HTTP request failed: \" + e.getMessage());")
        lines.append("}")
        
        for line in lines:
            self.constructor_code.append("        " + line)
    
    def emit_database_statement(self, stmt: DatabaseStatement):
        """Generate database operation using native JDBC or framework."""
        if self.framework == "plain":
            self._emit_native_database_operation(stmt)
        else:
            # Use existing framework method
            lines = self._generate_database_operation(stmt, 2)
            for line in lines:
                self.constructor_code.append("        " + line)
    
    def _emit_native_database_operation(self, stmt: DatabaseStatement):
        """Generate native JDBC database operation."""
        self.imports.add("java.sql.*")
        self.imports.add("javax.sql.DataSource")
        
        # Add database connection setup if not already present
        if not any("Connection connection" in line for line in self.constructor_code):
            self.constructor_code.extend([
                "        // Database setup",
                "        String dbUrl = \"jdbc:h2:mem:testdb\";",
                "        String dbUser = \"sa\";", 
                "        String dbPassword = \"\";",
                "        ",
                "        try {",
                "            Class.forName(\"org.h2.Driver\");",
                "            Connection connection = DriverManager.getConnection(dbUrl, dbUser, dbPassword);",
                "            ",
                "            // Create table if not exists",
                f"            String createTableSQL = \"CREATE TABLE IF NOT EXISTS {stmt.entity_name.lower()} (id BIGINT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(255))\";",
                "            Statement createStmt = connection.createStatement();", 
                "            createStmt.execute(createTableSQL);",
                "            "
            ])
        
        entity_name = stmt.entity_name.lower()
        
        if stmt.operation == "find":
            if stmt.conditions:
                # Find by condition
                condition_field = stmt.conditions[0].left.name if hasattr(stmt.conditions[0], 'left') else "id"
                condition_value = stmt.conditions[0].right.value if hasattr(stmt.conditions[0], 'right') else "1"
                
                self.constructor_code.extend([
                    f"            // Find {stmt.entity_name} by {condition_field}",
                    f"            String selectSQL = \"SELECT * FROM {entity_name} WHERE {condition_field} = ?\";",
                    "            PreparedStatement selectStmt = connection.prepareStatement(selectSQL);",
                    f"            selectStmt.setObject(1, {condition_value});",
                    "            ResultSet resultSet = selectStmt.executeQuery();",
                    "            ",
                    "            if (resultSet.next()) {",
                    f"                System.out.println(\"Found {stmt.entity_name}: \" + resultSet.getString(\"name\"));",
                    "            } else {",
                    f"                System.out.println(\"No {stmt.entity_name} found\");",
                    "            }",
                    ""
                ])
            else:
                # Find all
                self.constructor_code.extend([
                    f"            // Find all {stmt.entity_name}",
                    f"            String selectAllSQL = \"SELECT * FROM {entity_name}\";",
                    "            Statement selectStmt = connection.createStatement();",
                    "            ResultSet resultSet = selectStmt.executeQuery(selectAllSQL);",
                    "            ",
                    "            while (resultSet.next()) {",
                    f"                System.out.println(\"{stmt.entity_name}: \" + resultSet.getString(\"name\"));",
                    "            }",
                    ""
                ])
        
        elif stmt.operation == "create":
            # Insert operation
            self.constructor_code.extend([
                f"            // Create new {stmt.entity_name}",
                f"            String insertSQL = \"INSERT INTO {entity_name} (name) VALUES (?)\";",
                "            PreparedStatement insertStmt = connection.prepareStatement(insertSQL);",
                f"            insertStmt.setString(1, \"Sample {stmt.entity_name}\");",
                "            int rowsAffected = insertStmt.executeUpdate();",
                f"            System.out.println(\"Created {stmt.entity_name}, rows affected: \" + rowsAffected);",
                ""
            ])
        
        elif stmt.operation == "update":
            # Update operation
            self.constructor_code.extend([
                f"            // Update {stmt.entity_name}",
                f"            String updateSQL = \"UPDATE {entity_name} SET name = ? WHERE id = ?\";",
                "            PreparedStatement updateStmt = connection.prepareStatement(updateSQL);",
                f"            updateStmt.setString(1, \"Updated {stmt.entity_name}\");",
                "            updateStmt.setLong(2, 1);",
                "            int rowsAffected = updateStmt.executeUpdate();",
                f"            System.out.println(\"Updated {stmt.entity_name}, rows affected: \" + rowsAffected);",
                ""
            ])
        
        elif stmt.operation == "delete":
            # Delete operation
            self.constructor_code.extend([
                f"            // Delete {stmt.entity_name}",
                f"            String deleteSQL = \"DELETE FROM {entity_name} WHERE id = ?\";",
                "            PreparedStatement deleteStmt = connection.prepareStatement(deleteSQL);",
                "            deleteStmt.setLong(1, 1);",
                "            int rowsAffected = deleteStmt.executeUpdate();",
                f"            System.out.println(\"Deleted {stmt.entity_name}, rows affected: \" + rowsAffected);",
                ""
            ])
        
        # Close the try block if this is the first database operation
        if stmt.operation == "find" and not stmt.conditions:  # Simple way to detect first operation
            self.constructor_code.extend([
                "            connection.close();",
                "        } catch (ClassNotFoundException | SQLException e) {",
                "            System.err.println(\"Database operation failed: \" + e.getMessage());",
                "        }"
            ])
    
    def _generate_rest_controller(self, module: ModuleDefinition) -> List[str]:
        """Generate a REST controller from module with serve statements."""
        lines = []
        lines.append("@RestController")
        lines.append(f"@RequestMapping(\"/api\")")
        lines.append(f"public class {module.name}Controller {{")
        lines.append("")
        
        # Inject service
        lines.append("    @Autowired")
        lines.append(f"    private {module.name} {module.name.lower()}Service;")
        lines.append("")
        
        # Generate endpoints from serve statements
        for stmt in module.body:
            if isinstance(stmt, ServeStatement):
                lines.extend(self._generate_rest_endpoint(stmt))
        
        lines.append("}")
        return lines
    
    def _generate_rest_endpoint(self, serve: ServeStatement) -> List[str]:
        """Generate a REST endpoint from a serve statement."""
        lines = []
        
        # Determine Spring annotation
        method_map = {
            "get": "GetMapping",
            "post": "PostMapping", 
            "put": "PutMapping",
            "delete": "DeleteMapping"
        }
        
        spring_annotation = method_map.get(serve.method, "RequestMapping")
        lines.append(f"    @{spring_annotation}(\"{serve.endpoint}\")")
        
        # Build method signature
        method_name = f"{serve.method}{serve.endpoint.replace('/', '').replace('{', '').replace('}', '').capitalize()}"
        if not method_name.endswith("Mapping"):
            method_name = method_name.replace("Mapping", "")
        
        params = []
        for param_stmt in serve.body:
            if isinstance(param_stmt, ParamsStatement):
                java_type = self._get_java_type(self.map_user_type_to_internal(param_stmt.param_type))
                params.append(f"@PathVariable {java_type} {param_stmt.param_name}")
        
        # Add request body parameter if needed
        for stmt in serve.body:
            if isinstance(stmt, AcceptStatement):
                params.append(f"@RequestBody Object requestBody")
                break
        
        param_list = ", ".join(params)
        
        lines.append(f"    public ResponseEntity<Object> {method_name}({param_list}) {{")
        lines.append("        try {")
        
        # Generate method body
        for stmt in serve.body:
            if isinstance(stmt, RespondStatement):
                if serve.body and any(isinstance(s, ParamsStatement) for s in serve.body):
                    # Has parameters
                    param_names = [p.param_name for p in serve.body if isinstance(p, ParamsStatement)]
                    param_args = ", ".join(param_names)
                    lines.append(f"            Object result = {stmt.module_name.lower()}Service.{stmt.action_name}({param_args});")
                else:
                    lines.append(f"            Object result = {stmt.module_name.lower()}Service.{stmt.action_name}();")
                lines.append("            return ResponseEntity.ok(result);")
        
        lines.append("        } catch (Exception e) {")
        lines.append("            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).body(\"Error: \" + e.getMessage());")
        lines.append("        }")
        lines.append("    }")
        lines.append("")
        
        return lines
    
    def _generate_spring_boot_project(self, modules_found: List[ModuleDefinition], 
                                    data_definitions: List[DataDefinition], 
                                    serve_modules: List[ModuleDefinition]) -> str:
        """Generate a complete Spring Boot project structure."""
        from pathlib import Path
        import os
        
        # Create project structure
        project_name = self.class_name.lower().replace("application", "")
        base_package = f"com.example.{project_name}"
        
        # Define project root relative to current output file location
        if self.source_file_path:
            source_dir = Path(self.source_file_path).parent
            project_root = source_dir / "build" / f"{project_name}-spring-boot"
        else:
            project_root = Path("build") / f"{project_name}-spring-boot"
        
        # Create directory structure
        src_main_java = project_root / "src" / "main" / "java" / "com" / "example" / project_name
        src_main_resources = project_root / "src" / "main" / "resources"
        
        # Ensure directories exist
        os.makedirs(src_main_java / "entity", exist_ok=True)
        os.makedirs(src_main_java / "repository", exist_ok=True)
        os.makedirs(src_main_java / "service", exist_ok=True)
        os.makedirs(src_main_java / "controller", exist_ok=True)
        os.makedirs(src_main_resources, exist_ok=True)
        
        # Generate main application class
        main_app_lines = self._generate_spring_boot_application_with_package(base_package)
        main_app_file = src_main_java / "Application.java"
        with open(main_app_file, 'w') as f:
            f.write("\n".join(main_app_lines))
        
        # Generate JPA entities from data definitions
        entities_created = []
        for module in modules_found:
            for stmt in module.body:
                if isinstance(stmt, DataDefinition):
                    data_definitions.append(stmt)
        
        for data_def in data_definitions:
            entity_lines = self._generate_jpa_entity_with_package(data_def, base_package)
            entity_file = src_main_java / "entity" / f"{data_def.name}.java"
            with open(entity_file, 'w') as f:
                f.write("\n".join(entity_lines))
            entities_created.append(data_def.name)
            
            # Generate repository
            repo_lines = self._generate_repository_with_package(data_def.name, base_package)
            repo_file = src_main_java / "repository" / f"{data_def.name}Repository.java"
            with open(repo_file, 'w') as f:
                f.write("\n".join(repo_lines))
        
        # Generate services from modules
        for module in modules_found:
            service_lines = self._generate_service_with_package(module, base_package, entities_created)
            service_file = src_main_java / "service" / f"{module.name}Service.java"
            with open(service_file, 'w') as f:
                f.write("\n".join(service_lines))
        
        # Generate controllers from serve modules (if any)
        for module in serve_modules:
            controller_lines = self._generate_rest_controller_with_package(module, base_package)
            controller_file = src_main_java / "controller" / f"{module.name}Controller.java"
            with open(controller_file, 'w') as f:
                f.write("\n".join(controller_lines))
        
        # Generate Maven pom.xml
        pom_content = self._generate_maven_pom(project_name, base_package)
        pom_file = project_root / "pom.xml"
        with open(pom_file, 'w') as f:
            f.write(pom_content)
        
        # Generate application.properties
        props_content = self._generate_application_properties()
        props_file = src_main_resources / "application.properties"
        with open(props_file, 'w') as f:
            f.write(props_content)
        
        # Generate README for the Spring Boot project
        readme_content = self._generate_spring_boot_readme(project_name)
        readme_file = project_root / "README.md"
        with open(readme_file, 'w') as f:
            f.write(readme_content)
        
        return f"SPRING_PROJECT:{project_root}"
    
    def _build_class_file(self, class_lines: List[str], class_name: str) -> str:
        """Build a complete Java class file with imports."""
        lines = []
        
        # Add imports
        if self.imports:
            for imp in sorted(self.imports):
                lines.append(f"import {imp};")
            lines.append("")
        
        # Add class content
        lines.extend(class_lines)
        
        return "\n".join(lines)
    
    def _generate_spring_boot_application_with_package(self, package: str) -> List[str]:
        """Generate Spring Boot application class with package declaration."""
        lines = []
        lines.append(f"package {package};")
        lines.append("")
        lines.append("import org.springframework.boot.SpringApplication;")
        lines.append("import org.springframework.boot.autoconfigure.SpringBootApplication;")
        lines.append("")
        lines.append("@SpringBootApplication")
        lines.append("public class Application {")
        lines.append("")
        lines.append("    public static void main(String[] args) {")
        lines.append("        SpringApplication.run(Application.class, args);")
        lines.append("    }")
        lines.append("}")
        return lines
    
    def _generate_jpa_entity_with_package(self, data_def: DataDefinition, package: str) -> List[str]:
        """Generate JPA entity with package declaration."""
        lines = []
        lines.append(f"package {package}.entity;")
        lines.append("")
        lines.append("import jakarta.persistence.*;")
        lines.append("")
        lines.append("@Entity")
        lines.append(f"@Table(name = \"{data_def.name.lower()}s\")")
        lines.append(f"public class {data_def.name} {{")
        lines.append("")
        
        # Add ID field if not present
        id_field_added = any(field.name.lower() == "id" for field in data_def.fields)
        if not id_field_added:
            lines.append("    @Id")
            lines.append("    @GeneratedValue(strategy = GenerationType.IDENTITY)")
            lines.append("    @Column(name = \"id\")")
            lines.append("    private Long id;")
            lines.append("")
        
        # Generate fields
        for field in data_def.fields:
            if field.name.lower() == "id":
                lines.append("    @Id")
                lines.append("    @GeneratedValue(strategy = GenerationType.IDENTITY)")
            
            java_type = self._get_java_type_from_declared(field.type)
            lines.append(f"    @Column(name = \"{field.name.lower()}\")")
            lines.append(f"    private {java_type} {field.name};")
            lines.append("")
        
        # Generate getters and setters
        if not id_field_added:
            lines.extend(self._generate_getter_setter("Long", "id"))
            
        for field in data_def.fields:
            java_type = self._get_java_type_from_declared(field.type)
            lines.extend(self._generate_getter_setter(java_type, field.name))
        
        lines.append("}")
        return lines
    
    def _generate_repository_with_package(self, entity_name: str, package: str) -> List[str]:
        """Generate JPA repository with package declaration."""
        lines = []
        lines.append(f"package {package}.repository;")
        lines.append("")
        lines.append(f"import {package}.entity.{entity_name};")
        lines.append("import org.springframework.data.jpa.repository.JpaRepository;")
        lines.append("import org.springframework.stereotype.Repository;")
        lines.append("")
        lines.append("@Repository")
        lines.append(f"public interface {entity_name}Repository extends JpaRepository<{entity_name}, Long> {{")
        lines.append("    // Custom query methods can be added here")
        lines.append("}")
        return lines
    
    def _generate_service_with_package(self, module: ModuleDefinition, package: str, entities: List[str]) -> List[str]:
        """Generate service class with package declaration."""
        lines = []
        lines.append(f"package {package}.service;")
        lines.append("")
        
        # Add imports
        lines.append("import org.springframework.beans.factory.annotation.Autowired;")
        lines.append("import org.springframework.stereotype.Service;")
        lines.append("import java.util.List;")
        lines.append("import java.util.Optional;")
        lines.append("")
        
        # Add entity and repository imports
        for entity in entities:
            lines.append(f"import {package}.entity.{entity};")
            lines.append(f"import {package}.repository.{entity}Repository;")
        lines.append("")
        
        lines.append("@Service")
        lines.append(f"public class {module.name}Service {{")
        lines.append("")
        
        # Add repository injections
        for entity in entities:
            lines.append("    @Autowired")
            lines.append(f"    private {entity}Repository {entity.lower()}Repository;")
            lines.append("")
        
        # Generate service methods from actions
        for stmt in module.body:
            if isinstance(stmt, ActionDefinitionWithParams):
                lines.extend(self._generate_service_method_with_package(stmt))
        
        lines.append("}")
        return lines
    
    def _generate_service_method_with_package(self, action: ActionDefinitionWithParams) -> List[str]:
        """Generate service method with proper typing."""
        lines = []
        
        # Build parameter list
        params = []
        if action.parameters:
            for param in action.parameters:
                param_type = self._get_java_type(self.map_user_type_to_internal(param.type))
                params.append(f"{param_type} {param.name}")
        
        param_list = ", ".join(params)
        
        # Determine return type
        return_type = "Object"
        if action.return_type:
            return_type = self._get_java_type(self.map_user_type_to_internal(action.return_type))
        
        lines.append(f"    public {return_type} {action.name}({param_list}) {{")
        lines.append("        // TODO: Implement business logic")
        
        if return_type != "void":
            lines.append("        return null;")
        
        lines.append("    }")
        lines.append("")
        return lines
    
    def _generate_rest_controller_with_package(self, module: ModuleDefinition, package: str) -> List[str]:
        """Generate REST controller with package declaration."""
        lines = []
        lines.append(f"package {package}.controller;")
        lines.append("")
        lines.append(f"import {package}.service.{module.name}Service;")
        lines.append("import org.springframework.beans.factory.annotation.Autowired;")
        lines.append("import org.springframework.web.bind.annotation.*;")
        lines.append("")
        lines.append("@RestController")
        lines.append("@RequestMapping(\"/api\")")
        lines.append(f"public class {module.name}Controller {{")
        lines.append("")
        lines.append("    @Autowired")
        lines.append(f"    private {module.name}Service {module.name.lower()}Service;")
        lines.append("")
        lines.append("    // TODO: Add REST endpoints")
        lines.append("}")
        return lines
    
    def _generate_maven_pom(self, project_name: str, package: str) -> str:
        """Generate Maven pom.xml file."""
        return f"""<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 
         http://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>
    
    <parent>
        <groupId>org.springframework.boot</groupId>
        <artifactId>spring-boot-starter-parent</artifactId>
        <version>3.1.5</version>
        <relativePath/>
    </parent>
    
    <groupId>com.example</groupId>
    <artifactId>{project_name}-spring-boot</artifactId>
    <version>1.0.0</version>
    <name>{project_name.title()} Spring Boot Application</name>
    <description>Spring Boot application generated from Roelang DSL</description>
    
    <properties>
        <java.version>17</java.version>
    </properties>
    
    <dependencies>
        <dependency>
            <groupId>org.springframework.boot</groupId>
            <artifactId>spring-boot-starter-web</artifactId>
        </dependency>
        <dependency>
            <groupId>org.springframework.boot</groupId>
            <artifactId>spring-boot-starter-data-jpa</artifactId>
        </dependency>
        <dependency>
            <groupId>com.h2database</groupId>
            <artifactId>h2</artifactId>
            <scope>runtime</scope>
        </dependency>
        <dependency>
            <groupId>org.springframework.boot</groupId>
            <artifactId>spring-boot-starter-test</artifactId>
            <scope>test</scope>
        </dependency>
    </dependencies>
    
    <build>
        <plugins>
            <plugin>
                <groupId>org.springframework.boot</groupId>
                <artifactId>spring-boot-maven-plugin</artifactId>
            </plugin>
        </plugins>
    </build>
</project>"""
    
    def _generate_application_properties(self) -> str:
        """Generate application.properties file."""
        return """# Spring Boot Configuration
spring.application.name=roelang-spring-app

# H2 Database Configuration (for development)
spring.datasource.url=jdbc:h2:mem:testdb
spring.datasource.driverClassName=org.h2.Driver
spring.datasource.username=sa
spring.datasource.password=

# JPA/Hibernate Configuration
spring.jpa.database-platform=org.hibernate.dialect.H2Dialect
spring.jpa.hibernate.ddl-auto=create-drop
spring.jpa.show-sql=true
spring.jpa.properties.hibernate.format_sql=true

# H2 Console (for development)
spring.h2.console.enabled=true
spring.h2.console.path=/h2-console

# Server Configuration
server.port=8080
"""
    
    def _generate_spring_boot_readme(self, project_name: str) -> str:
        """Generate README for the Spring Boot project."""
        return f"""# {project_name.title()} Spring Boot Application

This Spring Boot application was generated from Roelang DSL.

## Features

- Spring Boot 3.1.5
- Spring Data JPA
- H2 Database (in-memory, for development)
- RESTful API endpoints
- Automatic database schema generation

## Project Structure

```
src/main/java/com/example/{project_name}/
├── Application.java              # Main Spring Boot application class
├── entity/                       # JPA entities
├── repository/                   # Data access layer
├── service/                      # Business logic layer
└── controller/                   # REST API controllers
```

## Running the Application

1. Ensure you have Java 17+ and Maven installed
2. Navigate to the project directory
3. Run the application:

```bash
mvn spring-boot:run
```

Or build and run the JAR:

```bash
mvn clean package
java -jar target/{project_name}-spring-boot-1.0.0.jar
```

## Accessing the Application

- Application: http://localhost:8080
- H2 Console: http://localhost:8080/h2-console
  - JDBC URL: `jdbc:h2:mem:testdb`
  - Username: `sa`
  - Password: (leave empty)

## API Endpoints

The application exposes REST endpoints under `/api/` path.

## Development

This is a standard Maven Spring Boot project. You can:
- Import it into any Java IDE
- Modify the generated classes
- Add additional dependencies to `pom.xml`
- Customize application properties

Generated by Roelang compiler with Spring Boot framework support.
"""