//! AST node definitions for Droe DSL compiler.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Base trait for all AST nodes
pub trait ASTNode {
    fn line_number(&self) -> Option<usize>;
    fn set_line_number(&mut self, line: usize);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Node {
    // Literals and identifiers
    Literal(Literal),
    Identifier(Identifier),
    
    // Expressions
    BinaryOp(BinaryOp),
    PropertyAccess(PropertyAccess),
    ArrayLiteral(ArrayLiteral),
    ArithmeticOp(ArithmeticOp),
    StringInterpolation(StringInterpolation),
    FormatExpression(FormatExpression),
    
    // Statements
    DisplayStatement(DisplayStatement),
    Assignment(Assignment),
    IfStatement(IfStatement),
    WhileLoop(WhileLoop),
    ForEachLoop(ForEachLoop),
    ReturnStatement(ReturnStatement),
    
    // Definitions
    ModuleDefinition(ModuleDefinition),
    DataDefinition(DataDefinition),
    ActionDefinition(ActionDefinition),
    ActionDefinitionWithParams(ActionDefinitionWithParams),
    TaskAction(TaskAction),
    FragmentDefinition(FragmentDefinition),
    ScreenDefinition(ScreenDefinition),
    FormDefinition(FormDefinition),
    
    // Components
    TitleComponent(TitleComponent),
    TextComponent(TextComponent),
    InputComponent(InputComponent),
    TextareaComponent(TextareaComponent),
    DropdownComponent(DropdownComponent),
    ToggleComponent(ToggleComponent),
    CheckboxComponent(CheckboxComponent),
    RadioComponent(RadioComponent),
    ButtonComponent(ButtonComponent),
    ImageComponent(ImageComponent),
    VideoComponent(VideoComponent),
    AudioComponent(AudioComponent),
    SlotComponent(SlotComponent),
    
    // API and Database
    ApiCallStatement(ApiCallStatement),
    DatabaseStatement(DatabaseStatement),
    ServeStatement(ServeStatement),
    
    // Includes and Metadata
    IncludeStatement(IncludeStatement),
    AssetInclude(AssetInclude),
    MetadataAnnotation(MetadataAnnotation),
    
    // Other
    TaskInvocation(TaskInvocation),
    ActionInvocation(ActionInvocation),
    ActionInvocationWithArgs(ActionInvocationWithArgs),
    DataInstance(DataInstance),
    FieldAssignment(FieldAssignment),
    AcceptStatement(AcceptStatement),
    RespondStatement(RespondStatement),
    ParamsStatement(ParamsStatement),
    Program(Program),
}

impl ASTNode for Node {
    fn line_number(&self) -> Option<usize> {
        match self {
            Node::Literal(n) => n.line_number,
            Node::Identifier(n) => n.line_number,
            Node::BinaryOp(n) => n.line_number,
            // Add for all variants...
            _ => None, // TODO: implement for all variants
        }
    }
    
    fn set_line_number(&mut self, line: usize) {
        match self {
            Node::Literal(n) => n.line_number = Some(line),
            Node::Identifier(n) => n.line_number = Some(line),
            Node::BinaryOp(n) => n.line_number = Some(line),
            // Add for all variants...
            _ => {} // TODO: implement for all variants
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Literal {
    pub value: LiteralValue,
    pub literal_type: String,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LiteralValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identifier {
    pub name: String,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryOp {
    pub left: Box<Node>,
    pub operator: String,
    pub right: Box<Node>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayStatement {
    pub expression: Box<Node>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfStatement {
    pub condition: Box<Node>,
    pub then_body: Vec<Node>,
    pub else_body: Option<Vec<Node>>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyAccess {
    pub object: Box<Node>,
    pub property: String,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    pub variable: String,
    pub value: Box<Node>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrayLiteral {
    pub elements: Vec<Node>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhileLoop {
    pub condition: Box<Node>,
    pub body: Vec<Node>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForEachLoop {
    pub variable: String,
    pub iterable: Box<Node>,
    pub body: Vec<Node>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArithmeticOp {
    pub left: Box<Node>,
    pub operator: String,
    pub right: Box<Node>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAction {
    pub name: String,
    pub parameters: Vec<ActionParameter>,
    pub body: Vec<Node>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInvocation {
    pub task_name: String,
    pub arguments: Vec<Node>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDefinition {
    pub name: String,
    pub body: Vec<Node>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnStatement {
    pub expression: Box<Node>,
    pub return_type: String,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionInvocation {
    pub action_name: String,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDefinition {
    pub name: String,
    pub body: Vec<Node>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataDefinition {
    pub name: String,
    pub fields: Vec<DataField>,
    pub storage_type: Option<String>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataField {
    pub name: String,
    pub field_type: String,
    pub annotations: Vec<String>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDefinitionWithParams {
    pub name: String,
    pub parameters: Vec<ActionParameter>,
    pub return_type: Option<String>,
    pub body: Vec<Node>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionParameter {
    pub name: String,
    pub param_type: String,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionInvocationWithArgs {
    pub module_name: Option<String>,
    pub action_name: String,
    pub arguments: Vec<Node>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringInterpolation {
    pub parts: Vec<Node>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataInstance {
    pub data_type: String,
    pub field_values: Vec<FieldAssignment>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldAssignment {
    pub field_name: String,
    pub value: Box<Node>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncludeStatement {
    pub module_name: String,
    pub file_path: String,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetInclude {
    pub asset_path: String,
    pub asset_type: String,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatExpression {
    pub expression: Box<Node>,
    pub format_pattern: String,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataAnnotation {
    pub key: String,
    pub value: MetadataValue,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetadataValue {
    String(String),
    Dict(HashMap<String, String>),
}

// UI Components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentDefinition {
    pub name: String,
    pub slots: Vec<SlotComponent>,
    pub attributes: Vec<AttributeDefinition>,
    pub classes: Vec<String>,
    pub styles: Option<String>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenDefinition {
    pub name: String,
    pub fragments: Vec<FragmentReference>,
    pub attributes: Vec<AttributeDefinition>,
    pub classes: Vec<String>,
    pub styles: Option<String>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentReference {
    pub fragment_name: String,
    pub slot_contents: HashMap<String, Vec<Node>>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotComponent {
    pub name: String,
    pub default_content: Vec<Node>,
    pub attributes: Vec<AttributeDefinition>,
    pub classes: Vec<String>,
    pub styles: Option<String>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormDefinition {
    pub name: String,
    pub children: Vec<Node>,
    pub attributes: Vec<AttributeDefinition>,
    pub classes: Vec<String>,
    pub styles: Option<String>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleComponent {
    pub text: String,
    pub attributes: Vec<AttributeDefinition>,
    pub classes: Vec<String>,
    pub styles: Option<String>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextComponent {
    pub text: String,
    pub attributes: Vec<AttributeDefinition>,
    pub classes: Vec<String>,
    pub styles: Option<String>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputComponent {
    pub input_type: String,
    pub binding: Option<String>,
    pub attributes: Vec<AttributeDefinition>,
    pub element_id: Option<String>,
    pub classes: Vec<String>,
    pub styles: Option<String>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextareaComponent {
    pub label: Option<String>,
    pub placeholder: Option<String>,
    pub rows: Option<i32>,
    pub binding: Option<String>,
    pub attributes: Vec<AttributeDefinition>,
    pub classes: Vec<String>,
    pub styles: Option<String>,
    pub element_id: Option<String>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropdownComponent {
    pub label: Option<String>,
    pub options: Vec<Node>,
    pub binding: Option<String>,
    pub attributes: Vec<AttributeDefinition>,
    pub element_id: Option<String>,
    pub classes: Vec<String>,
    pub styles: Option<String>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleComponent {
    pub binding: Option<String>,
    pub attributes: Vec<AttributeDefinition>,
    pub classes: Vec<String>,
    pub styles: Option<String>,
    pub element_id: Option<String>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckboxComponent {
    pub text: Option<String>,
    pub binding: Option<String>,
    pub attributes: Vec<AttributeDefinition>,
    pub element_id: Option<String>,
    pub classes: Vec<String>,
    pub styles: Option<String>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadioComponent {
    pub text: Option<String>,
    pub value: Option<String>,
    pub binding: Option<String>,
    pub attributes: Vec<AttributeDefinition>,
    pub element_id: Option<String>,
    pub classes: Vec<String>,
    pub styles: Option<String>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonComponent {
    pub text: String,
    pub action: Option<String>,
    pub attributes: Vec<AttributeDefinition>,
    pub classes: Vec<String>,
    pub styles: Option<String>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageComponent {
    pub src: String,
    pub alt: Option<String>,
    pub attributes: Vec<AttributeDefinition>,
    pub classes: Vec<String>,
    pub styles: Option<String>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoComponent {
    pub src: String,
    pub controls: bool,
    pub autoplay: bool,
    pub loop_video: bool,
    pub muted: bool,
    pub attributes: Vec<AttributeDefinition>,
    pub classes: Vec<String>,
    pub styles: Option<String>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioComponent {
    pub src: String,
    pub controls: bool,
    pub autoplay: bool,
    pub loop_audio: bool,
    pub attributes: Vec<AttributeDefinition>,
    pub classes: Vec<String>,
    pub styles: Option<String>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeDefinition {
    pub name: String,
    pub value: Option<Box<Node>>,
    pub line_number: Option<usize>,
}

// API and Database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCallStatement {
    pub verb: String,
    pub endpoint: String,
    pub method: String,
    pub payload: Option<String>,
    pub headers: Vec<ApiHeader>,
    pub response_variable: Option<String>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiHeader {
    pub name: String,
    pub value: String,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServeStatement {
    pub method: String,
    pub endpoint: String,
    pub body: Vec<Node>,
    pub params: Vec<String>,
    pub accept_type: Option<String>,
    pub response_action: Option<Box<Node>>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptStatement {
    pub module_name: String,
    pub action_name: String,
    pub param_name: Option<String>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RespondStatement {
    pub module_name: String,
    pub action_name: String,
    pub param_name: Option<String>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamsStatement {
    pub param_name: String,
    pub param_type: String,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStatement {
    pub operation: String,
    pub entity_name: String,
    pub conditions: Vec<Node>,
    pub fields: Vec<Node>,
    pub return_var: Option<String>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub statements: Vec<Node>,
    pub metadata: Vec<MetadataAnnotation>,
    pub included_modules: Option<Vec<IncludeStatement>>,
    pub line_number: Option<usize>,
}

// Error types
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[error("{message} at line {line}, column {column}")]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

pub type ParseResult<T> = Result<T, ParseError>;