"""WebAssembly Text (WAT) code generator for Roe DSL AST."""

from typing import List, Dict, Any
from .ast import (
    ASTNode, Program, DisplayStatement, IfStatement,
    Literal, Identifier, BinaryOp, PropertyAccess,
    Assignment, ArrayLiteral, WhileLoop, ForEachLoop, ArithmeticOp
)
from .symbols import SymbolTable, VariableType


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
        self.symbol_table = SymbolTable()
        self.loop_depth = 0  # Track nested loops for break/continue
        
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
        
        # Generate code for all statements to collect string constants
        for stmt in ast.statements:
            self.visit(stmt)
        
        # First pass: collect variables from assignments
        for stmt in ast.statements:
            self.collect_variables(stmt)
        
        # Main function wrapper with local variables
        if self.symbol_table.get_local_count() > 0:
            self.emit('(func $main')
            self.indent_level += 1
            
            # Declare local variables
            for var in self.symbol_table.get_all_variables().values():
                if var.type == VariableType.NUMBER:
                    self.emit('(local i32)')
                elif var.type == VariableType.BOOLEAN:
                    self.emit('(local i32)')
                elif var.type == VariableType.STRING:
                    self.emit('(local i32)')  # String offset
                    self.emit('(local i32)')  # String length
                elif var.type == VariableType.ARRAY:
                    self.emit('(local i32)')  # Array pointer
                    self.emit('(local i32)')  # Array length
        else:
            self.emit('(func $main')
            self.indent_level += 1
        
        # Generate function body
        for stmt in ast.statements:
            self.emit_statement(stmt)
        
        self.indent_level -= 1
        self.emit(')')
        
        # Data section for string constants (after function)
        if self.string_constants:
            self.emit_string_data()
        
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
    
    def collect_variables(self, node: ASTNode):
        """Collect variable declarations from the AST."""
        if isinstance(node, Assignment):
            # Determine variable type from value
            var_type = self.infer_type(node.value)
            self.symbol_table.declare_variable(node.variable, var_type)
        
        elif isinstance(node, WhileLoop):
            for stmt in node.body:
                self.collect_variables(stmt)
        
        elif isinstance(node, ForEachLoop):
            # The loop variable will be of the same type as array elements
            # For now, assume it's a string (most common case)
            self.symbol_table.declare_variable(node.variable, VariableType.STRING)
            for stmt in node.body:
                self.collect_variables(stmt)
        
        elif isinstance(node, IfStatement):
            for stmt in node.then_body:
                self.collect_variables(stmt)
            if node.else_body:
                for stmt in node.else_body:
                    self.collect_variables(stmt)
    
    def infer_type(self, node: ASTNode) -> VariableType:
        """Infer the type of a value node."""
        if isinstance(node, Literal):
            if node.type == 'string':
                return VariableType.STRING
            elif node.type == 'number':
                return VariableType.NUMBER
            elif node.type == 'boolean':
                return VariableType.BOOLEAN
        elif isinstance(node, ArrayLiteral):
            return VariableType.ARRAY
        elif isinstance(node, Identifier):
            # Look up existing variable type
            var = self.symbol_table.get_variable(node.name)
            if var:
                return var.type
            else:
                # Default to string if unknown
                return VariableType.STRING
        elif isinstance(node, ArithmeticOp):
            return VariableType.NUMBER
        elif isinstance(node, BinaryOp):
            return VariableType.BOOLEAN
        
        # Default fallback
        return VariableType.STRING
    
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
        
        elif isinstance(stmt, Assignment):
            self.emit_assignment(stmt)
        
        elif isinstance(stmt, WhileLoop):
            self.emit_while_loop(stmt)
        
        elif isinstance(stmt, ForEachLoop):
            self.emit_foreach_loop(stmt)
        
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
        
        elif isinstance(stmt.expression, Identifier):
            # Display a variable - for now, convert numbers to strings
            var = self.symbol_table.get_variable(stmt.expression.name)
            if not var:
                raise CodeGenError(f"Undeclared variable: {stmt.expression.name}")
            
            if var.type == VariableType.NUMBER:
                # For numbers, we'll need a helper function to convert to string
                # For now, just display a placeholder
                placeholder = f"<number:{stmt.expression.name}>"
                if placeholder not in self.string_constants:
                    self.string_constants[placeholder] = self.next_string_index
                    self.next_string_index += 1
                
                string_index = self.string_constants[placeholder]
                offset = sum(len(s) + 1 for s, i in self.string_constants.items() if i < string_index)
                string_length = len(placeholder)
                
                self.emit(f'i32.const {offset}')
                self.emit(f'i32.const {string_length}')
                self.emit('call $print')
            else:
                # For strings and other types, also use placeholder for now
                placeholder = f"<{var.type.value}:{stmt.expression.name}>"
                if placeholder not in self.string_constants:
                    self.string_constants[placeholder] = self.next_string_index
                    self.next_string_index += 1
                
                string_index = self.string_constants[placeholder]
                offset = sum(len(s) + 1 for s, i in self.string_constants.items() if i < string_index)
                string_length = len(placeholder)
                
                self.emit(f'i32.const {offset}')
                self.emit(f'i32.const {string_length}')
                self.emit('call $print')
        
        else:
            # TODO: Support other expression types
            raise CodeGenError(f"Display only supports string literals and variables currently")
    
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
            elif expr.type == 'string':
                # For string literals in expressions (like assignments), 
                # we store them as string constants and emit the offset
                if expr.value not in self.string_constants:
                    self.string_constants[expr.value] = self.next_string_index
                    self.next_string_index += 1
                
                string_index = self.string_constants[expr.value]
                offset = sum(len(s) + 1 for s, i in self.string_constants.items() if i < string_index)
                self.emit(f'i32.const {offset}')
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
            # Load variable value
            var = self.symbol_table.get_variable(expr.name)
            if not var:
                raise CodeGenError(f"Undeclared variable: {expr.name}")
            self.emit(f'local.get {var.wasm_index}')
        
        elif isinstance(expr, ArithmeticOp):
            # Emit left operand
            self.emit_expression(expr.left)
            
            # Emit right operand  
            self.emit_expression(expr.right)
            
            # Emit operator
            if expr.operator == '+':
                self.emit('i32.add')
            elif expr.operator == '-':
                self.emit('i32.sub')
            elif expr.operator == '*':
                self.emit('i32.mul')
            elif expr.operator == '/':
                self.emit('i32.div_s')
            else:
                raise CodeGenError(f"Unsupported arithmetic operator: {expr.operator}")
        
        elif isinstance(expr, ArrayLiteral):
            # For now, just emit the first element or 0 if empty
            # TODO: Implement proper array support
            if expr.elements:
                self.emit_expression(expr.elements[0])
            else:
                self.emit('i32.const 0')
        
        else:
            raise CodeGenError(f"Cannot emit expression of type: {type(expr).__name__}")
    
    def emit_assignment(self, stmt: Assignment):
        """Emit code for variable assignment."""
        self.emit(f';; set {stmt.variable}')
        
        var = self.symbol_table.get_variable(stmt.variable)
        if not var:
            raise CodeGenError(f"Undeclared variable: {stmt.variable}")
        
        # Emit the value expression
        self.emit_expression(stmt.value)
        
        # Store in local variable
        self.emit(f'local.set {var.wasm_index}')
    
    def emit_while_loop(self, stmt: WhileLoop):
        """Emit code for while loop."""
        self.emit(';; while loop')
        self.emit('loop $while_loop')
        self.indent_level += 1
        
        # Emit condition
        self.emit_expression(stmt.condition)
        
        # If condition is false, break out of loop
        self.emit('i32.eqz')
        self.emit('br_if 1')  # Break out of the loop
        
        # Emit loop body
        for s in stmt.body:
            self.emit_statement(s)
        
        # Continue loop
        self.emit('br $while_loop')
        
        self.indent_level -= 1
        self.emit('end')
    
    def emit_foreach_loop(self, stmt: ForEachLoop):
        """Emit code for foreach loop - simplified version."""
        # For now, only support simple array iteration
        # This is a simplified implementation
        self.emit(';; foreach loop (simplified)')
        
        if not isinstance(stmt.iterable, Identifier):
            raise CodeGenError("For each loops currently only support variable arrays")
        
        # TODO: Implement proper array iteration
        # For now, just emit the body once as a placeholder
        self.emit(';; TODO: implement proper array iteration')
        for s in stmt.body:
            self.emit_statement(s)


def generate_wat(ast: Program) -> str:
    """Generate WAT code from AST."""
    generator = WATCodeGenerator()
    return generator.generate(ast)