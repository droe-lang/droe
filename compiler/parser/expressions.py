"""Expression parsing module."""

import re
from typing import Optional, List
from ..ast import (
    ASTNode, Literal, Identifier, BinaryOp, PropertyAccess,
    ArrayLiteral, StringInterpolation, FormatExpression, ArithmeticOp
)
from .base import BaseParser, ParseError


class ExpressionParser(BaseParser):
    """Handles parsing of expressions."""
    
    def parse_expression(self, expr_str: str) -> ASTNode:
        """Parse an expression string into an AST node."""
        expr_str = expr_str.strip()
        
        if not expr_str:
            raise ParseError("Empty expression")
        
        # Check for format expression first
        if 'Format' in expr_str:
            return self.parse_format_expression(expr_str)
        
        # Check for array literal
        if expr_str.startswith('[') and expr_str.endswith(']'):
            return self.parse_array_literal(expr_str)
        
        # Check for string literal with interpolation
        if (expr_str.startswith('"') and expr_str.endswith('"')) or \
           (expr_str.startswith("'") and expr_str.endswith("'")):
            string_content = expr_str[1:-1]
            if '${' in string_content:
                return self.parse_string_interpolation(string_content)
            return Literal(string_content, 'string')
        
        # Check for boolean literals
        if expr_str == 'true':
            return Literal(True, 'boolean')
        if expr_str == 'false':
            return Literal(False, 'boolean')
        
        # Check for number literal
        try:
            if '.' in expr_str:
                value = float(expr_str)
            else:
                value = int(expr_str)
            return Literal(value, 'number')
        except ValueError:
            pass
        
        # Parse arithmetic operations
        arithmetic_result = self._parse_arithmetic(expr_str)
        if arithmetic_result:
            return arithmetic_result
        
        # Check for comparisons and logical operations
        for op_str, op_type in [
            ('==', 'equals'),
            ('!=', 'not_equals'),
            ('<=', 'less_equal'),
            ('>=', 'greater_equal'),
            ('<', 'less_than'),
            ('>', 'greater_than'),
            ('&&', 'and'),
            ('||', 'or'),
            ('+', 'concat')
        ]:
            if op_str in expr_str:
                parts = expr_str.split(op_str, 1)
                if len(parts) == 2:
                    left = self.parse_expression(parts[0].strip())
                    right = self.parse_expression(parts[1].strip())
                    return BinaryOp(left, op_type, right)
        
        # Check for property access
        if '.' in expr_str and not expr_str.replace('.', '').replace('-', '').isdigit():
            parts = expr_str.split('.', 1)
            base = parts[0].strip()
            
            # Check if base is an array element access
            if '[' in base and ']' in base:
                array_name = base[:base.index('[')]
                index_expr = base[base.index('[') + 1:base.index(']')]
                
                # Parse the index expression
                try:
                    index = int(index_expr)
                    index_node = Literal(index, 'number')
                except ValueError:
                    index_node = Identifier(index_expr)
                
                # Create property access for array[index].property
                array_access = PropertyAccess(Identifier(array_name), index_node)
                if len(parts) > 1:
                    return PropertyAccess(array_access, Identifier(parts[1].strip()))
                return array_access
            else:
                base_expr = Identifier(base)
                return PropertyAccess(base_expr, Identifier(parts[1].strip()))
        
        # Check for array element access
        if '[' in expr_str and ']' in expr_str:
            array_name = expr_str[:expr_str.index('[')]
            index_expr = expr_str[expr_str.index('[') + 1:expr_str.index(']')]
            
            # Parse the index expression
            try:
                index = int(index_expr)
                index_node = Literal(index, 'number')
            except ValueError:
                index_node = Identifier(index_expr)
            
            return PropertyAccess(Identifier(array_name), index_node)
        
        # Default to identifier
        return Identifier(expr_str)
    
    def _parse_arithmetic(self, expr_str: str) -> Optional[ArithmeticOp]:
        """Parse arithmetic expressions with proper precedence."""
        expr_str = expr_str.strip()
        
        # Remove outer parentheses if they enclose the entire expression
        if expr_str.startswith('(') and expr_str.endswith(')'):
            # Check if these parentheses are matched
            depth = 0
            for i, char in enumerate(expr_str):
                if char == '(':
                    depth += 1
                elif char == ')':
                    depth -= 1
                    if depth == 0 and i < len(expr_str) - 1:
                        # Parentheses don't enclose entire expression
                        break
            else:
                # Parentheses enclose entire expression, remove them
                expr_str = expr_str[1:-1].strip()
        
        # Parse addition and subtraction (lowest precedence)
        for op in ['+', '-']:
            # Find the operator at the correct precedence level
            depth = 0
            for i in range(len(expr_str) - 1, -1, -1):
                char = expr_str[i]
                if char == ')':
                    depth += 1
                elif char == '(':
                    depth -= 1
                elif depth == 0 and char == op:
                    # Skip unary minus at the beginning
                    if op == '-' and i == 0:
                        continue
                    
                    left_str = expr_str[:i].strip()
                    right_str = expr_str[i+1:].strip()
                    
                    if left_str and right_str:
                        left = self._parse_arithmetic(left_str) or self._parse_term(left_str)
                        right = self._parse_arithmetic(right_str) or self._parse_term(right_str)
                        
                        if left and right:
                            op_type = 'add' if op == '+' else 'subtract'
                            return ArithmeticOp(left, op_type, right)
        
        # Parse multiplication, division, and modulo (higher precedence)
        for op in ['*', '/', '%']:
            depth = 0
            for i in range(len(expr_str) - 1, -1, -1):
                char = expr_str[i]
                if char == ')':
                    depth += 1
                elif char == '(':
                    depth -= 1
                elif depth == 0 and char == op:
                    left_str = expr_str[:i].strip()
                    right_str = expr_str[i+1:].strip()
                    
                    if left_str and right_str:
                        left = self._parse_arithmetic(left_str) or self._parse_term(left_str)
                        right = self._parse_arithmetic(right_str) or self._parse_term(right_str)
                        
                        if left and right:
                            op_type = 'multiply' if op == '*' else ('divide' if op == '/' else 'modulo')
                            return ArithmeticOp(left, op_type, right)
        
        # Not an arithmetic expression
        return None
    
    def _parse_term(self, expr_str: str) -> Optional[ASTNode]:
        """Parse a term (number, identifier, or parenthesized expression)."""
        expr_str = expr_str.strip()
        
        # Handle parenthesized expressions
        if expr_str.startswith('(') and expr_str.endswith(')'):
            inner = expr_str[1:-1].strip()
            return self._parse_arithmetic(inner) or self.parse_expression(inner)
        
        # Try to parse as number
        try:
            if '.' in expr_str:
                return Literal(float(expr_str), 'number')
            else:
                return Literal(int(expr_str), 'number')
        except ValueError:
            pass
        
        # Parse as identifier or property access
        if '.' in expr_str:
            parts = expr_str.split('.', 1)
            base = Identifier(parts[0].strip())
            prop = Identifier(parts[1].strip())
            return PropertyAccess(base, prop)
        
        # Default to identifier
        return Identifier(expr_str)
    
    def parse_array_literal(self, expr_str: str) -> ArrayLiteral:
        """Parse an array literal like [1, 2, 3] or ["a", "b", "c"]."""
        expr_str = expr_str.strip()
        
        if not (expr_str.startswith('[') and expr_str.endswith(']')):
            raise ParseError(f"Invalid array literal: {expr_str}")
        
        content = expr_str[1:-1].strip()
        
        if not content:
            return ArrayLiteral([])
        
        # Parse array elements
        elements = []
        for elem_str in content.split(','):
            elem_str = elem_str.strip()
            elements.append(self.parse_expression(elem_str))
        
        return ArrayLiteral(elements)
    
    def parse_string_interpolation(self, string_content: str) -> StringInterpolation:
        """Parse string interpolation like 'Hello ${name}'."""
        parts = []
        current_pos = 0
        
        while current_pos < len(string_content):
            # Find next interpolation
            interp_start = string_content.find('${', current_pos)
            
            if interp_start == -1:
                # No more interpolations, add rest as literal
                if current_pos < len(string_content):
                    parts.append(Literal(string_content[current_pos:], 'string'))
                break
            
            # Add literal part before interpolation
            if interp_start > current_pos:
                parts.append(Literal(string_content[current_pos:interp_start], 'string'))
            
            # Find end of interpolation
            interp_end = string_content.find('}', interp_start + 2)
            if interp_end == -1:
                raise ParseError(f"Unclosed interpolation in string: {string_content}")
            
            # Parse the interpolated expression
            expr_str = string_content[interp_start + 2:interp_end].strip()
            parts.append(self.parse_expression(expr_str))
            
            current_pos = interp_end + 1
        
        return StringInterpolation(parts)
    
    def parse_format_expression(self, expr_str: str) -> FormatExpression:
        """Parse Format expression like Format(value, "%.2f")."""
        match = re.match(r'Format\s*\(\s*(.+?)\s*,\s*"([^"]+)"\s*\)', expr_str)
        if not match:
            match = re.match(r"Format\s*\(\s*(.+?)\s*,\s*'([^']+)'\s*\)", expr_str)
        
        if not match:
            raise ParseError(f"Invalid Format expression: {expr_str}")
        
        value_expr = self.parse_expression(match.group(1))
        format_string = match.group(2)
        
        return FormatExpression(value_expr, format_string)