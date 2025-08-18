//! Java code generator for Droe DSL

use crate::ast::{
    Node, Program, LiteralValue, BinaryOp, DisplayStatement,
    IfStatement, Assignment, ArrayLiteral, WhileLoop, ForEachLoop, ArithmeticOp,
    ActionDefinition, ActionDefinitionWithParams, ActionInvocation,
    ActionInvocationWithArgs, ModuleDefinition,
    StringInterpolation, FormatExpression,
};
use crate::codegen::CodeGenerator;
use std::collections::HashSet;

pub struct JavaCodeGenerator {
    source_file_path: Option<String>,
    is_main_file: bool,
    framework: String,
    package: Option<String>,
    class_name: String,
    imports: HashSet<String>,
    fields: Vec<String>,
    constructor_code: Vec<String>,
    methods: Vec<String>,
    module_classes: Vec<Vec<String>>,
    has_modules: bool,
}

impl Default for JavaCodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl JavaCodeGenerator {
    pub fn new() -> Self {
        Self::with_options(None, false, "plain", None)
    }
    
    pub fn with_options(
        source_file_path: Option<String>,
        is_main_file: bool,
        framework: &str,
        package: Option<String>,
    ) -> Self {
        let class_name = if let Some(ref path) = source_file_path {
            Self::path_to_class_name(path)
        } else {
            "RoelangProgram".to_string()
        };
        
        let mut imports = HashSet::new();
        imports.insert("java.util.*".to_string());
        imports.insert("java.time.*".to_string());
        imports.insert("java.time.format.*".to_string());
        imports.insert("java.text.*".to_string());
        
        Self {
            source_file_path,
            is_main_file,
            framework: framework.to_string(),
            package,
            class_name,
            imports,
            fields: Vec::new(),
            constructor_code: Vec::new(),
            methods: Vec::new(),
            module_classes: Vec::new(),
            has_modules: false,
        }
    }
    
    fn path_to_class_name(path: &str) -> String {
        let file_name = std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("RoelangProgram");
        
        Self::to_pascal_case(file_name)
    }
    
    fn to_pascal_case(name: &str) -> String {
        let binding = name.replace(['-', ' '], "_");
        let parts: Vec<&str> = binding.split('_').collect();
        let mut result = String::new();
        
        for part in parts {
            if !part.is_empty() {
                let mut chars = part.chars();
                if let Some(first) = chars.next() {
                    result.push(first.to_ascii_uppercase());
                    result.extend(chars);
                }
            }
        }
        
        // Ensure valid Java class name
        if result.is_empty() || !result.chars().next().unwrap().is_alphabetic() {
            "RoelangProgram".to_string()
        } else {
            result
        }
    }
    
    fn get_java_type(&mut self, type_name: &str) -> String {
        match type_name {
            "int" | "number" => "int".to_string(),
            "decimal" => "double".to_string(),
            "text" | "string" => "String".to_string(),
            "flag" | "yesno" | "boolean" => "boolean".to_string(),
            "date" => "String".to_string(),
            "list" | "array" => {
                self.imports.insert("java.util.*".to_string());
                "List<Object>".to_string()
            }
            _ if type_name.starts_with("list_of_") => {
                let element_type = &type_name[8..];
                self.imports.insert("java.util.*".to_string());
                format!("List<{}>", self.get_java_wrapper_type(element_type))
            }
            _ => "Object".to_string(),
        }
    }
    
    fn get_java_wrapper_type(&self, type_name: &str) -> String {
        match type_name {
            "int" | "number" => "Integer",
            "decimal" => "Double",
            "text" | "string" => "String",
            "flag" | "yesno" | "boolean" => "Boolean",
            "date" => "String",
            _ => "Object",
        }.to_string()
    }
    
    fn emit_expression(&mut self, expr: &Node) -> String {
        match expr {
            Node::Literal(lit) => self.emit_literal(&lit.value),
            Node::Identifier(id) => id.name.clone(),
            Node::BinaryOp(op) => self.emit_binary_op(op),
            Node::ArithmeticOp(op) => self.emit_arithmetic_op(op),
            Node::ArrayLiteral(arr) => self.emit_array_literal(arr),
            Node::StringInterpolation(interp) => self.emit_string_interpolation(interp),
            Node::FormatExpression(fmt) => self.emit_format_expression(fmt),
            Node::ActionInvocation(inv) => self.emit_action_invocation(inv),
            Node::ActionInvocationWithArgs(inv) => self.emit_action_invocation_with_args(inv),
            Node::PropertyAccess(access) => {
                let obj = self.emit_expression(&access.object);
                format!("{}.{}", obj, access.property)
            }
            _ => format!("/* TODO: {} */", self.node_type_name(expr)),
        }
    }
    
    fn emit_literal(&self, value: &LiteralValue) -> String {
        match value {
            LiteralValue::String(s) => {
                let escaped = s
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('\n', "\\n");
                format!("\"{}\"", escaped)
            }
            LiteralValue::Integer(n) => n.to_string(),
            LiteralValue::Float(f) => f.to_string(),
            LiteralValue::Boolean(b) => if *b { "true" } else { "false" }.to_string(),
        }
    }
    
    fn emit_binary_op(&mut self, op: &BinaryOp) -> String {
        let left = self.emit_expression(&op.left);
        let right = self.emit_expression(&op.right);
        
        match op.operator.as_str() {
            "==" => format!("Objects.equals({}, {})", left, right),
            "!=" => format!("!Objects.equals({}, {})", left, right),
            _ => format!("({} {} {})", left, op.operator, right),
        }
    }
    
    fn emit_arithmetic_op(&mut self, op: &ArithmeticOp) -> String {
        let left = self.emit_expression(&op.left);
        let right = self.emit_expression(&op.right);
        format!("({} {} {})", left, op.operator, right)
    }
    
    fn emit_array_literal(&mut self, arr: &ArrayLiteral) -> String {
        let elements: Vec<String> = arr.elements.iter()
            .map(|e| self.emit_expression(e))
            .collect();
        
        self.imports.insert("java.util.Arrays".to_string());
        format!("Arrays.asList({})", elements.join(", "))
    }
    
    fn emit_string_interpolation(&mut self, interp: &StringInterpolation) -> String {
        let mut format_parts = Vec::new();
        let mut args = Vec::new();
        
        for part in &interp.parts {
            match part {
                Node::Literal(lit) => {
                    if let LiteralValue::String(s) = &lit.value {
                        format_parts.push(s.clone());
                    }
                }
                expr => {
                    format_parts.push("%s".to_string());
                    args.push(self.emit_expression(expr));
                }
            }
        }
        
        let format_string = format_parts.join("");
        if args.is_empty() {
            format!("\"{}\"", format_string)
        } else {
            format!("String.format(\"{}\", {})", format_string, args.join(", "))
        }
    }
    
    fn emit_format_expression(&mut self, fmt: &FormatExpression) -> String {
        let expr_str = self.emit_expression(&fmt.expression);
        
        match fmt.format_pattern.as_str() {
            "MM/dd/yyyy" | "dd/MM/yyyy" | "MMM dd, yyyy" => {
                format!("LocalDate.parse({}).format(DateTimeFormatter.ofPattern(\"{}\"))",
                    expr_str, fmt.format_pattern)
            }
            "0.00" | "#,##0.00" | "$0.00" => {
                self.imports.insert("java.text.DecimalFormat".to_string());
                format!("new DecimalFormat(\"{}\").format({})", fmt.format_pattern, expr_str)
            }
            _ => expr_str,
        }
    }
    
    fn emit_action_invocation(&self, inv: &ActionInvocation) -> String {
        // TODO: Handle module invocations when ActionInvocation has module support
        format!("{}()", inv.action_name)
    }
    
    fn emit_action_invocation_with_args(&mut self, inv: &ActionInvocationWithArgs) -> String {
        let args: Vec<String> = inv.arguments.iter()
            .map(|arg| self.emit_expression(arg))
            .collect();
        
        // TODO: Handle module invocations when ActionInvocationWithArgs has module support
        format!("{}({})", inv.action_name, args.join(", "))
    }
    
    fn emit_statement(&mut self, stmt: &Node) {
        match stmt {
            Node::DisplayStatement(display) => self.emit_display_statement(display),
            Node::Assignment(assign) => self.emit_assignment(assign),
            Node::IfStatement(if_stmt) => self.emit_if_statement(if_stmt),
            Node::WhileLoop(while_loop) => self.emit_while_loop(while_loop),
            Node::ForEachLoop(foreach) => self.emit_foreach_loop(foreach),
            Node::ActionDefinition(action) => self.add_method_from_action(action),
            Node::ActionDefinitionWithParams(action) => self.add_method_from_parameterized_action(action),
            Node::ModuleDefinition(_) => {
                // Modules are processed separately
            }
            _ => {
                self.constructor_code.push(format!("// TODO: Implement {}", self.node_type_name(stmt)));
            }
        }
    }
    
    fn emit_display_statement(&mut self, stmt: &DisplayStatement) {
        let expr_str = self.emit_expression(&stmt.expression);
        self.constructor_code.push(format!("System.out.println({});", expr_str));
    }
    
    fn emit_assignment(&mut self, stmt: &Assignment) {
        let value_str = self.emit_expression(&stmt.value);
        
        // TODO: Implement proper type inference from AST
        let java_type = "Object".to_string();
        
        // Check if field exists
        let field_decl = format!("private {} {};", java_type, stmt.variable);
        if !self.fields.contains(&field_decl) {
            self.fields.push(field_decl);
        }
        
        self.constructor_code.push(format!("this.{} = {};", stmt.variable, value_str));
    }
    
    fn emit_if_statement(&mut self, stmt: &IfStatement) {
        let condition_str = self.emit_expression(&stmt.condition);
        self.constructor_code.push(format!("if ({}) {{", condition_str));
        
        for _then_stmt in &stmt.then_body {
            // Process then body
            self.constructor_code.push("    // Then body".to_string());
        }
        
        if let Some(ref else_body) = stmt.else_body {
            if !else_body.is_empty() {
                self.constructor_code.push("} else {".to_string());
                for _else_stmt in else_body {
                    // Process else body
                    self.constructor_code.push("    // Else body".to_string());
                }
            }
        }
        
        self.constructor_code.push("}".to_string());
    }
    
    fn emit_while_loop(&mut self, stmt: &WhileLoop) {
        let condition_str = self.emit_expression(&stmt.condition);
        self.constructor_code.push(format!("while ({}) {{", condition_str));
        self.constructor_code.push("    // Loop body".to_string());
        self.constructor_code.push("}".to_string());
    }
    
    fn emit_foreach_loop(&mut self, stmt: &ForEachLoop) {
        let collection_str = self.emit_expression(&stmt.iterable);
        self.constructor_code.push(format!(
            "for (Object {} : (List<Object>) {}) {{",
            stmt.variable, collection_str
        ));
        self.constructor_code.push("    // Loop body".to_string());
        self.constructor_code.push("}".to_string());
    }
    
    fn add_method_from_action(&mut self, action: &ActionDefinition) {
        let method_lines = self.generate_method_from_action(action, false);
        self.methods.extend(method_lines);
    }
    
    fn add_method_from_parameterized_action(&mut self, action: &ActionDefinitionWithParams) {
        let method_lines = self.generate_method_from_parameterized_action(action, false);
        self.methods.extend(method_lines);
    }
    
    fn generate_method_from_action(&mut self, action: &ActionDefinition, is_static: bool) -> Vec<String> {
        let mut lines = Vec::new();
        let static_modifier = if is_static { "static " } else { "" };
        
        // Determine return type
        let return_type = "Object"; // Simplified for now
        
        lines.push(format!("public {}{} {}() {{", static_modifier, return_type, action.name));
        
        if !action.body.is_empty() {
            for stmt in &action.body {
                if let Node::ReturnStatement(ret) = stmt {
                    let expr_str = self.emit_expression(&ret.expression);
                    lines.push(format!("    return {};", expr_str));
                } else {
                    lines.push("    // Method body statement".to_string());
                }
            }
        } else {
            lines.push("    return null;".to_string());
        }
        
        lines.push("}".to_string());
        lines
    }
    
    fn generate_method_from_parameterized_action(&mut self, action: &ActionDefinitionWithParams, is_static: bool) -> Vec<String> {
        let mut lines = Vec::new();
        let static_modifier = if is_static { "static " } else { "" };
        
        // Build parameter list
        let params: Vec<String> = action.parameters.iter()
            .map(|p| format!("{} {}", self.get_java_type(&p.param_type), p.name))
            .collect();
        
        let param_list = params.join(", ");
        
        // Determine return type
        let return_type = if let Some(ref ret_type) = action.return_type {
            self.get_java_type(ret_type)
        } else {
            "Object".to_string()
        };
        
        lines.push(format!("public {}{} {}({}) {{", static_modifier, return_type, action.name, param_list));
        
        if !action.body.is_empty() {
            for stmt in &action.body {
                if let Node::ReturnStatement(ret) = stmt {
                    let expr_str = self.emit_expression(&ret.expression);
                    lines.push(format!("    return {};", expr_str));
                } else {
                    lines.push("    // Method body statement".to_string());
                }
            }
        } else if return_type != "void" {
            lines.push("    return null;".to_string());
        }
        
        lines.push("}".to_string());
        lines
    }
    
    fn process_module(&mut self, module: &ModuleDefinition, is_main: bool) {
        if is_main {
            for stmt in &module.body {
                match stmt {
                    Node::ActionDefinition(action) => self.add_method_from_action(action),
                    Node::ActionDefinitionWithParams(action) => self.add_method_from_parameterized_action(action),
                    _ => self.emit_statement(stmt),
                }
            }
        } else {
            let module_class = self.generate_module_class(module);
            self.module_classes.push(module_class);
        }
    }
    
    fn generate_module_class(&mut self, module: &ModuleDefinition) -> Vec<String> {
        let mut lines = Vec::new();
        lines.push(format!("class {} {{", module.name));
        
        for stmt in &module.body {
            match stmt {
                Node::ActionDefinition(action) => {
                    let method_lines = self.generate_method_from_action(action, true);
                    for line in method_lines {
                        lines.push(format!("    {}", line));
                    }
                }
                Node::ActionDefinitionWithParams(action) => {
                    let method_lines = self.generate_method_from_parameterized_action(action, true);
                    for line in method_lines {
                        lines.push(format!("    {}", line));
                    }
                }
                _ => {}
            }
        }
        
        lines.push("}".to_string());
        lines
    }
    
    fn build_java_file(&self) -> String {
        let mut lines = Vec::new();
        
        // Add package declaration if specified
        if let Some(ref pkg) = self.package {
            lines.push(format!("package {};", pkg));
            lines.push(String::new());
        }
        
        // Add imports
        if !self.imports.is_empty() {
            let mut sorted_imports: Vec<_> = self.imports.iter().cloned().collect();
            sorted_imports.sort();
            for import in sorted_imports {
                lines.push(format!("import {};", import));
            }
            lines.push(String::new());
        }
        
        // Add module classes
        for module_class in &self.module_classes {
            lines.extend(module_class.clone());
            lines.push(String::new());
        }
        
        // Main class
        let class_modifier = if self.is_main_file { "public " } else { "" };
        lines.push(format!("{}class {} {{", class_modifier, self.class_name));
        
        // Add fields
        for field in &self.fields {
            lines.push(format!("    {}", field));
        }
        if !self.fields.is_empty() {
            lines.push(String::new());
        }
        
        // Add constructor
        if !self.constructor_code.is_empty() {
            lines.push(format!("    public {}() {{", self.class_name));
            for line in &self.constructor_code {
                lines.push(format!("        {}", line));
            }
            lines.push("    }".to_string());
            lines.push(String::new());
        }
        
        // Add methods
        for method_line in &self.methods {
            lines.push(format!("    {}", method_line));
        }
        if !self.methods.is_empty() {
            lines.push(String::new());
        }
        
        // Add main method
        if self.is_main_file || !self.has_modules {
            lines.push("    public static void main(String[] args) {".to_string());
            lines.push(format!("        new {}();", self.class_name));
            lines.push("    }".to_string());
        }
        
        lines.push("}".to_string());
        
        lines.join("\n")
    }
    
    fn node_type_name(&self, node: &Node) -> &str {
        match node {
            Node::Literal(_) => "Literal",
            Node::Identifier(_) => "Identifier",
            Node::BinaryOp(_) => "BinaryOp",
            Node::DisplayStatement(_) => "DisplayStatement",
            Node::IfStatement(_) => "IfStatement",
            Node::Assignment(_) => "Assignment",
            Node::ArrayLiteral(_) => "ArrayLiteral",
            Node::WhileLoop(_) => "WhileLoop",
            Node::ForEachLoop(_) => "ForEachLoop",
            Node::ArithmeticOp(_) => "ArithmeticOp",
            Node::TaskAction(_) => "TaskAction",
            Node::TaskInvocation(_) => "TaskInvocation",
            Node::ActionDefinition(_) => "ActionDefinition",
            Node::ReturnStatement(_) => "ReturnStatement",
            Node::ActionInvocation(_) => "ActionInvocation",
            Node::ModuleDefinition(_) => "ModuleDefinition",
            Node::DataDefinition(_) => "DataDefinition",
            // Node::DataField(_) => "DataField",
            Node::ActionDefinitionWithParams(_) => "ActionDefinitionWithParams",
            // Node::ActionParameter(_) => "ActionParameter",
            Node::ActionInvocationWithArgs(_) => "ActionInvocationWithArgs",
            Node::StringInterpolation(_) => "StringInterpolation",
            Node::DataInstance(_) => "DataInstance",
            Node::FieldAssignment(_) => "FieldAssignment",
            Node::PropertyAccess(_) => "PropertyAccess",
            Node::FormatExpression(_) => "FormatExpression",
            _ => "Unknown",
        }
    }
}

impl CodeGenerator for JavaCodeGenerator {
    fn generate(&self, program: &Program) -> Result<String, String> {
        let mut generator = JavaCodeGenerator::with_options(
            self.source_file_path.clone(),
            self.is_main_file,
            &self.framework,
            self.package.clone(),
        );
        
        // First pass: Check for modules
        let mut modules_found = Vec::new();
        let mut non_module_statements = Vec::new();
        
        for stmt in &program.statements {
            if let Node::ModuleDefinition(module) = stmt {
                modules_found.push(module);
                generator.has_modules = true;
            } else {
                non_module_statements.push(stmt);
            }
        }
        
        // Core Java generator - no framework-specific logic here
        // Framework routing is handled by CompilerFactory
        
        // Process modules
        for module in modules_found {
            generator.process_module(module, false);
        }
        
        // Process non-module statements
        for stmt in non_module_statements {
            generator.emit_statement(stmt);
        }
        
        Ok(generator.build_java_file())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pascal_case_conversion() {
        assert_eq!(JavaCodeGenerator::to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(JavaCodeGenerator::to_pascal_case("hello-world"), "HelloWorld");
        assert_eq!(JavaCodeGenerator::to_pascal_case("hello world"), "HelloWorld");
        assert_eq!(JavaCodeGenerator::to_pascal_case("helloWorld"), "HelloWorld");
    }
    
    #[test]
    fn test_java_type_mapping() {
        let mut gen = JavaCodeGenerator::new();
        assert_eq!(gen.get_java_type("int"), "int");
        assert_eq!(gen.get_java_type("text"), "String");
        assert_eq!(gen.get_java_type("boolean"), "boolean");
        assert_eq!(gen.get_java_type("list_of_int"), "List<Integer>");
    }
}