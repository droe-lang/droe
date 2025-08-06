use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Instruction {
    // Stack operations
    Push(Value),
    Pop,
    Dup,
    
    // Variables
    LoadVar(String),
    StoreVar(String),
    
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    
    // Comparison
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    
    // Control flow
    Jump(usize),
    JumpIfFalse(usize),
    JumpIfTrue(usize),
    
    // Functions/Actions
    Call(String, usize), // function name, arg count
    Return,
    
    // Tasks
    DefineTask(String, Vec<String>, usize), // name, params, body end
    RunTask(String, usize), // task name, arg count
    
    // Data structures
    CreateObject(String), // type name
    SetField(String),     // field name
    GetField(String),     // field name
    
    // Arrays
    CreateArray(usize),   // size
    GetIndex,
    SetIndex,
    
    // I/O
    Display,
    
    // Loops
    ForEach(String, usize), // variable name, loop end
    While(usize),           // loop end
    
    // Module system
    LoadModule(String),
    
    // Type operations
    TypeCheck(String),
    
    // Special
    Halt,
    Nop,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Array(Vec<Value>),
    Object(String, std::collections::HashMap<String, Value>), // type, fields
    Null,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BytecodeFile {
    pub version: u32,
    pub metadata: Metadata,
    pub constants: Vec<Value>,
    pub instructions: Vec<Instruction>,
    pub debug_info: Option<DebugInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub source_file: Option<String>,
    pub created_at: u64,
    pub compiler_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DebugInfo {
    pub line_numbers: Vec<(usize, usize)>, // instruction index -> line number
    pub source_map: Option<String>,
}

impl BytecodeFile {
    pub fn new(instructions: Vec<Instruction>) -> Self {
        Self {
            version: 1,
            metadata: Metadata {
                source_file: None,
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                compiler_version: "0.1.0".to_string(),
            },
            constants: Vec::new(),
            instructions,
            debug_info: None,
        }
    }
    
    pub fn serialize(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
    
    pub fn deserialize(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}