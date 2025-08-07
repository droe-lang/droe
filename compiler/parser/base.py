"""Base parser functionality and utilities."""

import re
from typing import List, Optional, Any, Dict
from ..ast import ASTNode


class ParseError(Exception):
    """Raised when parsing fails."""
    pass


class BaseParser:
    """Base parser with common utilities."""
    
    def __init__(self, source: str):
        self.source = source
        self.source = self._remove_comments(source)
        # Don't strip lines to preserve indentation
        self.lines = self.source.split('\n')
        self.current_line = 0
    
    def _remove_comments(self, source: str) -> str:
        """Remove single-line (//) and multi-line (/* */) comments from source code."""
        result = []
        lines = source.split('\n')
        in_multiline_comment = False
        
        for line in lines:
            processed_line = ""
            i = 0
            in_string = False
            string_char = None
            
            while i < len(line):
                char = line[i]
                
                # Handle string boundaries
                if not in_multiline_comment and not in_string and (char == '"' or char == "'"):
                    in_string = True
                    string_char = char
                    processed_line += char
                    i += 1
                    continue
                elif in_string and char == string_char:
                    # Check if it's escaped
                    if i > 0 and line[i-1] != '\\':
                        in_string = False
                        string_char = None
                    processed_line += char
                    i += 1
                    continue
                
                # Skip comment processing if inside a string
                if in_string:
                    processed_line += char
                    i += 1
                    continue
                
                # Check for multi-line comment start
                if not in_multiline_comment and i < len(line) - 1 and line[i:i+2] == '/*':
                    in_multiline_comment = True
                    i += 2
                    continue
                
                # Check for multi-line comment end
                if in_multiline_comment and i < len(line) - 1 and line[i:i+2] == '*/':
                    in_multiline_comment = False
                    i += 2
                    continue
                
                # Check for single-line comment start (only if not in multi-line comment)
                if not in_multiline_comment and i < len(line) - 1 and line[i:i+2] == '//':
                    # Rest of line is comment, break
                    break
                
                # Add character if not in comment
                if not in_multiline_comment:
                    processed_line += char
                
                i += 1
            
            result.append(processed_line.rstrip())
        
        return '\n'.join(result)
    
    def peek_line(self) -> Optional[str]:
        """Peek at the current line without consuming it."""
        if self.current_line < len(self.lines):
            return self.lines[self.current_line]
        return None
    
    def consume_line(self) -> Optional[str]:
        """Consume and return the current line."""
        if self.current_line < len(self.lines):
            line = self.lines[self.current_line]
            self.current_line += 1
            return line
        return None
    
    def skip_empty_lines(self):
        """Skip empty lines and lines containing only whitespace."""
        while self.current_line < len(self.lines):
            line = self.lines[self.current_line]
            if line and not line.isspace():
                break
            self.current_line += 1
    
    def extract_string_literal(self, expr_str: str) -> Optional[str]:
        """Extract string literal content from quoted string."""
        expr_str = expr_str.strip()
        if (expr_str.startswith('"') and expr_str.endswith('"')) or \
           (expr_str.startswith("'") and expr_str.endswith("'")):
            return expr_str[1:-1]
        return None