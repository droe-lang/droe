"""Parser for Roe DSL - converts DSL text to AST."""

import re
from typing import List, Optional, Tuple
from .ast import (
    ASTNode, Program, DisplayStatement, IfStatement,
    Literal, Identifier, BinaryOp, PropertyAccess,
    Assignment, ArrayLiteral, WhileLoop, ForEachLoop, ArithmeticOp
)


class ParseError(Exception):
    """Raised when parsing fails."""
    pass


class Parser:
    """Simple recursive-descent parser for Roe DSL."""
    
    def __init__(self, source: str):
        self.source = source
        self.lines = [line.strip() for line in source.strip().split('\n')]
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
        
        # Variable assignment (set x to value)
        elif line.startswith('set '):
            return self.parse_assignment(line)
        
        # While loop
        elif line.startswith('while '):
            return self.parse_while_loop()
        
        # For each loop
        elif line.startswith('for each ') or line.startswith('loop '):
            return self.parse_foreach_loop()
        
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
        
        # TODO: Support multi-line if statements and else clauses
        
        return IfStatement(condition=condition, then_body=then_body)
    
    def parse_assignment(self, line: str) -> Assignment:
        """Parse a variable assignment (set x to value or set x which are group of type to value)."""
        # Match pattern with optional type declaration:
        # set variable to value
        # set variable which are group of type to value
        
        # First try the typed array version
        typed_array_match = re.match(r'set\s+([a-zA-Z_][a-zA-Z0-9_]*)\s+which\s+are\s+group\s+of\s+(\w+)\s+to\s+(.+)', line, re.IGNORECASE)
        if typed_array_match:
            variable = typed_array_match.group(1)
            declared_type = typed_array_match.group(2).lower()
            value_str = typed_array_match.group(3)
            
            # Parse the value expression
            value = self.parse_expression(value_str)
            
            # Add type information to the assignment
            assignment = Assignment(variable=variable, value=value)
            assignment.declared_type = declared_type  # Store the declared type for arrays
            return assignment
        
        # Try the typed variable version  
        typed_var_match = re.match(r'set\s+([a-zA-Z_][a-zA-Z0-9_]*)\s+which\s+is\s+(?:a\s+)?(\w+)\s+to\s+(.+)', line, re.IGNORECASE)
        if typed_var_match:
            variable = typed_var_match.group(1)
            declared_type = typed_var_match.group(2).lower()
            value_str = typed_var_match.group(3)
            
            # Parse the value expression
            value = self.parse_expression(value_str)
            
            # Add type information to the assignment
            assignment = Assignment(variable=variable, value=value)
            assignment.declared_var_type = declared_type  # Store the declared type for variables
            return assignment
        
        # Fallback to regular assignment
        match = re.match(r'set\s+([a-zA-Z_][a-zA-Z0-9_]*)\s+to\s+(.+)', line, re.IGNORECASE)
        if not match:
            raise ParseError(f"Invalid assignment statement: {line}")
        
        variable = match.group(1)
        value_str = match.group(2)
        
        # Parse the value expression
        value = self.parse_expression(value_str)
        return Assignment(variable=variable, value=value)
    
    def parse_while_loop(self) -> WhileLoop:
        """Parse a while loop (multi-line)."""
        # Current line should be "while condition"
        line = self.lines[self.current_line]
        
        # Extract condition
        match = re.match(r'while\s+(.+)', line, re.IGNORECASE)
        if not match:
            raise ParseError(f"Invalid while statement: {line}")
        
        condition_str = match.group(1)
        condition = self.parse_expression(condition_str)
        
        # Parse body until "end while"
        body = []
        self.current_line += 1  # Move to next line
        
        while self.current_line < len(self.lines):
            line = self.lines[self.current_line].strip()
            
            if line.lower() == 'end while':
                break
            
            if line:  # Skip empty lines
                stmt = self.parse_statement(line)
                if stmt:
                    body.append(stmt)
            
            self.current_line += 1
        
        if self.current_line >= len(self.lines):
            raise ParseError("Missing 'end while' for while loop")
        
        return WhileLoop(condition=condition, body=body)
    
    def parse_foreach_loop(self) -> ForEachLoop:
        """Parse a for each loop (multi-line)."""
        # Current line should be "for each item in items" or "loop item in items"
        line = self.lines[self.current_line]
        
        # Extract variable and iterable
        match = re.match(r'(?:for each|loop)\s+([a-zA-Z_][a-zA-Z0-9_]*)\s+in\s+(.+)', line, re.IGNORECASE)
        if not match:
            raise ParseError(f"Invalid for each statement: {line}")
        
        variable = match.group(1)
        iterable_str = match.group(2)
        iterable = self.parse_expression(iterable_str)
        
        # Parse body until "end for" or "end loop"
        body = []
        self.current_line += 1  # Move to next line
        
        while self.current_line < len(self.lines):
            line = self.lines[self.current_line].strip()
            
            if line.lower() in ['end for', 'end loop']:
                break
            
            if line:  # Skip empty lines
                stmt = self.parse_statement(line)
                if stmt:
                    body.append(stmt)
            
            self.current_line += 1
        
        if self.current_line >= len(self.lines):
            raise ParseError("Missing 'end for' or 'end loop' for loop")
        
        return ForEachLoop(variable=variable, iterable=iterable, body=body)
    
    def parse_expression(self, expr_str: str) -> ASTNode:
        """Parse an expression."""
        expr_str = expr_str.strip()
        
        # Boolean literals (check these early to avoid "is" operator confusion)
        if expr_str.lower() == 'true' or expr_str.lower() == 'the condition is true':
            return Literal(value=True, type='boolean')
        elif expr_str.lower() == 'false' or expr_str.lower() == 'the condition is false':
            return Literal(value=False, type='boolean')
        
        # Array literal
        if expr_str.startswith('[') and expr_str.endswith(']'):
            return self.parse_array_literal(expr_str)
        
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
            (' plus ', '+'),
            (' minus ', '-'),
            (' times ', '*'),
            (' divided by ', '/'),
        ]
        
        for natural_op, symbol_op in natural_ops:
            if natural_op in expr_str:
                parts = expr_str.split(natural_op, 1)
                if len(parts) == 2:
                    left = self.parse_expression(parts[0].strip())
                    right = self.parse_expression(parts[1].strip())
                    # Use ArithmeticOp for arithmetic operators, BinaryOp for comparisons
                    if symbol_op in ['+', '-', '*', '/']:
                        return ArithmeticOp(left=left, operator=symbol_op, right=right)
                    else:
                        return BinaryOp(left=left, operator=symbol_op, right=right)
        
        # Binary operations (symbolic - check after natural language)
        for op in ['>=', '<=', '==', '!=', '>', '<', '+', '-', '*', '/']:
            if op in expr_str:
                parts = expr_str.split(op, 1)
                if len(parts) == 2:
                    left = self.parse_expression(parts[0].strip())
                    right = self.parse_expression(parts[1].strip())
                    # Use ArithmeticOp for arithmetic operators, BinaryOp for comparisons
                    if op in ['+', '-', '*', '/']:
                        return ArithmeticOp(left=left, operator=op, right=right)
                    else:
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
    
    def parse_array_literal(self, expr_str: str) -> ArrayLiteral:
        """Parse an array literal like ["a", "b", "c"]."""
        # Remove brackets
        content = expr_str[1:-1].strip()
        
        if not content:  # Empty array
            return ArrayLiteral(elements=[])
        
        # Split by commas and parse each element
        elements = []
        # Simple comma splitting (doesn't handle nested arrays properly, but sufficient for now)
        parts = content.split(',')
        
        for part in parts:
            part = part.strip()
            if part:
                element = self.parse_expression(part)
                elements.append(element)
        
        return ArrayLiteral(elements=elements)


def parse(source: str) -> Program:
    """Convenience function to parse Roe DSL source code."""
    parser = Parser(source)
    return parser.parse()