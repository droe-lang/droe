"""Parser for Roe DSL - converts DSL text to AST."""

import re
from typing import List, Optional, Tuple
from .ast import (
    ASTNode, Program, DisplayStatement, IfStatement,
    Literal, Identifier, BinaryOp, PropertyAccess,
    Assignment, ArrayLiteral, WhileLoop, ForEachLoop, ArithmeticOp,
    TaskAction, TaskInvocation, ActionDefinition, ReturnStatement, ActionInvocation,
    ModuleDefinition, DataDefinition, DataField, ActionDefinitionWithParams, 
    ActionParameter, ActionInvocationWithArgs, StringInterpolation,
    DataInstance, FieldAssignment, IncludeStatement, FormatExpression,
    MetadataAnnotation, LayoutDefinition, FormDefinition,
    TitleComponent, InputComponent, TextareaComponent, DropdownComponent,
    ToggleComponent, CheckboxComponent, RadioComponent, ButtonComponent,
    ImageComponent, VideoComponent, AudioComponent, AssetInclude,
    AttributeDefinition, ValidationAttribute, BindingAttribute, ActionAttribute
)


class ParseError(Exception):
    """Raised when parsing fails."""
    pass


class Parser:
    """Simple recursive-descent parser for Roe DSL."""
    
    def __init__(self, source: str):
        self.source = source
        # Process comments before splitting into lines
        self.source = self._remove_comments(source)
        self.lines = [line.strip() for line in self.source.strip().split('\n')]
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
    
    def parse(self) -> Program:
        """Parse the entire program."""
        metadata = []
        statements = []
        
        while self.current_line < len(self.lines):
            line = self.lines[self.current_line].strip()
            if not line:  # Skip empty lines
                self.current_line += 1
                continue
            
            # Check for metadata annotation
            if line.startswith('@'):
                meta = self.parse_metadata(line)
                if meta:
                    metadata.append(meta)
                self.current_line += 1
                continue
                
            stmt = self.parse_statement(line)
            if stmt:
                statements.append(stmt)
            self.current_line += 1
        
        return Program(statements=statements, metadata=metadata)
    
    def parse_metadata(self, line: str) -> Optional[MetadataAnnotation]:
        """Parse a metadata annotation like @target web or @name user_form."""
        line = line.strip()
        
        if not line.startswith('@'):
            return None
        
        # Remove the @ prefix
        content = line[1:].strip()
        
        # Handle quoted values: @name "user profile form"
        if ' "' in content and content.endswith('"'):
            key_part, quoted_value = content.split(' "', 1)
            key = key_part.strip()
            value = quoted_value[:-1]  # Remove closing quote
        # Handle single quoted values: @name 'user profile form'
        elif " '" in content and content.endswith("'"):
            key_part, quoted_value = content.split(" '", 1)
            key = key_part.strip()
            value = quoted_value[:-1]  # Remove closing quote
        # Handle unquoted values: @target web
        elif ' ' in content:
            key, value = content.split(' ', 1)
            key = key.strip()
            value = value.strip()
        else:
            # Single word annotation like @web (treat as key with empty value)
            key = content
            value = ""
        
        # Validate key format
        if not re.match(r'^[a-zA-Z_][a-zA-Z0-9_]*$', key):
            raise ParseError(f"Invalid metadata key format: @{key}")
        
        return MetadataAnnotation(key=key, value=value)
    
    def parse_statement(self, line: str) -> Optional[ASTNode]:
        """Parse a single statement."""
        line = line.strip()
        
        # Include statement
        if line.startswith('include '):
            # Check if it's an asset include or module include
            include_path = line[8:].strip()
            # Remove quotes if present
            if (include_path.startswith('"') and include_path.endswith('"')) or \
               (include_path.startswith("'") and include_path.endswith("'")):
                include_path = include_path[1:-1]
            
            # Check if it's an asset include
            if any(include_path.endswith(ext) for ext in ['.css', '.js', '.font', '.ttf', '.woff', '.woff2', '.otf']):
                return self.parse_asset_include(line)
            else:
                return self.parse_include(line)
        
        # Display/Show statement
        elif line.startswith('display ') or line.startswith('show '):
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
        
        # Task action definition
        elif line.startswith('task '):
            return self.parse_task_action()
        
        # Task invocation
        elif line.startswith('run '):
            return self.parse_task_invocation(line)
        
        # Module definition
        elif line.startswith('module '):
            return self.parse_module_definition()
        
        # Data definition
        elif line.startswith('data '):
            return self.parse_data_definition()
        
        # Layout definition
        elif line.startswith('layout '):
            return self.parse_layout_definition()
        
        # Form definition
        elif line.startswith('form '):
            return self.parse_form_definition()
        
        # UI components
        elif line.startswith('title '):
            return self.parse_title_component(line)
        elif line.startswith('input '):
            return self.parse_input_component(line)
        elif line.startswith('textarea '):
            return self.parse_textarea_component(line)
        elif line.startswith('dropdown '):
            return self.parse_dropdown_component(line)
        elif line.startswith('toggle '):
            return self.parse_toggle_component(line)
        elif line.startswith('checkbox '):
            return self.parse_checkbox_component(line)
        elif line.startswith('radio '):
            return self.parse_radio_component(line)
        elif line.startswith('button '):
            return self.parse_button_component(line)
        elif line.startswith('image '):
            return self.parse_image_component(line)
        elif line.startswith('video '):
            return self.parse_video_component(line)
        elif line.startswith('audio '):
            return self.parse_audio_component(line)
        elif any(line.lower().startswith(layout + ' ') or line.lower() == layout for layout in ['column', 'row', 'grid', 'stack', 'overlay']):
            return self.parse_inline_layout(line)
        elif line.lower().startswith('end '):
            return None
        
        # Action definition (check for parameterized actions first)
        elif line.startswith('action '):
            if ' with ' in line or ' gives ' in line:
                return self.parse_action_definition_with_params()
            else:
                return self.parse_action_definition()
        
        # Return statements (respond with, answer is, output, give)
        elif (line.startswith('respond with ') or line.startswith('answer is ') or 
              line.startswith('output ') or line.startswith('give ')):
            return self.parse_return_statement(line)
        
        # End statements (these should be handled by their respective parsing contexts)
        elif (line.lower().startswith('end ')):
            # This should not happen if parsing contexts are working correctly
            # But we'll return None to avoid errors
            return None
        
        else:
            raise ParseError(f"Unknown statement: {line}")
    
    def parse_include(self, line: str) -> IncludeStatement:
        """Parse an include statement (include ModuleName.roe or include "utils/ModuleName.roe")."""
        # Remove 'include ' prefix
        file_path = line[8:].strip()
        
        # Handle quoted paths (allows subdirectories)
        if file_path.startswith('"') and file_path.endswith('"'):
            file_path = file_path[1:-1]  # Remove quotes
        elif file_path.startswith("'") and file_path.endswith("'"):
            file_path = file_path[1:-1]  # Remove quotes
        
        # Validate file path format
        if not file_path.endswith('.roe'):
            raise ParseError(f"Include statement must specify a .roe file: {line}")
        
        # Extract module name from file path
        # For subdirectory paths like "utils/MathUtils.roe", use the full path structure
        if '/' in file_path:
            # For subdirectory paths, create a module name by replacing / with _
            # "utils/MathUtils.roe" -> "utils_MathUtils"
            module_name = file_path[:-4].replace('/', '_')
        else:
            # Simple filename without directory
            module_name = file_path[:-4]
        
        # Validate module name format (allow underscores for directory separators)
        if not re.match(r'^[a-zA-Z_][a-zA-Z0-9_]*$', module_name):
            raise ParseError(f"Invalid module name derived from include path: {module_name} (from {file_path})")
        
        return IncludeStatement(module_name=module_name, file_path=file_path)
    
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
        stmt = DisplayStatement(expression=expr)
        stmt.line_number = self.current_line + 1
        return stmt
    
    def parse_if(self, line: str) -> IfStatement:
        """Parse an if/when-then statement."""
        # Check for single-line format: "when condition then action"
        single_line_match = re.match(r'(?:if|when)\s+(.+?)\s+then\s+(.+)', line, re.IGNORECASE)
        if single_line_match:
            condition_str = single_line_match.group(1)
            then_str = single_line_match.group(2)
            
            # Parse condition
            condition = self.parse_expression(condition_str)
            
            # Parse then body (single statement)
            then_stmt = self.parse_statement(then_str)
            then_body = [then_stmt] if then_stmt else []
            
            stmt = IfStatement(condition=condition, then_body=then_body)
            stmt.line_number = self.current_line + 1
            return stmt
            
        # Check for multi-line format: "when condition then"
        multi_line_match = re.match(r'(?:if|when)\s+(.+?)\s+then\s*$', line, re.IGNORECASE)
        if multi_line_match:
            condition_str = multi_line_match.group(1)
            
            # Parse condition
            condition = self.parse_expression(condition_str)
            
            # Parse multi-line then body
            then_body = []
            else_body = None
            self.current_line += 1
            
            # Read then body until we hit 'otherwise', 'else', or 'end'
            while self.current_line < len(self.lines):
                line = self.lines[self.current_line].strip()
                
                if line.lower() in ['otherwise', 'else']:
                    # Start of else body
                    else_body = []
                    self.current_line += 1
                    
                    # Read else body
                    while self.current_line < len(self.lines):
                        line = self.lines[self.current_line].strip()
                        
                        if line.lower() in ['end when', 'end if', 'end']:
                            break
                        
                        if line:  # Skip empty lines
                            stmt = self.parse_statement(line)
                            if stmt:
                                else_body.append(stmt)
                        
                        self.current_line += 1
                    break
                    
                elif line.lower() in ['end when', 'end if', 'end']:
                    break
                
                if line:  # Skip empty lines
                    stmt = self.parse_statement(line)
                    if stmt:
                        then_body.append(stmt)
                
                self.current_line += 1
            
            if self.current_line >= len(self.lines):
                raise ParseError(f"Missing 'end when' or 'end if' for if statement")
            
            stmt = IfStatement(condition=condition, then_body=then_body, else_body=else_body)
            stmt.line_number = self.current_line + 1
            return stmt
        
        # Neither format matched
        raise ParseError(f"Invalid if/when statement: {line}")
    
    def parse_assignment(self, line: str) -> Assignment:
        """Parse a variable assignment (set x to value or set x which are group of type to value)."""
        # Match pattern with optional type declaration:
        # set variable to value
        # set variable which are group of type to value
        # set variable which is type from expression
        
        # First try data instance creation: set variable which is DataType with field1 is value1, field2 is value2
        data_instance_match = re.match(r'set\s+([a-zA-Z_][a-zA-Z0-9_]*)\s+which\s+is\s+([A-Z][a-zA-Z0-9_]*)\s+with\s+(.+)', line, re.IGNORECASE)
        if data_instance_match:
            variable = data_instance_match.group(1)
            data_type = data_instance_match.group(2)
            fields_str = data_instance_match.group(3)
            
            # Parse field assignments
            field_assignments = []
            field_parts = fields_str.split(',')
            for field_part in field_parts:
                field_part = field_part.strip()
                field_match = re.match(r'([a-zA-Z_][a-zA-Z0-9_]*)\s+is\s+(.+)', field_part, re.IGNORECASE)
                if field_match:
                    field_name = field_match.group(1)
                    field_value_str = field_match.group(2)
                    field_value = self.parse_expression(field_value_str)
                    field_assignments.append(FieldAssignment(field_name=field_name, value=field_value))
                else:
                    raise ParseError(f"Invalid field assignment: {field_part}")
            
            # Create data instance
            data_instance = DataInstance(data_type=data_type, field_values=field_assignments)
            
            # Create assignment with data instance
            assignment = Assignment(variable=variable, value=data_instance)
            assignment.line_number = self.current_line + 1
            assignment.declared_var_type = data_type  # Store the data type (keep original case)
            return assignment
        
        # Try the "from" syntax with type
        from_typed_match = re.match(r'set\s+([a-zA-Z_][a-zA-Z0-9_]*)\s+which\s+is\s+(?:a\s+)?(\w+)\s+from\s+(.+)', line, re.IGNORECASE)
        if from_typed_match:
            variable = from_typed_match.group(1)
            declared_type = from_typed_match.group(2).lower()
            value_str = from_typed_match.group(3)
            
            # Parse the value expression (might be a function call)
            value = self.parse_expression(value_str)
            
            # Add type information to the assignment
            assignment = Assignment(variable=variable, value=value)
            assignment.line_number = self.current_line + 1
            assignment.declared_var_type = declared_type  # Store the declared type for variables
            return assignment
        
        # Try the typed array versions (list of / group of)
        # Pattern: set variable which are (list of|group of) type to value
        typed_array_match = re.match(r'set\s+([a-zA-Z_][a-zA-Z0-9_]*)\s+which\s+are\s+(list\s+of|group\s+of)\s+(\w+)\s+to\s+(.+)', line, re.IGNORECASE)
        if typed_array_match:
            variable = typed_array_match.group(1)
            collection_type = typed_array_match.group(2).lower().replace(' ', '_')  # "list of" -> "list_of"
            element_type = typed_array_match.group(3).lower()
            value_str = typed_array_match.group(4)
            
            # Parse the value expression
            value = self.parse_expression(value_str)
            
            # Add type information to the assignment
            assignment = Assignment(variable=variable, value=value)
            assignment.line_number = self.current_line + 1
            assignment.declared_var_type = f"{collection_type}_{element_type}"  # Store the full collection type
            assignment.collection_type = collection_type  # Store the collection type
            return assignment
        
        # Legacy: set variable which are group of type to value
        legacy_array_match = re.match(r'set\s+([a-zA-Z_][a-zA-Z0-9_]*)\s+which\s+are\s+group\s+of\s+(\w+)\s+to\s+(.+)', line, re.IGNORECASE)
        if legacy_array_match:
            variable = legacy_array_match.group(1)
            declared_type = legacy_array_match.group(2).lower()
            value_str = legacy_array_match.group(3)
            
            # Parse the value expression
            value = self.parse_expression(value_str)
            
            # Add type information to the assignment
            assignment = Assignment(variable=variable, value=value)
            assignment.line_number = self.current_line + 1
            assignment.declared_var_type = f"group_of_{declared_type}"  # Store the full collection type
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
            assignment.line_number = self.current_line + 1
            assignment.declared_var_type = declared_type  # Store the declared type for variables
            return assignment
        
        # No fallback - Roelang requires explicit type declarations
        var_name = line.split()[1] if len(line.split()) > 1 else 'variable'
        raise ParseError(f"Line {self.current_line + 1}: Missing type declaration in assignment. Use 'set {var_name} which is <type> to <value>' for strong typing.")
    
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
    
    def _is_complete_quoted_string(self, expr_str: str) -> bool:
        """Check if the expression is a complete quoted string (not a concatenation with quotes)."""
        if not ((expr_str.startswith('"') and expr_str.endswith('"')) or 
                (expr_str.startswith("'") and expr_str.endswith("'"))):
            return False
        
        # Check if there are unescaped quotes in the middle that would indicate concatenation
        quote_char = expr_str[0]
        inside_content = expr_str[1:-1]
        
        # If the string is properly quoted at start and end, and there are no unescaped
        # quotes inside, then it's a complete quoted string regardless of operators inside
        # The + operators inside the quotes are part of the string content, not concatenation
        
        # Check for unescaped quotes inside
        i = 0
        while i < len(inside_content):
            if inside_content[i] == quote_char:
                # Check if it's escaped
                if i > 0 and inside_content[i-1] == '\\':
                    # It's escaped, continue
                    i += 1
                    continue
                else:
                    # Unescaped quote found inside, this is not a simple string
                    return False
            i += 1
        
        return True
    
    def _find_operator_outside_quotes(self, expr_str: str, operator: str) -> int:
        """Find the position of an operator that's not inside quoted strings."""
        i = 0
        while i < len(expr_str):
            if expr_str[i:i+len(operator)] == operator:
                # Check if we're inside quotes
                quote_count_single = expr_str[:i].count("'") - expr_str[:i].count("\\'")
                quote_count_double = expr_str[:i].count('"') - expr_str[:i].count('\\"')
                
                # If we have an even number of quotes (or zero), we're outside quotes
                if quote_count_single % 2 == 0 and quote_count_double % 2 == 0:
                    return i
            i += 1
        return -1
    
    def parse_expression(self, expr_str: str) -> ASTNode:
        """Parse an expression."""
        expr_str = expr_str.strip()
        
        # Format expression: format variable as "pattern"
        if expr_str.startswith('format ') and ' as ' in expr_str:
            return self.parse_format_expression(expr_str)
        
        # Boolean literals (check these early to avoid "is" operator confusion)
        if expr_str.lower() == 'true' or expr_str.lower() == 'the condition is true':
            return Literal(value=True, type='boolean')
        elif expr_str.lower() == 'false' or expr_str.lower() == 'the condition is false':
            return Literal(value=False, type='boolean')
        
        # Array literal
        if expr_str.startswith('[') and expr_str.endswith(']'):
            return self.parse_array_literal(expr_str)
        
        # String literal (check for interpolation first) - only if it's a complete quoted string
        if self._is_complete_quoted_string(expr_str):
            string_content = expr_str[1:-1]
            
            # Check if string contains interpolation markers [variable]
            if '[' in string_content and ']' in string_content:
                return self.parse_string_interpolation(string_content)
            else:
                return Literal(value=string_content, type='string')
        
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
            op_pos = self._find_operator_outside_quotes(expr_str, natural_op)
            if op_pos != -1:
                left_part = expr_str[:op_pos].strip()
                right_part = expr_str[op_pos + len(natural_op):].strip()
                if left_part and right_part:
                    left = self.parse_expression(left_part)
                    right = self.parse_expression(right_part)
                    # Use ArithmeticOp for arithmetic operators, BinaryOp for comparisons
                    if symbol_op in ['+', '-', '*', '/']:
                        return ArithmeticOp(left=left, operator=symbol_op, right=right)
                    else:
                        return BinaryOp(left=left, operator=symbol_op, right=right)
        
        # Binary operations (symbolic - check after natural language)
        for op in ['>=', '<=', '==', '!=', '>', '<', '+', '-', '*', '/', '%']:
            op_pos = self._find_operator_outside_quotes(expr_str, op)
            if op_pos != -1:
                left_part = expr_str[:op_pos].strip()
                right_part = expr_str[op_pos + len(op):].strip()
                if left_part and right_part:
                    left = self.parse_expression(left_part)
                    right = self.parse_expression(right_part)
                    # Use ArithmeticOp for arithmetic operators, BinaryOp for comparisons
                    if op in ['+', '-', '*', '/', '%']:
                        return ArithmeticOp(left=left, operator=op, right=right)
                    else:
                        return BinaryOp(left=left, operator=op, right=right)
        
        # Function call with arguments (run module.action with arg1, arg2)
        run_match = re.match(r'run\s+([a-zA-Z_][a-zA-Z0-9_]*(?:\.[a-zA-Z_][a-zA-Z0-9_]*)?)\s+with\s+(.+)', expr_str, re.IGNORECASE)
        if run_match:
            function_name = run_match.group(1)
            args_str = run_match.group(2)
            
            # Parse module.action format
            if '.' in function_name:
                module_name, action_name = function_name.split('.', 1)
            else:
                module_name = None
                action_name = function_name
            
            # Parse arguments
            arguments = []
            if args_str.strip():
                # Simple comma splitting (doesn't handle nested commas properly, but sufficient for now)
                arg_parts = args_str.split(',')
                for arg_part in arg_parts:
                    arg_part = arg_part.strip()
                    if arg_part:
                        arguments.append(self.parse_expression(arg_part))
            
            return ActionInvocationWithArgs(
                module_name=module_name, 
                action_name=action_name, 
                arguments=arguments
            )
        
        # Function call without arguments (run module.action)
        run_match_no_args = re.match(r'run\s+([a-zA-Z_][a-zA-Z0-9_]*(?:\.[a-zA-Z_][a-zA-Z0-9_]*)?)\s*$', expr_str, re.IGNORECASE)
        if run_match_no_args:
            function_name = run_match_no_args.group(1)
            
            # Parse module.action format
            if '.' in function_name:
                module_name, action_name = function_name.split('.', 1)
            else:
                module_name = None
                action_name = function_name
            
            return ActionInvocationWithArgs(
                module_name=module_name, 
                action_name=action_name, 
                arguments=[]
            )
        
        # Action call with arguments without run (for 'from' syntax: module.action with arg1, arg2)
        action_with_args_match = re.match(r'([a-zA-Z_][a-zA-Z0-9_]*(?:\.[a-zA-Z_][a-zA-Z0-9_]*)?)\s+with\s+(.+)', expr_str, re.IGNORECASE)
        if action_with_args_match:
            function_name = action_with_args_match.group(1)
            args_str = action_with_args_match.group(2)
            
            # Parse module.action format
            if '.' in function_name:
                module_name, action_name = function_name.split('.', 1)
            else:
                module_name = None
                action_name = function_name
            
            # Parse arguments
            arguments = []
            if args_str.strip():
                # Simple comma splitting (doesn't handle nested commas properly, but sufficient for now)
                arg_parts = args_str.split(',')
                for arg_part in arg_parts:
                    arg_part = arg_part.strip()
                    if arg_part:
                        arguments.append(self.parse_expression(arg_part))
            
            return ActionInvocationWithArgs(
                module_name=module_name, 
                action_name=action_name, 
                arguments=arguments
            )
        
        # Action call without arguments (for 'from' syntax: module.action or simple_action)
        action_no_args_match = re.match(r'^([a-zA-Z_][a-zA-Z0-9_]*(?:\.[a-zA-Z_][a-zA-Z0-9_]*)?)\s*$', expr_str, re.IGNORECASE)
        if action_no_args_match:
            function_name = action_no_args_match.group(1)
            
            # Only treat as action if it contains a dot (module.action) or if we're in a from context
            # For simple identifiers, we'll let it fall through to be treated as variables
            if '.' in function_name:
                # Parse module.action format
                module_name, action_name = function_name.split('.', 1)
                
                return ActionInvocationWithArgs(
                    module_name=module_name, 
                    action_name=action_name, 
                    arguments=[]
                )
            # For simple identifiers without dots, let them be parsed as Identifier below
            # The code generator will determine if it's an action or variable
        
        # Property access (e.g., user.age)
        if '.' in expr_str:
            parts = expr_str.split('.', 1)
            if len(parts) == 2:
                obj = Identifier(name=parts[0].strip())
                return PropertyAccess(object=obj, property=parts[1].strip())
        
        
        # Action invocation (for getting return values in expressions)
        # Check if this could be an action call - we'll determine at code gen if it's an action vs variable
        if re.match(r'^[a-zA-Z_][a-zA-Z0-9_]*$', expr_str):
            # This could be either a variable or an action invocation
            # We'll resolve this during code generation
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
    
    def parse_string_interpolation(self, string_content: str) -> StringInterpolation:
        """Parse a string with variable interpolation like 'Hello [name]'."""
        parts = []
        current_pos = 0
        
        while current_pos < len(string_content):
            # Find the next interpolation marker
            start_bracket = string_content.find('[', current_pos)
            
            if start_bracket == -1:
                # No more interpolation, add remaining text as literal
                if current_pos < len(string_content):
                    remaining_text = string_content[current_pos:]
                    if remaining_text:  # Only add non-empty text
                        parts.append(Literal(value=remaining_text, type='string'))
                break
            
            # Add text before the bracket as literal
            if start_bracket > current_pos:
                text_before = string_content[current_pos:start_bracket]
                if text_before:  # Only add non-empty text
                    parts.append(Literal(value=text_before, type='string'))
            
            # Find the closing bracket
            end_bracket = string_content.find(']', start_bracket)
            if end_bracket == -1:
                raise ParseError(f"Unclosed interpolation bracket in string: {string_content}")
            
            # Extract expression inside brackets
            expr_str = string_content[start_bracket + 1:end_bracket].strip()
            if not expr_str:
                raise ParseError(f"Empty interpolation bracket in string: {string_content}")
            
            # Parse the expression (could be variable, property access, or run expression)
            expr = self.parse_expression(expr_str)
            parts.append(expr)
            
            # Move past the closing bracket
            current_pos = end_bracket + 1
        
        return StringInterpolation(parts=parts)
    
    def parse_task_action(self) -> TaskAction:
        """Parse a task action definition (task name [with params] ... end)."""
        # Current line should be "task name [with params]"
        line = self.lines[self.current_line]
        
        # Extract task name
        task_match = re.match(r'task\s+([a-zA-Z_][a-zA-Z0-9_]*)', line, re.IGNORECASE)
        if not task_match:
            raise ParseError(f"Invalid task statement: {line}")
        
        task_name = task_match.group(1)
        parameters = []
        
        # Check for parameters
        with_match = re.search(r'\s+with\s+(.+)$', line, re.IGNORECASE)
        if with_match:
            params_str = with_match.group(1)
            
            # Parse parameters: "param1 which is type1, param2 which is type2"
            if params_str:
                param_parts = params_str.split(',')
                for param_part in param_parts:
                    param_part = param_part.strip()
                    # Handle compound types like "list of text" or "group of int"
                    compound_match = re.match(r'([a-zA-Z_][a-zA-Z0-9_]*)\s+which\s+is\s+(?:a\s+)?(list\s+of|group\s+of)\s+(\w+)', param_part, re.IGNORECASE)
                    if compound_match:
                        param_name = compound_match.group(1)
                        compound_type = compound_match.group(2).replace(' ', '_')  # "list of" -> "list_of"
                        base_type = compound_match.group(3)
                        param_type = f"{compound_type}_{base_type}"
                        parameters.append(ActionParameter(name=param_name, type=param_type))
                    else:
                        # Regular parameter: "param which is type" or "param which are type"
                        param_match = re.match(r'([a-zA-Z_][a-zA-Z0-9_]*)\s+which\s+(?:is|are)\s+(?:a\s+)?(\w+)', param_part, re.IGNORECASE)
                        if param_match:
                            param_name = param_match.group(1)
                            param_type = param_match.group(2)
                            parameters.append(ActionParameter(name=param_name, type=param_type))
                        else:
                            raise ParseError(f"Invalid parameter syntax: {param_part}")
        
        # Parse body until "end"
        body = []
        self.current_line += 1  # Move to next line
        
        while self.current_line < len(self.lines):
            line = self.lines[self.current_line].strip()
            
            # Check for end conditions first
            if line.lower() in ['end task', 'end']:
                break
            
            if line:  # Skip empty lines
                stmt = self.parse_statement(line)
                if stmt:
                    body.append(stmt)
            
            self.current_line += 1
        
        if self.current_line >= len(self.lines):
            raise ParseError(f"Missing 'end task' or 'end' for task '{task_name}'")
        
        return TaskAction(name=task_name, parameters=parameters, body=body)
    
    def parse_task_invocation(self, line: str) -> TaskInvocation:
        """Parse a task invocation (run task_name [with args])."""
        # Check for arguments: "run task_name with arg1, arg2, ..."
        with_match = re.match(r'run\s+([a-zA-Z_][a-zA-Z0-9_]*)\s+with\s+(.+)', line, re.IGNORECASE)
        if with_match:
            task_name = with_match.group(1)
            args_str = with_match.group(2)
            
            # Parse arguments
            arguments = []
            if args_str:
                arg_parts = args_str.split(',')
                for arg_part in arg_parts:
                    arg_part = arg_part.strip()
                    arg_expr = self.parse_expression(arg_part)
                    arguments.append(arg_expr)
            
            return TaskInvocation(task_name=task_name, arguments=arguments)
        
        # Simple task invocation without arguments: "run task_name"
        match = re.match(r'run\s+([a-zA-Z_][a-zA-Z0-9_]*)', line, re.IGNORECASE)
        if not match:
            raise ParseError(f"Invalid run statement: {line}")
        
        task_name = match.group(1)
        return TaskInvocation(task_name=task_name)
    
    def parse_action_definition(self) -> ActionDefinition:
        """Parse an action definition - now requires explicit return type for strong typing."""
        # Current line should be "action name gives return_type"
        line = self.lines[self.current_line]
        
        # Strong typing: All actions must declare return type
        if ' gives ' not in line:
            action_name = line.split()[1] if len(line.split()) > 1 else 'name'
            raise ParseError(f"Line {self.current_line + 1}: Action must declare return type: {line}. Use 'action {action_name} gives <type>'")
        
        # Extract action name and return type
        match = re.match(r'action\s+([a-zA-Z_][a-zA-Z0-9_]*)\s+gives\s+(\w+)', line, re.IGNORECASE)
        if not match:
            raise ParseError(f"Invalid action statement - must specify return type: {line}")
        
        action_name = match.group(1)
        return_type = match.group(2).lower()
        
        # Parse body until "end action" or "end"
        body = []
        self.current_line += 1  # Move to next line
        
        while self.current_line < len(self.lines):
            line = self.lines[self.current_line].strip()
            
            if line.lower() in ['end action', 'end']:
                break
            
            if line:  # Skip empty lines
                stmt = self.parse_statement(line)
                if stmt:
                    body.append(stmt)
            
            self.current_line += 1
        
        if self.current_line >= len(self.lines):
            raise ParseError(f"Missing 'end action' or 'end' for action '{action_name}'")
        
        # Convert to parameterized action with return type for consistency
        return ActionDefinitionWithParams(name=action_name, parameters=[], return_type=return_type, body=body)
    
    def parse_return_statement(self, line: str) -> ReturnStatement:
        """Parse return statements with natural language."""
        
        # Try different natural language patterns
        patterns = [
            (r'respond\s+with\s+(.+)', 'respond_with'),
            (r'answer\s+is\s+(.+)', 'answer_is'),
            (r'output\s+(.+)', 'output'),
            (r'give\s+(.+)', 'give')
        ]
        
        for pattern, return_type in patterns:
            match = re.match(pattern, line, re.IGNORECASE)
            if match:
                expression_str = match.group(1)
                expression = self.parse_expression(expression_str)
                return ReturnStatement(expression=expression, return_type=return_type)
        
        raise ParseError(f"Invalid return statement: {line}")
    
    def parse_module_definition(self) -> ModuleDefinition:
        """Parse a module definition (module name ... end module)."""
        # Current line should be "module name"
        line = self.lines[self.current_line]
        
        # Extract module name
        match = re.match(r'module\s+([a-zA-Z_][a-zA-Z0-9_]*)', line, re.IGNORECASE)
        if not match:
            raise ParseError(f"Invalid module statement: {line}")
        
        module_name = match.group(1)
        
        # Parse body until "end module" or "end"
        body = []
        self.current_line += 1  # Move to next line
        
        while self.current_line < len(self.lines):
            line = self.lines[self.current_line].strip()
            
            if line.lower() in ['end module', 'end']:
                break
            
            if line:  # Skip empty lines
                stmt = self.parse_statement(line)
                if stmt:
                    body.append(stmt)
            
            self.current_line += 1
        
        if self.current_line >= len(self.lines):
            raise ParseError(f"Missing 'end module' or 'end' for module '{module_name}'")
        
        return ModuleDefinition(name=module_name, body=body)
    
    def parse_data_definition(self) -> DataDefinition:
        """Parse a data structure definition (data Name ... end data)."""
        # Current line should be "data Name"
        line = self.lines[self.current_line]
        
        # Extract data name and optional storage type
        # Match patterns like: data Name, data Name short_store, data Name long_store
        match = re.match(r'data\s+([a-zA-Z_][a-zA-Z0-9_]*)(?:\s+(short_store|long_store))?', line, re.IGNORECASE)
        if not match:
            raise ParseError(f"Invalid data statement: {line}")
        
        data_name = match.group(1)
        storage_type = match.group(2) if match.group(2) else None
        
        # Parse fields until "end data" or "end"
        fields = []
        self.current_line += 1  # Move to next line
        
        while self.current_line < len(self.lines):
            line = self.lines[self.current_line].strip()
            
            if line.lower() in ['end data', 'end']:
                break
            
            if line:  # Skip empty lines
                # Parse field definition: "name is type"
                field_match = re.match(r'([a-zA-Z_][a-zA-Z0-9_]*)\s+is\s+(\w+)', line, re.IGNORECASE)
                if field_match:
                    field_name = field_match.group(1)
                    field_type = field_match.group(2).lower()
                    fields.append(DataField(name=field_name, type=field_type))
                else:
                    raise ParseError(f"Invalid field definition: {line}")
            
            self.current_line += 1
        
        if self.current_line >= len(self.lines):
            raise ParseError(f"Missing 'end data' or 'end' for data '{data_name}'")
        
        data_def = DataDefinition(name=data_name, fields=fields)
        if storage_type:
            data_def.storage_type = storage_type
        
        return data_def
    
    def parse_action_definition_with_params(self) -> ActionDefinitionWithParams:
        """Parse an action definition with parameters (action name with param1 which is type1, param2 which is type2 gives return_type)."""
        # Current line should be "action name [with params] [gives return_type]"
        line = self.lines[self.current_line]
        
        # Parse the action signature
        # Pattern: action name [with param1 which is type1, param2 which is type2] [gives return_type]
        action_match = re.match(r'action\s+([a-zA-Z_][a-zA-Z0-9_]*)', line, re.IGNORECASE)
        if not action_match:
            raise ParseError(f"Invalid action statement: {line}")
        
        action_name = action_match.group(1)
        parameters = []
        return_type = None
        
        # Check for parameters
        with_match = re.search(r'\s+with\s+(.+?)(?:\s+gives\s+(\w+))?$', line, re.IGNORECASE)
        if with_match:
            params_str = with_match.group(1)
            return_type = with_match.group(2)
            
            # Parse parameters: "param1 which is type1, param2 which is type2"
            if params_str:
                param_parts = params_str.split(',')
                for param_part in param_parts:
                    param_part = param_part.strip()
                    # Handle compound types like "list of text" or "group of int"
                    compound_match = re.match(r'([a-zA-Z_][a-zA-Z0-9_]*)\s+which\s+is\s+(?:a\s+)?(list\s+of|group\s+of)\s+(\w+)', param_part, re.IGNORECASE)
                    if compound_match:
                        param_name = compound_match.group(1)
                        collection_type = compound_match.group(2).lower().replace(' ', '_')
                        element_type = compound_match.group(3).lower()
                        param_type = f"{collection_type}_{element_type}"  # e.g., "list_of_text"
                        parameters.append(ActionParameter(name=param_name, type=param_type))
                    else:
                        # Regular type
                        param_match = re.match(r'([a-zA-Z_][a-zA-Z0-9_]*)\s+which\s+is\s+(?:a\s+)?(\w+)', param_part, re.IGNORECASE)
                        if param_match:
                            param_name = param_match.group(1)
                            param_type = param_match.group(2).lower()
                            parameters.append(ActionParameter(name=param_name, type=param_type))
                        else:
                            raise ParseError(f"Invalid parameter definition: {param_part}")
        
        # Check for return type without parameters
        elif ' gives ' in line:
            gives_match = re.search(r'\s+gives\s+(\w+)', line, re.IGNORECASE)
            if gives_match:
                return_type = gives_match.group(1).lower()
        
        # Strong typing: All actions must declare return type
        if return_type is None:
            raise ParseError(f"Line {self.current_line + 1}: Action must declare return type: {line}. Add 'gives <type>' to action signature")
        
        # Parse body until "end action" or "end"
        body = []
        self.current_line += 1  # Move to next line
        
        while self.current_line < len(self.lines):
            line = self.lines[self.current_line].strip()
            
            if line.lower() in ['end action', 'end']:
                break
            
            if line:  # Skip empty lines
                stmt = self.parse_statement(line)
                if stmt:
                    body.append(stmt)
            
            self.current_line += 1
        
        if self.current_line >= len(self.lines):
            raise ParseError(f"Missing 'end action' or 'end' for action '{action_name}'")
        
        return ActionDefinitionWithParams(
            name=action_name, 
            parameters=parameters, 
            return_type=return_type, 
            body=body
        )
    
    def parse_format_expression(self, expr_str: str) -> FormatExpression:
        """Parse a format expression: format variable as "pattern"."""
        # Split on " as " to get expression and pattern
        parts = expr_str.split(' as ', 1)
        if len(parts) != 2:
            raise ParseError(f"Invalid format expression: {expr_str}")
        
        # Extract the expression part (remove "format " prefix)
        expr_part = parts[0][7:].strip()  # Remove "format " prefix
        pattern_part = parts[1].strip()
        
        # Parse the expression to format
        expression = self.parse_expression(expr_part)
        
        # Extract format pattern (should be a quoted string)
        if not (pattern_part.startswith('"') and pattern_part.endswith('"')):
            raise ParseError(f"Format pattern must be a quoted string: {pattern_part}")
        
        pattern = pattern_part[1:-1]  # Remove quotes
        
        return FormatExpression(expression=expression, format_pattern=pattern)
    
    def parse_layout_definition(self) -> LayoutDefinition:
        """Parse a layout definition (layout name type ... end layout)."""
        line = self.lines[self.current_line]
        
        # Extract layout name and type: "layout login_screen column" or "layout name"
        match = re.match(r'layout\s+([a-zA-Z_][a-zA-Z0-9_]*)(?:\s+(\w+))?', line, re.IGNORECASE)
        if not match:
            raise ParseError(f"Invalid layout statement: {line}")
        
        layout_name = match.group(1)
        layout_type = match.group(2).lower() if match.group(2) else None
        
        # Parse children and attributes until "end layout" or "end"
        children = []
        attributes = []
        self.current_line += 1
        
        while self.current_line < len(self.lines):
            line = self.lines[self.current_line].strip()
            
            if line.lower() in ['end layout', 'end']:
                break
            
            if line:
                # Check if it's a layout type declaration (if not already specified)
                if layout_type is None and line.lower() in ['column', 'row', 'grid', 'stack', 'overlay']:
                    layout_type = line.lower()
                else:
                    stmt = self.parse_statement(line)
                    if stmt:
                        children.append(stmt)
            
            self.current_line += 1
        
        if self.current_line >= len(self.lines):
            raise ParseError(f"Missing 'end layout' for layout '{layout_name}'")
        
        if layout_type is None:
            layout_type = "column"  # Default to column layout
        
        return LayoutDefinition(name=layout_name, layout_type=layout_type, children=children, attributes=attributes)
    
    def parse_form_definition(self) -> FormDefinition:
        """Parse a form definition (form name ... end form)."""
        line = self.lines[self.current_line]
        
        # Extract form name: "form login_form"
        match = re.match(r'form\s+([a-zA-Z_][a-zA-Z0-9_]*)', line, re.IGNORECASE)
        if not match:
            raise ParseError(f"Invalid form statement: {line}")
        
        form_name = match.group(1)
        
        # Parse children until "end form" or "end"
        children = []
        attributes = []
        self.current_line += 1
        
        while self.current_line < len(self.lines):
            line = self.lines[self.current_line].strip()
            
            if line.lower() in ['end form', 'end']:
                break
            
            if line:
                stmt = self.parse_statement(line)
                if stmt:
                    children.append(stmt)
            
            self.current_line += 1
        
        if self.current_line >= len(self.lines):
            raise ParseError(f"Missing 'end form' for form '{form_name}'")
        
        return FormDefinition(name=form_name, children=children, attributes=attributes)
    
    def parse_inline_layout(self, line: str) -> LayoutDefinition:
        """Parse inline layout (column, row, etc.) - returns immediately for inline parsing."""
        parts = line.split()
        layout_type = parts[0].lower()
        
        # Parse class and style attributes
        css_classes = []
        style = None
        
        # Check for class attribute
        if 'class ' in line:
            class_match = re.search(r'class\s+"([^"]+)"', line)
            if not class_match:
                class_match = re.search(r"class\s+'([^']+)'", line)
            if class_match:
                css_classes = class_match.group(1).split()
        
        # Check for style attribute
        if 'style ' in line:
            style_match = re.search(r'style\s+"([^"]+)"', line)
            if not style_match:
                style_match = re.search(r"style\s+'([^']+)'", line)
            if style_match:
                style = style_match.group(1)
        
        # Create an anonymous layout with the specified type
        children = []
        self.current_line += 1
        
        # Parse children until corresponding end statement
        while self.current_line < len(self.lines):
            current_line = self.lines[self.current_line].strip()
            
            if current_line.lower() in [f'end {layout_type}', 'end']:
                break
            
            if current_line:
                stmt = self.parse_statement(current_line)
                if stmt:
                    children.append(stmt)
            
            self.current_line += 1
        
        if self.current_line >= len(self.lines):
            raise ParseError(f"Missing 'end {layout_type}' or 'end'")
        
        layout = LayoutDefinition(name=f"anonymous_{layout_type}", layout_type=layout_type, children=children, css_classes=css_classes)
        if style:
            layout.style = style
        return layout
    
    def parse_title_component(self, line: str) -> TitleComponent:
        """Parse a title component: title "Welcome to Roe" validate required."""
        # Extract text and attributes
        parts = line[6:].strip()  # Remove "title "
        
        # Find quoted text
        if parts.startswith('"') and '"' in parts[1:]:
            end_quote = parts.find('"', 1)
            text = parts[1:end_quote]
            attr_text = parts[end_quote + 1:].strip()
        elif parts.startswith("'") and "'" in parts[1:]:
            end_quote = parts.find("'", 1)
            text = parts[1:end_quote]
            attr_text = parts[end_quote + 1:].strip()
        else:
            # No quotes, extract first word as text
            words = parts.split()
            text = words[0] if words else ""
            attr_text = " ".join(words[1:]) if len(words) > 1 else ""
        
        attributes = self.parse_attributes(attr_text)
        return TitleComponent(text=text, attributes=attributes)
    
    def parse_input_component(self, line: str) -> InputComponent:
        """Parse input component: input id name_field email bind LoginForm.email validate required, email."""
        parts = line[6:].strip()  # Remove "input "
        
        # Extract input ID if present: "input id field_name ..."
        words = parts.split()
        element_id = None
        input_type = "text"
        binding = None
        
        if words and words[0] == "id" and len(words) > 1:
            element_id = words[1]
            remaining_words = words[2:]
        else:
            remaining_words = words
        
        if remaining_words:
            # Check for type specification: "input type password" or direct type
            if remaining_words[0] == "type" and len(remaining_words) > 1:
                input_type = remaining_words[1]
                attr_text = " ".join(remaining_words[2:]) if len(remaining_words) > 2 else ""
            else:
                # Check if first word is a known input type
                if remaining_words[0] in ['email', 'password', 'text', 'number', 'tel', 'url']:
                    input_type = remaining_words[0]
                    attr_text = " ".join(remaining_words[1:]) if len(remaining_words) > 1 else ""
                else:
                    attr_text = " ".join(remaining_words)
        else:
            attr_text = ""
        
        attributes = self.parse_attributes(attr_text)
        
        # Extract binding from attributes
        for attr in attributes:
            if isinstance(attr, BindingAttribute):
                binding = attr.binding_target
        
        # Create input component with ID
        input_comp = InputComponent(input_type=input_type, binding=binding, attributes=attributes)
        if element_id:
            # Store the element ID as a custom attribute
            input_comp.element_id = element_id
        
        return input_comp
    
    def parse_textarea_component(self, line: str) -> TextareaComponent:
        """Parse textarea component: textarea id notes_field bind LoginForm.message class "form-control" rows "4"."""
        parts = line[9:].strip()  # Remove "textarea "
        
        # Extract ID if present
        words = parts.split()
        element_id = None
        if words and words[0] == "id" and len(words) > 1:
            element_id = words[1]
            remaining_text = " ".join(words[2:])
        else:
            remaining_text = parts
        
        # Extract specific textarea attributes like rows
        if 'rows ' in remaining_text:
            rows_match = re.search(r'rows\s+"(\d+)"', remaining_text)
            if not rows_match:
                rows_match = re.search(r'rows\s+(\d+)', remaining_text)
            if rows_match:
                # Remove rows from text to avoid parsing it as attribute
                remaining_text = remaining_text.replace(rows_match.group(0), '')
        
        attributes = self.parse_attributes(remaining_text)
        binding = None
        css_classes = []
        
        # Extract binding and classes from attributes
        for attr in attributes:
            if isinstance(attr, BindingAttribute):
                binding = attr.binding_target
        
        # Check for classes
        if 'class ' in remaining_text:
            class_match = re.search(r'class\s+"([^"]+)"', remaining_text)
            if not class_match:
                class_match = re.search(r"class\s+'([^']+)'", remaining_text)
            if class_match:
                css_classes = class_match.group(1).split()
        
        textarea = TextareaComponent(binding=binding, attributes=attributes, css_classes=css_classes)
        if element_id:
            textarea.element_id = element_id
        
        return textarea
    
    def parse_dropdown_component(self, line: str) -> DropdownComponent:
        """Parse dropdown component: dropdown id country_field options ["Option1", "Option2"] bind Form.selection."""
        parts = line[9:].strip()  # Remove "dropdown "
        
        # Extract ID if present
        words = parts.split()
        element_id = None
        if words and words[0] == "id" and len(words) > 1:
            element_id = words[1]
            remaining_text = " ".join(words[2:])
        else:
            remaining_text = parts
        
        # Extract options if present
        options = []
        if 'options ' in remaining_text:
            options_start = remaining_text.find('options ')
            before_options = remaining_text[:options_start].strip()
            options_part = remaining_text[options_start + 8:].strip()
            
            if options_part.startswith('[') and ']' in options_part:
                end_bracket = options_part.find(']')
                options_str = options_part[1:end_bracket]
                attr_text = (before_options + " " + options_part[end_bracket + 1:]).strip()
                
                # Parse options array
                if options_str.strip():
                    option_parts = options_str.split(',')
                    for opt in option_parts:
                        opt = opt.strip()
                        if opt.startswith('"') and opt.endswith('"'):
                            options.append(Literal(value=opt[1:-1], type='string'))
                        elif opt.startswith("'") and opt.endswith("'"):
                            options.append(Literal(value=opt[1:-1], type='string'))
                        else:
                            options.append(Literal(value=opt, type='string'))
            else:
                attr_text = remaining_text
        else:
            attr_text = remaining_text
        
        attributes = self.parse_attributes(attr_text)
        binding = None
        
        # Extract binding from attributes
        for attr in attributes:
            if isinstance(attr, BindingAttribute):
                binding = attr.binding_target
        
        dropdown = DropdownComponent(options=options, binding=binding, attributes=attributes)
        if element_id:
            dropdown.element_id = element_id
        
        return dropdown
    
    def parse_toggle_component(self, line: str) -> ToggleComponent:
        """Parse toggle component: toggle bind Form.enabled."""
        parts = line[7:].strip()  # Remove "toggle "
        
        attributes = self.parse_attributes(parts)
        binding = None
        
        # Extract binding from attributes
        for attr in attributes:
            if isinstance(attr, BindingAttribute):
                binding = attr.binding_target
        
        return ToggleComponent(binding=binding, attributes=attributes)
    
    def parse_checkbox_component(self, line: str) -> CheckboxComponent:
        """Parse checkbox component: checkbox id newsletter_field "Accept terms" bind Form.accepted."""
        parts = line[9:].strip()  # Remove "checkbox "
        
        # Extract ID if present
        words = parts.split()
        element_id = None
        if words and words[0] == "id" and len(words) > 1:
            element_id = words[1]
            remaining_text = " ".join(words[2:])
        else:
            remaining_text = parts
        
        # Extract text if present
        text = None
        attr_text = remaining_text
        
        if remaining_text.startswith('"') and '"' in remaining_text[1:]:
            end_quote = remaining_text.find('"', 1)
            text = remaining_text[1:end_quote]
            attr_text = remaining_text[end_quote + 1:].strip()
        elif remaining_text.startswith("'") and "'" in remaining_text[1:]:
            end_quote = remaining_text.find("'", 1)
            text = remaining_text[1:end_quote]
            attr_text = remaining_text[end_quote + 1:].strip()
        
        attributes = self.parse_attributes(attr_text)
        binding = None
        
        # Extract binding from attributes
        for attr in attributes:
            if isinstance(attr, BindingAttribute):
                binding = attr.binding_target
        
        checkbox = CheckboxComponent(text=text, binding=binding, attributes=attributes)
        if element_id:
            checkbox.element_id = element_id
        
        return checkbox
    
    def parse_radio_component(self, line: str) -> RadioComponent:
        """Parse radio component: radio id role_admin "Option A" value "a" bind Form.choice."""
        parts = line[6:].strip()  # Remove "radio "
        
        # Extract ID if present
        words = parts.split()
        element_id = None
        if words and words[0] == "id" and len(words) > 1:
            element_id = words[1]
            remaining_text = " ".join(words[2:])
        else:
            remaining_text = parts
        
        # Extract text and value
        text = None
        value = None
        attr_text = remaining_text
        
        # Extract text if present
        if remaining_text.startswith('"') and '"' in remaining_text[1:]:
            end_quote = remaining_text.find('"', 1)
            text = remaining_text[1:end_quote]
            remaining = remaining_text[end_quote + 1:].strip()
            
            # Check for value specification
            if remaining.startswith('value '):
                value_part = remaining[6:].strip()
                if value_part.startswith('"') and '"' in value_part[1:]:
                    end_value_quote = value_part.find('"', 1)
                    value = value_part[1:end_value_quote]
                    attr_text = value_part[end_value_quote + 1:].strip()
                else:
                    words = value_part.split()
                    value = words[0] if words else ""
                    attr_text = " ".join(words[1:]) if len(words) > 1 else ""
            else:
                attr_text = remaining
        
        attributes = self.parse_attributes(attr_text)
        binding = None
        
        # Extract binding from attributes
        for attr in attributes:
            if isinstance(attr, BindingAttribute):
                binding = attr.binding_target
        
        radio = RadioComponent(text=text, value=value, binding=binding, attributes=attributes)
        if element_id:
            radio.element_id = element_id
        
        return radio
    
    def parse_button_component(self, line: str) -> ButtonComponent:
        """Parse button component: button "Login" run submit_login."""
        parts = line[7:].strip()  # Remove "button "
        
        # Extract text
        if parts.startswith('"') and '"' in parts[1:]:
            end_quote = parts.find('"', 1)
            text = parts[1:end_quote]
            attr_text = parts[end_quote + 1:].strip()
        elif parts.startswith("'") and "'" in parts[1:]:
            end_quote = parts.find("'", 1)
            text = parts[1:end_quote]
            attr_text = parts[end_quote + 1:].strip()
        else:
            # No quotes, extract first word as text
            words = parts.split()
            text = words[0] if words else "Button"
            attr_text = " ".join(words[1:]) if len(words) > 1 else ""
        
        attributes = self.parse_attributes(attr_text)
        action = None
        
        # Extract action from attributes
        for attr in attributes:
            if isinstance(attr, ActionAttribute):
                action = attr.action_name
        
        return ButtonComponent(text=text, action=action, attributes=attributes)
    
    def parse_image_component(self, line: str) -> ImageComponent:
        """Parse image component: image "path/to/image.png" alt "Description" class "img-fluid thumbnail"."""
        parts = line[6:].strip()  # Remove "image "
        
        # Extract source path
        if parts.startswith('"') and '"' in parts[1:]:
            end_quote = parts.find('"', 1)
            src = parts[1:end_quote]
            remaining = parts[end_quote + 1:].strip()
        elif parts.startswith("'") and "'" in parts[1:]:
            end_quote = parts.find("'", 1)
            src = parts[1:end_quote]
            remaining = parts[end_quote + 1:].strip()
        else:
            words = parts.split()
            src = words[0] if words else ""
            remaining = " ".join(words[1:]) if len(words) > 1 else ""
        
        # Parse alt text and classes
        alt = None
        css_classes = []
        
        # Check for alt text
        if remaining.startswith('alt '):
            alt_parts = remaining[4:].strip()
            if alt_parts.startswith('"') and '"' in alt_parts[1:]:
                end_alt = alt_parts.find('"', 1)
                alt = alt_parts[1:end_alt]
                remaining = alt_parts[end_alt + 1:].strip()
            elif alt_parts.startswith("'") and "'" in alt_parts[1:]:
                end_alt = alt_parts.find("'", 1)
                alt = alt_parts[1:end_alt]
                remaining = alt_parts[end_alt + 1:].strip()
        
        # Check for classes
        if remaining.startswith('class '):
            class_parts = remaining[6:].strip()
            if class_parts.startswith('"') and '"' in class_parts[1:]:
                end_class = class_parts.find('"', 1)
                css_classes = class_parts[1:end_class].split()
                remaining = class_parts[end_class + 1:].strip()
            elif class_parts.startswith("'") and "'" in class_parts[1:]:
                end_class = class_parts.find("'", 1)
                css_classes = class_parts[1:end_class].split()
                remaining = class_parts[end_class + 1:].strip()
        
        attributes = self.parse_attributes(remaining)
        return ImageComponent(src=src, alt=alt, attributes=attributes, css_classes=css_classes)
    
    def parse_video_component(self, line: str) -> VideoComponent:
        """Parse video component: video "path/to/video.mp4" controls autoplay muted class "video-player"."""
        parts = line[6:].strip()  # Remove "video "
        
        # Extract source path
        if parts.startswith('"') and '"' in parts[1:]:
            end_quote = parts.find('"', 1)
            src = parts[1:end_quote]
            remaining = parts[end_quote + 1:].strip()
        elif parts.startswith("'") and "'" in parts[1:]:
            end_quote = parts.find("'", 1)
            src = parts[1:end_quote]
            remaining = parts[end_quote + 1:].strip()
        else:
            words = parts.split()
            src = words[0] if words else ""
            remaining = " ".join(words[1:]) if len(words) > 1 else ""
        
        # Parse video attributes
        controls = True  # Default
        autoplay = False
        loop = False
        muted = False
        css_classes = []
        
        # Parse remaining attributes
        words = remaining.split()
        i = 0
        while i < len(words):
            word = words[i].lower()
            if word == 'controls':
                controls = True
            elif word == 'nocontrols':
                controls = False
            elif word == 'autoplay':
                autoplay = True
            elif word == 'loop':
                loop = True
            elif word == 'muted':
                muted = True
            elif word == 'class' and i + 1 < len(words):
                # Parse class value
                class_value = words[i + 1]
                if class_value.startswith('"') and class_value.endswith('"'):
                    css_classes = class_value[1:-1].split()
                elif class_value.startswith("'") and class_value.endswith("'"):
                    css_classes = class_value[1:-1].split()
                else:
                    css_classes = [class_value]
                i += 1
            i += 1
        
        # Get remaining attributes
        attr_text = " ".join(w for w in words if w.lower() not in ['controls', 'nocontrols', 'autoplay', 'loop', 'muted'] and not (words.index(w) > 0 and words[words.index(w)-1].lower() == 'class'))
        attributes = self.parse_attributes(attr_text)
        
        return VideoComponent(src=src, controls=controls, autoplay=autoplay, loop=loop, muted=muted, 
                            attributes=attributes, css_classes=css_classes)
    
    def parse_audio_component(self, line: str) -> AudioComponent:
        """Parse audio component: audio "path/to/audio.mp3" controls autoplay loop class "audio-player"."""
        parts = line[6:].strip()  # Remove "audio "
        
        # Extract source path
        if parts.startswith('"') and '"' in parts[1:]:
            end_quote = parts.find('"', 1)
            src = parts[1:end_quote]
            remaining = parts[end_quote + 1:].strip()
        elif parts.startswith("'") and "'" in parts[1:]:
            end_quote = parts.find("'", 1)
            src = parts[1:end_quote]
            remaining = parts[end_quote + 1:].strip()
        else:
            words = parts.split()
            src = words[0] if words else ""
            remaining = " ".join(words[1:]) if len(words) > 1 else ""
        
        # Parse audio attributes
        controls = True  # Default
        autoplay = False
        loop = False
        css_classes = []
        
        # Parse remaining attributes
        words = remaining.split()
        i = 0
        while i < len(words):
            word = words[i].lower()
            if word == 'controls':
                controls = True
            elif word == 'nocontrols':
                controls = False
            elif word == 'autoplay':
                autoplay = True
            elif word == 'loop':
                loop = True
            elif word == 'class' and i + 1 < len(words):
                # Parse class value
                class_value = words[i + 1]
                if class_value.startswith('"') and class_value.endswith('"'):
                    css_classes = class_value[1:-1].split()
                elif class_value.startswith("'") and class_value.endswith("'"):
                    css_classes = class_value[1:-1].split()
                else:
                    css_classes = [class_value]
                i += 1
            i += 1
        
        # Get remaining attributes
        attr_text = " ".join(w for w in words if w.lower() not in ['controls', 'nocontrols', 'autoplay', 'loop'] and not (words.index(w) > 0 and words[words.index(w)-1].lower() == 'class'))
        attributes = self.parse_attributes(attr_text)
        
        return AudioComponent(src=src, controls=controls, autoplay=autoplay, loop=loop, 
                            attributes=attributes, css_classes=css_classes)
    
    def parse_asset_include(self, line: str) -> AssetInclude:
        """Parse asset include statement: include "assets/style.css"."""
        parts = line[8:].strip()  # Remove "include "
        
        # Extract asset path
        if parts.startswith('"') and parts.endswith('"'):
            asset_path = parts[1:-1]
        elif parts.startswith("'") and parts.endswith("'"):
            asset_path = parts[1:-1]
        else:
            asset_path = parts
        
        # Determine asset type from extension
        if asset_path.endswith('.css'):
            asset_type = 'css'
        elif asset_path.endswith('.js'):
            asset_type = 'js'
        elif asset_path.endswith(('.ttf', '.woff', '.woff2', '.otf')):
            asset_type = 'font'
        else:
            asset_type = 'other'
        
        return AssetInclude(asset_path=asset_path, asset_type=asset_type)
    
    def parse_attributes(self, attr_text: str) -> List[AttributeDefinition]:
        """Parse component attributes like 'validate required, email' or 'bind LoginForm.email'."""
        attributes = []
        
        if not attr_text.strip():
            return attributes
        
        # Split by keywords: validate, bind, run, class
        parts = re.split(r'(?:^|\s+)(validate|bind|run|class)\s+', attr_text, flags=re.IGNORECASE)
        
        i = 1  # Skip first empty part
        while i < len(parts):
            attr_type = parts[i].lower()
            attr_value = parts[i + 1] if i + 1 < len(parts) else ""
            
            if attr_type == 'validate':
                # Parse validation types: "required, email" or "required"
                validations = [v.strip() for v in attr_value.split(',')]
                for validation in validations:
                    if validation:
                        attributes.append(ValidationAttribute(validation_type=validation))
            
            elif attr_type == 'bind':
                # Parse binding target: "LoginForm.email"
                binding_target = attr_value.strip()
                if binding_target:
                    attributes.append(BindingAttribute(binding_target=binding_target))
            
            elif attr_type == 'run':
                # Parse action name: "submit_login"
                action_name = attr_value.strip()
                if action_name:
                    attributes.append(ActionAttribute(action_name=action_name))
            
            i += 2
        
        return attributes


def parse(source: str) -> Program:
    """Convenience function to parse Roe DSL source code."""
    parser = Parser(source)
    return parser.parse()