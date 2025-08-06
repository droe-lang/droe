"""HTML code generator for Roelang frontend DSL."""

from typing import List, Dict, Any, Set
from ...ast import (
    ASTNode, Program, DisplayStatement, IfStatement,
    Literal, Identifier, BinaryOp, PropertyAccess,
    Assignment, ArrayLiteral, WhileLoop, ForEachLoop, ArithmeticOp,
    TaskAction, TaskInvocation, ActionDefinition, ReturnStatement, ActionInvocation,
    ModuleDefinition, DataDefinition, DataField, ActionDefinitionWithParams,
    ActionParameter, ActionInvocationWithArgs, StringInterpolation,
    DataInstance, FieldAssignment, FormatExpression,
    LayoutDefinition, FormDefinition, TitleComponent,
    InputComponent, TextareaComponent, DropdownComponent, ToggleComponent,
    CheckboxComponent, RadioComponent, ButtonComponent, AttributeDefinition,
    ValidationAttribute, BindingAttribute, ActionAttribute
)
from ...symbols import SymbolTable, VariableType
from ...codegen_base import BaseCodeGenerator, CodeGenError


class HTMLCodeGenerator(BaseCodeGenerator):
    """Generates HTML/CSS/JavaScript code from Roelang frontend DSL."""
    
    def __init__(self):
        super().__init__()
        self.data_models: Dict[str, DataDefinition] = {}
        self.actions: Dict[str, ActionDefinitionWithParams] = {}
        self.layouts: Dict[str, LayoutDefinition] = {}
        self.forms: Dict[str, FormDefinition] = {}
        self.component_counter = 0
        self.validation_rules: Set[str] = set()
        self.bindings: Dict[str, str] = {}  # component_id -> binding_target
        
    def generate(self, program: Program) -> str:
        """Generate complete HTML application from AST."""
        self.clear_output()
        self.data_models.clear()
        self.actions.clear()
        self.layouts.clear()
        self.forms.clear()
        self.component_counter = 0
        self.validation_rules.clear()
        self.bindings.clear()
        
        # First pass: collect data models, actions, layouts
        for stmt in program.statements:
            if isinstance(stmt, DataDefinition):
                self.data_models[stmt.name] = stmt
            elif isinstance(stmt, ActionDefinitionWithParams):
                self.actions[stmt.name] = stmt
            elif isinstance(stmt, LayoutDefinition):
                self.layouts[stmt.name] = stmt
            elif isinstance(stmt, FormDefinition):
                self.forms[stmt.name] = stmt
            elif isinstance(stmt, ModuleDefinition):
                # Process module contents
                for module_stmt in stmt.body:
                    if isinstance(module_stmt, DataDefinition):
                        self.data_models[module_stmt.name] = module_stmt
                    elif isinstance(module_stmt, ActionDefinitionWithParams):
                        self.actions[module_stmt.name] = module_stmt
                    elif isinstance(module_stmt, LayoutDefinition):
                        self.layouts[module_stmt.name] = module_stmt
                    elif isinstance(module_stmt, FormDefinition):
                        self.forms[module_stmt.name] = module_stmt
        
        # Generate HTML document
        html_content = self.generate_html_document(program)
        
        return html_content
    
    def generate_html_document(self, program: Program) -> str:
        """Generate complete HTML document."""
        self.emit("<!DOCTYPE html>")
        self.emit("<html lang=\"en\">")
        self.emit("<head>")
        self.indent_level += 1
        self.emit("<meta charset=\"UTF-8\">")
        self.emit("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">")
        self.emit("<title>Roelang Web App</title>")
        self.emit("<style>")
        self.indent_level += 1
        self.generate_css()
        self.indent_level -= 1
        self.emit("</style>")
        self.indent_level -= 1
        self.emit("</head>")
        
        self.emit("<body>")
        self.indent_level += 1
        
        # Generate body content
        for stmt in program.statements:
            if isinstance(stmt, LayoutDefinition):
                self.generate_layout(stmt)
            elif isinstance(stmt, FormDefinition):
                self.generate_form(stmt)
            elif isinstance(stmt, ModuleDefinition):
                # Generate module content
                for module_stmt in stmt.body:
                    if isinstance(module_stmt, LayoutDefinition):
                        self.generate_layout(module_stmt)
                    elif isinstance(module_stmt, FormDefinition):
                        self.generate_form(module_stmt)
        
        # Generate JavaScript
        self.emit("<script>")
        self.indent_level += 1
        self.generate_javascript(program)
        self.indent_level -= 1
        self.emit("</script>")
        
        self.indent_level -= 1
        self.emit("</body>")
        self.emit("</html>")
        
        return self.get_output()
    
    def generate_css(self):
        """Generate CSS styles for the web application."""
        # Base styles
        self.emit("* {")
        self.indent_level += 1
        self.emit("box-sizing: border-box;")
        self.emit("margin: 0;")
        self.emit("padding: 0;")
        self.indent_level -= 1
        self.emit("}")
        self.emit("")
        
        self.emit("body {")
        self.indent_level += 1
        self.emit("font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;")
        self.emit("line-height: 1.6;")
        self.emit("color: #333;")
        self.emit("background-color: #f5f5f5;")
        self.indent_level -= 1
        self.emit("}")
        self.emit("")
        
        # Layout styles
        self.emit(".layout-column {")
        self.indent_level += 1
        self.emit("display: flex;")
        self.emit("flex-direction: column;")
        self.emit("gap: 1rem;")
        self.indent_level -= 1
        self.emit("}")
        self.emit("")
        
        self.emit(".layout-row {")
        self.indent_level += 1
        self.emit("display: flex;")
        self.emit("flex-direction: row;")
        self.emit("gap: 1rem;")
        self.emit("align-items: center;")
        self.indent_level -= 1
        self.emit("}")
        self.emit("")
        
        self.emit(".layout-grid {")
        self.indent_level += 1
        self.emit("display: grid;")
        self.emit("gap: 1rem;")
        self.emit("grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));")
        self.indent_level -= 1
        self.emit("}")
        self.emit("")
        
        self.emit(".layout-stack {")
        self.indent_level += 1
        self.emit("position: relative;")
        self.indent_level -= 1
        self.emit("}")
        self.emit("")
        
        self.emit(".layout-overlay {")
        self.indent_level += 1
        self.emit("position: fixed;")
        self.emit("top: 0;")
        self.emit("left: 0;")
        self.emit("width: 100%;")
        self.emit("height: 100%;")
        self.emit("background-color: rgba(0, 0, 0, 0.5);")
        self.emit("display: flex;")
        self.emit("align-items: center;")
        self.emit("justify-content: center;")
        self.indent_level -= 1
        self.emit("}")
        self.emit("")
        
        # Component styles
        self.emit(".form-container {")
        self.indent_level += 1
        self.emit("background: white;")
        self.emit("padding: 2rem;")
        self.emit("border-radius: 8px;")
        self.emit("box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);")
        self.emit("max-width: 400px;")
        self.emit("width: 100%;")
        self.indent_level -= 1
        self.emit("}")
        self.emit("")
        
        self.emit(".title {")
        self.indent_level += 1
        self.emit("font-size: 1.5rem;")
        self.emit("font-weight: 600;")
        self.emit("margin-bottom: 1rem;")
        self.emit("text-align: center;")
        self.indent_level -= 1
        self.emit("}")
        self.emit("")
        
        self.emit(".input-field, .textarea-field {")
        self.indent_level += 1
        self.emit("width: 100%;")
        self.emit("padding: 0.75rem;")
        self.emit("border: 1px solid #ddd;")
        self.emit("border-radius: 4px;")
        self.emit("font-size: 1rem;")
        self.emit("transition: border-color 0.2s;")
        self.indent_level -= 1
        self.emit("}")
        self.emit("")
        
        self.emit(".input-field:focus, .textarea-field:focus {")
        self.indent_level += 1
        self.emit("outline: none;")
        self.emit("border-color: #007bff;")
        self.emit("box-shadow: 0 0 0 2px rgba(0, 123, 255, 0.25);")
        self.indent_level -= 1
        self.emit("}")
        self.emit("")
        
        self.emit(".button {")
        self.indent_level += 1
        self.emit("background-color: #007bff;")
        self.emit("color: white;")
        self.emit("border: none;")
        self.emit("padding: 0.75rem 1.5rem;")
        self.emit("border-radius: 4px;")
        self.emit("font-size: 1rem;")
        self.emit("cursor: pointer;")
        self.emit("transition: background-color 0.2s;")
        self.emit("width: 100%;")
        self.indent_level -= 1
        self.emit("}")
        self.emit("")
        
        self.emit(".button:hover {")
        self.indent_level += 1
        self.emit("background-color: #0056b3;")
        self.indent_level -= 1
        self.emit("}")
        self.emit("")
        
        self.emit(".error {")
        self.indent_level += 1
        self.emit("color: #dc3545;")
        self.emit("font-size: 0.875rem;")
        self.emit("margin-top: 0.25rem;")
        self.indent_level -= 1
        self.emit("}")
        self.emit("")
        
        # Validation styles
        self.emit(".input-error {")
        self.indent_level += 1
        self.emit("border-color: #dc3545 !important;")
        self.emit("box-shadow: 0 0 0 2px rgba(220, 53, 69, 0.25) !important;")
        self.indent_level -= 1
        self.emit("}")
    
    def generate_layout(self, layout: LayoutDefinition):
        """Generate HTML for a layout definition."""
        layout_id = f"layout-{layout.name}"
        css_class = f"layout-{layout.layout_type}"
        
        self.emit(f'<div id="{layout_id}" class="{css_class}">')
        self.indent_level += 1
        
        for child in layout.children:
            self.generate_component(child)
        
        self.indent_level -= 1
        self.emit("</div>")
        self.emit("")
    
    def generate_form(self, form: FormDefinition):
        """Generate HTML for a form definition."""
        form_id = f"form-{form.name}"
        
        self.emit(f'<div class="form-container">')
        self.indent_level += 1
        self.emit(f'<form id="{form_id}" class="layout-column">')
        self.indent_level += 1
        
        for child in form.children:
            self.generate_component(child)
        
        self.indent_level -= 1
        self.emit("</form>")
        self.indent_level -= 1
        self.emit("</div>")
        self.emit("")
    
    def generate_component(self, component: ASTNode):
        """Generate HTML for a UI component."""
        if isinstance(component, LayoutDefinition):
            self.generate_layout(component)
        elif isinstance(component, FormDefinition):
            self.generate_form(component)
        elif isinstance(component, TitleComponent):
            self.generate_title(component)
        elif isinstance(component, InputComponent):
            self.generate_input(component)
        elif isinstance(component, TextareaComponent):
            self.generate_textarea(component)
        elif isinstance(component, DropdownComponent):
            self.generate_dropdown(component)
        elif isinstance(component, ToggleComponent):
            self.generate_toggle(component)
        elif isinstance(component, CheckboxComponent):
            self.generate_checkbox(component)
        elif isinstance(component, RadioComponent):
            self.generate_radio(component)
        elif isinstance(component, ButtonComponent):
            self.generate_button(component)
        # Add other component types as needed
    
    def generate_title(self, title: TitleComponent):
        """Generate HTML for title component."""
        self.emit(f'<h2 class="title">{self.escape_html(title.text)}</h2>')
    
    def generate_input(self, input_comp: InputComponent):
        """Generate HTML for input component."""
        # Use element_id if provided, otherwise generate one
        if hasattr(input_comp, 'element_id') and input_comp.element_id:
            input_id = input_comp.element_id
        else:
            self.component_counter += 1
            input_id = f"input-{self.component_counter}"
        
        # Store binding if present
        if input_comp.binding:
            self.bindings[input_id] = input_comp.binding
        
        # Generate input with validation attributes
        input_attrs = [
            f'id="{input_id}"',
            f'type="{input_comp.input_type}"',
            'class="input-field"'
        ]
        
        # Add validation attributes
        for attr in input_comp.attributes:
            if isinstance(attr, ValidationAttribute):
                self.validation_rules.add(attr.validation_type)
                if attr.validation_type == 'required':
                    input_attrs.append('required')
                elif attr.validation_type == 'email':
                    input_attrs.append('pattern="[^@]+@[^@]+\.[^@]+"')
        
        # Add placeholder if input type suggests one
        if input_comp.input_type == 'email':
            input_attrs.append('placeholder="Enter your email"')
        elif input_comp.input_type == 'password':
            input_attrs.append('placeholder="Enter your password"')
        
        self.emit(f'<input {" ".join(input_attrs)}>')
        self.emit(f'<div id="{input_id}-error" class="error" style="display: none;"></div>')
    
    def generate_textarea(self, textarea: TextareaComponent):
        """Generate HTML for textarea component."""
        self.component_counter += 1
        textarea_id = f"textarea-{self.component_counter}"
        
        if textarea.binding:
            self.bindings[textarea_id] = textarea.binding
        
        self.emit(f'<textarea id="{textarea_id}" class="textarea-field" rows="4"></textarea>')
    
    def generate_dropdown(self, dropdown: DropdownComponent):
        """Generate HTML for dropdown component."""
        # Use element_id if provided, otherwise generate one
        if hasattr(dropdown, 'element_id') and dropdown.element_id:
            select_id = dropdown.element_id
        else:
            self.component_counter += 1
            select_id = f"select-{self.component_counter}"
        
        if dropdown.binding:
            self.bindings[select_id] = dropdown.binding
        
        self.emit(f'<select id="{select_id}" class="input-field">')
        self.indent_level += 1
        
        for option in dropdown.options:
            if isinstance(option, Literal):
                value = self.escape_html(str(option.value))
                self.emit(f'<option value="{value}">{value}</option>')
        
        self.indent_level -= 1
        self.emit("</select>")
    
    def generate_toggle(self, toggle: ToggleComponent):
        """Generate HTML for toggle component."""
        self.component_counter += 1
        toggle_id = f"toggle-{self.component_counter}"
        
        if toggle.binding:
            self.bindings[toggle_id] = toggle.binding
        
        self.emit('<div class="layout-row">')
        self.indent_level += 1
        self.emit(f'<input type="checkbox" id="{toggle_id}" class="toggle">')
        self.emit(f'<label for="{toggle_id}">Toggle</label>')
        self.indent_level -= 1
        self.emit("</div>")
    
    def generate_checkbox(self, checkbox: CheckboxComponent):
        """Generate HTML for checkbox component."""
        # Use element_id if provided, otherwise generate one
        if hasattr(checkbox, 'element_id') and checkbox.element_id:
            checkbox_id = checkbox.element_id
        else:
            self.component_counter += 1
            checkbox_id = f"checkbox-{self.component_counter}"
        
        if checkbox.binding:
            self.bindings[checkbox_id] = checkbox.binding
        
        self.emit('<div class="layout-row">')
        self.indent_level += 1
        self.emit(f'<input type="checkbox" id="{checkbox_id}">')
        if checkbox.text:
            self.emit(f'<label for="{checkbox_id}">{self.escape_html(checkbox.text)}</label>')
        self.indent_level -= 1
        self.emit("</div>")
    
    def generate_radio(self, radio: RadioComponent):
        """Generate HTML for radio component."""
        # Use element_id if provided, otherwise generate one
        if hasattr(radio, 'element_id') and radio.element_id:
            radio_id = radio.element_id
        else:
            self.component_counter += 1
            radio_id = f"radio-{self.component_counter}"
        
        # Use binding as radio group name if available
        group_name = radio.binding or "radio-group"
        
        self.emit('<div class="layout-row">')
        self.indent_level += 1
        
        radio_attrs = [
            f'type="radio"',
            f'id="{radio_id}"',
            f'name="{group_name}"'
        ]
        
        if radio.value:
            radio_attrs.append(f'value="{self.escape_html(radio.value)}"')
        
        self.emit(f'<input {" ".join(radio_attrs)}>')
        
        if radio.text:
            self.emit(f'<label for="{radio_id}">{self.escape_html(radio.text)}</label>')
        
        self.indent_level -= 1
        self.emit("</div>")
    
    def generate_button(self, button: ButtonComponent):
        """Generate HTML for button component."""
        self.component_counter += 1
        button_id = f"button-{self.component_counter}"
        
        button_attrs = [
            f'id="{button_id}"',
            'type="button"',
            'class="button"'
        ]
        
        if button.action:
            # Check if this is a form submit button that needs data
            if 'submit' in button.action.lower():
                # Find the form this button belongs to
                # Look for the form that contains this button
                form_names = list(self.forms.keys())
                form_id = f"form-{form_names[0]}" if form_names else "form-user_form"
                button_attrs.append(f'onclick="handleFormSubmit(\'{button.action}\', \'{form_id}\')"')
            else:
                button_attrs.append(f'onclick="{button.action}()"')
        
        self.emit(f'<button {" ".join(button_attrs)}>{self.escape_html(button.text)}</button>')
    
    def generate_javascript(self, program: Program):
        """Generate JavaScript for data binding and validation."""
        # Generate data models
        self.emit("// Data Models")
        for name, data_def in self.data_models.items():
            self.generate_data_model_js(name, data_def)
        
        self.emit("")
        
        # Generate validation functions
        self.emit("// Validation")
        self.generate_validation_js()
        
        self.emit("")
        
        # Generate data binding
        self.emit("// Data Binding")
        self.generate_binding_js()
        
        self.emit("")
        
        # Generate form handling
        self.emit("// Form Handling")
        self.generate_form_handling_js()
        
        self.emit("")
        
        # Generate action functions
        self.emit("// Actions")
        for name, action in self.actions.items():
            self.generate_action_js(name, action)
        
        # Generate initialization
        self.emit("")
        self.emit("// Initialize")
        self.emit("document.addEventListener('DOMContentLoaded', function() {")
        self.indent_level += 1
        self.emit("initializeDataBinding();")
        self.emit("initializeValidation();")
        self.indent_level -= 1
        self.emit("});")
    
    def generate_data_model_js(self, name: str, data_def: DataDefinition):
        """Generate JavaScript data model class."""
        self.emit(f"class {name} {{")
        self.indent_level += 1
        
        # Constructor
        self.emit("constructor() {")
        self.indent_level += 1
        for field in data_def.fields:
            default_value = self.get_default_value_js(field.type)
            self.emit(f"this.{field.name} = {default_value};")
        self.indent_level -= 1
        self.emit("}")
        self.emit("")
        
        # Validation method
        self.emit("validate() {")
        self.indent_level += 1
        self.emit("const errors = {};")
        
        for field in data_def.fields:
            field_validation = self.get_field_validation_js(field.name, field.type)
            if field_validation:
                self.emit(field_validation)
        
        self.emit("return Object.keys(errors).length === 0 ? null : errors;")
        self.indent_level -= 1
        self.emit("}")
        
        self.indent_level -= 1
        self.emit("}")
        self.emit("")
    
    def generate_validation_js(self):
        """Generate validation helper functions."""
        self.emit("function validateRequired(value) {")
        self.indent_level += 1
        self.emit("return value !== null && value !== undefined && value.toString().trim() !== '';")
        self.indent_level -= 1
        self.emit("}")
        self.emit("")
        
        self.emit("function validateEmail(value) {")
        self.indent_level += 1
        self.emit("const emailRegex = /^[^@]+@[^@]+\\.[^@]+$/;")
        self.emit("return emailRegex.test(value);")
        self.indent_level -= 1
        self.emit("}")
        self.emit("")
        
        self.emit("function showError(elementId, message) {")
        self.indent_level += 1
        self.emit("const element = document.getElementById(elementId);")
        self.emit("const errorElement = document.getElementById(elementId + '-error');")
        self.emit("if (element && errorElement) {")
        self.indent_level += 1
        self.emit("element.classList.add('input-error');")
        self.emit("errorElement.textContent = message;")
        self.emit("errorElement.style.display = 'block';")
        self.indent_level -= 1
        self.emit("}")
        self.indent_level -= 1
        self.emit("}")
        self.emit("")
        
        self.emit("function clearError(elementId) {")
        self.indent_level += 1
        self.emit("const element = document.getElementById(elementId);")
        self.emit("const errorElement = document.getElementById(elementId + '-error');")
        self.emit("if (element && errorElement) {")
        self.indent_level += 1
        self.emit("element.classList.remove('input-error');")
        self.emit("errorElement.style.display = 'none';")
        self.indent_level -= 1
        self.emit("}")
        self.indent_level -= 1
        self.emit("}")
    
    def generate_binding_js(self):
        """Generate data binding initialization."""
        self.emit("function initializeDataBinding() {")
        self.indent_level += 1
        
        for component_id, binding_target in self.bindings.items():
            self.emit(f"// Bind {component_id} to {binding_target}")
            self.emit(f"const {component_id}_element = document.getElementById('{component_id}');")
            self.emit(f"if ({component_id}_element) {{")
            self.indent_level += 1
            self.emit(f"{component_id}_element.addEventListener('input', function() {{")
            self.indent_level += 1
            self.emit(f"// Update data model: {binding_target}")
            self.emit(f"clearError('{component_id}');")
            self.indent_level -= 1
            self.emit("});")
            self.indent_level -= 1
            self.emit("}")
        
        self.indent_level -= 1
        self.emit("}")
        self.emit("")
        
        self.emit("function initializeValidation() {")
        self.indent_level += 1
        self.emit("// Set up real-time validation")
        self.indent_level -= 1
        self.emit("}")
    
    def generate_form_handling_js(self):
        """Generate form data collection and handling functions."""
        self.emit("// Create data model instances")
        for name, data_def in self.data_models.items():
            self.emit(f"const {name.lower()}Instance = new {name}();")
        
        self.emit("")
        
        # Generate form-specific collection functions
        for form_name in self.forms.keys():
            self.emit(f"function collect{form_name.title()}Data() {{")
            self.indent_level += 1
            self.emit("const formData = {};")
            self.emit("let isValid = true;")
            self.emit("")
            
            # Collect data from bindings
            for element_id, binding_target in self.bindings.items():
                if '.' in binding_target:
                    model_name, field_name = binding_target.split('.', 1)
                    if model_name in self.data_models:
                        data_def = self.data_models[model_name]
                        field = next((f for f in data_def.fields if f.name == field_name), None)
                        
                        if field:
                            self.emit(f"const {element_id}Element = document.getElementById('{element_id}');")
                            self.emit(f"if ({element_id}Element) {{")
                            self.indent_level += 1
                            
                            # Generate data collection based on field type
                            if field.type.lower() == 'boolean':
                                self.emit(f"formData.{field_name} = {element_id}Element.checked;")
                            elif field.type.lower() in ['number', 'int']:
                                self.emit(f"formData.{field_name} = parseInt({element_id}Element.value) || 0;")
                            else:
                                self.emit(f"formData.{field_name} = {element_id}Element.value;")
                            
                            # Generate validation for this field
                            self.emit(f"// Validate {field_name}")
                            self.emit(f"if (!validate{field_name.title()}({element_id}Element)) {{")
                            self.indent_level += 1
                            self.emit("isValid = false;")
                            self.indent_level -= 1
                            self.emit("}")
                            
                            self.indent_level -= 1
                            self.emit("}")
            
            self.emit("")
            self.emit("return { data: formData, isValid: isValid };")
            self.indent_level -= 1
            self.emit("}")
            self.emit("")
        
        # Generate individual field validation functions
        for element_id, binding_target in self.bindings.items():
            if '.' in binding_target:
                model_name, field_name = binding_target.split('.', 1)
                if model_name in self.data_models:
                    self.emit(f"function validate{field_name.title()}(element) {{")
                    self.indent_level += 1
                    self.emit("let isValid = true;")
                    self.emit("")
                    
                    # Find validation rules for this element
                    # This would be expanded to include specific validation logic
                    self.emit("// Add validation logic here")
                    self.emit("if (element.hasAttribute('required') && !element.value.trim()) {")
                    self.indent_level += 1
                    self.emit(f"showError('{element_id}', 'This field is required');")
                    self.emit("isValid = false;")
                    self.indent_level -= 1
                    self.emit("} else {")
                    self.indent_level += 1
                    self.emit(f"clearError('{element_id}');")
                    self.indent_level -= 1
                    self.emit("}")
                    
                    self.emit("")
                    self.emit("return isValid;")
                    self.indent_level -= 1
                    self.emit("}")
                    self.emit("")
        
        self.emit("function handleFormSubmit(actionName, formId) {")
        self.indent_level += 1
        self.emit("const form = document.getElementById(formId);")
        self.emit("if (!form) {")
        self.indent_level += 1
        self.emit("console.error('Form not found:', formId);")
        self.emit("return;")
        self.indent_level -= 1
        self.emit("}")
        self.emit("")
        
        # Collect form data using form elements
        self.emit("const formData = {};")
        self.emit("let isValid = true;")
        self.emit("")
        
        # Generate form data collection based on form elements
        for element_id, binding_target in self.bindings.items():
            if '.' in binding_target:
                model_name, field_name = binding_target.split('.', 1)
                if model_name in self.data_models:
                    data_def = self.data_models[model_name]
                    field = next((f for f in data_def.fields if f.name == field_name), None)
                    
                    if field:
                        self.emit(f"const {element_id}Element = document.getElementById('{element_id}');")
                        self.emit(f"if ({element_id}Element) {{")
                        self.indent_level += 1
                        
                        # Generate data collection based on field type
                        if field.type.lower() == 'boolean':
                            self.emit(f"formData.{field_name} = {element_id}Element.checked;")
                        elif field.type.lower() in ['number', 'int']:
                            self.emit(f"formData.{field_name} = parseInt({element_id}Element.value) || 0;")
                        else:
                            self.emit(f"formData.{field_name} = {element_id}Element.value || '';")
                        
                        # Generate validation for this field
                        self.emit(f"// Validate {field_name}")
                        self.emit(f"if (!validate{field_name.title()}({element_id}Element)) {{")
                        self.indent_level += 1
                        self.emit("isValid = false;")
                        self.indent_level -= 1
                        self.emit("}")
                        
                        self.indent_level -= 1
                        self.emit("}")
                        self.emit("")
        
        self.emit("if (!isValid) {")
        self.indent_level += 1
        self.emit("console.log('Form validation failed');")
        self.emit("return;")
        self.indent_level -= 1
        self.emit("}")
        self.emit("")
        
        self.emit("console.log('Form data collected:', formData);")
        self.emit("")
        self.emit("// Call the appropriate action")
        self.emit("if (typeof window[actionName] === 'function') {")
        self.indent_level += 1
        self.emit("window[actionName](formData);")
        self.indent_level -= 1
        self.emit("} else {")
        self.indent_level += 1
        self.emit("console.error('Action not found:', actionName);")
        self.indent_level -= 1
        self.emit("}")
        self.indent_level -= 1
        self.emit("}")
    
    def generate_action_js(self, name: str, action: ActionDefinitionWithParams):
        """Generate JavaScript function for action."""
        params = [param.name for param in action.parameters]
        param_list = ", ".join(params)
        
        self.emit(f"function {name}({param_list}) {{")
        self.indent_level += 1
        
        # Generate action body
        for stmt in action.body:
            self.generate_js_statement(stmt)
        
        self.indent_level -= 1
        self.emit("}")
        self.emit("")
    
    def generate_js_statement(self, stmt: ASTNode):
        """Generate JavaScript statement."""
        if isinstance(stmt, DisplayStatement):
            expr = self.generate_js_expression(stmt.expression)
            self.emit(f"console.log({expr});")
        elif isinstance(stmt, ReturnStatement):
            expr = self.generate_js_expression(stmt.expression)
            self.emit(f"return {expr};")
    
    def generate_js_expression(self, expr: ASTNode) -> str:
        """Generate JavaScript expression."""
        if isinstance(expr, Literal):
            if expr.type == 'string':
                return f'"{self.escape_js_string(str(expr.value))}"'
            else:
                return str(expr.value).lower() if isinstance(expr.value, bool) else str(expr.value)
        elif isinstance(expr, Identifier):
            return expr.name
        elif isinstance(expr, StringInterpolation):
            parts = []
            for part in expr.parts:
                if isinstance(part, Literal):
                    parts.append(f'"{self.escape_js_string(str(part.value))}"')
                else:
                    parts.append(self.generate_js_expression(part))
            return " + ".join(parts)
        else:
            return "null"
    
    def get_default_value_js(self, type_name: str) -> str:
        """Get default value for JavaScript based on type."""
        type_defaults = {
            'text': '""',
            'string': '""',
            'number': '0',
            'int': '0',
            'float': '0.0',
            'boolean': 'false',
            'bool': 'false'
        }
        return type_defaults.get(type_name.lower(), 'null')
    
    def get_field_validation_js(self, field_name: str, field_type: str) -> str:
        """Generate field validation JavaScript."""
        # This would be expanded based on validation rules
        return f"// Validation for {field_name} ({field_type})"
    
    def escape_html(self, text: str) -> str:
        """Escape HTML special characters."""
        return (text.replace('&', '&amp;')
                   .replace('<', '&lt;')
                   .replace('>', '&gt;')
                   .replace('"', '&quot;')
                   .replace("'", '&#x27;'))
    
    def escape_js_string(self, text: str) -> str:
        """Escape JavaScript string."""
        return (text.replace('\\', '\\\\')
                   .replace('"', '\\"')
                   .replace('\n', '\\n')
                   .replace('\r', '\\r')
                   .replace('\t', '\\t'))
    
    def emit_statement(self, stmt: ASTNode):
        """Emit code for a statement - required by BaseCodeGenerator."""
        # For HTML generation, we handle statements in generate_component
        # This is just to satisfy the abstract method requirement
        if isinstance(stmt, DisplayStatement):
            self.generate_js_statement(stmt)
        elif isinstance(stmt, LayoutDefinition):
            self.generate_layout(stmt)
        elif isinstance(stmt, FormDefinition):
            self.generate_form(stmt)
        # Add more statement types as needed
    
    def emit_expression(self, expr: ASTNode):
        """Emit code for an expression - required by BaseCodeGenerator."""
        # For HTML generation, we handle expressions in generate_js_expression
        # This is just to satisfy the abstract method requirement
        return self.generate_js_expression(expr)