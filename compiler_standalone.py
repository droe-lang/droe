#!/usr/bin/env python3
"""
Standalone compiler for Roelang that gets installed to ~/.roelang/compiler.py
This file contains all the compiler functionality in a single module.
"""

import re
from dataclasses import dataclass
from typing import Any, List, Optional, Union


# ============= AST Definitions =============

@dataclass
class ASTNode:
    """Base class for all AST nodes."""
    pass


@dataclass
class Literal(ASTNode):
    """Represents a literal value (string, number, boolean)."""
    value: Union[str, int, float, bool]
    type: str  # 'string', 'number', 'boolean'


@dataclass
class Identifier(ASTNode):
    """Represents an identifier (variable name)."""
    name: str


@dataclass
class BinaryOp(ASTNode):
    """Represents a binary operation (e.g., >, <, ==, +, -)."""
    left: ASTNode
    operator: str
    right: ASTNode


@dataclass
class DisplayStatement(ASTNode):
    """Represents a display statement."""
    expression: ASTNode


@dataclass
class IfStatement(ASTNode):
    """Represents an if-then statement."""
    condition: ASTNode
    then_body: List[ASTNode]
    else_body: Optional[List[ASTNode]] = None


@dataclass
class PropertyAccess(ASTNode):
    """Represents property access (e.g., user.age)."""
    object: ASTNode
    property: str


@dataclass
class Program(ASTNode):
    """Root node containing all statements in the program."""
    statements: List[ASTNode]


# ============= Parser =============

class ParseError(Exception):
    """Raised when parsing fails."""
    pass


class Parser:
    """Simple recursive-descent parser for Roe DSL."""
    
    def __init__(self, source: str):
        self.source = source
        self.lines = source.strip().split('\n')
        self.current_line = 0
    
    def parse(self) -> Program:
        """Parse the entire program."""
        statements = []
        
        while self.current_line < len(self.lines):
            line = self.lines[self.current_line].strip()
            if not line:  # Skip empty lines
                self.current_line += 1
                continue
                
            stmt = self.parse_statement(line)
            if stmt:
                statements.append(stmt)
            self.current_line += 1
        
        return Program(statements=statements)
    
    def parse_statement(self, line: str) -> Optional[ASTNode]:
        """Parse a single statement."""
        line = line.strip()
        
        # Display/Show statement
        if line.startswith('display ') or line.startswith('show '):
            return self.parse_display(line)
        
        # If/When statement
        elif line.startswith('if ') or line.startswith('when '):
            return self.parse_if(line)
        
        else:
            raise ParseError(f"Unknown statement: {line}")
    
    def parse_display(self, line: str) -> DisplayStatement:
        """Parse a display/show statement."""
        # Remove 'display ' or 'show ' prefix
        if line.startswith('display '):
            content = line[8:].strip()
        elif line.startswith('show '):
            content = line[5:].strip()
        else:
            raise ParseError(f"Invalid display statement: {line}")
        
        # Parse the expression
        expr = self.parse_expression(content)
        return DisplayStatement(expression=expr)
    
    def parse_if(self, line: str) -> IfStatement:
        """Parse an if/when-then statement."""
        # Simple regex to extract condition and then-body
        match = re.match(r'(?:if|when)\s+(.+?)\s+then\s+(.+)', line, re.IGNORECASE)
        if not match:
            raise ParseError(f"Invalid if/when statement: {line}")
        
        condition_str = match.group(1)
        then_str = match.group(2)
        
        # Parse condition
        condition = self.parse_expression(condition_str)
        
        # Parse then body (for now, just a single statement)
        then_stmt = self.parse_statement(then_str)
        then_body = [then_stmt] if then_stmt else []
        
        return IfStatement(condition=condition, then_body=then_body)
    
    def parse_expression(self, expr_str: str) -> ASTNode:
        """Parse an expression."""
        expr_str = expr_str.strip()
        
        # Boolean literals (check these early to avoid "is" operator confusion)
        if expr_str.lower() == 'true' or expr_str.lower() == 'the condition is true':
            return Literal(value=True, type='boolean')
        elif expr_str.lower() == 'false' or expr_str.lower() == 'the condition is false':
            return Literal(value=False, type='boolean')
        
        # String literal
        if (expr_str.startswith('"') and expr_str.endswith('"')) or \
           (expr_str.startswith("'") and expr_str.endswith("'")):
            return Literal(value=expr_str[1:-1], type='string')
        
        # Number literal
        try:
            if '.' in expr_str:
                return Literal(value=float(expr_str), type='number')
            else:
                return Literal(value=int(expr_str), type='number')
        except ValueError:
            pass
        
        # Natural language operators (check these first)
        natural_ops = [
            (' is greater than or equal to ', '>='),
            (' is less than or equal to ', '<='),
            (' is greater than ', '>'),
            (' is less than ', '<'),
            (' does not equal ', '!='),
            (' is not equal to ', '!='),
            (' is not ', '!='),
            (' equals ', '=='),
            (' is equal to ', '=='),
            (' is ', '=='),  # Generic 'is' maps to equality
        ]
        
        for natural_op, symbol_op in natural_ops:
            if natural_op in expr_str:
                parts = expr_str.split(natural_op, 1)
                if len(parts) == 2:
                    left = self.parse_expression(parts[0].strip())
                    right = self.parse_expression(parts[1].strip())
                    return BinaryOp(left=left, operator=symbol_op, right=right)
        
        # Binary operations (symbolic - check after natural language)
        for op in ['>=', '<=', '==', '!=', '>', '<', '+', '-', '*', '/']:
            if op in expr_str:
                parts = expr_str.split(op, 1)
                if len(parts) == 2:
                    left = self.parse_expression(parts[0].strip())
                    right = self.parse_expression(parts[1].strip())
                    return BinaryOp(left=left, operator=op, right=right)
        
        # Property access (e.g., user.age)
        if '.' in expr_str:
            parts = expr_str.split('.', 1)
            if len(parts) == 2:
                obj = Identifier(name=parts[0].strip())
                return PropertyAccess(object=obj, property=parts[1].strip())
        
        
        # Simple identifier
        if re.match(r'^[a-zA-Z_][a-zA-Z0-9_]*$', expr_str):
            return Identifier(name=expr_str)
        
        raise ParseError(f"Unable to parse expression: {expr_str}")


# ============= Code Generator =============

class CodeGenError(Exception):
    """Raised when code generation fails."""
    pass


class WATCodeGenerator:
    """Generates WebAssembly Text format from AST."""
    
    def __init__(self):
        self.indent_level = 0
        self.output = []
        self.string_constants = {}
        self.next_string_index = 0
        
    def generate(self, ast: Program) -> str:
        """Generate WAT code from AST."""
        self.output = []
        self.string_constants = {}
        self.next_string_index = 0
        
        # Module header
        self.emit("(module")
        self.indent_level += 1
        
        # Import display function
        self.emit('(import "env" "display" (func $display (param i32)))')
        
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
            # Get string index
            string_index = self.string_constants.get(stmt.expression.value, 0)
            offset = sum(len(s) + 1 for s, i in self.string_constants.items() if i < string_index)
            
            # Push string address and call display
            self.emit(f'i32.const {offset}')
            self.emit('call $display')
        
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


# ============= Compiler Interface =============

class CompilerError(Exception):
    """General compiler error."""
    pass


def compile(source: str) -> str:
    """Compile Roe DSL source code to WebAssembly Text format."""
    try:
        parser = Parser(source)
        ast = parser.parse()
        
        generator = WATCodeGenerator()
        wat = generator.generate(ast)
        
        return wat
        
    except ParseError as e:
        raise CompilerError(f"Parse error: {str(e)}")
    except CodeGenError as e:
        raise CompilerError(f"Code generation error: {str(e)}")
    except Exception as e:
        raise CompilerError(f"Unexpected error: {str(e)}")


def compile_roe_to_wat(input_path, output_path):
    """
    Compile a Roe DSL file to WebAssembly Text format.
    
    This function provides backward compatibility with the old compiler interface
    and also supports the legacy "Display" syntax.
    """
    input_path = str(input_path)
    output_path = str(output_path)
    
    # Read source file
    with open(input_path, "r") as f:
        content = f.read().strip()
    
    try:
        # Try new syntax first
        wat = compile(content)
        
    except CompilerError:
        # Check if it's legacy syntax
        if content.startswith("Display "):
            # Convert old syntax to new syntax
            message = content[len("Display "):]
            new_content = f'display "{message}"'
            
            try:
                wat = compile(new_content)
                
                # Adjust WAT to match old format (using print instead of display)
                wat = wat.replace('"env" "display"', '"env" "print"')
                wat = wat.replace('$display (param i32)', '$print (param i32 i32)')
                wat = wat.replace('call $display', f'i32.const {len(message)}\n    call $print')
                
            except Exception as e:
                raise ValueError(f"Compilation failed: {str(e)}")
        else:
            raise
    
    # Write output file
    with open(output_path, "w") as f:
        f.write(wat)
    
    print(f"✅ Compiled {input_path} to {output_path}")