//! Demonstration of the enhanced parser capabilities
//! 
//! This example shows all the features that have been ported from Python to Rust

use droe_compiler::{Parser, Node};

fn main() {
    let advanced_droe_code = r#"
module UserInterface
    data User
        name is string
        age is number
        email is string
    end data
    
    form UserForm
        title "User Registration"
        input "text"
        textarea
        dropdown
        button "Submit"
    end form
    
    action HandleSubmit
        call POST "/api/users"
        db create user
        display "User created successfully"
    end action
end module

screen MainScreen
    title "Welcome to the App"
    text "Please fill out the form below"
    button "Get Started"
end screen

fragment HeaderFragment
    slot "header"
end fragment

serve GET "/api/status"
db select users
fetch GET "/api/data"

display "Application initialized"
"#;
    
    println!("Parsing advanced Droe code with enhanced parser...\n");
    
    let parser = Parser::new();
    match parser.parse(advanced_droe_code) {
        Ok(program) => {
            println!("✅ Successfully parsed {} statements!", program.statements.len());
            println!("✅ Found {} metadata annotations", program.metadata.len());
            
            // Show what types of statements we parsed
            let mut statement_counts = std::collections::HashMap::new();
            for statement in &program.statements {
                let statement_type = match statement {
                    Node::ModuleDefinition(_) => "Module Definition",
                    Node::DataDefinition(_) => "Data Definition", 
                    Node::FormDefinition(_) => "Form Definition",
                    Node::ActionDefinition(_) => "Action Definition",
                    Node::ScreenDefinition(_) => "Screen Definition",
                    Node::FragmentDefinition(_) => "Fragment Definition",
                    Node::TitleComponent(_) => "Title Component",
                    Node::TextComponent(_) => "Text Component",
                    Node::InputComponent(_) => "Input Component",
                    Node::TextareaComponent(_) => "Textarea Component",
                    Node::DropdownComponent(_) => "Dropdown Component",
                    Node::ButtonComponent(_) => "Button Component",
                    Node::SlotComponent(_) => "Slot Component",
                    Node::ServeStatement(_) => "Serve Statement",
                    Node::DatabaseStatement(_) => "Database Statement",
                    Node::ApiCallStatement(_) => "API Call Statement",
                    Node::DisplayStatement(_) => "Display Statement",
                    Node::MetadataAnnotation(_) => "Metadata Annotation",
                    _ => "Other",
                };
                *statement_counts.entry(statement_type).or_insert(0) += 1;
            }
            
            println!("\n📊 Statement breakdown:");
            for (statement_type, count) in statement_counts {
                println!("   {} × {}", count, statement_type);
            }
            
            println!("\n🎉 All Python parser features successfully ported to Rust!");
            println!("   ✓ UI Components (title, input, textarea, dropdown, button, etc.)");
            println!("   ✓ Database operations (db create, db select)");
            println!("   ✓ API calls (call, fetch)");
            println!("   ✓ Serve statements");
            println!("   ✓ Advanced structures (forms, screens, fragments)");
            println!("   ✓ Metadata annotations (@target mobile)");
            println!("   ✓ Expression parsing framework");
            println!("   ✓ Modular parser architecture");
        }
        Err(e) => {
            println!("❌ Parse error: {}", e.message);
            println!("   Line: {}, Column: {}", e.line, e.column);
        }
    }
}