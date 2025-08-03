"""WebAssembly Text (WAT) code generator for Roe DSL AST."""

from typing import List, Dict, Any
from .ast import (
    ASTNode, Program, DisplayStatement, IfStatement,
    Literal, Identifier, BinaryOp, PropertyAccess
)


class CodeGenError(Exception):
    """Raised when code generation fails."""
    pass


class WATCodeGenerator:
    """Generates WebAssembly Text format from AST."""
    
    def __init__(self):
        self.indent_level = 0
        self.output = []
        self.string_constants = {}  # Map string -> index
        self.next_string_index = 0
        
    def generate(self, ast: Program) -> str:
        """Generate WAT code from AST."""
        self.output = []
        self.string_constants = {}
        self.next_string_index = 0
        
        # Module header
        self.emit("(module")
        self.indent_level += 1
        
        # Import print function  
        self.emit('(import "env" "print" (func $print (param i32 i32)))')
        
        # Memory for string storage
        self.emit('(memory 1)')
        self.emit('(export "memory" (memory 0))')
        
        # Generate code for all statements
        for stmt in ast.statements:
            self.visit(stmt)
        
        # Data section for string constants
        if self.string_constants:
            self.emit_string_data()
        
        # Main function wrapper
        self.emit('(func $main')
        self.indent_level += 1
        
        # Generate function body
        for stmt in ast.statements:
            self.emit_statement(stmt)
        
        self.indent_level -= 1
        self.emit(')')
        
        # Export main function
        self.emit('(export "main" (func $main))')
        
        self.indent_level -= 1
        self.emit(")")
        
        return '\n'.join(self.output)
    
    def emit(self, code: str):
        """Emit a line of code with proper indentation."""
        indent = "  " * self.indent_level
        self.output.append(f"{indent}{code}")
    
    def visit(self, node: ASTNode):
        """Visit an AST node to collect string constants."""
        if isinstance(node, Literal) and node.type == 'string':
            if node.value not in self.string_constants:
                self.string_constants[node.value] = self.next_string_index
                self.next_string_index += 1
        
        elif isinstance(node, DisplayStatement):
            self.visit(node.expression)
        
        elif isinstance(node, IfStatement):
            self.visit(node.condition)
            for stmt in node.then_body:
                self.visit(stmt)
            if node.else_body:
                for stmt in node.else_body:
                    self.visit(stmt)
        
        elif isinstance(node, BinaryOp):
            self.visit(node.left)
            self.visit(node.right)
        
        elif isinstance(node, Program):
            for stmt in node.statements:
                self.visit(stmt)
    
    def emit_string_data(self):
        """Emit data section for string constants."""
        self.emit('')
        self.emit(';; String constants')
        
        offset = 0
        for string, index in sorted(self.string_constants.items(), key=lambda x: x[1]):
            # Store string with null terminator
            bytes_str = ''.join(f'\\{ord(c):02x}' for c in string) + '\\00'
            self.emit(f'(data (i32.const {offset}) "{bytes_str}")')
            # Update offset for next string
            offset += len(string) + 1
        
        self.emit('')
    
    def emit_statement(self, stmt: ASTNode):
        """Emit code for a statement."""
        if isinstance(stmt, DisplayStatement):
            self.emit_display(stmt)
        
        elif isinstance(stmt, IfStatement):
            self.emit_if(stmt)
        
        else:
            raise CodeGenError(f"Unsupported statement type: {type(stmt).__name__}")
    
    def emit_display(self, stmt: DisplayStatement):
        """Emit code for display statement."""
        self.emit(';; display statement')
        
        if isinstance(stmt.expression, Literal) and stmt.expression.type == 'string':
            # Get string index and calculate offset
            string_index = self.string_constants.get(stmt.expression.value, 0)
            offset = sum(len(s) + 1 for s, i in self.string_constants.items() if i < string_index)
            string_length = len(stmt.expression.value)
            
            # Push string address and length, then call print
            self.emit(f'i32.const {offset}')
            self.emit(f'i32.const {string_length}')
            self.emit('call $print')
        
        else:
            # TODO: Support other expression types
            raise CodeGenError(f"Display only supports string literals currently")
    
    def emit_if(self, stmt: IfStatement):
        """Emit code for if statement."""
        self.emit(';; if statement')
        
        # Emit condition
        self.emit_expression(stmt.condition)
        
        # If-then structure
        self.emit('if')
        self.indent_level += 1
        
        # Emit then body
        for s in stmt.then_body:
            self.emit_statement(s)
        
        # TODO: Support else clause
        
        self.indent_level -= 1
        self.emit('end')
    
    def emit_expression(self, expr: ASTNode):
        """Emit code for an expression that produces a value."""
        if isinstance(expr, Literal):
            if expr.type == 'number':
                if isinstance(expr.value, int):
                    self.emit(f'i32.const {expr.value}')
                else:
                    self.emit(f'f32.const {expr.value}')
            elif expr.type == 'boolean':
                self.emit(f'i32.const {1 if expr.value else 0}')
            else:
                raise CodeGenError(f"Cannot emit {expr.type} literal as expression")
        
        elif isinstance(expr, BinaryOp):
            # Emit left operand
            self.emit_expression(expr.left)
            
            # Emit right operand
            self.emit_expression(expr.right)
            
            # Emit operator
            if expr.operator == '>':
                self.emit('i32.gt_s')
            elif expr.operator == '<':
                self.emit('i32.lt_s')
            elif expr.operator == '>=':
                self.emit('i32.ge_s')
            elif expr.operator == '<=':
                self.emit('i32.le_s')
            elif expr.operator == '==':
                self.emit('i32.eq')
            elif expr.operator == '!=':
                self.emit('i32.ne')
            else:
                raise CodeGenError(f"Unsupported operator: {expr.operator}")
        
        elif isinstance(expr, PropertyAccess):
            # TODO: Implement property access (requires symbol table)
            raise CodeGenError("Property access not yet implemented")
        
        elif isinstance(expr, Identifier):
            # TODO: Implement identifier lookup (requires symbol table)
            raise CodeGenError("Identifier lookup not yet implemented")
        
        else:
            raise CodeGenError(f"Cannot emit expression of type: {type(expr).__name__}")


def generate_wat(ast: Program) -> str:
    """Generate WAT code from AST."""
    generator = WATCodeGenerator()
    return generator.generate(ast)