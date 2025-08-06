use crate::bytecode::{BytecodeFile, Instruction, Value};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

pub struct VM {
    stack: Vec<Value>,
    variables: HashMap<String, Value>,
    globals: HashMap<String, Value>,
    call_stack: Vec<CallFrame>,
    pc: usize,
    instructions: Vec<Instruction>,
    tasks: HashMap<String, TaskDefinition>,
    modules: HashMap<String, HashMap<String, Value>>,
}

struct CallFrame {
    return_address: usize,
    locals: HashMap<String, Value>,
}

struct TaskDefinition {
    parameters: Vec<String>,
    body_start: usize,
    body_end: usize,
}

impl VM {
    pub fn new(bytecode: BytecodeFile) -> Self {
        Self {
            stack: Vec::new(),
            variables: HashMap::new(),
            globals: HashMap::new(),
            call_stack: Vec::new(),
            pc: 0,
            instructions: bytecode.instructions,
            tasks: HashMap::new(),
            modules: HashMap::new(),
        }
    }
    
    pub fn run(&mut self) -> Result<()> {
        while self.pc < self.instructions.len() {
            let instruction = self.instructions[self.pc].clone();
            self.execute_instruction(instruction)?;
        }
        Ok(())
    }
    
    fn execute_instruction(&mut self, instruction: Instruction) -> Result<()> {
        match instruction {
            Instruction::Push(value) => {
                self.stack.push(value);
                self.pc += 1;
            }
            
            Instruction::Pop => {
                self.stack.pop().ok_or_else(|| anyhow!("Stack underflow"))?;
                self.pc += 1;
            }
            
            Instruction::Dup => {
                let value = self.stack.last()
                    .ok_or_else(|| anyhow!("Stack underflow"))?
                    .clone();
                self.stack.push(value);
                self.pc += 1;
            }
            
            Instruction::LoadVar(name) => {
                let value = self.variables.get(&name)
                    .or_else(|| self.globals.get(&name))
                    .ok_or_else(|| anyhow!("Variable '{}' not found", name))?
                    .clone();
                self.stack.push(value);
                self.pc += 1;
            }
            
            Instruction::StoreVar(name) => {
                let value = self.stack.pop()
                    .ok_or_else(|| anyhow!("Stack underflow"))?;
                self.variables.insert(name, value);
                self.pc += 1;
            }
            
            Instruction::Add => {
                let b = self.pop_number()?;
                let a = self.pop_number()?;
                self.stack.push(Value::Number(a + b));
                self.pc += 1;
            }
            
            Instruction::Sub => {
                let b = self.pop_number()?;
                let a = self.pop_number()?;
                self.stack.push(Value::Number(a - b));
                self.pc += 1;
            }
            
            Instruction::Mul => {
                let b = self.pop_number()?;
                let a = self.pop_number()?;
                self.stack.push(Value::Number(a * b));
                self.pc += 1;
            }
            
            Instruction::Div => {
                let b = self.pop_number()?;
                let a = self.pop_number()?;
                if b == 0.0 {
                    return Err(anyhow!("Division by zero"));
                }
                self.stack.push(Value::Number(a / b));
                self.pc += 1;
            }
            
            Instruction::Eq => {
                let b = self.stack.pop().ok_or_else(|| anyhow!("Stack underflow"))?;
                let a = self.stack.pop().ok_or_else(|| anyhow!("Stack underflow"))?;
                self.stack.push(Value::Boolean(a == b));
                self.pc += 1;
            }
            
            Instruction::Neq => {
                let b = self.stack.pop().ok_or_else(|| anyhow!("Stack underflow"))?;
                let a = self.stack.pop().ok_or_else(|| anyhow!("Stack underflow"))?;
                self.stack.push(Value::Boolean(a != b));
                self.pc += 1;
            }
            
            Instruction::Lt => {
                let b = self.pop_number()?;
                let a = self.pop_number()?;
                self.stack.push(Value::Boolean(a < b));
                self.pc += 1;
            }
            
            Instruction::Gt => {
                let b = self.pop_number()?;
                let a = self.pop_number()?;
                self.stack.push(Value::Boolean(a > b));
                self.pc += 1;
            }
            
            Instruction::Lte => {
                let b = self.pop_number()?;
                let a = self.pop_number()?;
                self.stack.push(Value::Boolean(a <= b));
                self.pc += 1;
            }
            
            Instruction::Gte => {
                let b = self.pop_number()?;
                let a = self.pop_number()?;
                self.stack.push(Value::Boolean(a >= b));
                self.pc += 1;
            }
            
            Instruction::Jump(addr) => {
                self.pc = addr;
            }
            
            Instruction::JumpIfFalse(addr) => {
                let condition = self.pop_boolean()?;
                if !condition {
                    self.pc = addr;
                } else {
                    self.pc += 1;
                }
            }
            
            Instruction::JumpIfTrue(addr) => {
                let condition = self.pop_boolean()?;
                if condition {
                    self.pc = addr;
                } else {
                    self.pc += 1;
                }
            }
            
            Instruction::Display => {
                let value = self.stack.pop()
                    .ok_or_else(|| anyhow!("Stack underflow"))?;
                println!("{}", self.value_to_string(&value));
                self.pc += 1;
            }
            
            Instruction::DefineTask(name, params, body_end) => {
                self.tasks.insert(name, TaskDefinition {
                    parameters: params,
                    body_start: self.pc + 1,
                    body_end,
                });
                self.pc = body_end + 1;
            }
            
            Instruction::RunTask(name, arg_count) => {
                let task = self.tasks.get(&name)
                    .ok_or_else(|| anyhow!("Task '{}' not found", name))?
                    .clone();
                
                if arg_count != task.parameters.len() {
                    return Err(anyhow!("Task '{}' expects {} arguments, got {}", 
                        name, task.parameters.len(), arg_count));
                }
                
                let mut args = Vec::new();
                for _ in 0..arg_count {
                    args.push(self.stack.pop()
                        .ok_or_else(|| anyhow!("Stack underflow"))?);
                }
                args.reverse();
                
                let mut locals = HashMap::new();
                for (param, arg) in task.parameters.iter().zip(args.iter()) {
                    locals.insert(param.clone(), arg.clone());
                }
                
                self.call_stack.push(CallFrame {
                    return_address: self.pc + 1,
                    locals: std::mem::take(&mut self.variables),
                });
                
                self.variables = locals;
                self.pc = task.body_start;
            }
            
            Instruction::CreateArray(size) => {
                let mut elements = Vec::new();
                for _ in 0..size {
                    elements.push(self.stack.pop()
                        .ok_or_else(|| anyhow!("Stack underflow"))?);
                }
                elements.reverse();
                self.stack.push(Value::Array(elements));
                self.pc += 1;
            }
            
            Instruction::Halt => {
                self.pc = self.instructions.len();
            }
            
            Instruction::Nop => {
                self.pc += 1;
            }
            
            _ => {
                return Err(anyhow!("Unimplemented instruction: {:?}", instruction));
            }
        }
        
        Ok(())
    }
    
    fn pop_number(&mut self) -> Result<f64> {
        match self.stack.pop() {
            Some(Value::Number(n)) => Ok(n),
            Some(_) => Err(anyhow!("Expected number")),
            None => Err(anyhow!("Stack underflow")),
        }
    }
    
    fn pop_boolean(&mut self) -> Result<bool> {
        match self.stack.pop() {
            Some(Value::Boolean(b)) => Ok(b),
            Some(_) => Err(anyhow!("Expected boolean")),
            None => Err(anyhow!("Stack underflow")),
        }
    }
    
    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Array(arr) => {
                let elements: Vec<String> = arr.iter()
                    .map(|v| self.value_to_string(v))
                    .collect();
                format!("[{}]", elements.join(", "))
            }
            Value::Object(typ, fields) => {
                let field_strs: Vec<String> = fields.iter()
                    .map(|(k, v)| format!("{}: {}", k, self.value_to_string(v)))
                    .collect();
                format!("{} {{ {} }}", typ, field_strs.join(", "))
            }
            Value::Null => "null".to_string(),
        }
    }
}