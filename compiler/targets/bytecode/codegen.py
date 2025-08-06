"""Bytecode code generator for Roe DSL compiler."""

import struct
import json
import time
from typing import List, Dict, Any, Optional
from ...ast import *
from ...codegen_base import BaseCodeGenerator, CodeGenError


class BytecodeGenerator(BaseCodeGenerator):
    """Generates bytecode for the Roe VM."""
    
    def __init__(self):
        super().__init__()
        self.instructions = []
        self.constants = []
        self.labels = {}
        self.label_refs = {}
        self.current_loop_end = None
        self.task_definitions = {}
        
    def generate(self, ast: Program) -> str:
        """Generate bytecode from AST and return as string."""
        try:
            # Generate instructions
            self.visit_program(ast)
            
            # Add halt instruction at the end
            self.emit("Halt")
            
            # Resolve label references
            self._resolve_labels()
            
            # Create bytecode file structure
            bytecode_file = {
                "version": 1,
                "metadata": {
                    "source_file": None,
                    "created_at": int(time.time()),
                    "compiler_version": "0.1.0"
                },
                "constants": self.constants,
                "instructions": self._serialize_instructions(),
                "debug_info": None
            }
            
            # For now, use JSON serialization
            # We'll update the Rust VM to handle JSON
            return json.dumps(bytecode_file)
            
        except Exception as e:
            raise CodeGenError(f"Bytecode generation failed: {str(e)}")
    
    def emit_expression(self, expr: ASTNode):
        """Emit bytecode for an expression."""
        self.visit(expr)
    
    def emit_statement(self, stmt: ASTNode):
        """Emit bytecode for a statement."""
        self.visit(stmt)
    
    def visit(self, node: ASTNode):
        """Visit an AST node using the visitor pattern."""
        method_name = f"visit_{node.__class__.__name__.lower()}"
        if hasattr(self, method_name):
            method = getattr(self, method_name)
            return method(node)
        else:
            raise CodeGenError(f"No visitor method for {node.__class__.__name__}")
    
    def emit(self, opcode: str, *args):
        """Emit a bytecode instruction."""
        instruction = {"op": opcode}
        if args:
            instruction["args"] = list(args)
        self.instructions.append(instruction)
        
    def emit_value(self, value):
        """Emit a value push instruction."""
        if isinstance(value, str):
            self.emit("Push", {"type": "String", "value": value})
        elif isinstance(value, (int, float)):
            self.emit("Push", {"type": "Number", "value": float(value)})
        elif isinstance(value, bool):
            self.emit("Push", {"type": "Boolean", "value": value})
        else:
            self.emit("Push", {"type": "Null"})
    
    def create_label(self) -> str:
        """Create a new unique label."""
        label = f"L{len(self.labels)}"
        return label
        
    def mark_label(self, label: str):
        """Mark the current position with a label."""
        self.labels[label] = len(self.instructions)
        
    def emit_jump(self, opcode: str, label: str):
        """Emit a jump instruction with a label reference."""
        self.label_refs[len(self.instructions)] = label
        self.emit(opcode, 0)  # Placeholder address
        
    def _resolve_labels(self):
        """Resolve all label references to actual addresses."""
        for instruction_idx, label in self.label_refs.items():
            if label not in self.labels:
                raise CodeGenError(f"Undefined label: {label}")
            self.instructions[instruction_idx]["args"][0] = self.labels[label]
    
    def _serialize_instructions(self) -> List[Any]:
        """Convert instructions to serializable format matching Rust enum."""
        result = []
        for inst in self.instructions:
            op = inst["op"]
            args = inst.get("args", [])
            
            # Convert to Rust-compatible format
            if op == "Push" and args:
                value = args[0]
                if value["type"] == "String":
                    result.append({"Push": {"String": value["value"]}})
                elif value["type"] == "Number":
                    result.append({"Push": {"Number": value["value"]}})
                elif value["type"] == "Boolean":
                    result.append({"Push": {"Boolean": value["value"]}})
                else:
                    result.append({"Push": "Null"})
            elif op == "LoadVar" and args:
                result.append({"LoadVar": args[0]})
            elif op == "StoreVar" and args:
                result.append({"StoreVar": args[0]})
            elif op in ["Display", "Add", "Sub", "Mul", "Div", "Eq", "Neq", "Lt", "Gt", "Lte", "Gte", "Pop", "Dup", "Return", "Halt", "Nop"]:
                result.append(op)
            elif op == "Jump" and args:
                result.append({"Jump": args[0]})
            elif op == "JumpIfFalse" and args:
                result.append({"JumpIfFalse": args[0]})
            elif op == "JumpIfTrue" and args:
                result.append({"JumpIfTrue": args[0]})
            elif op == "CreateArray" and args:
                result.append({"CreateArray": args[0]})
            elif op == "DefineTask" and len(args) >= 3:
                result.append({"DefineTask": [args[0], args[1], args[2]]})
            elif op == "RunTask" and len(args) >= 2:
                result.append({"RunTask": [args[0], args[1]]})
            else:
                # Fallback for unknown instructions
                result.append(op)
        
        return result
    
    def _convert_to_rust_format(self, bytecode_file: dict) -> dict:
        """Convert bytecode to format matching Rust structures."""
        instructions = []
        
        for inst in bytecode_file["instructions"]:
            op = inst["op"]
            args = inst.get("args", [])
            
            # Convert to Rust enum format
            if op == "Push":
                value = args[0]
                if value["type"] == "String":
                    instructions.append({"Push": {"String": value["value"]}})
                elif value["type"] == "Number":
                    instructions.append({"Push": {"Number": value["value"]}})
                elif value["type"] == "Boolean":
                    instructions.append({"Push": {"Boolean": value["value"]}})
                else:
                    instructions.append({"Push": "Null"})
            elif op == "LoadVar":
                instructions.append({"LoadVar": args[0]})
            elif op == "StoreVar":
                instructions.append({"StoreVar": args[0]})
            elif op == "Display":
                instructions.append("Display")
            elif op == "Halt":
                instructions.append("Halt")
            elif op in ["Add", "Sub", "Mul", "Div", "Eq", "Neq", "Lt", "Gt", "Lte", "Gte"]:
                instructions.append(op)
            elif op == "Jump":
                instructions.append({"Jump": args[0]})
            elif op == "JumpIfFalse":
                instructions.append({"JumpIfFalse": args[0]})
            elif op == "JumpIfTrue":
                instructions.append({"JumpIfTrue": args[0]})
            elif op == "CreateArray":
                instructions.append({"CreateArray": args[0]})
            elif op == "DefineTask":
                instructions.append({"DefineTask": [args[0], args[1], args[2]]})
            elif op == "RunTask":
                instructions.append({"RunTask": [args[0], args[1]]})
            else:
                instructions.append(op)
        
        return {
            "version": bytecode_file["version"],
            "metadata": bytecode_file["metadata"],
            "constants": bytecode_file["constants"],
            "instructions": instructions,
            "debug_info": bytecode_file["debug_info"]
        }
    
    def visit_program(self, node: Program):
        """Visit program node."""
        # Process included modules first
        if node.included_modules:
            for include in node.included_modules:
                self.visit_include_statement(include)
        
        # Then process all statements
        for stmt in node.statements:
            self.visit(stmt)
    
    def visit_displaystatement(self, node: DisplayStatement):
        """Visit display statement."""
        self.visit(node.expression)
        self.emit("Display")
    
    def visit_literal(self, node: Literal):
        """Visit literal value."""
        self.emit_value(node.value)
    
    def visit_identifier(self, node: Identifier):
        """Visit identifier."""
        self.emit("LoadVar", node.name)
    
    def visit_binaryop(self, node: BinaryOp):
        """Visit binary operation."""
        self.visit(node.left)
        self.visit(node.right)
        
        op_map = {
            '>': 'Gt',
            '<': 'Lt',
            '>=': 'Gte',
            '<=': 'Lte',
            '==': 'Eq',
            '!=': 'Neq',
            '+': 'Add',
            '-': 'Sub',
            '*': 'Mul',
            '/': 'Div',
        }
        
        if node.operator in op_map:
            self.emit(op_map[node.operator])
        else:
            raise CodeGenError(f"Unknown operator: {node.operator}")
    
    def visit_assignment(self, node: Assignment):
        """Visit assignment."""
        self.visit(node.value)
        self.emit("StoreVar", node.variable)
    
    def visit_if_statement(self, node: IfStatement):
        """Visit if statement."""
        else_label = self.create_label()
        end_label = self.create_label()
        
        # Evaluate condition
        self.visit(node.condition)
        
        # Jump to else if false
        self.emit_jump("JumpIfFalse", else_label)
        
        # Then body
        for stmt in node.then_body:
            self.visit(stmt)
        
        # Jump to end
        self.emit_jump("Jump", end_label)
        
        # Else label
        self.mark_label(else_label)
        
        # Else body
        if node.else_body:
            for stmt in node.else_body:
                self.visit(stmt)
        
        # End label
        self.mark_label(end_label)
    
    def visit_while_loop(self, node: WhileLoop):
        """Visit while loop."""
        start_label = self.create_label()
        end_label = self.create_label()
        
        # Save previous loop end
        prev_loop_end = self.current_loop_end
        self.current_loop_end = end_label
        
        # Start of loop
        self.mark_label(start_label)
        
        # Evaluate condition
        self.visit(node.condition)
        
        # Exit if false
        self.emit_jump("JumpIfFalse", end_label)
        
        # Loop body
        for stmt in node.body:
            self.visit(stmt)
        
        # Jump back to start
        self.emit_jump("Jump", start_label)
        
        # End of loop
        self.mark_label(end_label)
        
        # Restore previous loop end
        self.current_loop_end = prev_loop_end
    
    def visit_for_each_loop(self, node: ForEachLoop):
        """Visit for each loop."""
        # This is a simplified implementation
        # In a real VM, we'd need iterator support
        raise CodeGenError("ForEach loops not yet implemented in bytecode")
    
    def visit_array_literal(self, node: ArrayLiteral):
        """Visit array literal."""
        # Push all elements
        for element in node.elements:
            self.visit(element)
        
        # Create array
        self.emit("CreateArray", len(node.elements))
    
    def visit_task_action(self, node: TaskAction):
        """Visit task definition."""
        end_label = self.create_label()
        
        # Store task definition
        self.task_definitions[node.name] = {
            "params": [p.name for p in node.parameters],
            "start": len(self.instructions)
        }
        
        # Emit task definition
        params = [p.name for p in node.parameters]
        self.emit_jump("DefineTask", end_label)
        self.instructions[-1]["args"] = [node.name, params, 0]  # Will fix end address
        
        # Task body
        for stmt in node.body:
            self.visit(stmt)
        
        # Mark end
        self.mark_label(end_label)
        
        # Fix the task end address
        for i, inst in enumerate(self.instructions):
            if (inst["op"] == "DefineTask" and 
                len(inst["args"]) > 0 and 
                inst["args"][0] == node.name):
                inst["args"][2] = self.labels[end_label]
                break
    
    def visit_task_invocation(self, node: TaskInvocation):
        """Visit task invocation."""
        # Push arguments
        for arg in node.arguments:
            self.visit(arg)
        
        # Run task
        self.emit("RunTask", node.task_name, len(node.arguments))
    
    def visit_property_access(self, node: PropertyAccess):
        """Visit property access."""
        self.visit(node.object)
        self.emit("GetField", node.property)
    
    def visit_return_statement(self, node: ReturnStatement):
        """Visit return statement."""
        self.visit(node.expression)
        self.emit("Return")
    
    def visit_string_interpolation(self, node: StringInterpolation):
        """Visit string interpolation."""
        result_parts = []
        
        for part in node.parts:
            if isinstance(part, Literal):
                result_parts.append(part.value)
            else:
                # For now, just convert to string
                # In a real implementation, we'd need string conversion
                result_parts.append("[var]")
        
        self.emit_value("".join(result_parts))
    
    def visit_data_instance(self, node: DataInstance):
        """Visit data instance creation."""
        # Create object
        self.emit("CreateObject", node.data_type)
        
        # Set fields
        for field_assignment in node.field_values:
            self.emit("Dup")  # Duplicate object reference
            self.visit(field_assignment.value)
            self.emit("SetField", field_assignment.field_name)
    
    def visit_include_statement(self, node: IncludeStatement):
        """Visit include statement."""
        # In bytecode, includes are resolved at compile time
        # The included module's code is already part of the AST
        pass
    
    def visit_module_definition(self, node: ModuleDefinition):
        """Visit module definition."""
        # For now, just process the body
        # In a real implementation, we'd create module scope
        for stmt in node.body:
            self.visit(stmt)
    
    def visit_action_definition(self, node: ActionDefinition):
        """Visit action definition (simple)."""
        # Similar to task but simpler
        self.visit_task_action(TaskAction(
            name=node.name,
            parameters=[],
            body=node.body
        ))
    
    def visit_action_definition_with_params(self, node: ActionDefinitionWithParams):
        """Visit action definition with parameters."""
        # Convert to task for now
        self.visit_task_action(TaskAction(
            name=node.name,
            parameters=node.parameters,
            body=node.body
        ))
    
    def visit_data_definition(self, node: DataDefinition):
        """Visit data definition."""
        # Data definitions are compile-time only in bytecode
        # The VM doesn't need them explicitly
        pass


def generate(ast: Program, options: Optional[Dict[str, Any]] = None) -> str:
    """Entry point for bytecode generation."""
    generator = BytecodeGenerator()
    bytecode_bytes = generator.generate(ast)
    
    # For file writing, we'll return the bytes as a string
    # The file writer should handle this as binary
    return bytecode_bytes.decode('utf-8')