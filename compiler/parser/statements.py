"""Statement parsing module."""

import re
from typing import Optional, List
from ..ast import (
    ASTNode, DisplayStatement, IfStatement, Assignment, WhileLoop,
    ForEachLoop, TaskAction, TaskInvocation, ActionDefinition,
    ReturnStatement, ActionInvocationWithArgs, IncludeStatement,
    DataInstance, FieldAssignment, Identifier, Literal,
    ApiCallStatement, ApiHeader, DatabaseStatement, ServeStatement,
    BinaryOp
)
from .expressions import ExpressionParser
from .base import ParseError


class StatementParser(ExpressionParser):
    """Handles parsing of statements."""
    
    def parse_statement(self, line: str) -> Optional[ASTNode]:
        """Parse a statement from a line."""
        line = line.strip()
        
        if not line:
            return None
        
        
        # Include statements
        if line.startswith('@include '):
            return self.parse_include(line)
        
        # Database statements
        if line.startswith('db '):
            return self.parse_database_statement(line)
        
        # Serve statements
        if line.startswith('serve '):
            return self.parse_serve_statement(line)
        
        # Display statements
        if line.startswith('display '):
            return self.parse_display(line)
        
        # API call statements
        if line.startswith(('call ', 'fetch ', 'update ', 'delete ')):
            return self.parse_api_call(line)
        
        # Conditional statements  
        if line.startswith('when '):
            return self.parse_when(line)
        
        # Loop statements
        if line.startswith('while '):
            return self.parse_while_loop(line)
        if line.startswith('for each '):
            return self.parse_foreach_loop(line)
        
        # Task statements
        if line.startswith('task '):
            return self.parse_task_action(line)
        if line.startswith('run task '):
            return self.parse_task_invocation(line)
        
        # Action statements
        if line.startswith('action '):
            return self.parse_action_definition(line)
        if line.startswith('give '):
            return self.parse_return_statement(line)
        
        # Assignment and variable creation (set statements)
        if line.startswith('set '):
            return self.parse_set_statement(line)
        
        # Action invocation (before assignment check)
        if '(' in line and ')' in line and ' is ' not in line:
            return self.parse_action_invocation(line)
        
        # Assignment statements (must be last)
        if ' is ' in line and not line.startswith('when ') and ' which is ' not in line:
            return self.parse_assignment(line)
        
        return None
    
    def parse_include(self, line: str) -> IncludeStatement:
        """Parse @include statement."""
        match = re.match(r'@include\s+(\w+)\s+from\s+"([^"]+)"', line)
        if not match:
            match = re.match(r"@include\s+(\w+)\s+from\s+'([^']+)'", line)
        
        if not match:
            # Try without quotes
            match = re.match(r'@include\s+(\w+)\s+from\s+(\S+)', line)
        
        if not match:
            raise ParseError(f"Invalid @include statement: {line}")
        
        module_name = match.group(1)
        file_path = match.group(2)
        
        return IncludeStatement(module_name, file_path)
    
    def parse_display(self, line: str) -> DisplayStatement:
        """Parse display statement."""
        if line.startswith('display '):
            content = line[8:].strip()  # Remove "display "
        else:
            raise ParseError(f"Invalid display statement: {line}")
        
        # Check if content is a string literal
        if (content.startswith('"') and content.endswith('"')) or \
           (content.startswith("'") and content.endswith("'")):
            value = Literal(content[1:-1], 'string')
        else:
            # Parse as expression
            value = self.parse_expression(content)
        
        return DisplayStatement(value)
    
    def parse_when(self, line: str) -> IfStatement:
        """Parse when statement with condition and body."""
        # Extract condition from 'when <condition> then'
        if ' then' not in line:
            raise ParseError(f"Invalid when statement - missing 'then': {line}")
        
        condition_str = line[5:].split(' then')[0].strip()  # Remove 'when ' and everything after ' then'
        condition = self.parse_expression(condition_str)
        
        # Check if this is a single-line when statement
        then_part = line.split(' then', 1)[1].strip() if ' then' in line else ''
        if then_part:
            # Single line: when <condition> then <statement>
            stmt = self.parse_statement(then_part)
            return IfStatement(condition, [stmt] if stmt else [], None)
        
        # Multi-line when statement
        body = []
        else_body = []
        in_otherwise = False
        
        while self.current_line < len(self.lines):
            next_line = self.peek_line()
            
            if not next_line:
                break
            
            # Check for end marker or otherwise clause
            if next_line.strip() == 'end when':
                self.consume_line()
                break
            elif next_line.strip() == 'otherwise':
                self.consume_line()
                in_otherwise = True
                continue
            
            # Parse any non-empty line that's not an end marker
            self.consume_line()
            statement_line = next_line.strip()
            
            # Parse the statement
            stmt = self.parse_statement(statement_line)
            if stmt:
                if in_otherwise:
                    else_body.append(stmt)
                else:
                    body.append(stmt)
        
        return IfStatement(condition, body, else_body if else_body else None)
    
    def parse_assignment(self, line: str) -> Assignment:
        """Parse assignment statement using 'is' syntax."""
        # Split on ' is ' (variable is value)
        if ' is ' not in line:
            raise ParseError(f"Invalid assignment syntax - missing 'is': {line}")
        
        parts = line.split(' is ', 1)
        if len(parts) != 2:
            raise ParseError(f"Invalid assignment: {line}")
        
        var_name = parts[0].strip()
        value_str = parts[1].strip()
        
        # Parse the value expression
        value = self.parse_expression(value_str)
        
        return Assignment(var_name, value)
    
    def parse_while_loop(self, line: str) -> WhileLoop:
        """Parse while loop."""
        # Extract condition from 'while <condition>'
        condition_str = line[6:].strip()  # Remove 'while '
        condition = self.parse_expression(condition_str)
        
        # Parse body
        body = []
        while self.current_line < len(self.lines):
            next_line = self.peek_line()
            
            if not next_line:
                break
            
            if next_line.strip() == 'end while':
                self.consume_line()
                break
            
            self.consume_line()
            statement_line = next_line.strip()
            
            stmt = self.parse_statement(statement_line)
            if stmt:
                body.append(stmt)
        
        return WhileLoop(condition, body)
    
    def parse_foreach_loop(self, line: str) -> ForEachLoop:
        """Parse for each loop."""
        # Check for character iteration: 'for each char in <string>'
        char_match = re.match(r'for each\s+char\s+in\s+(.+)', line)
        if char_match:
            string_expr = char_match.group(1).strip()
            collection = self.parse_expression(string_expr)
            item_var = 'char'  # Default variable name for character iteration
        else:
            # Extract iteration spec from 'for each <item> in <collection>'
            match = re.match(r'for each\s+(\w+)\s+in\s+(.+)', line)
            if not match:
                raise ParseError(f"Invalid for each syntax: {line}")
            
            item_var = match.group(1)
            collection_expr = match.group(2).strip()
            
            # Parse collection as expression
            collection = self.parse_expression(collection_expr)
        
        # Parse body
        body = []
        while self.current_line < len(self.lines):
            next_line = self.peek_line()
            
            if not next_line:
                break
            
            if next_line.strip() == 'end for':
                self.consume_line()
                break
            
            self.consume_line()
            statement_line = next_line.strip()
            
            stmt = self.parse_statement(statement_line)
            if stmt:
                body.append(stmt)
        
        return ForEachLoop(item_var, collection, body)
    
    def parse_task_action(self, line: str) -> TaskAction:
        """Parse task definition."""
        # Extract task name from 'task <name>'
        name = line[5:].strip()  # Remove 'task '
        name = self.extract_string_literal(name) or name
        
        # Parse steps
        steps = []
        while self.current_line < len(self.lines):
            next_line = self.peek_line()
            
            if not next_line:
                break
            
            if next_line.strip() == 'end task':
                self.consume_line()
                break
            
            if next_line.strip().startswith('step '):
                self.consume_line()
                step_content = next_line.strip()[5:].strip()  # Remove 'step '
                step_content = self.extract_string_literal(step_content) or step_content
                steps.append(step_content)
            else:
                self.consume_line()
        
        return TaskAction(name, steps)
    
    def parse_task_invocation(self, line: str) -> TaskInvocation:
        """Parse run task statement."""
        # Extract task name from 'run task <name>'
        task_name = line[9:].strip()  # Remove 'run task '
        task_name = self.extract_string_literal(task_name) or task_name
        return TaskInvocation(task_name)
    
    def parse_action_definition(self, line: str) -> ActionDefinition:
        """Parse action definition."""
        # Parse action signature: 'action <name>' or 'action <name> with <params>'
        parts = line[7:].strip().split()  # Remove 'action '
        if not parts:
            raise ParseError(f"Invalid action definition: {line}")
        
        name = parts[0]
        
        # Parse parameters if present (action name with param1 which is type, param2 which is type)
        params = []
        if 'with' in line:
            # Extract parameters after 'with'
            with_index = line.find('with')
            param_str = line[with_index + 4:].strip()
            # Simple parameter parsing for now
            # TODO: Implement full parameter parsing with types
        
        # Parse body
        body = []
        while self.current_line < len(self.lines):
            next_line = self.peek_line()
            
            if not next_line:
                break
            
            if next_line.strip() == 'end action':
                self.consume_line()
                break
            
            self.consume_line()
            statement_line = next_line.strip()
            
            stmt = self.parse_statement(statement_line)
            if stmt:
                body.append(stmt)
        
        return ActionDefinition(name, body)
    
    
    def parse_return_statement(self, line: str) -> ReturnStatement:
        """Parse give statement."""
        value_str = line[5:].strip()  # Remove "give "
        
        if value_str:
            value = self.parse_expression(value_str)
        else:
            value = None
        
        return ReturnStatement(value)
    
    def parse_set_statement(self, line: str) -> Assignment:
        """Parse set statement for variable declaration and assignment."""
        # Parse 'set variable which is type to value' or 'set variable to value'
        
        # Remove 'set '
        content = line[4:].strip()
        
        # Look for ' to ' to split variable declaration from value
        if ' to ' not in content:
            raise ParseError(f"Invalid set statement - missing 'to': {line}")
        
        var_part, value_str = content.split(' to ', 1)
        
        # Extract variable name (may include type declaration)
        if ' which is ' in var_part:
            var_name = var_part.split(' which is ')[0].strip()
            var_type = var_part.split(' which is ')[1].strip()
            # Type information extracted but current AST doesn't store it
            # TODO: Extend AST to include type information
        else:
            var_name = var_part.strip()
        
        # Parse the value expression
        value = self.parse_expression(value_str.strip())
        
        return Assignment(var_name, value)
    
    def parse_action_invocation(self, line: str) -> Optional[ActionInvocationWithArgs]:
        """Parse action invocation with arguments."""
        match = re.match(r'(\w+)\s*\(([^)]*)\)', line)
        if not match:
            return None
        
        action_name = match.group(1)
        args_str = match.group(2).strip()
        
        # Parse arguments
        args = []
        if args_str:
            # Simple split by comma (doesn't handle nested expressions well)
            for arg in args_str.split(','):
                arg = arg.strip()
                args.append(self.parse_expression(arg))
        
        return ActionInvocationWithArgs(action_name, args)
    
    def parse_api_call(self, line: str) -> Optional[ApiCallStatement]:
        """Parse API call statements like 'call /login method POST with loginForm'."""
        # Parse the first line: call /login method POST with loginForm into response
        parts = line.split()
        if len(parts) < 4:  # Minimum: verb endpoint method action
            return None
            
        verb = parts[0]  # 'call', 'fetch', 'update', 'delete'
        endpoint = parts[1]  # '/login'
        
        # Find method
        method_index = -1
        for i, part in enumerate(parts):
            if part.lower() == 'method':
                if i + 1 < len(parts):
                    method = parts[i + 1].upper()  # 'POST', 'GET', etc.
                    method_index = i
                break
        else:
            raise ParseError(f"Missing 'method' keyword in API call: {line}")
        
        # Find payload (optional)
        payload = None
        if 'with' in parts:
            with_index = parts.index('with')
            if with_index + 1 < len(parts):
                payload_part = parts[with_index + 1]
                # Remove 'into' if it follows
                if 'into' in parts and parts.index('into') == with_index + 2:
                    payload = payload_part
                else:
                    payload = payload_part
        
        # Find response variable
        response_variable = None
        if 'into' in parts:
            into_index = parts.index('into')
            if into_index + 1 < len(parts):
                response_variable = parts[into_index + 1]
        
        # Create initial API call (headers will be added by multiline parser)
        api_call = ApiCallStatement(
            verb=verb,
            endpoint=endpoint,
            method=method,
            payload=payload,
            headers=[],
            response_variable=response_variable
        )
        
        return api_call
    
    def parse_api_headers(self, api_call: ApiCallStatement, lines: List[str]) -> None:
        """Parse headers for an API call from subsequent lines."""
        i = 0
        while i < len(lines):
            line = lines[i].strip()
            
            if line == "using headers":
                # Parse header lines that follow
                i += 1
                while i < len(lines):
                    header_line = lines[i].strip()
                    if ':' in header_line:
                        name, value = header_line.split(':', 1)
                        name = name.strip()
                        value = value.strip().strip('"\'')  # Remove quotes
                        api_call.headers.append(ApiHeader(name=name, value=value))
                    else:
                        break  # End of headers
                    i += 1
                break
            i += 1

    def parse_database_statement(self, line: str) -> Optional[DatabaseStatement]:
        """Parse database statements like 'db find User where id equals id'."""
        line = line.strip()
        
        # Remove 'db ' prefix
        if not line.startswith('db '):
            return None
        
        line = line[3:].strip()  # Remove 'db ' prefix
        
        # Parse different database operations
        if line.startswith('find '):
            return self._parse_db_find(line[5:])  # Remove 'find '
        elif line.startswith('create '):
            return self._parse_db_create(line[7:])  # Remove 'create '
        elif line.startswith('update '):
            return self._parse_db_update(line[7:])  # Remove 'update '
        elif line.startswith('delete '):
            return self._parse_db_delete(line[7:])  # Remove 'delete '
        else:
            raise ParseError(f"Unknown database operation: {line}")

    def _parse_db_find(self, line: str) -> DatabaseStatement:
        """Parse db find operations like 'User where id equals id' or 'all User'."""
        line = line.strip()
        
        # Handle 'all User' case
        if line.startswith('all '):
            entity_name = line[4:].strip()
            return DatabaseStatement(
                operation='find_all',
                entity_name=entity_name,
                conditions=[],
                fields=[]
            )
        
        # Handle 'User where ...' case
        if ' where ' in line:
            entity_part, condition_part = line.split(' where ', 1)
            entity_name = entity_part.strip()
            
            # Parse conditions (simplified - just handle single condition for now)
            conditions = []
            condition_part = condition_part.strip()
            
            # Parse condition like "id equals id" or "name equals 'John'"
            if ' equals ' in condition_part:
                field_name, value_part = condition_part.split(' equals ', 1)
                field_name = field_name.strip()
                value_part = value_part.strip()
                
                # Create condition nodes
                field_node = Identifier(field_name)
                value_node = self.parse_expression(value_part)
                conditions.append(BinaryOp(field_node, '==', value_node))
            
            return DatabaseStatement(
                operation='find',
                entity_name=entity_name,
                conditions=conditions,
                fields=[]
            )
        else:
            # Simple find without conditions
            entity_name = line.strip()
            return DatabaseStatement(
                operation='find',
                entity_name=entity_name,
                conditions=[],
                fields=[]
            )

    def _parse_db_create(self, line: str) -> DatabaseStatement:
        """Parse db create operations like 'User with name is input.name and email is input.email'."""
        line = line.strip()
        
        if ' with ' in line:
            entity_part, fields_part = line.split(' with ', 1)
            entity_name = entity_part.strip()
            
            # Parse field assignments
            fields = []
            field_assignments = fields_part.split(' and ')
            
            for assignment in field_assignments:
                assignment = assignment.strip()
                if ' is ' in assignment:
                    field_name, value_part = assignment.split(' is ', 1)
                    field_name = field_name.strip()
                    value_part = value_part.strip()
                    
                    value_node = self.parse_expression(value_part)
                    fields.append(FieldAssignment(field_name, value_node))
            
            return DatabaseStatement(
                operation='create',
                entity_name=entity_name,
                conditions=[],
                fields=fields
            )
        else:
            # Create without fields
            entity_name = line.strip()
            return DatabaseStatement(
                operation='create',
                entity_name=entity_name,
                conditions=[],
                fields=[]
            )

    def _parse_db_update(self, line: str) -> DatabaseStatement:
        """Parse db update operations like 'User where id equals id set name to input.name'."""
        line = line.strip()
        
        if ' where ' in line and ' set ' in line:
            # Split into entity, where clause, and set clause
            entity_where_part, set_part = line.split(' set ', 1)
            entity_part, condition_part = entity_where_part.split(' where ', 1)
            
            entity_name = entity_part.strip()
            
            # Parse conditions (simplified)
            conditions = []
            condition_part = condition_part.strip()
            if ' equals ' in condition_part:
                field_name, value_part = condition_part.split(' equals ', 1)
                field_name = field_name.strip()
                value_part = value_part.strip()
                
                field_node = Identifier(field_name)
                value_node = self.parse_expression(value_part)
                conditions.append(BinaryOp(field_node, '==', value_node))
            
            # Parse set fields
            fields = []
            set_assignments = set_part.split(' and ')
            
            for assignment in set_assignments:
                assignment = assignment.strip()
                if ' to ' in assignment:
                    field_name, value_part = assignment.split(' to ', 1)
                    field_name = field_name.strip()
                    value_part = value_part.strip()
                    
                    value_node = self.parse_expression(value_part)
                    fields.append(FieldAssignment(field_name, value_node))
            
            return DatabaseStatement(
                operation='update',
                entity_name=entity_name,
                conditions=conditions,
                fields=fields
            )
        else:
            raise ParseError(f"Invalid update syntax: {line}")

    def _parse_db_delete(self, line: str) -> DatabaseStatement:
        """Parse db delete operations like 'User where id equals id'."""
        line = line.strip()
        
        if ' where ' in line:
            entity_part, condition_part = line.split(' where ', 1)
            entity_name = entity_part.strip()
            
            # Parse conditions (simplified)
            conditions = []
            condition_part = condition_part.strip()
            if ' equals ' in condition_part:
                field_name, value_part = condition_part.split(' equals ', 1)
                field_name = field_name.strip()
                value_part = value_part.strip()
                
                field_node = Identifier(field_name)
                value_node = self.parse_expression(value_part)
                conditions.append(BinaryOp(field_node, '==', value_node))
            
            return DatabaseStatement(
                operation='delete',
                entity_name=entity_name,
                conditions=conditions,
                fields=[]
            )
        else:
            raise ParseError(f"Delete operation requires where clause: {line}")
    
    def parse_serve_statement(self, line: str) -> Optional[ServeStatement]:
        """Parse serve statement like 'serve get /users'."""
        line = line.strip()
        
        # Remove 'serve ' prefix
        if not line.startswith('serve '):
            return None
        
        line = line[6:].strip()  # Remove 'serve ' prefix
        
        # Parse method and endpoint
        parts = line.split(' ', 1)
        if len(parts) != 2:
            raise ParseError(f"Invalid serve syntax: {line}")
        
        method = parts[0].lower()
        endpoint = parts[1].strip()
        
        # Parse serve body until 'end serve'
        body = []
        while self.current_line < len(self.lines):
            next_line = self.peek_line()
            
            if not next_line:
                break
            
            if next_line.strip() == 'end serve':
                self.consume_line()  # consume 'end serve'
                break
            
            # Parse any non-empty line that's not an end marker
            self.consume_line()
            stmt_line = next_line.strip()
            if stmt_line:
                # Parse the statement inside serve block
                stmt = self.parse_statement(stmt_line)
                if stmt:
                    body.append(stmt)
        
        return ServeStatement(method, endpoint, body)