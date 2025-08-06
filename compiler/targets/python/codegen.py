"""Python code generator for Roelang compiler."""

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


class PythonCodeGenerator(BaseCodeGenerator):
    """Generates Python code from Roelang AST."""
    
    def __init__(self):
        super().__init__()
        self.imports = set()
        self.class_definitions = []
        self.function_definitions = []
        self.main_code = []
    
    def generate(self, program: Program) -> str:
        """Generate Python code from AST."""
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
        return self._build_python_file()
    
    def _build_python_file(self) -> str:
        """Build the complete Python file."""
        lines = []
        
        # Add imports
        lines.extend(sorted(self.imports))
        # No runtime import needed - using inline code generation
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
            # Add as comment for unsupported statements
            self.main_code.append(f"# TODO: Implement {type(stmt).__name__}")
    
    def emit_expression(self, expr: ASTNode) -> str:
        """Emit code for an expression and return the expression string."""
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