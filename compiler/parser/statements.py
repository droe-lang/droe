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
        if line.startswith('Include '):
            return self.parse_include(line)
        
        # Database statements
        if line.startswith('db '):
            return self.parse_database_statement(line)
        
        # Serve statements
        if line.startswith('serve '):
            return self.parse_serve_statement(line)
        
        # Display statements
        if line.startswith('Display ') or line.startswith('display '):
            return self.parse_display(line)
        
        # API call statements
        if line.startswith(('call ', 'fetch ', 'update ', 'delete ')):
            return self.parse_api_call(line)
        
        # Conditional statements
        if line.startswith('If '):
            return self.parse_if(line)
        
        # Loop statements
        if line == 'While:':
            return self.parse_while_loop()
        if line == 'ForEach:':
            return self.parse_foreach_loop()
        
        # Task statements
        if line == 'Task:':
            return self.parse_task_action()
        if line.startswith('RunTask '):
            return self.parse_task_invocation(line)
        
        # Action statements
        if line == 'Action:' or line.startswith('Action '):
            return self.parse_action_definition()
        if line.startswith('Return '):
            return self.parse_return_statement(line)
        
        # Data instance creation
        if line.startswith('Create '):
            return self.parse_data_instance(line)
        
        # Action invocation (before assignment check)
        if '(' in line and ')' in line and not '=' in line:
            return self.parse_action_invocation(line)
        
        # Assignment statements (must be last)
        if '=' in line:
            return self.parse_assignment(line)
        
        return None
    
    def parse_include(self, line: str) -> IncludeStatement:
        """Parse Include statement."""
        match = re.match(r'Include\s+(\w+)\s+from\s+"([^"]+)"', line)
        if not match:
            match = re.match(r"Include\s+(\w+)\s+from\s+'([^']+)'", line)
        
        if not match:
            # Try without quotes
            match = re.match(r'Include\s+(\w+)\s+from\s+(\S+)', line)
        
        if not match:
            raise ParseError(f"Invalid Include statement: {line}")
        
        module_name = match.group(1)
        file_path = match.group(2)
        
        return IncludeStatement(module_name, file_path)
    
    def parse_display(self, line: str) -> DisplayStatement:
        """Parse Display/display statement."""
        if line.startswith('display '):
            content = line[8:].strip()  # Remove "display "
        else:
            content = line[8:].strip()  # Remove "Display "
        
        # Check if content is a string literal
        if (content.startswith('"') and content.endswith('"')) or \
           (content.startswith("'") and content.endswith("'")):
            value = Literal(content[1:-1], 'string')
        else:
            # Parse as expression
            value = self.parse_expression(content)
        
        return DisplayStatement(value)
    
    def parse_if(self, line: str) -> IfStatement:
        """Parse If statement with condition and body."""
        # Extract condition
        condition_match = re.match(r'If\s+(.+?):', line)
        if not condition_match:
            raise ParseError(f"Invalid If statement: {line}")
        
        condition_str = condition_match.group(1).strip()
        condition = self.parse_expression(condition_str)
        
        # Parse body statements
        body = []
        else_body = []
        in_else = False
        
        while self.current_line < len(self.lines):
            next_line = self.peek_line()
            
            if not next_line or not next_line.startswith('    '):
                # Check for Else clause
                if next_line and next_line.strip() == 'Else:':
                    self.consume_line()
                    in_else = True
                    continue
                else:
                    break
            
            # Consume indented line
            self.consume_line()
            statement_line = next_line[4:]  # Remove indentation
            
            # Parse the statement
            stmt = self.parse_statement(statement_line)
            if stmt:
                if in_else:
                    else_body.append(stmt)
                else:
                    body.append(stmt)
        
        return IfStatement(condition, body, else_body if else_body else None)
    
    def parse_assignment(self, line: str) -> Assignment:
        """Parse assignment statement."""
        # Find the = sign (not ==)
        eq_pos = -1
        i = 0
        while i < len(line):
            if line[i] == '=' and (i + 1 >= len(line) or line[i + 1] != '=') and \
               (i == 0 or line[i - 1] not in '!<>='):
                eq_pos = i
                break
            i += 1
        
        if eq_pos == -1:
            raise ParseError(f"Invalid assignment: {line}")
        
        var_name = line[:eq_pos].strip()
        value_str = line[eq_pos + 1:].strip()
        
        # Parse the value expression
        value = self.parse_expression(value_str)
        
        return Assignment(var_name, value)
    
    def parse_while_loop(self) -> WhileLoop:
        """Parse While loop."""
        # Next line should contain condition
        condition_line = self.consume_line()
        if not condition_line or not condition_line.startswith('    Condition:'):
            raise ParseError("While loop missing Condition")
        
        condition_str = condition_line.strip()[10:].strip()
        condition = self.parse_expression(condition_str)
        
        # Parse body
        body = []
        while self.current_line < len(self.lines):
            next_line = self.peek_line()
            
            if not next_line or not next_line.startswith('    '):
                break
            
            self.consume_line()
            statement_line = next_line[4:]
            
            # Skip Condition line
            if statement_line.strip().startswith('Condition:'):
                continue
            
            stmt = self.parse_statement(statement_line)
            if stmt:
                body.append(stmt)
        
        return WhileLoop(condition, body)
    
    def parse_foreach_loop(self) -> ForEachLoop:
        """Parse ForEach loop."""
        # Next line should contain the iteration spec
        iter_line = self.consume_line()
        if not iter_line or not iter_line.startswith('    '):
            raise ParseError("ForEach loop missing iteration specification")
        
        iter_spec = iter_line.strip()
        
        # Parse iteration specification (e.g., "item in items:")
        match = re.match(r'(\w+)\s+in\s+(.+?):', iter_spec)
        if not match:
            raise ParseError(f"Invalid ForEach iteration: {iter_spec}")
        
        item_var = match.group(1)
        collection_expr = match.group(2).strip()
        
        # Parse collection as expression
        collection = self.parse_expression(collection_expr)
        
        # Parse body
        body = []
        while self.current_line < len(self.lines):
            next_line = self.peek_line()
            
            if not next_line or not next_line.startswith('        '):
                break
            
            self.consume_line()
            statement_line = next_line[8:]  # Remove 8 spaces
            
            stmt = self.parse_statement(statement_line)
            if stmt:
                body.append(stmt)
        
        return ForEachLoop(item_var, collection, body)
    
    def parse_task_action(self) -> TaskAction:
        """Parse Task action block."""
        name_line = self.consume_line()
        if not name_line or not name_line.startswith('    Name:'):
            raise ParseError("Task missing Name")
        
        name = name_line.strip()[5:].strip()
        name = self.extract_string_literal(name) or name
        
        # Parse Steps
        steps = []
        while self.current_line < len(self.lines):
            next_line = self.peek_line()
            
            if not next_line or not next_line.startswith('    '):
                break
            
            if next_line.strip().startswith('Step:'):
                self.consume_line()
                step_content = next_line.strip()[5:].strip()
                step_content = self.extract_string_literal(step_content) or step_content
                steps.append(step_content)
        
        return TaskAction(name, steps)
    
    def parse_task_invocation(self, line: str) -> TaskInvocation:
        """Parse RunTask statement."""
        match = re.match(r'RunTask\s+"([^"]+)"', line)
        if not match:
            match = re.match(r"RunTask\s+'([^']+)'", line)
        if not match:
            match = re.match(r'RunTask\s+(\w+)', line)
        
        if not match:
            raise ParseError(f"Invalid RunTask statement: {line}")
        
        task_name = match.group(1)
        return TaskInvocation(task_name)
    
    def parse_action_definition(self) -> ActionDefinition:
        """Parse Action definition."""
        # Check if the Action line has parameters
        first_line = self.lines[self.current_line - 1].strip() if self.current_line > 0 else ""
        
        if '(' in first_line and ')' in first_line:
            return self.parse_action_definition_with_params()
        
        # Check if action name is inline (e.g., "Action sharePhoto:")
        if first_line.startswith('Action ') and first_line.endswith(':'):
            name = first_line[7:-1].strip()  # Remove "Action " and ":"
        else:
            # Old style with Name: on next line
            name_line = self.consume_line()
            if not name_line or not name_line.strip().startswith('Name:'):
                raise ParseError("Action missing Name")
            name = name_line.strip()[5:].strip()
        
        name = self.extract_string_literal(name) or name
        
        # Parse body
        body = []
        while self.current_line < len(self.lines):
            next_line = self.peek_line()
            
            if not next_line or not next_line.startswith('    '):
                break
            
            self.consume_line()
            statement_line = next_line[4:]
            
            # Skip Name line
            if statement_line.strip().startswith('Name:'):
                continue
            
            stmt = self.parse_statement(statement_line)
            if stmt:
                body.append(stmt)
        
        return ActionDefinition(name, body)
    
    def parse_action_definition_with_params(self) -> ActionDefinition:
        """Parse Action definition with parameters."""
        # Get the action line that was already consumed
        action_line = self.lines[self.current_line - 1]
        
        # Extract name and parameters
        match = re.match(r'Action\s+(\w+)\s*\(([^)]*)\)\s*:', action_line)
        if not match:
            raise ParseError(f"Invalid Action definition: {action_line}")
        
        name = match.group(1)
        params_str = match.group(2).strip()
        
        # Parse parameters
        params = []
        if params_str:
            for param in params_str.split(','):
                param = param.strip()
                params.append(param)
        
        # Parse body
        body = []
        while self.current_line < len(self.lines):
            next_line = self.peek_line()
            
            if not next_line or not next_line.startswith('    '):
                break
            
            self.consume_line()
            statement_line = next_line[4:]
            
            stmt = self.parse_statement(statement_line)
            if stmt:
                body.append(stmt)
        
        # For now, treat as regular ActionDefinition
        # You might want to create ActionDefinitionWithParams
        return ActionDefinition(name, body)
    
    def parse_return_statement(self, line: str) -> ReturnStatement:
        """Parse Return statement."""
        value_str = line[7:].strip()  # Remove "Return "
        
        if value_str:
            value = self.parse_expression(value_str)
        else:
            value = None
        
        return ReturnStatement(value)
    
    def parse_data_instance(self, line: str) -> DataInstance:
        """Parse data instance creation."""
        match = re.match(r'Create\s+(\w+)\s+(\w+):', line)
        if not match:
            raise ParseError(f"Invalid Create statement: {line}")
        
        data_type = match.group(1)
        instance_name = match.group(2)
        
        # Parse field assignments
        fields = []
        while self.current_line < len(self.lines):
            next_line = self.peek_line()
            
            if not next_line or not next_line.startswith('    '):
                break
            
            self.consume_line()
            field_line = next_line[4:]
            
            # Parse field assignment
            if '=' in field_line:
                field_name, value_str = field_line.split('=', 1)
                field_name = field_name.strip()
                value_str = value_str.strip()
                
                value = self.parse_expression(value_str)
                fields.append(FieldAssignment(field_name, value))
        
        return DataInstance(data_type, instance_name, fields)
    
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
            
            if next_line.startswith('    '):  # Indented content
                self.consume_line()
                stmt_line = next_line.strip()
                if stmt_line:
                    # Parse the statement inside serve block
                    stmt = self.parse_statement(stmt_line)
                    if stmt:
                        body.append(stmt)
            else:
                break
        
        return ServeStatement(method, endpoint, body)