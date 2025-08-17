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
    FragmentDefinition, ScreenDefinition, SlotComponent, FormDefinition, TitleComponent, TextComponent,
    InputComponent, TextareaComponent, DropdownComponent, ToggleComponent,
    CheckboxComponent, RadioComponent, ButtonComponent, FragmentReference,
    ImageComponent, VideoComponent, AudioComponent, AssetInclude,
    AttributeDefinition, ValidationAttribute, BindingAttribute, ActionAttribute,
    ApiCallStatement, ApiHeader
)
from ...symbols import SymbolTable, VariableType
from ...codegen_base import BaseCodeGenerator, CodeGenError


class HTMLCodeGenerator(BaseCodeGenerator):
    """Generates HTML/CSS/JavaScript code from Roelang frontend DSL."""
    
    def __init__(self):
        super().__init__()
        self.data_models: Dict[str, DataDefinition] = {}
        self.actions: Dict[str, ActionDefinitionWithParams] = {}
        self.fragments: Dict[str, FragmentDefinition] = {}
        self.screens: Dict[str, ScreenDefinition] = {}
        self.forms: Dict[str, FormDefinition] = {}
        self.component_counter = 0
        self.validation_rules: Set[str] = set()
        self.bindings: Dict[str, str] = {}  # component_id -> binding_target
        self.asset_includes: List[AssetInclude] = []  # Track included assets
        self.use_external_css = True  # Flag to use external CSS
        
    def generate(self, program: Program) -> str:
        """Generate complete HTML application from AST."""
        self.clear_output()
        self.data_models.clear()
        self.actions.clear()
        self.fragments.clear()
        self.screens.clear()
        self.forms.clear()
        self.component_counter = 0
        self.validation_rules.clear()
        self.bindings.clear()
        
        
        # First pass: collect data models, actions, layouts, and assets
        for stmt in program.statements:
            if isinstance(stmt, AssetInclude):
                self.asset_includes.append(stmt)
            elif isinstance(stmt, DataDefinition):
                self.data_models[stmt.name] = stmt
            elif isinstance(stmt, ActionDefinitionWithParams):
                self.actions[stmt.name] = stmt
            elif isinstance(stmt, ActionDefinition):
                # Convert ActionDefinition to ActionDefinitionWithParams for consistency
                action_with_params = ActionDefinitionWithParams(
                    name=stmt.name,
                    parameters=[],
                    body=stmt.body,
                    return_type=getattr(stmt, 'return_type', None)
                )
                self.actions[stmt.name] = action_with_params
            elif isinstance(stmt, FragmentDefinition):
                self.fragments[stmt.name] = stmt
            elif isinstance(stmt, ScreenDefinition):
                self.screens[stmt.name] = stmt
            elif isinstance(stmt, FormDefinition):
                self.forms[stmt.name] = stmt
            elif isinstance(stmt, ModuleDefinition):
                # Process module contents
                for module_stmt in stmt.body:
                    if isinstance(module_stmt, AssetInclude):
                        self.asset_includes.append(module_stmt)
                    elif isinstance(module_stmt, DataDefinition):
                        self.data_models[module_stmt.name] = module_stmt
                    elif isinstance(module_stmt, ActionDefinitionWithParams):
                        self.actions[module_stmt.name] = module_stmt
                    elif isinstance(module_stmt, ActionDefinition):
                        # Convert ActionDefinition to ActionDefinitionWithParams for consistency
                        action_with_params = ActionDefinitionWithParams(
                            name=module_stmt.name,
                            parameters=[],
                            body=module_stmt.body,
                            return_type=getattr(module_stmt, 'return_type', None)
                        )
                        self.actions[module_stmt.name] = action_with_params
                    elif isinstance(module_stmt, FragmentDefinition):
                        self.fragments[module_stmt.name] = module_stmt
                    elif isinstance(module_stmt, ScreenDefinition):
                        self.screens[module_stmt.name] = module_stmt
                        # Recursively collect forms from screen children
                        self._collect_forms_from_children(module_stmt.children)
                    elif isinstance(module_stmt, FormDefinition):
                        self.forms[module_stmt.name] = module_stmt
        
        
        
        # Generate HTML document
        html_content = self.generate_html_document(program)
        
        return html_content
    
    def _collect_forms_from_children(self, children):
        """Recursively collect forms from layout children."""
        for child in children:
            if isinstance(child, FormDefinition):
                self.forms[child.name] = child
            elif isinstance(child, LayoutDefinition):
                # Recursively process nested layouts
                self._collect_forms_from_children(child.children)
    
    def generate_html_document(self, program: Program) -> str:
        """Generate complete HTML document."""
        self.emit("<!DOCTYPE html>")
        self.emit("<html lang=\"en\">")
        self.emit("<head>")
        self.indent_level += 1
        self.emit("<meta charset=\"UTF-8\">")
        self.emit("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">")
        self.emit("<title>Roelang Web App</title>")
        
        # Include external CSS files
        css_includes = [asset for asset in self.asset_includes if asset.asset_type == 'css']
        if css_includes or self.use_external_css:
            # Always include global.css if using external CSS
            if self.use_external_css:
                self.emit('<link rel="stylesheet" href="assets/global.css">')
            
            # Include any additional CSS files
            for asset in css_includes:
                self.emit(f'<link rel="stylesheet" href="{asset.asset_path}">')
        
        else:
            # Inline CSS for standalone HTML
            self.emit("<style>")
            self.indent_level += 1
            self.generate_css()
            self.indent_level -= 1
            self.emit("</style>")
        
        # Always include main.js for HTML target
        self.emit('<script src="assets/main.js"></script>')
        
        # Include external fonts
        font_includes = [asset for asset in self.asset_includes if asset.asset_type == 'font']
        if font_includes:
            self.emit("<style>")
            self.indent_level += 1
            for asset in font_includes:
                font_name = asset.asset_path.split('/')[-1].split('.')[0]
                self.emit(f"@font-face {{")
                self.indent_level += 1
                self.emit(f'font-family: "{font_name}";')
                self.emit(f'src: url("{asset.asset_path}");')
                self.indent_level -= 1
                self.emit("}")
            self.indent_level -= 1
            self.emit("</style>")
        self.indent_level -= 1
        self.emit("</head>")
        
        self.emit("<body>")
        self.indent_level += 1
        
        
        # Generate body content from collected screens and layouts
        # If we have screens, render them (which will use their layouts)
        if self.screens:
            for screen_name, screen_def in self.screens.items():
                self.generate_screen(screen_def)
        else:
            # Fallback: Generate standalone layouts if no screens
            for layout_name, layout_def in self.layouts.items():
                self.generate_layout(layout_def)
        
        # Generate all forms that were collected during the first pass
        for form_name, form_def in self.forms.items():
            self.generate_form(form_def)
        
        # Include external JavaScript files
        js_includes = [asset for asset in self.asset_includes if asset.asset_type == 'js']
        for asset in js_includes:
            self.emit(f'<script src="{asset.asset_path}"></script>')
        
        # Generate inline JavaScript
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
        # Generate unique ID
        if not hasattr(self, 'layout_counter'):
            self.layout_counter = 0
        self.layout_counter += 1
        layout_id = f"layout-{layout.name}-{self.layout_counter}"
        
        # Only use explicitly declared CSS classes
        classes = layout.css_classes if hasattr(layout, 'css_classes') and layout.css_classes else []
        
        # Build attributes - only add class attribute if there are explicit classes
        attrs = [f'id="{layout_id}"']
        if classes:
            attrs.append(f'class="{" ".join(classes)}"')
        
        # Add style if present
        if hasattr(layout, 'style') and layout.style:
            attrs.append(f'style="{self.escape_html(layout.style)}"')
        
        # Choose appropriate HTML tag based on layout type
        html_tag = self.get_html_tag_for_layout(layout)
        
        self.emit(f'<{html_tag} {" ".join(attrs)}>')
        self.indent_level += 1
        
        for child in layout.children:
            self.generate_component(child)
        
        self.indent_level -= 1
        self.emit(f"</{html_tag}>")
        self.emit("")
    
    def get_html_tag_for_layout(self, layout: LayoutDefinition) -> str:
        """Get appropriate HTML tag for a layout based on its type."""
        layout_type = getattr(layout, 'layout_type', 'div')
        
        # Map layout types to HTML tags
        tag_mapping = {
            'header': 'header',
            'main': 'main', 
            'footer': 'footer',
            'nav': 'nav',
            'section': 'section',
            'article': 'article',
            'aside': 'aside',
            'column': 'div',
            'row': 'div',
            'grid': 'div',
            'stack': 'div',
            'overlay': 'div'
        }
        
        return tag_mapping.get(layout_type, 'div')
    
    def generate_screen(self, screen: ScreenDefinition):
        """Generate HTML for a screen definition using fragments."""
        # Generate unique ID for the screen
        if not hasattr(self, 'screen_counter'):
            self.screen_counter = 0
        self.screen_counter += 1
        screen_id = f"screen-{screen.name}-{self.screen_counter}"
        
        # Build screen container attributes
        attrs = [f'id="{screen_id}"']
        if hasattr(screen, 'classes') and screen.classes:
            attrs.append(f'class="{" ".join(screen.classes)}"')
        if hasattr(screen, 'styles') and screen.styles:
            attrs.append(f'style="{screen.styles}"')
        
        self.emit(f'<div {" ".join(attrs)}>')
        self.indent()
        
        # Process each fragment reference in the screen
        for fragment_ref in screen.fragments:
            self.generate_fragment_reference(fragment_ref)
        
        self.dedent()
        self.emit('</div>')
    
    def generate_fragment_reference(self, fragment_ref: FragmentReference):
        """Generate HTML for a fragment reference with slot content."""
        # Look up the fragment definition
        if fragment_ref.fragment_name not in self.fragments:
            raise CodeGenError(f"Fragment '{fragment_ref.fragment_name}' not found")
        
        fragment = self.fragments[fragment_ref.fragment_name]
        
        # Generate fragment container
        fragment_id = f"fragment-{fragment.name}-{self.component_counter}"
        self.component_counter += 1
        
        # Build fragment container attributes
        attrs = [f'id="{fragment_id}"', f'data-fragment="{fragment.name}"']
        if hasattr(fragment, 'classes') and fragment.classes:
            attrs.append(f'class="{" ".join(fragment.classes)}"')
        if hasattr(fragment, 'styles') and fragment.styles:
            attrs.append(f'style="{fragment.styles}"')
        
        # Use semantic tag based on fragment name or default to div
        tag = self.get_semantic_tag_for_fragment(fragment.name)
        
        self.emit(f'<{tag} {" ".join(attrs)}>')
        self.indent()
        
        # Generate content for each slot in the fragment
        for slot in fragment.slots:
            self.generate_slot_with_content(slot, fragment_ref.slot_contents.get(slot.name, []))
        
        self.dedent()
        self.emit(f'</{tag}>')
    
    def get_semantic_tag_for_fragment(self, fragment_name: str) -> str:
        """Get appropriate semantic HTML tag based on fragment name."""
        semantic_mapping = {
            'header': 'header',
            'footer': 'footer', 
            'nav': 'nav',
            'navigation': 'nav',
            'main': 'main',
            'content': 'main',
            'sidebar': 'aside',
            'aside': 'aside',
            'article': 'article',
            'section': 'section'
        }
        
        # Check for partial matches
        for keyword, tag in semantic_mapping.items():
            if keyword in fragment_name.lower():
                return tag
        
        return 'div'  # Default fallback
    
    def generate_slot_with_content(self, slot: SlotComponent, content: List[ASTNode]):
        """Generate HTML for a slot with its assigned content."""
        slot_id = f"slot-{slot.name}-{self.component_counter}"
        self.component_counter += 1
        
        # Build slot attributes
        attrs = [f'id="{slot_id}"', f'data-slot="{slot.name}"']
        if hasattr(slot, 'classes') and slot.classes:
            attrs.append(f'class="{" ".join(slot.classes)}"')
        if hasattr(slot, 'styles') and slot.styles:
            attrs.append(f'style="{slot.styles}"')
        
        self.emit(f'<div {" ".join(attrs)}>')
        self.indent()
        
        # Generate content assigned to this slot, or default content if no assignment
        if content:
            for component in content:
                self.generate_component(component)
        elif slot.default_content:
            for component in slot.default_content:
                self.generate_component(component)
        
        self.dedent()
        self.emit('</div>')
    
    def generate_screen_with_layout(self, screen: ScreenDefinition, layout: LayoutDefinition):
        """Generate screen content within a layout, filling slots appropriately."""
        # Generate unique ID for the screen
        if not hasattr(self, 'screen_counter'):
            self.screen_counter = 0
        self.screen_counter += 1
        screen_id = f"screen-{screen.name}-{self.screen_counter}"
        
        # Create a copy of the layout but fill slots with screen content
        layout_id = f"layout-{layout.name}-{self.screen_counter}"
        
        # Only use explicitly declared CSS classes
        classes = layout.css_classes if hasattr(layout, 'css_classes') and layout.css_classes else []
        
        # Build attributes
        attrs = [f'id="{layout_id}"']
        if classes:
            attrs.append(f'class="{" ".join(classes)}"')
        
        # Choose appropriate HTML tag based on layout type
        html_tag = self.get_html_tag_for_layout(layout)
        
        self.emit(f'<{html_tag} {" ".join(attrs)}>')
        self.indent_level += 1
        
        # Process layout children, replacing slots with screen content
        for child in layout.children:
            self.generate_component_with_slot_filling(child, screen)
        
        self.indent_level -= 1
        self.emit(f"</{html_tag}>")
        self.emit("")
    
    def generate_component_with_slot_filling(self, component: ASTNode, screen: ScreenDefinition):
        """Generate a component, filling slots with screen content if found."""
        if isinstance(component, SlotComponent):
            # Fill this slot with screen content
            if component.name == "content" or component.name in screen.slot_contents:
                # Use specific slot content if available, otherwise use main screen content
                slot_content = screen.slot_contents.get(component.name, screen.children if component.name == "content" else [])
                for content_item in slot_content:
                    self.generate_component(content_item)
            else:
                # Use default slot content if no screen content for this slot
                for default_item in component.default_content:
                    self.generate_component(default_item)
        elif isinstance(component, LayoutDefinition):
            # For layout components (like containers), render them but process their children for slots
            self.generate_layout_with_slot_filling(component, screen)
        else:
            # Regular component
            self.generate_component(component)
    
    def generate_layout_with_slot_filling(self, layout: LayoutDefinition, screen: ScreenDefinition):
        """Generate a layout container but process its children for slot filling."""
        # Generate unique ID
        if not hasattr(self, 'layout_counter'):
            self.layout_counter = 0
        self.layout_counter += 1
        layout_id = f"layout-{layout.name}-{self.layout_counter}"
        
        # Only use explicitly declared CSS classes
        classes = layout.css_classes if hasattr(layout, 'css_classes') and layout.css_classes else []
        
        # Build attributes - only add class attribute if there are explicit classes
        attrs = [f'id="{layout_id}"']
        if classes:
            attrs.append(f'class="{" ".join(classes)}"')
        
        # Add style if present
        if hasattr(layout, 'style') and layout.style:
            attrs.append(f'style="{self.escape_html(layout.style)}"')
        
        # Choose appropriate HTML tag based on layout type
        html_tag = self.get_html_tag_for_layout(layout)
        
        self.emit(f'<{html_tag} {" ".join(attrs)}>')
        self.indent_level += 1
        
        # Process children with slot filling
        for child in layout.children:
            self.generate_component_with_slot_filling(child, screen)
        
        self.indent_level -= 1
        self.emit(f"</{html_tag}>")
        self.emit("")
    
    def generate_screen_standalone(self, screen: ScreenDefinition):
        """Generate screen without a layout."""
        # Generate unique ID
        if not hasattr(self, 'screen_counter'):
            self.screen_counter = 0
        self.screen_counter += 1
        screen_id = f"screen-{screen.name}-{self.screen_counter}"
        
        # Build attributes
        classes = screen.css_classes if hasattr(screen, 'css_classes') and screen.css_classes else []
        attrs = [f'id="{screen_id}"']
        if classes:
            attrs.append(f'class="{" ".join(classes)}"')
        
        self.emit(f'<div {" ".join(attrs)}>')
        self.indent_level += 1
        
        # Generate screen content directly
        for child in screen.children:
            self.generate_component(child)
        
        self.indent_level -= 1
        self.emit("</div>")
        self.emit("")
    
    def generate_form(self, form: FormDefinition):
        """Generate HTML for a form definition."""
        form_id = f"form-{form.name}"
        
        self.emit(f'<div>')
        self.indent_level += 1
        self.emit(f'<form id="{form_id}">')
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
        elif isinstance(component, TextComponent):
            self.generate_text(component)
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
        elif isinstance(component, ImageComponent):
            self.generate_image(component)
        elif isinstance(component, VideoComponent):
            self.generate_video(component)
        elif isinstance(component, AudioComponent):
            self.generate_audio(component)
        elif isinstance(component, SlotComponent):
            self.generate_slot(component)
    
    def generate_title(self, title: TitleComponent):
        """Generate HTML for title component."""
        # Build attributes
        attrs = []
        
        # Add classes if present
        if hasattr(title, 'classes') and title.classes:
            attrs.append(f'class="{" ".join(title.classes)}"')
        
        # Add styles if present
        if hasattr(title, 'styles') and title.styles:
            attrs.append(f'style="{title.styles}"')
        
        # Generate with attributes if any, or plain if none
        if attrs:
            self.emit(f'<h2 {" ".join(attrs)}>{self.escape_html(title.text)}</h2>')
        else:
            self.emit(f'<h2>{self.escape_html(title.text)}</h2>')
    
    def generate_text(self, text: TextComponent):
        """Generate HTML for text component."""
        # Build attributes
        attrs = []
        
        # Add classes if present
        if hasattr(text, 'classes') and text.classes:
            attrs.append(f'class="{" ".join(text.classes)}"')
        
        # Add styles if present
        if hasattr(text, 'styles') and text.styles:
            attrs.append(f'style="{text.styles}"')
        
        # Generate with attributes if any, or plain if none
        if attrs:
            self.emit(f'<p {" ".join(attrs)}>{self.escape_html(text.text)}</p>')
        else:
            self.emit(f'<p>{self.escape_html(text.text)}</p>')
    
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
        
        # Get label from attributes or generate one
        label = ""
        placeholder = ""
        name = input_id  # Default name to id
        required = False
        disabled = False
        readonly = False
        
        # Extract attributes from the component
        for attr in input_comp.attributes:
            if isinstance(attr, AttributeDefinition):
                if attr.name == 'label':
                    label = attr.value
                elif attr.name == 'placeholder':
                    placeholder = attr.value
                elif attr.name == 'name':
                    name = attr.value
                elif attr.name == 'required':
                    required = attr.value.lower() == 'true'
                elif attr.name == 'disabled':
                    disabled = attr.value.lower() == 'true'
                elif attr.name == 'readonly':
                    readonly = attr.value.lower() == 'true'
            elif isinstance(attr, ValidationAttribute):
                self.validation_rules.add(attr.validation_type)
                if attr.validation_type == 'required':
                    required = True
        
        # Wrap in div container
        self.emit('<div class="form-group">')
        self.indent_level += 1
        
        # Generate label if present
        if label:
            self.emit(f'<label for="{input_id}">{self.escape_html(label)}</label>')
        
        # Generate input with all attributes
        input_attrs = [
            f'id="{input_id}"',
            f'name="{name}"',
            f'type="{input_comp.input_type}"'
        ]
        
        if placeholder:
            input_attrs.append(f'placeholder="{self.escape_html(placeholder)}"')
        elif input_comp.input_type == 'email' and not placeholder:
            input_attrs.append('placeholder="Enter your email"')
        elif input_comp.input_type == 'password' and not placeholder:
            input_attrs.append('placeholder="Enter your password"')
        
        if required:
            input_attrs.append('required')
        if disabled:
            input_attrs.append('disabled')
        if readonly:
            input_attrs.append('readonly')
        
        # Add validation patterns
        if input_comp.input_type == 'email':
            input_attrs.append('pattern="[^@]+@[^@]+\.[^@]+"')
        
        self.emit(f'<input {" ".join(input_attrs)}>')
        self.emit(f'<div id="{input_id}-error" class="error-message" style="display: none;"></div>')
        
        self.indent_level -= 1
        self.emit('</div>')
    
    def generate_textarea(self, textarea: TextareaComponent):
        """Generate HTML for textarea component."""
        self.component_counter += 1
        textarea_id = f"textarea-{self.component_counter}"
        
        if textarea.binding:
            self.bindings[textarea_id] = textarea.binding
        
        # Get attributes from the new fields and attributes
        rows = textarea.rows if hasattr(textarea, 'rows') and textarea.rows else 4
        placeholder = textarea.placeholder if hasattr(textarea, 'placeholder') and textarea.placeholder else ""
        label = textarea.label if hasattr(textarea, 'label') and textarea.label else ""
        name = textarea_id  # Default name to id
        required = False
        disabled = False
        readonly = False
        cols = None
        maxlength = None
        
        # Extract additional attributes
        for attr in textarea.attributes:
            if isinstance(attr, AttributeDefinition):
                if attr.name == 'name':
                    name = attr.value
                elif attr.name == 'required':
                    required = attr.value.lower() == 'true'
                elif attr.name == 'disabled':
                    disabled = attr.value.lower() == 'true'
                elif attr.name == 'readonly':
                    readonly = attr.value.lower() == 'true'
                elif attr.name == 'cols':
                    cols = attr.value
                elif attr.name == 'maxlength':
                    maxlength = attr.value
        
        # Wrap in div container
        self.emit('<div class="form-group">')
        self.indent_level += 1
        
        # Generate label if present
        if label:
            self.emit(f'<label for="{textarea_id}">{self.escape_html(label)}</label>')
        
        # Generate textarea with all attributes
        attrs = [
            f'id="{textarea_id}"',
            f'name="{name}"',
            f'rows="{rows}"'
        ]
        
        if cols:
            attrs.append(f'cols="{cols}"')
        if placeholder:
            attrs.append(f'placeholder="{self.escape_html(placeholder)}"')
        if maxlength:
            attrs.append(f'maxlength="{maxlength}"')
        if required:
            attrs.append('required')
        if disabled:
            attrs.append('disabled')
        if readonly:
            attrs.append('readonly')
        
        self.emit(f'<textarea {" ".join(attrs)}></textarea>')
        
        self.indent_level -= 1
        self.emit('</div>')
    
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
        
        # Get attributes from the new fields and attributes
        label = dropdown.label if hasattr(dropdown, 'label') and dropdown.label else ""
        name = select_id  # Default name to id
        required = False
        disabled = False
        multiple = False
        size = None
        
        # Extract additional attributes
        for attr in dropdown.attributes:
            if isinstance(attr, AttributeDefinition):
                if attr.name == 'name':
                    name = attr.value
                elif attr.name == 'required':
                    required = attr.value.lower() == 'true'
                elif attr.name == 'disabled':
                    disabled = attr.value.lower() == 'true'
                elif attr.name == 'multiple':
                    multiple = attr.value.lower() == 'true'
                elif attr.name == 'size':
                    size = attr.value
        
        # Wrap in div container
        self.emit('<div class="form-group">')
        self.indent_level += 1
        
        # Generate label if present
        if label:
            self.emit(f'<label for="{select_id}">{self.escape_html(label)}</label>')
        
        # Generate select with all attributes
        select_attrs = [
            f'id="{select_id}"',
            f'name="{name}"'
        ]
        
        if required:
            select_attrs.append('required')
        if disabled:
            select_attrs.append('disabled')
        if multiple:
            select_attrs.append('multiple')
        if size:
            select_attrs.append(f'size="{size}"')
        
        self.emit(f'<select {" ".join(select_attrs)}>')
        self.indent_level += 1
        
        # Handle both old structure (Literal objects) and new structure (string list)
        if hasattr(dropdown, 'options') and dropdown.options:
            if isinstance(dropdown.options, list):
                for option in dropdown.options:
                    if isinstance(option, str):
                        # New format: simple string options
                        value = self.escape_html(option)
                        self.emit(f'<option value="{value}">{value}</option>')
                    elif isinstance(option, Literal):
                        # Old format: Literal objects
                        value = self.escape_html(str(option.value))
                        self.emit(f'<option value="{value}">{value}</option>')
        
        self.indent_level -= 1
        self.emit("</select>")
        
        self.indent_level -= 1
        self.emit('</div>')
    
    def generate_toggle(self, toggle: ToggleComponent):
        """Generate HTML for toggle component."""
        self.component_counter += 1
        toggle_id = f"toggle-{self.component_counter}"
        
        if toggle.binding:
            self.bindings[toggle_id] = toggle.binding
        
        self.emit('<div>')
        self.indent_level += 1
        self.emit(f'<input type="checkbox" id="{toggle_id}">')
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
        
        self.emit('<div>')
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
        
        self.emit('<div>')
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
            'type="button"'
        ]
        
        if button.action:
            # Check if this button is inside a form and should submit form data
            # Look for actions that require form data (have parameters) OR 
            # actions with common form-related names like save, submit, update
            action_needs_form_data = False
            form_action_keywords = ['save', 'submit', 'update', 'create', 'register', 'login']
            
            if button.action in self.actions:
                action_def = self.actions[button.action]
                if hasattr(action_def, 'parameters') and action_def.parameters:
                    action_needs_form_data = True
            
            # Also check if action name suggests form submission
            if any(keyword in button.action.lower() for keyword in form_action_keywords):
                action_needs_form_data = True
            
            if action_needs_form_data:
                # This is a form submit button that needs data
                # Find the form this button belongs to
                form_names = list(self.forms.keys())
                if form_names:
                    form_name = form_names[0]  # Use the first form found
                    handler_name = f"submit_{form_name}"
                    button_attrs.append(f'onclick="{handler_name}(\'{button.action}\')"')
                else:
                    # Fallback to regular action
                    button_attrs.append(f'onclick="{button.action}()"')
            else:
                # Regular action button
                button_attrs.append(f'onclick="{button.action}()"')
        
        self.emit(f'<button {" ".join(button_attrs)}>{self.escape_html(button.text)}</button>')
    
    def generate_image(self, image: ImageComponent):
        """Generate HTML for image component."""
        self.component_counter += 1
        image_id = f"image-{self.component_counter}"
        
        img_attrs = [
            f'id="{image_id}"',
            f'src="{self.escape_html(image.src)}"'
        ]
        
        if image.alt:
            img_attrs.append(f'alt="{self.escape_html(image.alt)}"')
        else:
            img_attrs.append('alt=""')
        
        # Add CSS classes
        classes = image.css_classes if image.css_classes else []
        if classes:
            img_attrs.append(f'class="{" ".join(classes)}"')
        
        self.emit(f'<img {" ".join(img_attrs)}>')
    
    def generate_video(self, video: VideoComponent):
        """Generate HTML for video component."""
        self.component_counter += 1
        video_id = f"video-{self.component_counter}"
        
        video_attrs = [
            f'id="{video_id}"',
            f'src="{self.escape_html(video.src)}"'
        ]
        
        if video.controls:
            video_attrs.append('controls')
        if video.autoplay:
            video_attrs.append('autoplay')
        if video.loop:
            video_attrs.append('loop')
        if video.muted:
            video_attrs.append('muted')
        
        # Add CSS classes
        classes = video.css_classes if video.css_classes else []
        if classes:
            video_attrs.append(f'class="{" ".join(classes)}"')
        
        self.emit(f'<video {" ".join(video_attrs)}>')
        self.emit('  Your browser does not support the video tag.')
        self.emit('</video>')
    
    def generate_audio(self, audio: AudioComponent):
        """Generate HTML for audio component."""
        self.component_counter += 1
        audio_id = f"audio-{self.component_counter}"
        
        audio_attrs = [
            f'id="{audio_id}"',
            f'src="{self.escape_html(audio.src)}"'
        ]
        
        if audio.controls:
            audio_attrs.append('controls')
        if audio.autoplay:
            audio_attrs.append('autoplay')
        if audio.loop:
            audio_attrs.append('loop')
        
        # Add CSS classes
        classes = audio.css_classes if audio.css_classes else []
        if classes:
            audio_attrs.append(f'class="{" ".join(classes)}"')
        
        self.emit(f'<audio {" ".join(audio_attrs)}>')
        self.emit('  Your browser does not support the audio element.')
        self.emit('</audio>')
    
    def generate_slot(self, slot: SlotComponent):
        """Generate HTML for slot component - renders default content."""
        # Generate unique ID
        self.component_counter += 1
        slot_id = f"slot-{slot.name}-{self.component_counter}"
        
        # Build attributes
        classes = slot.css_classes if slot.css_classes else []
        attrs = [f'id="{slot_id}"', f'data-slot="{slot.name}"']
        if classes:
            attrs.append(f'class="{" ".join(classes)}"')
        
        self.emit(f'<div {" ".join(attrs)}>')
        self.indent_level += 1
        
        # Render default content
        for content_item in slot.default_content:
            self.generate_component(content_item)
        
        self.indent_level -= 1
        self.emit('</div>')
    
    def generate_javascript(self, program: Program):
        """Generate JavaScript for data binding and validation."""
        self.emit("// Roelang Application Code")
        self.emit("")
        
        # Generate data models with state persistence
        self.emit("// Data Models")
        for name, data_def in self.data_models.items():
            self.generate_data_model_js(name, data_def)
        
        self.emit("")
        
        # Generate initialization object
        self.emit("// Initialize Roelang")
        self.emit("window.RoelangInit = {")
        self.indent_level += 1
        
        # Generate bindings initialization
        self.emit("initializeBindings: function() {")
        self.indent_level += 1
        
        # Create model instances with storage support
        for name, data_def in self.data_models.items():
            storage_type = data_def.storage_type if hasattr(data_def, 'storage_type') else None
            storage_arg = f"'{storage_type}'" if storage_type else "null"
            self.emit(f"Roelang.DataBinding.createModel('{name}', {name}, {storage_arg});")
        
        self.emit("")
        
        # Set up data bindings
        for component_id, binding_target in self.bindings.items():
            self.emit(f"Roelang.DataBinding.bind('{component_id}', '{binding_target}');")
        
        self.indent_level -= 1
        self.emit("}")
        
        self.indent_level -= 1
        self.emit("};")
        self.emit("")
        
        # Generate action functions
        self.emit("// Actions")
        for name, action in self.actions.items():
            self.generate_action_js(name, action)
        
        # Generate top-level statements (like API calls)
        self.emit("")
        self.emit("// Top-level statements")
        for stmt in program.statements:
            if isinstance(stmt, ApiCallStatement):
                self.generate_api_call_js(stmt)
        
        # Generate form submission handlers
        self.emit("")
        self.emit("// Form Handlers")
        self.generate_form_handlers()
    
    def generate_form_handlers(self):
        """Generate form submission handlers."""
        for form_name, form_def in self.forms.items():
            form_id = f"form-{form_name}"
            
            # Build validation rules for this form
            self.emit(f"// Handler for {form_name}")
            self.emit(f"window.submit_{form_name} = async function(actionName) {{")
            self.indent_level += 1
            
            # Generate validation rules object
            self.emit("const validationRules = {")
            self.indent_level += 1
            
            # Collect validation rules from components
            for component_id, binding_target in self.bindings.items():
                # Add basic validation rules
                self.emit(f"'{component_id}': [")
                self.indent_level += 1
                self.emit("{ type: 'required', message: 'This field is required' },")
                if 'email' in component_id.lower():
                    self.emit("{ type: 'email', message: 'Please enter a valid email' }")
                self.indent_level -= 1
                self.emit("],")
            
            self.indent_level -= 1
            self.emit("};")
            self.emit("")
            
            # Collect form data
            self.emit("const formData = {};")
            self.emit("let isValid = true;")
            self.emit("")
            
            # Generate form data collection based on bindings
            for element_id, binding_target in self.bindings.items():
                if '.' in binding_target:
                    model_name, field_name = binding_target.split('.', 1)
                    if model_name in self.data_models:
                        data_def = self.data_models[model_name]
                        field = next((f for f in data_def.fields if f.name == field_name), None)
                        
                        if field:
                            safe_element_var = element_id.replace('-', '_')
                            self.emit(f"const {safe_element_var}Element = document.getElementById('{element_id}');")
                            self.emit(f"if ({safe_element_var}Element) {{")
                            self.indent_level += 1
                            
                            # Generate data collection based on field type
                            if field.type.lower() == 'boolean':
                                self.emit(f"formData.{field_name} = {safe_element_var}Element.checked;")
                            elif field.type.lower() in ['number', 'int']:
                                self.emit(f"formData.{field_name} = parseInt({safe_element_var}Element.value) || 0;")
                            else:
                                self.emit(f"formData.{field_name} = {safe_element_var}Element.value || '';")
                            
                            self.indent_level -= 1
                            self.emit("}")
                            self.emit("")
            
            # Display form data for verification
            self.emit("// Display form data for verification")
            self.emit("const displayElement = document.querySelector('.form-result-display');")
            self.emit("if (displayElement) {")
            self.indent_level += 1
            self.emit("let displayText = 'Form Data: ';")
            self.emit("for (const [key, value] of Object.entries(formData)) {")
            self.indent_level += 1
            self.emit("displayText += `${key}: \"${value}\", `;")
            self.indent_level -= 1
            self.emit("}")
            self.emit("const titleElement = displayElement.querySelector('.title');")
            self.emit("if (titleElement) {")
            self.indent_level += 1
            self.emit("titleElement.textContent = displayText.slice(0, -2); // Remove trailing comma")
            self.emit("titleElement.className = 'title text-success';")
            self.indent_level -= 1
            self.emit("}")
            self.indent_level -= 1
            self.emit("}")
            self.emit("")
            
            # Call the action
            self.emit("// Call the action")
            self.emit("if (typeof window[actionName] === 'function') {")
            self.indent_level += 1
            self.emit("return await window[actionName](formData);")
            self.indent_level -= 1
            self.emit("} else {")
            self.indent_level += 1
            self.emit("console.error('Action not found:', actionName);")
            self.emit("return null;")
            self.indent_level -= 1
            self.emit("}")
            self.indent_level -= 1
            self.emit("}")
            self.emit("")
    
    def generate_data_model_js(self, name: str, data_def: DataDefinition):
        """Generate JavaScript data model class."""
        self.emit(f"class {name} {{")
        self.indent_level += 1
        
        # Constructor with default values
        self.emit("constructor() {")
        self.indent_level += 1
        for field in data_def.fields:
            default_value = self.get_default_value_js(field.type)
            self.emit(f"this.{field.name} = {default_value};")
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
            # Create safe JavaScript variable name by replacing hyphens with underscores
            safe_var_name = component_id.replace('-', '_')
            self.emit(f"const {safe_var_name}_element = document.getElementById('{component_id}');")
            self.emit(f"if ({safe_var_name}_element) {{")
            self.indent_level += 1
            self.emit(f"{safe_var_name}_element.addEventListener('input', function() {{")
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
                            safe_element_var = element_id.replace('-', '_')
                            self.emit(f"const {safe_element_var}Element = document.getElementById('{element_id}');")
                            self.emit(f"if ({safe_element_var}Element) {{")
                            self.indent_level += 1
                            
                            # Generate data collection based on field type
                            if field.type.lower() == 'boolean':
                                self.emit(f"formData.{field_name} = {safe_element_var}Element.checked;")
                            elif field.type.lower() in ['number', 'int']:
                                self.emit(f"formData.{field_name} = parseInt({safe_element_var}Element.value) || 0;")
                            else:
                                self.emit(f"formData.{field_name} = {safe_element_var}Element.value;")
                            
                            # Generate validation for this field
                            self.emit(f"// Validate {field_name}")
                            self.emit(f"if (!validate{field_name.title()}({safe_element_var}Element)) {{")
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
        
        self.emit("async function handleFormSubmit(actionName, formId) {")
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
                        safe_element_var = element_id.replace('-', '_')
                        self.emit(f"const {safe_element_var}Element = document.getElementById('{element_id}');")
                        self.emit(f"if ({safe_element_var}Element) {{")
                        self.indent_level += 1
                        
                        # Generate data collection based on field type
                        if field.type.lower() == 'boolean':
                            self.emit(f"formData.{field_name} = {safe_element_var}Element.checked;")
                        elif field.type.lower() in ['number', 'int']:
                            self.emit(f"formData.{field_name} = parseInt({safe_element_var}Element.value) || 0;")
                        else:
                            self.emit(f"formData.{field_name} = {safe_element_var}Element.value || '';")
                        
                        # Generate validation for this field
                        self.emit(f"// Validate {field_name}")
                        self.emit(f"if (!validate{field_name.title()}({safe_element_var}Element)) {{")
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
        
        # Add form data display for verification
        self.emit("// Display form data for verification")
        self.emit("const displayElement = document.querySelector('.form-result-display');")
        self.emit("if (displayElement) {")
        self.indent_level += 1
        self.emit("let displayText = 'Form Data: ';")
        self.emit("for (const [key, value] of Object.entries(formData)) {")
        self.indent_level += 1
        self.emit("displayText += `${key}: \"${value}\", `;")
        self.indent_level -= 1
        self.emit("}")
        self.emit("const titleElement = displayElement.querySelector('.title');")
        self.emit("if (titleElement) {")
        self.indent_level += 1
        self.emit("titleElement.textContent = displayText.slice(0, -2); // Remove trailing comma")
        self.emit("titleElement.className = 'title text-success';")
        self.indent_level -= 1
        self.emit("}")
        self.indent_level -= 1
        self.emit("}")
        self.emit("")
        
        self.emit("// Call the appropriate action")
        self.emit("if (typeof window[actionName] === 'function') {")
        self.indent_level += 1
        self.emit("await window[actionName](formData);")
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
        
        # Check if action contains API calls (needs to be async)
        has_api_calls = self._has_api_calls(action.body)
        async_keyword = "async " if has_api_calls else ""
        
        self.emit(f"{async_keyword}function {name}({param_list}) {{")
        self.indent_level += 1
        
        # Generate action body
        for stmt in action.body:
            self.generate_js_statement(stmt)
        
        self.indent_level -= 1
        self.emit("}")
        self.emit("")
    
    def _has_api_calls(self, statements: List[ASTNode]) -> bool:
        """Check if any statement in the list is an API call."""
        for stmt in statements:
            if isinstance(stmt, ApiCallStatement):
                return True
            # Could check nested statements in if/while/etc. here if needed
        return False
    
    def generate_js_statement(self, stmt: ASTNode):
        """Generate JavaScript statement."""
        if isinstance(stmt, DisplayStatement):
            expr = self.generate_js_expression(stmt.expression)
            self.emit(f"console.log({expr});")
        elif isinstance(stmt, ReturnStatement):
            expr = self.generate_js_expression(stmt.expression)
            self.emit(f"return {expr};")
        elif isinstance(stmt, ApiCallStatement):
            self.generate_api_call_js(stmt)
    
    def generate_api_call_js(self, api_call: ApiCallStatement):
        """Generate JavaScript Fetch API call."""
        # Generate async function call with fetch
        endpoint = api_call.endpoint
        method = api_call.method.upper()
        
        # Build fetch options
        fetch_options = {
            'method': method,
            'headers': {}
        }
        
        # Add headers
        for header in api_call.headers:
            header_name = header.name
            header_value = header.value
            # Remove quotes if present
            if header_value.startswith('"') and header_value.endswith('"'):
                header_value = header_value[1:-1]
            fetch_options['headers'][header_name] = header_value
        
        # Add Content-Type for POST/PUT requests with payload
        if api_call.payload and method in ['POST', 'PUT', 'PATCH']:
            fetch_options['headers']['Content-Type'] = 'application/json'
        
        # Build fetch call
        self.emit("try {")
        self.indent_level += 1
        
        # Construct fetch options
        self.emit("const fetchOptions = {")
        self.indent_level += 1
        self.emit(f"method: '{method}',")
        
        if fetch_options['headers']:
            self.emit("headers: {")
            self.indent_level += 1
            for header_name, header_value in fetch_options['headers'].items():
                self.emit(f"'{header_name}': '{self.escape_js_string(header_value)}',")
            self.indent_level -= 1
            self.emit("},")
        
        # Add body for POST/PUT requests
        if api_call.payload and method in ['POST', 'PUT', 'PATCH']:
            self.emit(f"body: JSON.stringify({api_call.payload})")
        
        self.indent_level -= 1
        self.emit("};")
        self.emit("")
        
        # Make the fetch call
        self.emit(f"const response = await fetch('{endpoint}', fetchOptions);")
        self.emit("")
        
        # Handle response
        self.emit("if (!response.ok) {")
        self.indent_level += 1
        self.emit("throw new Error(`HTTP error! status: ${response.status}`);")
        self.indent_level -= 1
        self.emit("}")
        self.emit("")
        
        # Parse response and assign to variable
        if api_call.response_variable:
            self.emit(f"const {api_call.response_variable} = await response.json();")
            self.emit(f"console.log('API Response:', {api_call.response_variable});")
        else:
            self.emit("const apiResponse = await response.json();")
            self.emit("console.log('API Response:', apiResponse);")
        
        self.indent_level -= 1
        self.emit("} catch (error) {")
        self.indent_level += 1
        self.emit("console.error('API call failed:', error);")
        if api_call.response_variable:
            self.emit(f"const {api_call.response_variable} = null;")
        self.indent_level -= 1
        self.emit("}")
        self.emit("")
    
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
    
    def get_css_classes(self, component: ASTNode, default_classes: List[str] = None) -> str:
        """Get CSS class attribute for a component."""
        classes = default_classes if default_classes else []
        if hasattr(component, 'css_classes') and component.css_classes:
            classes.extend(component.css_classes)
        return " ".join(classes) if classes else ""
    
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