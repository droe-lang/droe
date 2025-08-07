"""Core parser that orchestrates all parsing modules."""

from typing import List, Optional
from ..ast import Program, ASTNode
from .base import ParseError
from .statements import StatementParser
from .structures import StructureParser


class Parser(StructureParser, StatementParser):
    """Main parser that combines all parsing functionality."""
    
    def parse(self) -> Program:
        """Parse the entire source code into a Program AST."""
        statements = []
        metadata_for_next = None
        
        while self.current_line < len(self.lines):
            self.skip_empty_lines()
            
            if self.current_line >= len(self.lines):
                break
            
            line = self.peek_line()
            if not line:
                self.consume_line()
                continue
            
            # Work with stripped version for comparisons
            line_stripped = line.strip()
            
            # Check for metadata annotations
            if line_stripped.startswith('@'):
                metadata = self.parse_metadata(line_stripped)
                if metadata:
                    # Store metadata both for next statement AND as standalone statement
                    metadata_for_next = metadata
                    statements.append(metadata)  # Add as standalone statement for target detection
                    self.consume_line()
                    continue
            
            # Consume the line
            self.consume_line()
            
            # Parse structural definitions
            if line_stripped == 'Module:':
                stmt = self.parse_module_definition()
            elif line_stripped == 'Data:':
                stmt = self.parse_data_definition()
            elif line_stripped.startswith('module '):
                stmt = self.parse_module_spec_syntax(line_stripped)
            elif line_stripped.startswith('data '):
                stmt = self.parse_data_spec_syntax(line_stripped)
            elif line_stripped == 'Layout:':
                stmt = self.parse_layout_definition()
            elif line_stripped == 'Form:':
                stmt = self.parse_form_definition()
            elif line_stripped.startswith('Layout ') and ':' in line_stripped and '[' in line_stripped:
                stmt = self.parse_inline_layout(line_stripped)
            
            # Parse UI components
            elif any(line_stripped.startswith(prefix + ':') for prefix in [
                'Title', 'Input', 'Textarea', 'Dropdown', 'Toggle',
                'Checkbox', 'Radio', 'Button', 'Image', 'Video', 'Audio',
                'Asset', 'Camera', 'Location', 'Notification', 'Storage',
                'Sensor', 'Contact'
            ]):
                stmt = self.parse_component(line_stripped)
            
            # Parse statements
            elif line_stripped.startswith(('call ', 'fetch ', 'update ', 'delete ')):
                # Handle multiline API calls
                stmt = self.parse_multiline_api_call(line_stripped)
            else:
                stmt = self.parse_statement(line_stripped)
            
            if stmt:
                # Attach metadata if present
                if metadata_for_next:
                    if hasattr(stmt, 'metadata'):
                        stmt.metadata = metadata_for_next
                    metadata_for_next = None
                
                statements.append(stmt)
        
        return Program(statements)
    
    def parse_multiline_api_call(self, first_line: str) -> Optional[ASTNode]:
        """Parse a multiline API call with headers."""
        # Parse the main API call line
        api_call = self.parse_api_call(first_line)
        if not api_call:
            return None
        
        # Look ahead for "using headers" and collect header lines
        self.consume_line()  # Consume the API call line
        header_lines = []
        
        # Check if the next line is "using headers"
        if self.current_line < len(self.lines):
            next_line = self.peek_line()
            if next_line and next_line.strip() == "using headers":
                self.consume_line()  # Consume "using headers"
                
                # Collect indented header lines
                while self.current_line < len(self.lines):
                    line = self.peek_line()
                    if not line or not line.startswith('    '):  # Not indented
                        break
                    if ':' in line:
                        header_lines.append(line.strip())
                        self.consume_line()
                    else:
                        break
        
        # Parse headers and add to API call
        if header_lines:
            self.parse_api_headers(api_call, ['using headers'] + header_lines)
        
        return api_call
    
    @classmethod
    def parse_source(cls, source: str) -> Program:
        """Convenience method to parse source code."""
        parser = cls(source)
        return parser.parse()