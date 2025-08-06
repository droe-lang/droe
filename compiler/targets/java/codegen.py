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
    DataInstance, FieldAssignment, FormatExpression
)
from ...symbols import SymbolTable, VariableType
from ...codegen_base import BaseCodeGenerator, CodeGenError


class JavaCodeGenerator(BaseCodeGenerator):
    """Generates Java code from Roelang AST."""
    
    def __init__(self, source_file_path: Optional[str] = None, is_main_file: bool = False):
        super().__init__()
        self.source_file_path = source_file_path
        self.is_main_file = is_main_file
        self.class_name = "RoelangProgram"  # Default
        self.imports = set()
        self.fields = []  # Instance variables
        self.constructor_code = []  # Procedural code goes here
        self.methods = []  # Action definitions become methods
        self.module_classes = []  # Module definitions become separate classes
        self.has_modules = False
        
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
        self.has_modules = False
        
        # Add core imports
        self.imports.add("java.util.*")
        self.imports.add("java.time.*")
        self.imports.add("java.time.format.*")
        self.imports.add("java.text.*")
        
        # First pass: Check for modules and extract them
        modules_found = []
        non_module_statements = []
        
        for stmt in program.statements:
            if isinstance(stmt, ModuleDefinition):
                modules_found.append(stmt)
                self.has_modules = True
            else:
                non_module_statements.append(stmt)
        
        # If we have modules, treat all modules as separate classes
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