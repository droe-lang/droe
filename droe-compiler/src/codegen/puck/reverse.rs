//! Reverse code generator: Convert Puck JSON back to Droelang DSL.

use serde_json::Value;
use std::collections::HashMap;

pub struct PuckToDSLConverter {
    indent_level: usize,
    output_lines: Vec<String>,
    component_counter: usize,
}

impl PuckToDSLConverter {
    pub fn new() -> Self {
        Self {
            indent_level: 0,
            output_lines: Vec::new(),
            component_counter: 0,
        }
    }

    pub fn convert(&mut self, puck_json: &Value, original_metadata: Option<&HashMap<String, String>>) -> Result<String, String> {
        self.output_lines.clear();
        self.indent_level = 0;
        self.component_counter = 0;

        // Add original metadata (target directive, etc.)
        if let Some(metadata) = original_metadata {
            if let Some(target) = metadata.get("target") {
                self.output_lines.push(format!("@target {}", target));
                self.output_lines.push(String::new());
            }
        }

        // Convert root to get screen information
        let empty_root_map = serde_json::Map::new();
        let root = puck_json.get("root").and_then(|r| r.as_object()).unwrap_or(&empty_root_map);
        let empty_props_map = serde_json::Map::new();
        let root_props = root.get("props").and_then(|p| p.as_object()).unwrap_or(&empty_props_map);
        let title = root_props.get("title").and_then(|t| t.as_str()).unwrap_or("Screen");
        let layout_type = root_props.get("layout").and_then(|l| l.as_str()).unwrap_or("default");
        let description = root_props.get("description").and_then(|d| d.as_str()).unwrap_or("");

        // Create meaningful module name from title
        let module_name = self.to_pascal_case(title);
        let module_name = if module_name.is_empty() || !module_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            "Screen".to_string()
        } else {
            module_name
        };

        // Add module wrapper with meaningful name
        self.output_lines.push(format!("module {}", module_name));
        self.indent_level += 1;

        // Generate layout definition based on layout type
        self.generate_layout_definition(layout_type);

        // Generate screen that uses the layout
        let screen_name = format!("{}Screen", module_name);
        let layout_name = self.get_layout_name(layout_type);

        self.add_line(&format!("screen {} layout=\"{}\"", screen_name, layout_name));
        self.indent_level += 1;

        if !description.is_empty() {
            self.add_line(&format!("// {}", description));
        }

        // Convert content as screen content
        if let Some(content) = puck_json.get("content").and_then(|c| c.as_array()) {
            if !content.is_empty() {
                self.convert_content_items(content);
            } else {
                self.add_line("// Empty screen content");
            }
        } else {
            self.add_line("// Empty screen content");
        }

        // Close screen
        self.indent_level -= 1;
        self.add_line("end screen");

        // Close module
        self.indent_level -= 1;
        self.output_lines.push("end module".to_string());

        Ok(self.output_lines.join("\n"))
    }

    fn to_pascal_case(&self, text: &str) -> String {
        // Remove special characters and split on word boundaries
        let processed_text = text.replace('-', " ").replace('_', " ");
        let words: Vec<&str> = processed_text.split_whitespace().collect();
        // Capitalize each word and join
        words.iter()
            .filter(|word| word.chars().all(|c| c.is_alphanumeric()))
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                }
            })
            .collect::<String>()
    }

    fn generate_layout_definition(&mut self, layout_type: &str) {
        let layout_name = self.get_layout_name(layout_type);

        self.add_line(&format!("layout {}", layout_name));
        self.indent_level += 1;

        match layout_type {
            "header-footer" => {
                self.add_line("header");
                self.indent_level += 1;
                self.add_line("nav \"Main Navigation\"");
                self.indent_level -= 1;
                self.add_line("end header");
                self.add_line("");
                self.add_line("main");
                self.indent_level += 1;
                self.add_line("slot \"content\"");
                self.indent_level -= 1;
                self.add_line("end main");
                self.add_line("");
                self.add_line("footer");
                self.indent_level += 1;
                self.add_line("text \"© 2024 Company\"");
                self.indent_level -= 1;
                self.add_line("end footer");
            }
            "sidebar" => {
                self.add_line("div class=\"flex\"");
                self.indent_level += 1;
                self.add_line("aside class=\"w-64\"");
                self.indent_level += 1;
                self.add_line("nav \"Sidebar Navigation\"");
                self.indent_level -= 1;
                self.add_line("end aside");
                self.add_line("main class=\"flex-1\"");
                self.indent_level += 1;
                self.add_line("slot \"content\"");
                self.indent_level -= 1;
                self.add_line("end main");
                self.indent_level -= 1;
                self.add_line("end div");
            }
            "fullwidth" => {
                self.add_line("div class=\"w-full\"");
                self.indent_level += 1;
                self.add_line("slot \"content\"");
                self.indent_level -= 1;
                self.add_line("end div");
            }
            "landing" => {
                self.add_line("header class=\"hero\"");
                self.indent_level += 1;
                self.add_line("nav \"Top Navigation\"");
                self.indent_level -= 1;
                self.add_line("end header");
                self.add_line("main");
                self.indent_level += 1;
                self.add_line("slot \"content\"");
                self.indent_level -= 1;
                self.add_line("end main");
                self.add_line("footer");
                self.indent_level += 1;
                self.add_line("text \"Contact Info\"");
                self.indent_level -= 1;
                self.add_line("end footer");
            }
            _ => { // default layout
                self.add_line("main");
                self.indent_level += 1;
                self.add_line("slot \"content\"");
                self.indent_level -= 1;
                self.add_line("end main");
            }
        }

        self.indent_level -= 1;
        self.add_line("end layout");
        self.add_line("");
    }

    fn get_layout_name(&self, layout_type: &str) -> &str {
        match layout_type {
            "default" => "DefaultLayout",
            "header-footer" => "HeaderFooterLayout",
            "sidebar" => "SidebarLayout",
            "fullwidth" => "FullWidthLayout",
            "landing" => "LandingPageLayout",
            _ => "DefaultLayout",
        }
    }

    fn convert_content_items(&mut self, content: &[Value]) {
        for component in content {
            if let Some(component_type) = component.get("type").and_then(|t| t.as_str()) {
                match component_type {
                    "Column" => self.convert_column(component),
                    "Container" => {
                        // Check if this looks like a form
                        let empty_items = Vec::new();
                        let items = component.get("props")
                            .and_then(|p| p.get("items"))
                            .and_then(|i| i.as_array())
                            .unwrap_or(&empty_items);

                        let has_form_elements = items.iter().any(|item| {
                            if let Some(item_type) = item.get("type").and_then(|t| t.as_str()) {
                                matches!(item_type, "TextInput" | "Textarea" | "Select" | "Button" | "Checkbox" | "Radio")
                            } else {
                                false
                            }
                        });

                        if has_form_elements {
                            self.convert_form_container(component);
                        } else {
                            self.convert_container(component);
                        }
                    }
                    _ => {
                        // Handle direct components (Heading, Text, etc.)
                        self.convert_component_to_dsl(component);
                    }
                }
            }
        }
    }

    fn convert_column(&mut self, column: &Value) {
        let empty_map = serde_json::Map::new();
        let props = column.get("props").and_then(|p| p.as_object()).unwrap_or(&empty_map);

        // Extract CSS classes if present
        let class_attr = if let Some(css_classes) = props.get("classes").and_then(|c| c.as_str()) {
            if !css_classes.is_empty() {
                format!(" class \"{}\"", css_classes)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        self.add_line(&format!("column{}", class_attr));
        self.indent_level += 1;

        // Handle items array in props
        if let Some(items) = props.get("items").and_then(|i| i.as_array()) {
            for item in items {
                self.convert_component_to_dsl(item);
            }
        }

        self.indent_level -= 1;
        self.add_line("end column");
    }

    fn convert_form_container(&mut self, container: &Value) {
        let form_name = format!("Form{}", self.component_counter);
        self.component_counter += 1;

        self.add_line(&format!("form {}", form_name));
        self.indent_level += 1;

        if let Some(items) = container.get("props")
            .and_then(|p| p.get("items"))
            .and_then(|i| i.as_array()) {
            for child in items {
                self.convert_form_component(child);
            }
        }

        self.indent_level -= 1;
        self.add_line("end form");
    }

    fn convert_container(&mut self, container: &Value) {
        self.add_line("layout Container");
        self.indent_level += 1;

        if let Some(items) = container.get("props")
            .and_then(|p| p.get("items"))
            .and_then(|i| i.as_array()) {
            for child in items {
                self.convert_component_to_dsl(child);
            }
        }

        self.indent_level -= 1;
        self.add_line("end layout");
    }

    fn convert_form_component(&mut self, component: &Value) {
        let component_type = component.get("type").and_then(|t| t.as_str()).unwrap_or("");
        let empty_map = serde_json::Map::new();
        let props = component.get("props").and_then(|p| p.as_object()).unwrap_or(&empty_map);

        match component_type {
            "TextInput" => {
                let label = props.get("label").and_then(|l| l.as_str()).unwrap_or("Input");
                let placeholder = props.get("placeholder").and_then(|p| p.as_str()).unwrap_or("");
                let required = props.get("required").and_then(|r| r.as_str()).unwrap_or("false") == "true";

                let mut line = format!("input \"{}\"", label);
                if !placeholder.is_empty() {
                    line.push_str(&format!(" placeholder=\"{}\"", placeholder));
                }
                if required {
                    line.push_str(" required");
                }
                self.add_line(&line);
            }
            "Textarea" => {
                let label = props.get("label").and_then(|l| l.as_str()).unwrap_or("Message");
                let rows = props.get("rows").and_then(|r| r.as_u64()).unwrap_or(4);

                let line = format!("textarea \"{}\" rows={}", label, rows);
                self.add_line(&line);
            }
            "Select" => {
                let label = props.get("label").and_then(|l| l.as_str()).unwrap_or("Select");
                let empty_vec = Vec::new();
                let options = props.get("options").and_then(|o| o.as_array()).unwrap_or(&empty_vec);

                let mut line = format!("dropdown \"{}\"", label);
                if !options.is_empty() {
                    let option_values: Vec<String> = options.iter()
                        .filter_map(|opt| {
                            opt.get("label").and_then(|l| l.as_str())
                                .or_else(|| opt.get("value").and_then(|v| v.as_str()))
                        })
                        .map(|s| format!("\"{}\"", s))
                        .collect();
                    if !option_values.is_empty() {
                        line.push_str(&format!(" options=[{}]", option_values.join(", ")));
                    }
                }
                self.add_line(&line);
            }
            "Button" => {
                let text = props.get("text").and_then(|t| t.as_str()).unwrap_or("Button");
                let variant = props.get("variant").and_then(|v| v.as_str()).unwrap_or("default");

                if variant == "default" && matches!(text.to_lowercase().as_str(), "submit" | "send" | "save") {
                    self.add_line(&format!("submit \"{}\"", text));
                } else {
                    self.add_line(&format!("button \"{}\"", text));
                }
            }
            "Checkbox" => {
                let label = props.get("label").and_then(|l| l.as_str()).unwrap_or("Checkbox");
                self.add_line(&format!("checkbox \"{}\"", label));
            }
            "Radio" => {
                let label = props.get("label").and_then(|l| l.as_str()).unwrap_or("Radio");
                self.add_line(&format!("radio \"{}\"", label));
            }
            _ => {
                // Fallback to general component conversion
                self.convert_component_to_dsl(component);
            }
        }
    }

    fn convert_component_to_dsl(&mut self, component: &Value) {
        let component_type = component.get("type").and_then(|t| t.as_str()).unwrap_or("");
        let empty_map = serde_json::Map::new();
        let props = component.get("props").and_then(|p| p.as_object()).unwrap_or(&empty_map);

        match component_type {
            "Heading" => {
                let text = props.get("text").and_then(|t| t.as_str()).unwrap_or("Heading");
                let level = props.get("level").and_then(|l| l.as_u64()).unwrap_or(1);

                // Extract CSS classes if present
                let class_attr = if let Some(css_classes) = props.get("cssClasses").and_then(|c| c.as_array()) {
                    let classes: Vec<String> = css_classes.iter()
                        .filter_map(|c| c.as_str())
                        .map(|s| s.to_string())
                        .collect();
                    if !classes.is_empty() {
                        format!(" class \"{}\"", classes.join(" "))
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                // Use appropriate component name based on level
                if level == 1 {
                    self.add_line(&format!("title \"{}\"{}", text, class_attr));
                } else {
                    self.add_line(&format!("heading{} \"{}\"{}", level, text, class_attr));
                }
            }
            "Text" => {
                let text = props.get("text").and_then(|t| t.as_str()).unwrap_or("Text");
                self.add_line(&format!("text \"{}\"", text));
            }
            "Image" => {
                let src = props.get("src").and_then(|s| s.as_str()).unwrap_or("image.jpg");
                let alt = props.get("alt").and_then(|a| a.as_str()).unwrap_or("Image");
                self.add_line(&format!("image source \"{}\" alt \"{}\"", src, alt));
            }
            "Button" => {
                let text = props.get("text").and_then(|t| t.as_str()).unwrap_or("Button");
                self.add_line(&format!("button \"{}\"", text));
            }
            "Spacer" => {
                let height = props.get("height").and_then(|h| h.as_str()).unwrap_or("20px");
                self.add_line(&format!("spacer height={}", height));
            }
            "Textarea" => {
                let label = props.get("label").and_then(|l| l.as_str()).unwrap_or("Message");
                let rows = props.get("rows").and_then(|r| r.as_u64()).unwrap_or(4);
                let placeholder = props.get("placeholder").and_then(|p| p.as_str()).unwrap_or("");

                let mut line = format!("textarea \"{}\" rows={}", label, rows);
                if !placeholder.is_empty() {
                    line.push_str(&format!(" placeholder=\"{}\"", placeholder));
                }
                self.add_line(&line);
            }
            "Select" => {
                let label = props.get("label").and_then(|l| l.as_str()).unwrap_or("Select Option");
                let empty_vec2 = Vec::new();
                let options = props.get("options").and_then(|o| o.as_array()).unwrap_or(&empty_vec2);

                let mut line = format!("dropdown \"{}\"", label);
                if !options.is_empty() {
                    let option_values: Vec<String> = options.iter()
                        .filter_map(|opt| {
                            opt.get("label").and_then(|l| l.as_str())
                                .or_else(|| opt.get("value").and_then(|v| v.as_str()))
                        })
                        .map(|s| format!("\"{}\"", s))
                        .collect();
                    if !option_values.is_empty() {
                        line.push_str(&format!(" options=[{}]", option_values.join(", ")));
                    }
                }
                self.add_line(&line);
            }
            "TextInput" => {
                let label = props.get("label").and_then(|l| l.as_str()).unwrap_or("Input");
                let placeholder = props.get("placeholder").and_then(|p| p.as_str()).unwrap_or("");
                let required = props.get("required").and_then(|r| r.as_str()).unwrap_or("false");

                let mut line = format!("input \"{}\" type=\"text\"", label);
                if !placeholder.is_empty() {
                    line.push_str(&format!(" placeholder=\"{}\"", placeholder));
                }
                if required == "true" {
                    line.push_str(" required=\"true\"");
                }
                self.add_line(&line);
            }
            "FileInput" => {
                let label = props.get("label").and_then(|l| l.as_str()).unwrap_or("File Upload");
                let accept = props.get("accept").and_then(|a| a.as_str()).unwrap_or("");
                let multiple = props.get("multiple").and_then(|m| m.as_str()).unwrap_or("false");
                let required = props.get("required").and_then(|r| r.as_str()).unwrap_or("false");

                let mut line = format!("input \"{}\" type=\"file\"", label);
                if !accept.is_empty() {
                    line.push_str(&format!(" accept=\"{}\"", accept));
                }
                if multiple == "true" {
                    line.push_str(" multiple=\"true\"");
                }
                if required == "true" {
                    line.push_str(" required=\"true\"");
                }
                self.add_line(&line);
            }
            "Divider" => {
                let style = props.get("style").and_then(|s| s.as_str()).unwrap_or("solid");
                self.add_line(&format!("divider style={}", style));
            }
            _ => {
                // Unknown component - add as comment
                self.add_line(&format!("// Unknown component: {}", component_type));
            }
        }
    }

    fn add_line(&mut self, content: &str) {
        if !content.trim().is_empty() {
            let indent = "    ".repeat(self.indent_level);
            self.output_lines.push(format!("{}{}", indent, content));
        } else {
            self.output_lines.push(String::new());
        }
    }
}

impl Default for PuckToDSLConverter {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert Puck JSON string back to DSL.
pub fn convert_puck_to_dsl(puck_json_str: &str, original_metadata: Option<&HashMap<String, String>>) -> Result<String, String> {
    let puck_data: Value = serde_json::from_str(puck_json_str)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;
    
    let mut converter = PuckToDSLConverter::new();
    converter.convert(&puck_data, original_metadata)
        .map_err(|e| format!("Failed to convert Puck JSON to DSL: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_converter_creation() {
        let converter = PuckToDSLConverter::new();
        assert_eq!(converter.indent_level, 0);
        assert!(converter.output_lines.is_empty());
        assert_eq!(converter.component_counter, 0);
    }

    #[test]
    fn test_to_pascal_case() {
        let converter = PuckToDSLConverter::new();
        assert_eq!(converter.to_pascal_case("hello world"), "HelloWorld");
        assert_eq!(converter.to_pascal_case("sample-screen"), "SampleScreen");
        assert_eq!(converter.to_pascal_case("test_case"), "TestCase");
        assert_eq!(converter.to_pascal_case(""), "");
    }

    #[test]
    fn test_get_layout_name() {
        let converter = PuckToDSLConverter::new();
        assert_eq!(converter.get_layout_name("default"), "DefaultLayout");
        assert_eq!(converter.get_layout_name("header-footer"), "HeaderFooterLayout");
        assert_eq!(converter.get_layout_name("sidebar"), "SidebarLayout");
        assert_eq!(converter.get_layout_name("unknown"), "DefaultLayout");
    }

    #[test]
    fn test_convert_sample_json() {
        let sample_json = json!({
            "content": [
                {
                    "type": "Container",
                    "id": "form-container",
                    "props": {
                        "padding": 16,
                        "items": [
                            {
                                "type": "TextInput",
                                "id": "input-1",
                                "props": {
                                    "label": "Name",
                                    "placeholder": "Enter your name",
                                    "required": "true"
                                }
                            }
                        ]
                    }
                }
            ],
            "root": {"props": {"title": "Sample Screen"}}
        });

        let mut metadata = HashMap::new();
        metadata.insert("target".to_string(), "html".to_string());

        let result = convert_puck_to_dsl(&sample_json.to_string(), Some(&metadata));
        assert!(result.is_ok());
        let dsl = result.unwrap();
        assert!(dsl.contains("@target html"));
        assert!(dsl.contains("module SampleScreen"));
        assert!(dsl.contains("form Form0"));
        assert!(dsl.contains("input \"Name\" placeholder=\"Enter your name\" required"));
    }
}