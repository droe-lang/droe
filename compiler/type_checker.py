"""Strong type checker for Roelang DSL."""

from typing import List, Dict, Optional
from .ast import *
from .symbols import VariableType, SymbolTable
from .codegen_base import BaseCodeGenerator


class TypeCheckError(Exception):
    """Type checking error."""
    pass


class TypeChecker:
    """Enforces strong typing throughout Roelang programs."""
    
    def __init__(self):
        self.symbol_table = SymbolTable()
        self.action_types: Dict[str, str] = {}  # action_name -> return_type
        self.task_types: Dict[str, str] = {}    # task_name -> return_type (if any)
        
    def check_program(self, program: Program):
        """Perform strong type checking on entire program."""
        # First pass: collect all action and task signatures
        self._collect_signatures(program)
        
        # Second pass: validate all statements and expressions
        for statement in program.statements:
            self._check_statement(statement)
    
    def _collect_signatures(self, program: Program):
        """Collect all action and task type signatures."""
        for statement in program.statements:
            if isinstance(statement, ActionDefinitionWithParams):
                self.action_types[statement.name] = statement.return_type
            elif isinstance(statement, ModuleDefinition):
                for module_stmt in statement.body:
                    if isinstance(module_stmt, ActionDefinitionWithParams):
                        full_name = f"{statement.name}.{module_stmt.name}"
                        self.action_types[full_name] = module_stmt.return_type
    
    def _check_statement(self, statement: ASTNode):
        """Type check a statement."""
        if isinstance(statement, Assignment):
            self._check_assignment(statement)
        elif isinstance(statement, ActionDefinitionWithParams):
            self._check_action_definition(statement)
        elif isinstance(statement, TaskAction):
            self._check_task_definition(statement)
        elif isinstance(statement, ModuleDefinition):
            for module_stmt in statement.body:
                self._check_statement(module_stmt)
        elif isinstance(statement, IfStatement):
            self._check_conditional(statement)
        elif isinstance(statement, WhileLoop):
            self._check_while_loop(statement)
        elif isinstance(statement, ForEachLoop):
            self._check_foreach_loop(statement)
    
    def _check_assignment(self, assignment: Assignment):
        """Check assignment type compatibility."""
        line_info = f" at line {assignment.line_number}" if assignment.line_number else ""
        
        # Get declared type if present
        declared_type = getattr(assignment, 'declared_var_type', None)
        
        if declared_type is None:
            raise TypeCheckError(f"Variable '{assignment.variable}' must have explicit type declaration{line_info}")
        
        # Check if declared type is valid
        try:
            mapped_type = self._map_user_type_to_internal(declared_type)
        except:
            raise TypeCheckError(f"Unknown type '{declared_type}' for variable '{assignment.variable}'{line_info}")
        
        # Infer type from value
        inferred_type = self._infer_expression_type(assignment.value)
        
        # Check compatibility
        if not self._are_types_compatible(mapped_type, inferred_type):
            raise TypeCheckError(
                f"Type mismatch: Cannot assign {inferred_type.value} to {declared_type} variable '{assignment.variable}'{line_info}"
            )
        
        # Register variable
        self.symbol_table.declare_variable(assignment.variable, mapped_type)
    
    def _check_action_definition(self, action: ActionDefinitionWithParams):
        """Check action definition type safety."""
        # Create new scope for this action with parameters
        old_symbol_table = self.symbol_table
        self.symbol_table = SymbolTable()
        
        # Add parameters to symbol table
        if action.parameters:
            for param in action.parameters:
                param_type = self._map_user_type_to_internal(param.type)
                self.symbol_table.declare_variable(param.name, param_type)
        
        # Check that all return statements match declared return type
        declared_return_type = self._map_user_type_to_internal(action.return_type)
        
        has_return = False
        for stmt in action.body:
            if isinstance(stmt, ReturnStatement):
                has_return = True
                return_expr_type = self._infer_expression_type(stmt.expression)
                if not self._are_types_compatible(declared_return_type, return_expr_type):
                    raise TypeCheckError(
                        f"Action '{action.name}' return type mismatch: declared {action.return_type}, "
                        f"but returns {return_expr_type.value}"
                    )
        
        # Actions must have at least one return statement
        if not has_return:
            raise TypeCheckError(f"Action '{action.name}' must have at least one 'give' statement")
        
        # Restore previous symbol table
        self.symbol_table = old_symbol_table
    
    def _check_task_definition(self, task: TaskAction):
        """Check task definition - currently tasks don't return values."""
        # For now, just validate parameter types if any
        pass
    
    def _check_conditional(self, if_stmt: IfStatement):
        """Check if statement condition type."""
        condition_type = self._infer_expression_type(if_stmt.condition)
        if not self._is_boolean_type(condition_type):
            raise TypeCheckError(f"If condition must be boolean, got {condition_type.value}")
        
        # Check bodies
        if if_stmt.then_body:
            for stmt in if_stmt.then_body:
                self._check_statement(stmt)
        if if_stmt.else_body:
            for stmt in if_stmt.else_body:
                self._check_statement(stmt)
    
    def _check_while_loop(self, while_loop: WhileLoop):
        """Check while loop condition type."""
        condition_type = self._infer_expression_type(while_loop.condition)
        if not self._is_boolean_type(condition_type):
            raise TypeCheckError(f"While condition must be boolean, got {condition_type.value}")
        
        # Check body
        if while_loop.body:
            for stmt in while_loop.body:
                self._check_statement(stmt)
    
    def _check_foreach_loop(self, foreach: ForEachLoop):
        """Check foreach loop collection type."""
        collection_type = self._infer_expression_type(foreach.collection)
        if not self._is_collection_type(collection_type):
            raise TypeCheckError(f"Foreach requires collection type, got {collection_type.value}")
    
    def _infer_expression_type(self, expr: ASTNode) -> VariableType:
        """Infer the type of an expression."""
        if isinstance(expr, Literal):
            if isinstance(expr.value, bool):
                return VariableType.FLAG
            elif isinstance(expr.value, int):
                return VariableType.INT
            elif isinstance(expr.value, float):
                return VariableType.DECIMAL
            elif isinstance(expr.value, str):
                return VariableType.TEXT
        elif isinstance(expr, Identifier):
            var = self.symbol_table.get_variable(expr.name)
            if var:
                return var.type
            else:
                # Check if it's an action call
                if expr.name in self.action_types:
                    return_type = self.action_types[expr.name]
                    return self._map_user_type_to_internal(return_type)
                raise TypeCheckError(f"Undefined variable or action: {expr.name}")
        elif isinstance(expr, ArrayLiteral):
            return VariableType.LIST_OF
        elif isinstance(expr, BinaryOp):
            left_type = self._infer_expression_type(expr.left)
            right_type = self._infer_expression_type(expr.right)
            
            if expr.operator in ['+', '-', '*', '/']:
                # Numeric operations
                if self._is_numeric_type(left_type) and self._is_numeric_type(right_type):
                    # Return higher precision type
                    if left_type == VariableType.DECIMAL or right_type == VariableType.DECIMAL:
                        return VariableType.DECIMAL
                    return VariableType.INT
                # String concatenation (only for + operator)
                elif expr.operator == '+' and (self._is_text_type(left_type) or self._is_text_type(right_type)):
                    return VariableType.TEXT
                else:
                    raise TypeCheckError(f"Invalid operation {expr.operator} between {left_type.value} and {right_type.value}")
            elif expr.operator in ['==', '!=', '<', '>', '<=', '>=']:
                # Comparison operations return boolean
                return VariableType.FLAG
            elif expr.operator in ['and', 'or']:
                # Logical operations
                if self._is_boolean_type(left_type) and self._is_boolean_type(right_type):
                    return VariableType.FLAG
                raise TypeCheckError(f"Logical operators require boolean operands")
        elif isinstance(expr, ArithmeticOp):
            # Handle ArithmeticOp same as BinaryOp
            left_type = self._infer_expression_type(expr.left)  
            right_type = self._infer_expression_type(expr.right)
            
            if expr.operator in ['+', '-', '*', '/']:
                if self._is_numeric_type(left_type) and self._is_numeric_type(right_type):
                    if left_type == VariableType.DECIMAL or right_type == VariableType.DECIMAL:
                        return VariableType.DECIMAL
                    return VariableType.INT
                elif expr.operator == '+' and (self._is_text_type(left_type) or self._is_text_type(right_type)):
                    return VariableType.TEXT
                else:
                    raise TypeCheckError(f"Invalid arithmetic operation {expr.operator} between {left_type.value} and {right_type.value}")
        elif isinstance(expr, ActionInvocationWithArgs):
            action_name = expr.action_name
            if expr.module_name:
                action_name = f"{expr.module_name}.{action_name}"
            if action_name in self.action_types:
                return_type = self.action_types[action_name]
                return self._map_user_type_to_internal(return_type)
            raise TypeCheckError(f"Unknown action: {action_name}")
        
        return VariableType.TEXT  # Default fallback
    
    def _map_user_type_to_internal(self, user_type: str) -> VariableType:
        """Map user-facing type names to internal VariableType enum."""
        type_map = {
            'int': VariableType.INT,
            'decimal': VariableType.DECIMAL,
            'text': VariableType.TEXT,
            'flag': VariableType.FLAG,
            'yesno': VariableType.YESNO,
            'date': VariableType.DATE,
            'list_of': VariableType.LIST_OF,
            'group_of': VariableType.GROUP_OF,
            # Legacy support
            'number': VariableType.NUMBER,
            'string': VariableType.STRING,
            'boolean': VariableType.BOOLEAN,
            'array': VariableType.ARRAY,
        }
        
        if user_type not in type_map:
            raise TypeCheckError(f"Unknown type: {user_type}")
        
        return type_map[user_type]
    
    def _are_types_compatible(self, declared_type: VariableType, inferred_type: VariableType) -> bool:
        """Check if two types are compatible for assignment."""
        # Exact match
        if declared_type == inferred_type:
            return True
        
        # Numeric type compatibility
        if self._is_numeric_type(declared_type) and self._is_numeric_type(inferred_type):
            return True
        
        # Text type compatibility
        if self._is_text_type(declared_type) and self._is_text_type(inferred_type):
            return True
        
        # Boolean type compatibility
        if self._is_boolean_type(declared_type) and self._is_boolean_type(inferred_type):
            return True
        
        # Date-string compatibility (strings can be assigned to dates)
        if declared_type == VariableType.DATE and self._is_text_type(inferred_type):
            return True
        
        # Collection type compatibility
        if self._is_collection_type(declared_type) and self._is_collection_type(inferred_type):
            return True
        
        return False
    
    def _is_numeric_type(self, var_type: VariableType) -> bool:
        """Check if type is numeric."""
        return var_type in [VariableType.INT, VariableType.DECIMAL, VariableType.NUMBER]
    
    def _is_text_type(self, var_type: VariableType) -> bool:
        """Check if type is text-like."""
        return var_type in [VariableType.TEXT, VariableType.STRING]
    
    def _is_boolean_type(self, var_type: VariableType) -> bool:
        """Check if type is boolean."""
        return var_type in [VariableType.FLAG, VariableType.YESNO, VariableType.BOOLEAN]
    
    def _is_collection_type(self, var_type: VariableType) -> bool:
        """Check if type is a collection."""
        return var_type in [VariableType.LIST_OF, VariableType.GROUP_OF, VariableType.ARRAY]