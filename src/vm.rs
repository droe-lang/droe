use anyhow::Result;
use std::path::Path;
use console::style;
use wasmtime::{Engine, Module, Store, Instance, Linker};
use serde_json::Value;
use std::collections::HashMap;

pub struct DroeVM {
    engine: Engine,
    store: Store<()>,
    linker: Linker<()>,
}

impl DroeVM {
    pub fn new() -> Result<Self> {
        let engine = Engine::default();
        let store = Store::new(&engine, ());
        let mut linker = Linker::new(&engine);
        
        // Define host functions
        linker.func_wrap("env", "display", |msg: i32| {
            println!("VM Output: {}", msg);
        })?;
        
        linker.func_wrap("env", "log", |level: i32, msg: i32| {
            println!("VM Log[{}]: {}", level, msg);
        })?;
        
        Ok(Self {
            engine,
            store,
            linker,
        })
    }
    
    pub fn load_module(&mut self, wasm_bytes: &[u8]) -> Result<Module> {
        Module::new(&self.engine, wasm_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to load WASM module: {}", e))
    }
    
    pub fn instantiate(&mut self, module: &Module) -> Result<Instance> {
        self.linker.instantiate(&mut self.store, module)
            .map_err(|e| anyhow::anyhow!("Failed to instantiate module: {}", e))
    }
    
    pub fn call_function(&mut self, instance: &Instance, func_name: &str, _args: &[wasmtime::Val]) -> Result<Vec<wasmtime::Val>> {
        let func = instance
            .get_typed_func::<(), ()>(&mut self.store, func_name)
            .map_err(|e| anyhow::anyhow!("Function '{}' not found: {}", func_name, e))?;
        
        func.call(&mut self.store, ())
            .map_err(|e| anyhow::anyhow!("Function call failed: {}", e))?;
        
        Ok(vec![])
    }
}

/// Droe Bytecode Virtual Machine
pub struct DroeVirtualMachine {
    stack: Vec<Value>,
    variables: HashMap<String, Value>,
    program_counter: usize,
}

impl DroeVirtualMachine {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            variables: HashMap::new(),
            program_counter: 0,
        }
    }
    
    pub fn execute(&mut self, instructions: &[Value]) -> Result<()> {
        self.program_counter = 0;
        
        while self.program_counter < instructions.len() {
            let instruction = &instructions[self.program_counter];
            
            match instruction {
                Value::String(op) => {
                    match op.as_str() {
                        "Display" => self.execute_display()?,
                        "Halt" => break,
                        "Add" => self.execute_add()?,
                        "Sub" => self.execute_sub()?,
                        "Mul" => self.execute_mul()?,
                        "Div" => self.execute_div()?,
                        "Eq" => self.execute_eq()?,
                        "Neq" => self.execute_neq()?,
                        "Lt" => self.execute_lt()?,
                        "Gt" => self.execute_gt()?,
                        "Lte" => self.execute_lte()?,
                        "Gte" => self.execute_gte()?,
                        "Pop" => self.execute_pop()?,
                        "Dup" => self.execute_dup()?,
                        "ToString" => self.execute_to_string()?,
                        "Concat" => self.execute_concat()?,
                        "Return" => break, // For now, just stop execution
                        "Nop" => {}, // No operation
                        _ => return Err(anyhow::anyhow!("Unknown instruction: {}", op)),
                    }
                }
                Value::Object(obj) => {
                    if let Some(push_value) = obj.get("Push") {
                        self.execute_push(push_value)?;
                    } else if let Some(var_name) = obj.get("LoadVar") {
                        if let Some(name) = var_name.as_str() {
                            self.execute_load_var(name)?;
                        }
                    } else if let Some(var_name) = obj.get("StoreVar") {
                        if let Some(name) = var_name.as_str() {
                            self.execute_store_var(name)?;
                        }
                    } else if let Some(addr) = obj.get("Jump") {
                        if let Some(address) = addr.as_u64() {
                            self.execute_jump(address as usize)?;
                            continue; // Don't increment PC
                        }
                    } else if let Some(addr) = obj.get("JumpIfFalse") {
                        if let Some(address) = addr.as_u64() {
                            if self.execute_jump_if_false(address as usize)? {
                                continue; // Don't increment PC if jump occurred
                            }
                        }
                    } else if let Some(addr) = obj.get("JumpIfTrue") {
                        if let Some(address) = addr.as_u64() {
                            if self.execute_jump_if_true(address as usize)? {
                                continue; // Don't increment PC if jump occurred
                            }
                        }
                    } else if let Some(size) = obj.get("CreateArray") {
                        if let Some(array_size) = size.as_u64() {
                            self.execute_create_array(array_size as usize)?;
                        }
                    } else {
                        return Err(anyhow::anyhow!("Unknown instruction object: {:?}", obj));
                    }
                }
                _ => return Err(anyhow::anyhow!("Invalid instruction format: {:?}", instruction)),
            }
            
            self.program_counter += 1;
        }
        
        Ok(())
    }
    
    fn execute_push(&mut self, value: &Value) -> Result<()> {
        match value {
            Value::Object(obj) => {
                if let Some(s) = obj.get("String").and_then(|v| v.as_str()) {
                    self.stack.push(Value::String(s.to_string()));
                } else if let Some(n) = obj.get("Number").and_then(|v| v.as_f64()) {
                    self.stack.push(Value::Number(serde_json::Number::from_f64(n).unwrap()));
                } else if let Some(b) = obj.get("Boolean").and_then(|v| v.as_bool()) {
                    self.stack.push(Value::Bool(b));
                } else {
                    self.stack.push(Value::Null);
                }
            }
            _ => {
                self.stack.push(value.clone());
            }
        }
        Ok(())
    }
    
    fn execute_display(&mut self) -> Result<()> {
        if let Some(value) = self.stack.pop() {
            match value {
                Value::String(s) => println!("{}", s),
                Value::Number(n) => println!("{}", n),
                Value::Bool(b) => println!("{}", b),
                Value::Null => println!("null"),
                _ => println!("{}", value),
            }
        } else {
            return Err(anyhow::anyhow!("Stack underflow: no value to display"));
        }
        Ok(())
    }
    
    fn execute_load_var(&mut self, var_name: &str) -> Result<()> {
        if let Some(value) = self.variables.get(var_name) {
            self.stack.push(value.clone());
        } else {
            return Err(anyhow::anyhow!("Undefined variable: {}", var_name));
        }
        Ok(())
    }
    
    fn execute_store_var(&mut self, var_name: &str) -> Result<()> {
        if let Some(value) = self.stack.pop() {
            self.variables.insert(var_name.to_string(), value);
        } else {
            return Err(anyhow::anyhow!("Stack underflow: no value to store"));
        }
        Ok(())
    }
    
    fn execute_add(&mut self) -> Result<()> {
        let b = self.stack.pop().ok_or_else(|| anyhow::anyhow!("Stack underflow"))?;
        let a = self.stack.pop().ok_or_else(|| anyhow::anyhow!("Stack underflow"))?;
        
        match (&a, &b) {
            (Value::Number(a_num), Value::Number(b_num)) => {
                let result = a_num.as_f64().unwrap() + b_num.as_f64().unwrap();
                self.stack.push(Value::Number(serde_json::Number::from_f64(result).unwrap()));
            }
            (Value::String(a_str), Value::String(b_str)) => {
                self.stack.push(Value::String(format!("{}{}", a_str, b_str)));
            }
            _ => return Err(anyhow::anyhow!("Type error: cannot add {:?} and {:?}", a, b)),
        }
        Ok(())
    }
    
    fn execute_sub(&mut self) -> Result<()> {
        let b = self.stack.pop().ok_or_else(|| anyhow::anyhow!("Stack underflow"))?;
        let a = self.stack.pop().ok_or_else(|| anyhow::anyhow!("Stack underflow"))?;
        
        match (&a, &b) {
            (Value::Number(a_num), Value::Number(b_num)) => {
                let result = a_num.as_f64().unwrap() - b_num.as_f64().unwrap();
                self.stack.push(Value::Number(serde_json::Number::from_f64(result).unwrap()));
            }
            _ => return Err(anyhow::anyhow!("Type error: cannot subtract {:?} and {:?}", a, b)),
        }
        Ok(())
    }
    
    fn execute_mul(&mut self) -> Result<()> {
        let b = self.stack.pop().ok_or_else(|| anyhow::anyhow!("Stack underflow"))?;
        let a = self.stack.pop().ok_or_else(|| anyhow::anyhow!("Stack underflow"))?;
        
        match (&a, &b) {
            (Value::Number(a_num), Value::Number(b_num)) => {
                let result = a_num.as_f64().unwrap() * b_num.as_f64().unwrap();
                self.stack.push(Value::Number(serde_json::Number::from_f64(result).unwrap()));
            }
            _ => return Err(anyhow::anyhow!("Type error: cannot multiply {:?} and {:?}", a, b)),
        }
        Ok(())
    }
    
    fn execute_div(&mut self) -> Result<()> {
        let b = self.stack.pop().ok_or_else(|| anyhow::anyhow!("Stack underflow"))?;
        let a = self.stack.pop().ok_or_else(|| anyhow::anyhow!("Stack underflow"))?;
        
        match (&a, &b) {
            (Value::Number(a_num), Value::Number(b_num)) => {
                let divisor = b_num.as_f64().unwrap();
                if divisor == 0.0 {
                    return Err(anyhow::anyhow!("Division by zero"));
                }
                let result = a_num.as_f64().unwrap() / divisor;
                self.stack.push(Value::Number(serde_json::Number::from_f64(result).unwrap()));
            }
            _ => return Err(anyhow::anyhow!("Type error: cannot divide {:?} and {:?}", a, b)),
        }
        Ok(())
    }
    
    fn execute_eq(&mut self) -> Result<()> {
        let b = self.stack.pop().ok_or_else(|| anyhow::anyhow!("Stack underflow"))?;
        let a = self.stack.pop().ok_or_else(|| anyhow::anyhow!("Stack underflow"))?;
        self.stack.push(Value::Bool(a == b));
        Ok(())
    }
    
    fn execute_neq(&mut self) -> Result<()> {
        let b = self.stack.pop().ok_or_else(|| anyhow::anyhow!("Stack underflow"))?;
        let a = self.stack.pop().ok_or_else(|| anyhow::anyhow!("Stack underflow"))?;
        self.stack.push(Value::Bool(a != b));
        Ok(())
    }
    
    fn execute_lt(&mut self) -> Result<()> {
        let b = self.stack.pop().ok_or_else(|| anyhow::anyhow!("Stack underflow"))?;
        let a = self.stack.pop().ok_or_else(|| anyhow::anyhow!("Stack underflow"))?;
        
        match (&a, &b) {
            (Value::Number(a_num), Value::Number(b_num)) => {
                self.stack.push(Value::Bool(a_num.as_f64().unwrap() < b_num.as_f64().unwrap()));
            }
            _ => return Err(anyhow::anyhow!("Type error: cannot compare {:?} and {:?}", a, b)),
        }
        Ok(())
    }
    
    fn execute_gt(&mut self) -> Result<()> {
        let b = self.stack.pop().ok_or_else(|| anyhow::anyhow!("Stack underflow"))?;
        let a = self.stack.pop().ok_or_else(|| anyhow::anyhow!("Stack underflow"))?;
        
        match (&a, &b) {
            (Value::Number(a_num), Value::Number(b_num)) => {
                self.stack.push(Value::Bool(a_num.as_f64().unwrap() > b_num.as_f64().unwrap()));
            }
            _ => return Err(anyhow::anyhow!("Type error: cannot compare {:?} and {:?}", a, b)),
        }
        Ok(())
    }
    
    fn execute_lte(&mut self) -> Result<()> {
        let b = self.stack.pop().ok_or_else(|| anyhow::anyhow!("Stack underflow"))?;
        let a = self.stack.pop().ok_or_else(|| anyhow::anyhow!("Stack underflow"))?;
        
        match (&a, &b) {
            (Value::Number(a_num), Value::Number(b_num)) => {
                self.stack.push(Value::Bool(a_num.as_f64().unwrap() <= b_num.as_f64().unwrap()));
            }
            _ => return Err(anyhow::anyhow!("Type error: cannot compare {:?} and {:?}", a, b)),
        }
        Ok(())
    }
    
    fn execute_gte(&mut self) -> Result<()> {
        let b = self.stack.pop().ok_or_else(|| anyhow::anyhow!("Stack underflow"))?;
        let a = self.stack.pop().ok_or_else(|| anyhow::anyhow!("Stack underflow"))?;
        
        match (&a, &b) {
            (Value::Number(a_num), Value::Number(b_num)) => {
                self.stack.push(Value::Bool(a_num.as_f64().unwrap() >= b_num.as_f64().unwrap()));
            }
            _ => return Err(anyhow::anyhow!("Type error: cannot compare {:?} and {:?}", a, b)),
        }
        Ok(())
    }
    
    fn execute_pop(&mut self) -> Result<()> {
        self.stack.pop().ok_or_else(|| anyhow::anyhow!("Stack underflow"))?;
        Ok(())
    }
    
    fn execute_dup(&mut self) -> Result<()> {
        if let Some(value) = self.stack.last() {
            self.stack.push(value.clone());
        } else {
            return Err(anyhow::anyhow!("Stack underflow: cannot duplicate"));
        }
        Ok(())
    }
    
    fn execute_jump(&mut self, address: usize) -> Result<()> {
        self.program_counter = address;
        Ok(())
    }
    
    fn execute_jump_if_false(&mut self, address: usize) -> Result<bool> {
        if let Some(condition) = self.stack.pop() {
            match condition {
                Value::Bool(false) | Value::Null => {
                    self.program_counter = address;
                    Ok(true) // Jump occurred
                }
                _ => Ok(false) // No jump
            }
        } else {
            Err(anyhow::anyhow!("Stack underflow: no condition for jump"))
        }
    }
    
    fn execute_jump_if_true(&mut self, address: usize) -> Result<bool> {
        if let Some(condition) = self.stack.pop() {
            match condition {
                Value::Bool(true) => {
                    self.program_counter = address;
                    Ok(true) // Jump occurred
                }
                Value::Number(n) if n.as_f64().unwrap() != 0.0 => {
                    self.program_counter = address;
                    Ok(true) // Jump occurred
                }
                Value::String(s) if !s.is_empty() => {
                    self.program_counter = address;
                    Ok(true) // Jump occurred
                }
                _ => Ok(false) // No jump
            }
        } else {
            Err(anyhow::anyhow!("Stack underflow: no condition for jump"))
        }
    }
    
    fn execute_create_array(&mut self, size: usize) -> Result<()> {
        let mut array = Vec::with_capacity(size);
        for _ in 0..size {
            if let Some(element) = self.stack.pop() {
                array.push(element);
            } else {
                return Err(anyhow::anyhow!("Stack underflow: not enough elements for array"));
            }
        }
        array.reverse(); // Reverse because we popped in reverse order
        self.stack.push(Value::Array(array));
        Ok(())
    }
    
    fn execute_to_string(&mut self) -> Result<()> {
        if let Some(value) = self.stack.pop() {
            let string_value = match value {
                Value::String(s) => s,
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => "null".to_string(),
                Value::Array(arr) => format!("{:?}", arr),
                Value::Object(obj) => format!("{:?}", obj),
            };
            self.stack.push(Value::String(string_value));
        } else {
            return Err(anyhow::anyhow!("Stack underflow: no value to convert to string"));
        }
        Ok(())
    }
    
    fn execute_concat(&mut self) -> Result<()> {
        let b = self.stack.pop().ok_or_else(|| anyhow::anyhow!("Stack underflow"))?;
        let a = self.stack.pop().ok_or_else(|| anyhow::anyhow!("Stack underflow"))?;
        
        // Convert both to strings and concatenate
        let a_str = match a {
            Value::String(s) => s,
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            _ => format!("{:?}", a),
        };
        
        let b_str = match b {
            Value::String(s) => s,
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            _ => format!("{:?}", b),
        };
        
        self.stack.push(Value::String(format!("{}{}", a_str, b_str)));
        Ok(())
    }
}

pub async fn run_wasm(file: &Path, function: &str, args: &[String]) -> Result<()> {
    println!("{} Running WASM module: {}", 
        style("[INFO]").cyan(), 
        file.display()
    );
    
    let wasm_bytes = std::fs::read(file)
        .map_err(|e| anyhow::anyhow!("Failed to read WASM file: {}", e))?;
    
    let mut vm = DroeVM::new()?;
    let module = vm.load_module(&wasm_bytes)?;
    let instance = vm.instantiate(&module)?;
    
    println!("{} Calling function: {}", style("[INFO]").yellow(), function);
    
    // Convert string args to WASM values if needed
    let wasm_args = args.iter().map(|_| wasmtime::Val::I32(0)).collect::<Vec<_>>();
    
    let _result = vm.call_function(&instance, function, &wasm_args)?;
    
    println!("{} Execution completed!", style("[SUCCESS]").green());
    Ok(())
}

pub async fn validate_wasm(file: &Path) -> Result<()> {
    println!("{} Validating WASM module: {}", 
        style("[INFO]").cyan(), 
        file.display()
    );
    
    let wasm_bytes = std::fs::read(file)
        .map_err(|e| anyhow::anyhow!("Failed to read WASM file: {}", e))?;
    
    let engine = Engine::default();
    let _module = Module::new(&engine, &wasm_bytes)
        .map_err(|e| anyhow::anyhow!("Invalid WASM module: {}", e))?;
    
    println!("{} WASM module is valid!", style("[SUCCESS]").green());
    Ok(())
}

pub async fn show_info(file: &Path) -> Result<()> {
    println!("{} Analyzing WASM module: {}", 
        style("[INFO]").cyan(), 
        file.display()
    );
    
    let wasm_bytes = std::fs::read(file)
        .map_err(|e| anyhow::anyhow!("Failed to read WASM file: {}", e))?;
    
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_bytes)
        .map_err(|e| anyhow::anyhow!("Invalid WASM module: {}", e))?;
    
    println!("\n{}", style("=== WASM Module Information ===").bold());
    println!("File: {}", file.display());
    println!("Size: {} bytes", wasm_bytes.len());
    
    // Get exports
    println!("\n{}", style("Exports:").bold());
    for export in module.exports() {
        println!("  - {} ({:?})", export.name(), export.ty());
    }
    
    // Get imports
    println!("\n{}", style("Imports:").bold());
    for import in module.imports() {
        println!("  - {}.{} ({:?})", import.module(), import.name(), import.ty());
    }
    
    Ok(())
}

pub async fn run_bytecode(file: &Path, _function: &str, _args: &[String]) -> Result<()> {
    println!("{} Running bytecode module: {}", 
        style("[INFO]").cyan(), 
        file.display()
    );
    
    let bytecode_content = std::fs::read_to_string(file)
        .map_err(|e| anyhow::anyhow!("Failed to read bytecode file: {}", e))?;
    
    let bytecode: serde_json::Value = serde_json::from_str(&bytecode_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse bytecode JSON: {}", e))?;
    
    let instructions = bytecode.get("instructions")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("Invalid bytecode format: missing instructions array"))?;
    
    let mut vm = DroeVirtualMachine::new();
    vm.execute(instructions)?;
    
    println!("{} Execution completed!", style("[SUCCESS]").green());
    Ok(())
}