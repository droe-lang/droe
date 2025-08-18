//! HTML code generation for Droe DSL frontend applications
//! 
//! This module generates complete HTML/CSS/JavaScript web applications from Droe DSL,
//! supporting responsive layouts, forms, data binding, and interactive components.

pub mod javascript;

use crate::ast::*;
use crate::codegen::{CodeGenerator};
use crate::codegen_base::{BaseCodeGenerator, CodeGenContext, CodeGenError, TypeSystemHelpers};
// use crate::symbols::VariableType;
use std::collections::{HashMap, HashSet};

pub use javascript::JavaScriptGenerator;

/// HTML/CSS/JavaScript code generator for web frontend applications
pub struct HTMLGenerator {
    context: CodeGenContext,
    // Component tracking
    data_models: HashMap<String, DataDefinition>,
    actions: HashMap<String, ActionDefinitionWithParams>,
    fragments: HashMap<String, FragmentDefinition>,
    screens: HashMap<String, ScreenDefinition>,
    forms: HashMap<String, FormDefinition>,
    component_counter: usize,
    validation_rules: HashSet<String>,
    bindings: HashMap<String, String>, // component_id -> binding_target
    asset_includes: Vec<AssetInclude>,
    use_external_css: bool,
}

impl HTMLGenerator {
    pub fn new() -> Self {
        let mut context = CodeGenContext::new();
        context.enable_core_lib("string_utils");
        context.enable_core_lib("formatting");
        
        Self {
            context,
            data_models: HashMap::new(),
            actions: HashMap::new(),
            fragments: HashMap::new(),
            screens: HashMap::new(),
            forms: HashMap::new(),
            component_counter: 0,
            validation_rules: HashSet::new(),
            bindings: HashMap::new(),
            asset_includes: Vec::new(),
            use_external_css: true,
        }
    }
    
    /// Generate complete HTML application with CSS and JavaScript
    pub fn generate_web_app(&mut self, program: &Program) -> Result<HashMap<String, String>, CodeGenError> {
        self.context.clear_output();
        self.data_models.clear();
        self.actions.clear();
        self.fragments.clear();
        self.screens.clear();
        self.forms.clear();
        self.component_counter = 0;
        self.validation_rules.clear();
        self.bindings.clear();
        self.asset_includes.clear();
        
        // First pass: collect all definitions
        self.collect_definitions(program)?;
        
        let mut files = HashMap::new();
        
        // Generate main HTML file
        let html_content = self.generate_html_document(program)?;
        files.insert("index.html".to_string(), html_content);
        
        // Generate external CSS if enabled
        if self.use_external_css {
            let css_content = self.generate_css_file()?;
            files.insert("assets/global.css".to_string(), css_content);
        }
        
        // Generate JavaScript module
        let js_content = self.generate_javascript_file(program)?;
        files.insert("assets/main.js".to_string(), js_content);
        
        Ok(files)
    }
    
    fn collect_definitions(&mut self, program: &Program) -> Result<(), CodeGenError> {
        for stmt in &program.statements {
            self.collect_statement_definitions(stmt)?;
        }
        Ok(())
    }
    
    fn collect_statement_definitions(&mut self, stmt: &Node) -> Result<(), CodeGenError> {
        match stmt {
            Node::AssetInclude(asset) => {
                self.asset_includes.push(asset.clone());
            },
            Node::DataDefinition(data) => {
                self.data_models.insert(data.name.clone(), data.clone());
            },
            Node::ActionDefinitionWithParams(action) => {
                self.actions.insert(action.name.clone(), action.clone());
            },
            Node::ActionDefinition(action) => {
                // Convert to ActionDefinitionWithParams for consistency
                let action_with_params = ActionDefinitionWithParams {
                    name: action.name.clone(),
                    parameters: vec![],
                    body: action.body.clone(),
                    return_type: None,
                    line_number: action.line_number,
                };
                self.actions.insert(action.name.clone(), action_with_params);
            },
            Node::FragmentDefinition(fragment) => {
                self.fragments.insert(fragment.name.clone(), fragment.clone());
            },
            Node::ScreenDefinition(screen) => {
                self.screens.insert(screen.name.clone(), screen.clone());
            },
            Node::FormDefinition(form) => {
                self.forms.insert(form.name.clone(), form.clone());
            },
            Node::ModuleDefinition(module) => {
                for module_stmt in &module.body {
                    self.collect_statement_definitions(module_stmt)?;
                }
            },
            _ => {} // Other statements don't define reusable components
        }
        Ok(())
    }
    
    fn generate_html_document(&mut self, program: &Program) -> Result<String, CodeGenError> {
        self.context.clear_output();
        
        self.context.emit("<!DOCTYPE html>");
        self.context.emit("<html lang=\"en\">");
        self.context.emit("<head>");
        self.context.indent();
        
        // Meta tags
        self.context.emit("<meta charset=\"UTF-8\">");
        self.context.emit("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">");
        self.context.emit("<title>Droe Web App</title>");
        
        // CSS includes
        self.generate_css_includes()?;
        
        // JavaScript includes
        self.context.emit("<script src=\"assets/main.js\" defer></script>");
        
        self.context.dedent();
        self.context.emit("</head>");
        
        self.context.emit("<body>");
        self.context.indent();
        
        // Generate body content
        self.generate_body_content(program)?;
        
        self.context.dedent();
        self.context.emit("</body>");
        self.context.emit("</html>");
        
        Ok(self.context.get_output())
    }
    
    fn generate_css_includes(&mut self) -> Result<(), CodeGenError> {
        // Include external CSS files
        let css_includes: Vec<_> = self.asset_includes.iter()
            .filter(|asset| asset.asset_type == "css")
            .collect();
            
        if self.use_external_css {
            self.context.emit("<link rel=\"stylesheet\" href=\"assets/global.css\">");
        }
        
        for asset in css_includes {
            self.context.emit(&format!("<link rel=\"stylesheet\" href=\"{}\">", asset.asset_path));
        }
        
        // Include font files
        let font_includes: Vec<_> = self.asset_includes.iter()
            .filter(|asset| asset.asset_type == "font")
            .collect();
            
        if !font_includes.is_empty() {
            self.context.emit("<style>");
            self.context.indent();
            for asset in font_includes {
                let font_name = asset.asset_path.split('/').last().unwrap_or("CustomFont")
                    .split('.').next().unwrap_or("CustomFont");
                self.context.emit("@font-face {");
                self.context.indent();
                self.context.emit(&format!("font-family: \"{}\";", font_name));
                self.context.emit(&format!("src: url(\"{}\");", asset.asset_path));
                self.context.dedent();
                self.context.emit("}");
            }
            self.context.dedent();
            self.context.emit("</style>");
        }
        
        Ok(())
    }
    
    fn generate_body_content(&mut self, program: &Program) -> Result<(), CodeGenError> {
        // Generate screens and layouts
        let screens = self.screens.clone();
        for (_, screen) in &screens {
            self.generate_screen(screen)?;
        }
        
        // Generate standalone forms
        let forms = self.forms.clone();
        for (_, form) in &forms {
            self.generate_form(form)?;
        }
        
        // Include external JavaScript files
        let js_includes: Vec<_> = self.asset_includes.iter()
            .filter(|asset| asset.asset_type == "js")
            .collect();
            
        for asset in js_includes {
            self.context.emit(&format!("<script src=\"{}\"></script>", asset.asset_path));
        }
        
        Ok(())
    }
    
    fn generate_screen(&mut self, screen: &ScreenDefinition) -> Result<(), CodeGenError> {
        let screen_id = format!("screen-{}-{}", screen.name, self.component_counter);
        self.component_counter += 1;
        
        let mut attrs = vec![format!("id=\"{}\"", screen_id)];
        if !screen.classes.is_empty() {
            attrs.push(format!("class=\"{}\"", screen.classes.join(" ")));
        }
        
        self.context.emit(&format!("<div {}>", attrs.join(" ")));
        self.context.indent();
        
        // Process fragment references in the screen
        for fragment_ref in &screen.fragments {
            self.generate_fragment_reference(fragment_ref)?;
        }
        
        self.context.dedent();
        self.context.emit("</div>");
        
        Ok(())
    }
    
    fn generate_fragment_reference(&mut self, fragment_ref: &FragmentReference) -> Result<(), CodeGenError> {
        let fragment = self.fragments.get(&fragment_ref.fragment_name)
            .ok_or_else(|| CodeGenError::GenerationFailed {
                message: format!("Fragment '{}' not found", fragment_ref.fragment_name)
            })?;
        
        let fragment_id = format!("fragment-{}-{}", fragment.name, self.component_counter);
        self.component_counter += 1;
        
        let mut attrs = vec![
            format!("id=\"{}\"", fragment_id),
            format!("data-fragment=\"{}\"", fragment.name)
        ];
        
        if !fragment.classes.is_empty() {
            attrs.push(format!("class=\"{}\"", fragment.classes.join(" ")));
        }
        
        let tag = self.get_semantic_tag_for_fragment(&fragment.name);
        
        self.context.emit(&format!("<{} {}>", tag, attrs.join(" ")));
        self.context.indent();
        
        // Generate content for each slot
        let slots = fragment.slots.clone();
        for slot in &slots {
            let slot_content = fragment_ref.slot_contents.get(&slot.name).cloned().unwrap_or_default();
            self.generate_slot_with_content(slot, &slot_content)?;
        }
        
        self.context.dedent();
        self.context.emit(&format!("</{}>", tag));
        
        Ok(())
    }
    
    fn get_semantic_tag_for_fragment(&self, fragment_name: &str) -> &'static str {
        let name_lower = fragment_name.to_lowercase();
        
        if name_lower.contains("header") { "header" }
        else if name_lower.contains("footer") { "footer" }
        else if name_lower.contains("nav") { "nav" }
        else if name_lower.contains("main") || name_lower.contains("content") { "main" }
        else if name_lower.contains("aside") || name_lower.contains("sidebar") { "aside" }
        else if name_lower.contains("article") { "article" }
        else if name_lower.contains("section") { "section" }
        else { "div" }
    }
    
    fn generate_slot_with_content(&mut self, slot: &SlotComponent, content: &[Node]) -> Result<(), CodeGenError> {
        let slot_id = format!("slot-{}-{}", slot.name, self.component_counter);
        self.component_counter += 1;
        
        let mut attrs = vec![
            format!("id=\"{}\"", slot_id),
            format!("data-slot=\"{}\"", slot.name)
        ];
        
        if !slot.classes.is_empty() {
            attrs.push(format!("class=\"{}\"", slot.classes.join(" ")));
        }
        
        self.context.emit(&format!("<div {}>", attrs.join(" ")));
        self.context.indent();
        
        // Generate assigned content or default content
        if !content.is_empty() {
            for component in content {
                self.generate_component(component)?;
            }
        } else {
            for component in &slot.default_content {
                self.generate_component(component)?;
            }
        }
        
        self.context.dedent();
        self.context.emit("</div>");
        
        Ok(())
    }
    
    fn generate_form(&mut self, form: &FormDefinition) -> Result<(), CodeGenError> {
        let form_id = format!("form-{}", form.name);
        
        self.context.emit("<div>");
        self.context.indent();
        self.context.emit(&format!("<form id=\"{}\">", form_id));
        self.context.indent();
        
        for child in &form.children {
            self.generate_component(child)?;
        }
        
        self.context.dedent();
        self.context.emit("</form>");
        self.context.dedent();
        self.context.emit("</div>");
        
        Ok(())
    }
    
    fn generate_component(&mut self, component: &Node) -> Result<(), CodeGenError> {
        match component {
            Node::FormDefinition(form) => self.generate_form(form),
            Node::TitleComponent(title) => self.generate_title(title),
            Node::TextComponent(text) => self.generate_text(text),
            Node::InputComponent(input) => self.generate_input(input),
            Node::TextareaComponent(textarea) => self.generate_textarea(textarea),
            Node::DropdownComponent(dropdown) => self.generate_dropdown(dropdown),
            Node::CheckboxComponent(checkbox) => self.generate_checkbox(checkbox),
            Node::RadioComponent(radio) => self.generate_radio(radio),
            Node::ButtonComponent(button) => self.generate_button(button),
            Node::ImageComponent(image) => self.generate_image(image),
            Node::VideoComponent(video) => self.generate_video(video),
            Node::AudioComponent(audio) => self.generate_audio(audio),
            _ => {
                self.context.emit("<!-- Unsupported component -->");
                Ok(())
            }
        }
    }
    
    fn generate_title(&mut self, title: &TitleComponent) -> Result<(), CodeGenError> {
        let mut attrs = Vec::new();
        
        if !title.classes.is_empty() {
            attrs.push(format!("class=\"{}\"", title.classes.join(" ")));
        }
        
        if let Some(styles) = &title.styles {
            attrs.push(format!("style=\"{}\"", styles));
        }
        
        let attrs_str = if attrs.is_empty() { String::new() } else { format!(" {}", attrs.join(" ")) };
        let escaped_text = self.escape_html(&title.text);
        
        self.context.emit(&format!("<h2{}>{}</h2>", attrs_str, escaped_text));
        Ok(())
    }
    
    fn generate_text(&mut self, text: &TextComponent) -> Result<(), CodeGenError> {
        let mut attrs = Vec::new();
        
        if !text.classes.is_empty() {
            attrs.push(format!("class=\"{}\"", text.classes.join(" ")));
        }
        
        if let Some(styles) = &text.styles {
            attrs.push(format!("style=\"{}\"", styles));
        }
        
        let attrs_str = if attrs.is_empty() { String::new() } else { format!(" {}", attrs.join(" ")) };
        let escaped_text = self.escape_html(&text.text);
        
        self.context.emit(&format!("<p{}>{}</p>", attrs_str, escaped_text));
        Ok(())
    }
    
    fn generate_input(&mut self, input: &InputComponent) -> Result<(), CodeGenError> {
        let input_id = if let Some(id) = &input.element_id {
            id.clone()
        } else {
            self.component_counter += 1;
            format!("input-{}", self.component_counter)
        };
        
        // Store binding if present
        if let Some(binding) = &input.binding {
            self.bindings.insert(input_id.clone(), binding.clone());
        }
        
        // Extract attributes
        let mut label = String::new();
        let mut placeholder = String::new();
        let mut name = input_id.clone();
        let mut required = false;
        
        for attr in &input.attributes {
            if let Some(value_node) = &attr.value {
                if let Node::Literal(literal) = value_node.as_ref() {
                    if let LiteralValue::String(value) = &literal.value {
                        match attr.name.as_str() {
                            "label" => label = value.clone(),
                            "placeholder" => placeholder = value.clone(),
                            "name" => name = value.clone(),
                            "required" => required = value.to_lowercase() == "true",
                            _ => {}
                        }
                    }
                }
            }
            
            // Handle validation attributes
            if attr.name == "required" {
                self.validation_rules.insert("required".to_string());
                required = true;
            }
        }
        
        // Generate form group
        self.context.emit("<div class=\"form-group\">");
        self.context.indent();
        
        // Generate label
        if !label.is_empty() {
            self.context.emit(&format!("<label for=\"{}\">{}</label>", input_id, self.escape_html(&label)));
        }
        
        // Generate input
        let mut input_attrs = vec![
            format!("id=\"{}\"", input_id),
            format!("name=\"{}\"", name),
            format!("type=\"{}\"", input.input_type),
        ];
        
        if !placeholder.is_empty() {
            input_attrs.push(format!("placeholder=\"{}\"", self.escape_html(&placeholder)));
        }
        
        if required {
            input_attrs.push("required".to_string());
        }
        
        if input.input_type == "email" {
            input_attrs.push("pattern=\"[^@]+@[^@]+\\.[^@]+\"".to_string());
        }
        
        self.context.emit(&format!("<input {}>", input_attrs.join(" ")));
        self.context.emit(&format!("<div id=\"{}-error\" class=\"error-message\" style=\"display: none;\"></div>", input_id));
        
        self.context.dedent();
        self.context.emit("</div>");
        
        Ok(())
    }
    
    fn generate_textarea(&mut self, textarea: &TextareaComponent) -> Result<(), CodeGenError> {
        self.component_counter += 1;
        let textarea_id = format!("textarea-{}", self.component_counter);
        
        if let Some(binding) = &textarea.binding {
            self.bindings.insert(textarea_id.clone(), binding.clone());
        }
        
        let rows = textarea.rows.unwrap_or(4);
        let placeholder = textarea.placeholder.as_deref().unwrap_or("");
        let label = textarea.label.as_deref().unwrap_or("");
        
        self.context.emit("<div class=\"form-group\">");
        self.context.indent();
        
        if !label.is_empty() {
            self.context.emit(&format!("<label for=\"{}\">{}</label>", textarea_id, self.escape_html(label)));
        }
        
        let mut attrs = vec![
            format!("id=\"{}\"", textarea_id),
            format!("name=\"{}\"", textarea_id),
            format!("rows=\"{}\"", rows),
        ];
        
        if !placeholder.is_empty() {
            attrs.push(format!("placeholder=\"{}\"", self.escape_html(placeholder)));
        }
        
        self.context.emit(&format!("<textarea {}></textarea>", attrs.join(" ")));
        
        self.context.dedent();
        self.context.emit("</div>");
        
        Ok(())
    }
    
    fn generate_dropdown(&mut self, dropdown: &DropdownComponent) -> Result<(), CodeGenError> {
        let select_id = if let Some(id) = &dropdown.element_id {
            id.clone()
        } else {
            self.component_counter += 1;
            format!("select-{}", self.component_counter)
        };
        
        if let Some(binding) = &dropdown.binding {
            self.bindings.insert(select_id.clone(), binding.clone());
        }
        
        let label = dropdown.label.as_deref().unwrap_or("");
        
        self.context.emit("<div class=\"form-group\">");
        self.context.indent();
        
        if !label.is_empty() {
            self.context.emit(&format!("<label for=\"{}\">{}</label>", select_id, self.escape_html(label)));
        }
        
        self.context.emit(&format!("<select id=\"{}\" name=\"{}\">", select_id, select_id));
        self.context.indent();
        
        for option in &dropdown.options {
            let option_text = match option {
                Node::Literal(literal) => {
                    match &literal.value {
                        LiteralValue::String(s) => s.clone(),
                        _ => format!("{:?}", literal.value),
                    }
                },
                _ => "Option".to_string(),
            };
            let escaped_option = self.escape_html(&option_text);
            self.context.emit(&format!("<option value=\"{}\">{}</option>", escaped_option, escaped_option));
        }
        
        self.context.dedent();
        self.context.emit("</select>");
        
        self.context.dedent();
        self.context.emit("</div>");
        
        Ok(())
    }
    
    fn generate_checkbox(&mut self, checkbox: &CheckboxComponent) -> Result<(), CodeGenError> {
        let checkbox_id = if let Some(id) = &checkbox.element_id {
            id.clone()
        } else {
            self.component_counter += 1;
            format!("checkbox-{}", self.component_counter)
        };
        
        if let Some(binding) = &checkbox.binding {
            self.bindings.insert(checkbox_id.clone(), binding.clone());
        }
        
        self.context.emit("<div>");
        self.context.indent();
        self.context.emit(&format!("<input type=\"checkbox\" id=\"{}\">", checkbox_id));
        if let Some(text) = &checkbox.text {
            self.context.emit(&format!("<label for=\"{}\">{}</label>", checkbox_id, self.escape_html(text)));
        }
        self.context.dedent();
        self.context.emit("</div>");
        
        Ok(())
    }
    
    fn generate_radio(&mut self, radio: &RadioComponent) -> Result<(), CodeGenError> {
        let radio_id = if let Some(id) = &radio.element_id {
            id.clone()
        } else {
            self.component_counter += 1;
            format!("radio-{}", self.component_counter)
        };
        
        let group_name = radio.binding.as_deref().unwrap_or("radio-group");
        
        self.context.emit("<div>");
        self.context.indent();
        
        let mut attrs = vec![
            "type=\"radio\"".to_string(),
            format!("id=\"{}\"", radio_id),
            format!("name=\"{}\"", group_name),
        ];
        
        if let Some(value) = &radio.value {
            attrs.push(format!("value=\"{}\"", self.escape_html(value)));
        }
        
        self.context.emit(&format!("<input {}>", attrs.join(" ")));
        
        if let Some(text) = &radio.text {
            self.context.emit(&format!("<label for=\"{}\">{}</label>", radio_id, self.escape_html(text)));
        }
        
        self.context.dedent();
        self.context.emit("</div>");
        
        Ok(())
    }
    
    fn generate_button(&mut self, button: &ButtonComponent) -> Result<(), CodeGenError> {
        self.component_counter += 1;
        let button_id = format!("button-{}", self.component_counter);
        
        let mut attrs = vec![
            format!("id=\"{}\"", button_id),
            "type=\"button\"".to_string(),
        ];
        
        if let Some(action) = &button.action {
            // Determine if this is a form submit button
            let is_form_action = self.actions.get(action)
                .map(|a| !a.parameters.is_empty())
                .unwrap_or(false) ||
                action.to_lowercase().contains("save") ||
                action.to_lowercase().contains("submit") ||
                action.to_lowercase().contains("register");
            
            if is_form_action && !self.forms.is_empty() {
                let form_name = self.forms.keys().next().unwrap();
                attrs.push(format!("onclick=\"submit_{}('{}')\"", form_name, action));
            } else {
                attrs.push(format!("onclick=\"{}()\"", action));
            }
        }
        
        self.context.emit(&format!("<button {}>{}</button>", attrs.join(" "), self.escape_html(&button.text)));
        
        Ok(())
    }
    
    fn generate_image(&mut self, image: &ImageComponent) -> Result<(), CodeGenError> {
        self.component_counter += 1;
        let image_id = format!("image-{}", self.component_counter);
        
        let mut attrs = vec![
            format!("id=\"{}\"", image_id),
            format!("src=\"{}\"", self.escape_html(&image.src)),
        ];
        
        if let Some(alt) = &image.alt {
            attrs.push(format!("alt=\"{}\"", self.escape_html(alt)));
        } else {
            attrs.push("alt=\"\"".to_string());
        }
        
        if !image.classes.is_empty() {
            attrs.push(format!("class=\"{}\"", image.classes.join(" ")));
        }
        
        self.context.emit(&format!("<img {}>", attrs.join(" ")));
        
        Ok(())
    }
    
    fn generate_video(&mut self, video: &VideoComponent) -> Result<(), CodeGenError> {
        self.component_counter += 1;
        let video_id = format!("video-{}", self.component_counter);
        
        let mut attrs = vec![
            format!("id=\"{}\"", video_id),
            format!("src=\"{}\"", self.escape_html(&video.src)),
        ];
        
        if video.controls { attrs.push("controls".to_string()); }
        if video.autoplay { attrs.push("autoplay".to_string()); }
        if video.loop_video { attrs.push("loop".to_string()); }
        if video.muted { attrs.push("muted".to_string()); }
        
        if !video.classes.is_empty() {
            attrs.push(format!("class=\"{}\"", video.classes.join(" ")));
        }
        
        self.context.emit(&format!("<video {}>", attrs.join(" ")));
        self.context.emit("  Your browser does not support the video tag.");
        self.context.emit("</video>");
        
        Ok(())
    }
    
    fn generate_audio(&mut self, audio: &AudioComponent) -> Result<(), CodeGenError> {
        self.component_counter += 1;
        let audio_id = format!("audio-{}", self.component_counter);
        
        let mut attrs = vec![
            format!("id=\"{}\"", audio_id),
            format!("src=\"{}\"", self.escape_html(&audio.src)),
        ];
        
        if audio.controls { attrs.push("controls".to_string()); }
        if audio.autoplay { attrs.push("autoplay".to_string()); }
        if audio.loop_audio { attrs.push("loop".to_string()); }
        
        if !audio.classes.is_empty() {
            attrs.push(format!("class=\"{}\"", audio.classes.join(" ")));
        }
        
        self.context.emit(&format!("<audio {}>", attrs.join(" ")));
        self.context.emit("  Your browser does not support the audio element.");
        self.context.emit("</audio>");
        
        Ok(())
    }
    
    fn generate_css_file(&mut self) -> Result<String, CodeGenError> {
        self.context.clear_output();
        
        // Base reset styles
        self.context.emit("* {");
        self.context.indent();
        self.context.emit("box-sizing: border-box;");
        self.context.emit("margin: 0;");
        self.context.emit("padding: 0;");
        self.context.dedent();
        self.context.emit("}");
        self.context.emit("");
        
        // Body styles
        self.context.emit("body {");
        self.context.indent();
        self.context.emit("font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;");
        self.context.emit("line-height: 1.6;");
        self.context.emit("color: #333;");
        self.context.emit("background-color: #f5f5f5;");
        self.context.dedent();
        self.context.emit("}");
        self.context.emit("");
        
        // Form styles
        self.context.emit(".form-group {");
        self.context.indent();
        self.context.emit("margin-bottom: 1rem;");
        self.context.dedent();
        self.context.emit("}");
        self.context.emit("");
        
        self.context.emit(".form-group label {");
        self.context.indent();
        self.context.emit("display: block;");
        self.context.emit("margin-bottom: 0.5rem;");
        self.context.emit("font-weight: 500;");
        self.context.dedent();
        self.context.emit("}");
        self.context.emit("");
        
        self.context.emit("input, textarea, select {");
        self.context.indent();
        self.context.emit("width: 100%;");
        self.context.emit("padding: 0.75rem;");
        self.context.emit("border: 1px solid #ddd;");
        self.context.emit("border-radius: 4px;");
        self.context.emit("font-size: 1rem;");
        self.context.emit("transition: border-color 0.2s;");
        self.context.dedent();
        self.context.emit("}");
        self.context.emit("");
        
        self.context.emit("input:focus, textarea:focus, select:focus {");
        self.context.indent();
        self.context.emit("outline: none;");
        self.context.emit("border-color: #007bff;");
        self.context.emit("box-shadow: 0 0 0 2px rgba(0, 123, 255, 0.25);");
        self.context.dedent();
        self.context.emit("}");
        self.context.emit("");
        
        self.context.emit("button {");
        self.context.indent();
        self.context.emit("background-color: #007bff;");
        self.context.emit("color: white;");
        self.context.emit("border: none;");
        self.context.emit("padding: 0.75rem 1.5rem;");
        self.context.emit("border-radius: 4px;");
        self.context.emit("font-size: 1rem;");
        self.context.emit("cursor: pointer;");
        self.context.emit("transition: background-color 0.2s;");
        self.context.dedent();
        self.context.emit("}");
        self.context.emit("");
        
        self.context.emit("button:hover {");
        self.context.indent();
        self.context.emit("background-color: #0056b3;");
        self.context.dedent();
        self.context.emit("}");
        self.context.emit("");
        
        self.context.emit(".error-message {");
        self.context.indent();
        self.context.emit("color: #dc3545;");
        self.context.emit("font-size: 0.875rem;");
        self.context.emit("margin-top: 0.25rem;");
        self.context.dedent();
        self.context.emit("}");
        
        Ok(self.context.get_output())
    }
    
    fn generate_javascript_file(&mut self, program: &Program) -> Result<String, CodeGenError> {
        self.context.clear_output();
        
        self.context.emit("// Droe Application JavaScript");
        self.context.emit("");
        
        // Generate data models
        self.context.emit("// Data Models");
        let data_models = self.data_models.clone();
        for (name, data_def) in &data_models {
            self.generate_data_model_js(name, data_def)?;
        }
        self.context.emit("");
        
        // Generate action functions
        self.context.emit("// Actions");
        let actions = self.actions.clone();
        for (name, action) in &actions {
            self.generate_action_js(name, action)?;
        }
        
        // Generate form handlers
        self.context.emit("// Form Handlers");
        self.generate_form_handlers()?;
        
        // Generate initialization
        self.context.emit("// Initialize application");
        self.context.emit("document.addEventListener('DOMContentLoaded', function() {");
        self.context.indent();
        self.context.emit("console.log('Droe application loaded');");
        self.context.dedent();
        self.context.emit("});");
        
        Ok(self.context.get_output())
    }
    
    fn generate_data_model_js(&mut self, name: &str, data_def: &DataDefinition) -> Result<(), CodeGenError> {
        self.context.emit(&format!("class {} {{", name));
        self.context.indent();
        
        self.context.emit("constructor() {");
        self.context.indent();
        for field in &data_def.fields {
            let default_value = self.get_default_value_js(&field.field_type);
            self.context.emit(&format!("this.{} = {};", field.name, default_value));
        }
        self.context.dedent();
        self.context.emit("}");
        
        self.context.dedent();
        self.context.emit("}");
        self.context.emit("");
        
        Ok(())
    }
    
    fn generate_action_js(&mut self, name: &str, action: &ActionDefinitionWithParams) -> Result<(), CodeGenError> {
        let params: Vec<String> = action.parameters.iter().map(|p| p.name.clone()).collect();
        let param_list = params.join(", ");
        
        // Check if action contains API calls (needs to be async)
        let has_api_calls = action.body.iter().any(|stmt| matches!(stmt, Node::ApiCallStatement(_)));
        let async_keyword = if has_api_calls { "async " } else { "" };
        
        self.context.emit(&format!("{}function {}({}) {{", async_keyword, name, param_list));
        self.context.indent();
        
        // Generate action body
        for stmt in &action.body {
            self.generate_js_statement(stmt)?;
        }
        
        self.context.dedent();
        self.context.emit("}");
        self.context.emit("");
        
        Ok(())
    }
    
    fn generate_form_handlers(&mut self) -> Result<(), CodeGenError> {
        for (form_name, _) in &self.forms {
            self.context.emit(&format!("// Handler for {}", form_name));
            self.context.emit(&format!("window.submit_{} = async function(actionName) {{", form_name));
            self.context.indent();
            
            // Collect form data
            self.context.emit("const formData = {};");
            self.context.emit("let isValid = true;");
            self.context.emit("");
            
            // Generate form data collection based on bindings
            for (element_id, binding_target) in &self.bindings {
                if let Some((model_name, field_name)) = binding_target.split_once('.') {
                    if let Some(data_def) = self.data_models.get(model_name) {
                        if let Some(field) = data_def.fields.iter().find(|f| f.name == field_name) {
                            let safe_var = element_id.replace('-', "_");
                            self.context.emit(&format!("const {}Element = document.getElementById('{}');", safe_var, element_id));
                            self.context.emit(&format!("if ({}Element) {{", safe_var));
                            self.context.indent();
                            
                            // Generate data collection based on field type
                            match field.field_type.to_lowercase().as_str() {
                                "boolean" | "bool" => {
                                    self.context.emit(&format!("formData.{} = {}Element.checked;", field_name, safe_var));
                                },
                                "number" | "int" | "integer" => {
                                    self.context.emit(&format!("formData.{} = parseInt({}Element.value) || 0;", field_name, safe_var));
                                },
                                _ => {
                                    self.context.emit(&format!("formData.{} = {}Element.value || '';", field_name, safe_var));
                                }
                            }
                            
                            self.context.dedent();
                            self.context.emit("}");
                        }
                    }
                }
            }
            
            self.context.emit("");
            self.context.emit("// Call the action");
            self.context.emit("if (typeof window[actionName] === 'function') {");
            self.context.indent();
            self.context.emit("return await window[actionName](formData);");
            self.context.dedent();
            self.context.emit("} else {");
            self.context.indent();
            self.context.emit("console.error('Action not found:', actionName);");
            self.context.emit("return null;");
            self.context.dedent();
            self.context.emit("}");
            
            self.context.dedent();
            self.context.emit("};");
            self.context.emit("");
        }
        
        Ok(())
    }
    
    fn generate_js_statement(&mut self, stmt: &Node) -> Result<(), CodeGenError> {
        match stmt {
            Node::DisplayStatement(display) => {
                let expr = self.emit_expression(&display.expression)?;
                self.context.emit(&format!("console.log({});", expr));
            },
            Node::ReturnStatement(ret) => {
                let expr = self.emit_expression(&ret.expression)?;
                self.context.emit(&format!("return {};", expr));
            },
            Node::ApiCallStatement(api_call) => {
                self.generate_api_call_js(api_call)?;
            },
            _ => {
                self.context.emit("// TODO: Implement statement");
            }
        }
        Ok(())
    }
    
    fn generate_api_call_js(&mut self, api_call: &ApiCallStatement) -> Result<(), CodeGenError> {
        let endpoint = &api_call.endpoint;
        let method = api_call.method.to_uppercase();
        
        self.context.emit("try {");
        self.context.indent();
        
        // Construct fetch options
        self.context.emit("const fetchOptions = {");
        self.context.indent();
        self.context.emit(&format!("method: '{}',", method));
        
        // Add headers
        if !api_call.headers.is_empty() {
            self.context.emit("headers: {");
            self.context.indent();
            for header in &api_call.headers {
                let value = if header.value.starts_with('"') && header.value.ends_with('"') {
                    &header.value[1..header.value.len()-1]
                } else {
                    &header.value
                };
                self.context.emit(&format!("'{}': '{}',", header.name, self.escape_js_string(value)));
            }
            self.context.dedent();
            self.context.emit("},");
        }
        
        // Add body for POST/PUT requests
        if let Some(payload) = &api_call.payload {
            if matches!(method.as_str(), "POST" | "PUT" | "PATCH") {
                self.context.emit(&format!("body: JSON.stringify({})", payload));
            }
        }
        
        self.context.dedent();
        self.context.emit("};");
        self.context.emit("");
        
        // Make the fetch call
        self.context.emit(&format!("const response = await fetch('{}', fetchOptions);", endpoint));
        self.context.emit("");
        
        // Handle response
        self.context.emit("if (!response.ok) {");
        self.context.indent();
        self.context.emit("throw new Error(`HTTP error! status: ${response.status}`);");
        self.context.dedent();
        self.context.emit("}");
        self.context.emit("");
        
        // Parse response
        if let Some(response_var) = &api_call.response_variable {
            self.context.emit(&format!("const {} = await response.json();", response_var));
            self.context.emit(&format!("console.log('API Response:', {});", response_var));
        } else {
            self.context.emit("const apiResponse = await response.json();");
            self.context.emit("console.log('API Response:', apiResponse);");
        }
        
        self.context.dedent();
        self.context.emit("} catch (error) {");
        self.context.indent();
        self.context.emit("console.error('API call failed:', error);");
        if let Some(response_var) = &api_call.response_variable {
            self.context.emit(&format!("const {} = null;", response_var));
        }
        self.context.dedent();
        self.context.emit("}");
        
        Ok(())
    }
    
    fn get_default_value_js(&self, type_name: &str) -> &'static str {
        match type_name.to_lowercase().as_str() {
            "text" | "string" => "\"\"",
            "number" | "int" | "integer" => "0",
            "float" | "decimal" => "0.0",
            "boolean" | "bool" => "false",
            _ => "null"
        }
    }
    
    fn escape_html(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
    }
    
    fn escape_js_string(&self, text: &str) -> String {
        text.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }
}

impl TypeSystemHelpers for HTMLGenerator {}

impl CodeGenerator for HTMLGenerator {
    fn generate(&self, program: &Program) -> Result<String, String> {
        let mut generator = Self::new();
        generator.generate_html_document(program).map_err(|e| format!("{}", e))
    }
}

impl BaseCodeGenerator for HTMLGenerator {
    fn generate(&self, program: &Program) -> Result<String, CodeGenError> {
        let mut generator = Self::new();
        generator.generate_html_document(program)
    }
    
    fn emit_expression(&mut self, expr: &Node) -> Result<String, CodeGenError> {
        match expr {
            Node::Literal(literal) => {
                Ok(match &literal.value {
                    LiteralValue::String(s) => format!("\"{}\"", self.escape_js_string(s)),
                    LiteralValue::Integer(i) => i.to_string(),
                    LiteralValue::Float(f) => f.to_string(),
                    LiteralValue::Boolean(b) => b.to_string().to_lowercase(),
                })
            },
            Node::Identifier(identifier) => {
                Ok(identifier.name.clone())
            },
            _ => Ok("null".to_string()),
        }
    }
    
    fn emit_statement(&mut self, stmt: &Node) -> Result<(), CodeGenError> {
        self.generate_component(stmt)
    }
    
    fn target_language(&self) -> &str {
        "html"
    }
}

impl Default for HTMLGenerator {
    fn default() -> Self {
        Self::new()
    }
}