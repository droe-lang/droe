"""Node.js code generator for Roelang compiler."""

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


class NodeCodeGenerator(BaseCodeGenerator):
    """Generates Node.js JavaScript code from Roelang AST."""
    
    def __init__(self):
        super().__init__()
        self.requires = set()
        self.class_definitions = []
        self.function_definitions = []
        self.main_code = []
    
    def generate(self, program: Program) -> str:
        """Generate Node.js code from AST."""
        self.clear_output()
        self.requires.clear()
        self.class_definitions.clear()
        self.function_definitions.clear()
        self.main_code.clear()
        
        # Add core requires
        if self.is_core_lib_enabled('formatting'):
            # No specific requires needed for basic formatting
            pass
        
        # Process all statements
        for stmt in program.statements:
            self.emit_statement(stmt)
        
        # Generate final JavaScript code
        return self._build_js_file()
    
    def _build_js_file(self) -> str:
        """Build the complete JavaScript file."""
        lines = []
        
        # Add requires
        for require in sorted(self.requires):
            lines.append(require)
        if self.requires:
            lines.append("")
        
        # Add runtime library
        lines.extend(self._generate_runtime_library())
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
        lines.append("function main() {")
        for line in self.main_code:
            lines.append(f"  {line}")
        lines.append("}")
        lines.append("")
        lines.append("// Run main if this is the entry point")
        lines.append("if (require.main === module) {")
        lines.append("  main();")
        lines.append("}")
        
        return "\\n".join(lines)
    
    def _generate_runtime_library(self) -> List[str]:
        """Generate runtime library functions."""
        runtime = [
            "// Roelang Runtime Library for Node.js",
            "",
            "class RoelangRuntime {",
            "  static display(value) {",
            "    if (typeof value === 'boolean') {",
            "      console.log(value ? 'true' : 'false');",
            "    } else if (Array.isArray(value)) {",
            "      console.log(`[${value.join(', ')}]`);",
            "    } else {",
            "      console.log(value);",
            "    }",
            "  }",
            "",
            "  static formatDate(dateValue, pattern) {",
            "    let date;",
            "    if (typeof dateValue === 'string') {",
            "      date = new Date(dateValue);",
            "    } else {",
            "      date = dateValue;",
            "    }",
            "",
            "    switch (pattern) {",
            "      case 'MM/dd/yyyy':",
            "        const month = (date.getMonth() + 1).toString().padStart(2, '0');",
            "        const day = date.getDate().toString().padStart(2, '0');",
            "        const year = date.getFullYear();",
            "        return `${month}/${day}/${year}`;",
            "      case 'dd/MM/yyyy':",
            "        const month2 = (date.getMonth() + 1).toString().padStart(2, '0');",
            "        const day2 = date.getDate().toString().padStart(2, '0');",
            "        const year2 = date.getFullYear();",
            "        return `${day2}/${month2}/${year2}`;",
            "      case 'MMM dd, yyyy':",
            "        return date.toLocaleDateString('en-US', {",
            "          year: 'numeric',",
            "          month: 'short',",
            "          day: 'numeric'",
            "        });",
            "      case 'long':",
            "        return date.toLocaleDateString('en-US', {",
            "          weekday: 'long',",
            "          year: 'numeric',",
            "          month: 'long',",
            "          day: 'numeric'",
            "        });",
            "      case 'short':",
            "        return date.toLocaleDateString('en-US', {",
            "          year: '2-digit',",
            "          month: '2-digit',",
            "          day: '2-digit'",
            "        });",
            "      case 'iso':",
            "        return date.toISOString().split('T')[0];",
            "      default:",
            "        return dateValue.toString();",
            "    }",
            "  }",
            "",
            "  static formatDecimal(decimalValue, pattern) {",
            "    const num = parseFloat(decimalValue);",
            "    switch (pattern) {",
            "      case '0.00':",
            "        return num.toFixed(2);",
            "      case '#,##0.00':",
            "        return num.toLocaleString('en-US', {",
            "          minimumFractionDigits: 2,",
            "          maximumFractionDigits: 2",
            "        });",
            "      case '$0.00':",
            "        return `$${num.toFixed(2)}`;",
            "      case 'percent':",
            "        return `${num.toFixed(2)}%`;",
            "      default:",
            "        return num.toString();",
            "    }",
            "  }",
            "",
            "  static formatNumber(numberValue, pattern) {",
            "    const num = parseInt(numberValue);",
            "    switch (pattern) {",
            "      case '#,##0':",
            "        return num.toLocaleString('en-US');",
            "      case '0000':",
            "        return num.toString().padStart(4, '0');",
            "      case 'hex':",
            "        return `0x${num.toString(16).toUpperCase()}`;",
            "      case 'oct':",
            "        return `0o${num.toString(8)}`;",
            "      case 'bin':",
            "        return `0b${num.toString(2)}`;",
            "      default:",
            "        return num.toString();",
            "    }",
            "  }",
            "",
            "  // Math utility functions",
            "  static mathPower(base, exponent) {",
            "    return Math.pow(base, exponent);",
            "  }",
            "",
            "  static mathSqrt(value) {",
            "    return Math.sqrt(value);",
            "  }",
            "",
            "  static mathAbs(value) {",
            "    return Math.abs(value);",
            "  }",
            "",
            "  static mathMin(a, b) {",
            "    return Math.min(a, b);",
            "  }",
            "",
            "  static mathMax(a, b) {",
            "    return Math.max(a, b);",
            "  }",
            "}",
            ""
        ]
        return runtime
    
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
                # Escape quotes and newlines
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
            
            # Handle JavaScript-specific operators
            if expr.operator == "==":
                return f"({left} === {right})"
            elif expr.operator == "!=":
                return f"({left} !== {right})"
            else:
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
                return f"RoelangRuntime.formatDate({expr_str}, {pattern})"
            elif expr_type == VariableType.DECIMAL:
                return f"RoelangRuntime.formatDecimal({expr_str}, {pattern})"
            elif self._is_numeric_type(expr_type):
                return f"RoelangRuntime.formatNumber({expr_str}, {pattern})"
            else:
                return expr_str
        else:
            return f"/* TODO: {type(expr).__name__} */"
    
    def emit_display_statement(self, stmt: DisplayStatement):
        """Emit display statement."""
        expr_str = self.emit_expression(stmt.expression)
        self.main_code.append(f"RoelangRuntime.display({expr_str});")
    
    def emit_assignment(self, stmt: Assignment):
        """Emit assignment statement."""
        value_str = self.emit_expression(stmt.value)
        
        # JavaScript uses let for variable declaration
        self.main_code.append(f"let {stmt.variable} = {value_str};")
        
        # Track variable in symbol table with inferred type
        inferred_type = self.infer_type(stmt.value)
        self.symbol_table.declare_variable(stmt.variable, inferred_type)
    
    def emit_if_statement(self, stmt: IfStatement):
        """Emit if statement."""
        condition_str = self.emit_expression(stmt.condition)
        self.main_code.append(f"if ({condition_str}) {{")
        
        # Emit then block (simplified)
        if stmt.then_body:
            for then_stmt in stmt.then_body:
                if isinstance(then_stmt, DisplayStatement):
                    expr_str = self.emit_expression(then_stmt.expression)
                    self.main_code.append(f"  RoelangRuntime.display({expr_str});")
        
        if stmt.else_body:
            self.main_code.append("} else {")
            for else_stmt in stmt.else_body:
                if isinstance(else_stmt, DisplayStatement):
                    expr_str = self.emit_expression(else_stmt.expression)
                    self.main_code.append(f"  RoelangRuntime.display({expr_str});")
        
        self.main_code.append("}")
    
    def emit_while_loop(self, stmt: WhileLoop):
        """Emit while loop."""
        condition_str = self.emit_expression(stmt.condition)
        self.main_code.append(f"while ({condition_str}) {{")
        self.main_code.append("  // Loop body")
        self.main_code.append("}")
    
    def emit_foreach_loop(self, stmt: ForEachLoop):
        """Emit for-each loop."""
        collection_str = self.emit_expression(stmt.collection)
        self.main_code.append(f"for (const {stmt.variable} of {collection_str}) {{")
        self.main_code.append("  // Loop body")
        self.main_code.append("}")
    
    def emit_action_definition(self, stmt: ActionDefinition):
        """Emit action definition as JavaScript function."""
        func_lines = [f"function {stmt.name}() {{"]
        
        if stmt.body:
            for body_stmt in stmt.body:
                if isinstance(body_stmt, ReturnStatement):
                    return_expr = self.emit_expression(body_stmt.expression)
                    func_lines.append(f"  return {return_expr};")
                else:
                    func_lines.append("  // Function body")
        else:
            func_lines.append("  return undefined;")
        
        func_lines.append("}")
        self.function_definitions.append(func_lines)
    
    def emit_module_definition(self, stmt: ModuleDefinition):
        """Emit module definition as JavaScript class."""
        class_lines = [f"class {stmt.name} {{"]
        
        if stmt.body:
            for body_stmt in stmt.body:
                if isinstance(body_stmt, ActionDefinition):
                    class_lines.append(f"  static {body_stmt.name}() {{")
                    class_lines.append("    // Method body")
                    class_lines.append("    return undefined;")
                    class_lines.append("  }")
                    class_lines.append("")
        else:
            class_lines.append("  // Empty class")
        
        class_lines.append("}")
        self.class_definitions.append(class_lines)
    
    def emit_string_interpolation(self, expr: StringInterpolation) -> str:
        """Emit string interpolation as template literal."""
        parts = []
        for part in expr.parts:
            if isinstance(part, str):
                # Escape template literal special characters
                escaped = part.replace('\\\\', '\\\\\\\\').replace('`', '\\\\`').replace('$', '\\\\$')
                parts.append(escaped)
            else:
                part_expr = self.emit_expression(part)
                parts.append(f"${{{part_expr}}}")
        
        interpolated = "".join(parts)
        return f"`{interpolated}`"