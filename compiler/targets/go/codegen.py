"""Go code generator for Roelang compiler."""

from typing import List, Dict, Any
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


class GoCodeGenerator(BaseCodeGenerator):
    """Generates Go code from Roelang AST."""
    
    def __init__(self):
        super().__init__()
        self.imports = set()
        self.type_definitions = []
        self.function_definitions = []
        self.main_code = []
        self.variables = {}  # Track variable types for Go typing
    
    def generate(self, program: Program) -> str:
        """Generate Go code from AST."""
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
        
        # Add runtime library
        lines.extend(self._generate_runtime_library())
        lines.append("")
        
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
        
        return "\\n".join(lines)
    
    def _generate_runtime_library(self) -> List[str]:
        """Generate runtime library functions."""
        runtime = [
            "// Roelang Runtime Library",
            "",
            "// Display prints a value with appropriate formatting",
            "func Display(value interface{}) {",
            "\\tswitch v := value.(type) {",
            "\\tcase bool:",
            "\\t\\tif v {",
            '\\t\\t\\tfmt.Println("true")',
            "\\t\\t} else {",
            '\\t\\t\\tfmt.Println("false")',
            "\\t\\t}",
            "\\tcase []interface{}:",
            '\\t\\tfmt.Print("[")',
            "\\t\\tfor i, item := range v {",
            "\\t\\t\\tif i > 0 {",
            '\\t\\t\\t\\tfmt.Print(", ")',
            "\\t\\t\\t}",
            "\\t\\t\\tfmt.Print(item)",
            "\\t\\t}",
            '\\t\\tfmt.Println("]")',
            "\\tdefault:",
            "\\t\\tfmt.Println(v)",
            "\\t}",
            "}",
            "",
            "// FormatDate formats a date string according to pattern",
            "func FormatDate(dateStr, pattern string) string {",
            '\\tt, err := time.Parse("2006-01-02", dateStr)',
            "\\tif err != nil {",
            "\\t\\treturn dateStr",
            "\\t}",
            "",
            "\\tswitch pattern {",
            '\\tcase "MM/dd/yyyy":',
            '\\t\\treturn t.Format("01/02/2006")',
            '\\tcase "dd/MM/yyyy":',
            '\\t\\treturn t.Format("02/01/2006")',
            '\\tcase "MMM dd, yyyy":',
            '\\t\\treturn t.Format("Jan 02, 2006")',
            '\\tcase "long":',
            '\\t\\treturn t.Format("Monday, January 02, 2006")',
            '\\tcase "short":',
            '\\t\\treturn t.Format("01/02/06")',
            '\\tcase "iso":',
            '\\t\\treturn t.Format("2006-01-02")',
            "\\tdefault:",
            "\\t\\treturn dateStr",
            "\\t}",
            "}",
            "",
            "// FormatDecimal formats a decimal value according to pattern",
            "func FormatDecimal(value float64, pattern string) string {",
            "\\tswitch pattern {",
            '\\tcase "0.00":',
            '\\t\\treturn fmt.Sprintf("%.2f", value)',
            '\\tcase "$0.00":',
            '\\t\\treturn fmt.Sprintf("$%.2f", value)',
            '\\tcase "percent":',
            '\\t\\treturn fmt.Sprintf("%.2f%%", value)',
            "\\tdefault:",
            '\\t\\treturn fmt.Sprintf("%.2f", value)',
            "\\t}",
            "}",
            "",
            "// FormatNumber formats an integer according to pattern",
            "func FormatNumber(value int, pattern string) string {",
            "\\tswitch pattern {",
            '\\tcase "0000":',
            '\\t\\treturn fmt.Sprintf("%04d", value)',
            '\\tcase "hex":',
            '\\t\\treturn fmt.Sprintf("0x%X", value)',
            '\\tcase "oct":',
            '\\t\\treturn fmt.Sprintf("0o%o", value)',
            '\\tcase "bin":',
            '\\t\\treturn fmt.Sprintf("0b%b", value)',
            "\\tdefault:",
            '\\t\\treturn fmt.Sprintf("%d", value)',
            "\\t}",
            "}",
            ""
        ]
        return runtime
    
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
                return f"FormatDate({expr_str}, {pattern})"
            elif expr_type == VariableType.DECIMAL:
                return f"FormatDecimal({expr_str}, {pattern})"
            elif self._is_numeric_type(expr_type):
                return f"FormatNumber({expr_str}, {pattern})"
            else:
                return expr_str
        else:
            return f"/* TODO: {type(expr).__name__} */"
    
    def emit_display_statement(self, stmt: DisplayStatement):
        """Emit display statement."""
        expr_str = self.emit_expression(stmt.expression)
        self.main_code.append(f"Display({expr_str})")
    
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