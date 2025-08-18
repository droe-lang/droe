//! Puck editor JSON format code generator for Droelang DSL.

use crate::ast::*;
use crate::codegen::CodeGenerator;
use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;

pub mod reverse;
pub use reverse::{PuckToDSLConverter, convert_puck_to_dsl};

pub struct PuckCodeGenerator {
    component_counter: usize,
    data_models: HashMap<String, DataDefinition>,
    actions: HashMap<String, ActionDefinition>,
    forms: Vec<FormDefinition>,
}

impl PuckCodeGenerator {
    pub fn new() -> Self {
        Self {
            component_counter: 0,
            data_models: HashMap::new(),
            actions: HashMap::new(),
            forms: Vec::new(),
        }
    }

    pub fn clear_state(&mut self) {
        self.component_counter = 0;
        self.data_models.clear();
        self.actions.clear();
        self.forms.clear();
    }

    fn collect_definitions(&mut self, program: &Program) {
        for stmt in &program.statements {
            match stmt {
                Node::ModuleDefinition(module) => {
                    for module_stmt in &module.body {
                        self.process_statement(module_stmt);
                    }
                }
                _ => self.process_statement(stmt),
            }
        }
    }

    fn process_statement(&mut self, stmt: &Node) {
        match stmt {
            Node::DataDefinition(data_def) => {
                self.data_models.insert(data_def.name.clone(), data_def.clone());
            }
            Node::ActionDefinition(action_def) => {
                self.actions.insert(action_def.name.clone(), action_def.clone());
            }
            Node::FormDefinition(form_def) => {
                self.forms.push(form_def.clone());
            }
            // Note: LayoutDefinition is not defined in the current AST
            // We'll handle this when the AST is updated
            _ => {}
        }
    }

    fn generate_puck_data(&mut self) -> Value {
        let mut content = Vec::new();
        let root_props = json!({"title": "Droelang Page"});

        // Convert forms to Puck components
        for form in &self.forms.clone() {
            let form_components = self.convert_form(form);
            content.extend(form_components);
        }

        // If no layouts or forms, create default content from other components
        if content.is_empty() && (!self.data_models.is_empty() || !self.actions.is_empty()) {
            content = self.create_default_layout();
        }

        json!({
            "root": {"props": root_props},
            "content": content,
            "zones": {}
        })
    }

    fn convert_form(&mut self, form: &FormDefinition) -> Vec<Value> {
        let mut components = Vec::new();

        // Create a Container component for the form
        let mut container_props = json!({
            "padding": 16,
            "background": "white",
            "border": "light",
            "items": []
        });

        // Add form title if exists
        if !form.name.is_empty() {
            let title = json!({
                "type": "Heading",
                "props": {
                    "id": self.generate_id("Heading"),
                    "text": form.name,
                    "level": 2,
                    "align": "left"
                }
            });
            container_props["items"].as_array_mut().unwrap().push(title);
        }

        // Process form children (instead of fields)
        for child in &form.children {
            if let Some(field_component) = self.convert_form_field(child) {
                container_props["items"].as_array_mut().unwrap().push(field_component);
            }
        }

        // Add submit button (forms don't have action field in current AST)
        let submit_button = json!({
            "type": "Button",
            "props": {
                "id": self.generate_id("Button"),
                "text": "Submit",
                "variant": "default",
                "size": "default",
                "fullWidth": "false"
            }
        });
        container_props["items"].as_array_mut().unwrap().push(submit_button);

        let container = json!({
            "type": "Container",
            "id": self.generate_id("Container"),
            "props": container_props
        });

        components.push(container);
        components
    }

    fn convert_component(&mut self, component: &Node) -> Option<Value> {
        match component {
            Node::TitleComponent(title_comp) => {
                let mut props = json!({
                    "text": title_comp.text,
                    "level": 1, // TitleComponent doesn't have level field in current AST
                    "align": "left",
                    "id": self.generate_id("Heading")
                });

                // Add CSS classes to props for round-trip preservation
                if !title_comp.classes.is_empty() {
                    props["cssClasses"] = json!(title_comp.classes);
                    props["className"] = json!(title_comp.classes.join(" "));
                }

                Some(json!({
                    "type": "Heading",
                    "props": props
                }))
            }
            Node::InputComponent(input_comp) => {
                // Extract label from attributes if present
                let label = input_comp.attributes.iter()
                    .find(|attr| attr.name == "label")
                    .and_then(|attr| attr.value.as_ref())
                    .map(|value| self.extract_text_value(value))
                    .unwrap_or_else(|| "Input".to_string());

                let placeholder = input_comp.attributes.iter()
                    .find(|attr| attr.name == "placeholder")
                    .and_then(|attr| attr.value.as_ref())
                    .map(|value| self.extract_text_value(value))
                    .unwrap_or_else(|| "".to_string());

                let required = input_comp.attributes.iter()
                    .any(|attr| attr.name == "required");

                Some(json!({
                    "type": "TextInput",
                    "props": {
                        "label": label,
                        "placeholder": placeholder,
                        "required": if required { "true" } else { "false" },
                        "fullWidth": "true",
                        "id": self.generate_id("TextInput")
                    }
                }))
            }
            Node::TextareaComponent(textarea_comp) => {
                Some(json!({
                    "type": "Textarea",
                    "props": {
                        "id": self.generate_id("Textarea"),
                        "label": textarea_comp.label.as_ref().unwrap_or(&"Message".to_string()),
                        "placeholder": textarea_comp.placeholder.as_ref().unwrap_or(&"".to_string()),
                        "rows": textarea_comp.rows.unwrap_or(4),
                        "required": "false"
                    }
                }))
            }
            Node::ButtonComponent(button_comp) => {
                Some(json!({
                    "type": "Button",
                    "props": {
                        "id": self.generate_id("Button"),
                        "text": button_comp.text,
                        "variant": "default",
                        "size": "default",
                        "fullWidth": "false"
                    }
                }))
            }
            Node::ImageComponent(image_comp) => {
                Some(json!({
                    "type": "Image",
                    "props": {
                        "id": self.generate_id("Image"),
                        "src": image_comp.src,
                        "alt": image_comp.alt.as_ref().unwrap_or(&"Image".to_string()),
                        "width": "auto",
                        "rounded": "md"
                    }
                }))
            }
            Node::DropdownComponent(dropdown_comp) => {
                let options: Vec<Value> = dropdown_comp.options
                    .iter()
                    .map(|opt| {
                        let opt_text = self.extract_text_value(opt);
                        json!({"label": opt_text, "value": opt_text})
                    })
                    .collect();

                Some(json!({
                    "type": "Select",
                    "props": {
                        "id": self.generate_id("Select"),
                        "label": dropdown_comp.label.as_ref().unwrap_or(&"Select".to_string()),
                        "options": options,
                        "required": "false"
                    }
                }))
            }
            Node::CheckboxComponent(checkbox_comp) => {
                Some(json!({
                    "type": "Checkbox",
                    "props": {
                        "id": self.generate_id("Checkbox"),
                        "label": checkbox_comp.text.as_ref().unwrap_or(&"Options".to_string()),
                        "name": "checkbox-group",
                        "options": [{"label": "Option 1", "value": "option1"}],
                        "required": "false"
                    }
                }))
            }
            Node::RadioComponent(radio_comp) => {
                Some(json!({
                    "type": "Radio",
                    "props": {
                        "id": self.generate_id("Radio"),
                        "label": radio_comp.text.as_ref().unwrap_or(&"Choose".to_string()),
                        "name": "radio-group",
                        "options": [{"label": "Option 1", "value": "option1"}],
                        "required": "false"
                    }
                }))
            }
            Node::DisplayStatement(display_stmt) => {
                let text_value = self.extract_text_value(&display_stmt.expression);
                Some(json!({
                    "type": "Text",
                    "props": {
                        "id": self.generate_id("Text"),
                        "text": text_value,
                        "size": "base",
                        "align": "left"
                    }
                }))
            }
            _ => None,
        }
    }

    fn convert_form_field(&mut self, field: &Node) -> Option<Value> {
        // Reuse component conversion logic
        self.convert_component(field)
    }

    fn create_default_layout(&mut self) -> Vec<Value> {
        let mut section_items = Vec::new();

        // Add a heading
        let heading = json!({
            "type": "Heading",
            "id": self.generate_id("Heading"),
            "props": {
                "text": "Droelang Application",
                "level": 1,
                "align": "center"
            }
        });
        section_items.push(heading);

        // Add data model info if available
        if !self.data_models.is_empty() {
            let data_models_text = self.data_models.keys().cloned().collect::<Vec<_>>().join(", ");
            let text = json!({
                "type": "Text",
                "props": {
                    "id": self.generate_id("Text"),
                    "text": format!("Data Models: {}", data_models_text),
                    "size": "base",
                    "align": "left"
                }
            });
            section_items.push(text);
        }

        // Add action info if available
        if !self.actions.is_empty() {
            let actions_text = self.actions.keys().cloned().collect::<Vec<_>>().join(", ");
            let text = json!({
                "type": "Text",
                "props": {
                    "id": self.generate_id("Text"),
                    "text": format!("Actions: {}", actions_text),
                    "size": "base",
                    "align": "left"
                }
            });
            section_items.push(text);
        }

        let section = json!({
            "type": "Section",
            "id": self.generate_id("Section"),
            "props": {
                "padding": 32,
                "background": "transparent",
                "items": section_items
            }
        });

        vec![section]
    }

    fn extract_text_value(&self, node: &Node) -> String {
        match node {
            Node::Literal(literal) => match &literal.value {
                crate::ast::LiteralValue::String(s) => s.clone(),
                crate::ast::LiteralValue::Integer(i) => i.to_string(),
                crate::ast::LiteralValue::Float(f) => f.to_string(),
                crate::ast::LiteralValue::Boolean(b) => b.to_string(),
            },
            Node::StringInterpolation(_) => "{dynamic text}".to_string(),
            Node::Identifier(identifier) => format!("{{{{{}}}}}", identifier.name),
            _ => format!("{:?}", node),
        }
    }

    fn generate_id(&mut self, component_type: &str) -> String {
        format!("{}-{}", component_type, Uuid::new_v4())
    }
}

impl Default for PuckCodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator for PuckCodeGenerator {
    fn generate(&self, program: &Program) -> Result<String, String> {
        let mut generator = Self::new();
        generator.clear_state();

        // First pass: collect definitions
        generator.collect_definitions(program);

        // Generate Puck data structure
        let puck_data = generator.generate_puck_data();

        // Return as JSON string
        serde_json::to_string_pretty(&puck_data)
            .map_err(|e| format!("Failed to serialize Puck JSON: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_puck_generator_creation() {
        let generator = PuckCodeGenerator::new();
        assert_eq!(generator.component_counter, 0);
        assert!(generator.data_models.is_empty());
        assert!(generator.actions.is_empty());
        assert!(generator.forms.is_empty());
    }

    #[test]
    fn test_generate_id() {
        let mut generator = PuckCodeGenerator::new();
        let id1 = generator.generate_id("Button");
        let id2 = generator.generate_id("Button");
        assert!(id1.starts_with("Button-"));
        assert!(id2.starts_with("Button-"));
        assert_ne!(id1, id2);
    }
}