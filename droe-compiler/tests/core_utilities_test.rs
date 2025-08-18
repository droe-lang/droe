//! Test suite for core utility functions

use droe_compiler::codegen::utils::{CoreUtilities, MathUtils, StringUtils, FormattingUtils};
use droe_compiler::codegen_base::CoreLibraries;

#[test]
fn test_math_utils_creation() {
    let math_utils = MathUtils::new();
    assert!(math_utils.get_functions().contains_key("math_abs_i32"));
    assert!(math_utils.get_functions().contains_key("math_decimal_multiply"));
    assert_eq!(math_utils.get_functions().len(), 8);
}

#[test]
fn test_string_utils_creation() {
    let string_utils = StringUtils::new();
    assert!(string_utils.get_functions().contains_key("print_no_newline"));
    assert!(string_utils.get_functions().contains_key("string_concat"));
    assert_eq!(string_utils.get_functions().len(), 6);
}

#[test]
fn test_formatting_utils_creation() {
    let formatting_utils = FormattingUtils::new();
    assert!(formatting_utils.get_functions().contains_key("format_date"));
    assert!(formatting_utils.get_functions().contains_key("format_decimal"));
    assert_eq!(formatting_utils.get_functions().len(), 8);
}

#[test]
fn test_core_utilities_integration() {
    let core_utils = CoreUtilities::new();
    
    // Test function lookup across all utilities
    assert!(core_utils.has_function("math_abs_i32"));
    assert!(core_utils.has_function("string_concat"));
    assert!(core_utils.has_function("format_date"));
    assert!(!core_utils.has_function("nonexistent_function"));
}

#[test]
fn test_core_libraries_integration() {
    let mut core_libs = CoreLibraries::new();
    
    // Test enabling libraries
    core_libs.enable("math_utils");
    core_libs.enable("string_utils");
    core_libs.enable("formatting");
    
    assert!(core_libs.is_enabled("math_utils"));
    assert!(core_libs.is_enabled("string_utils"));
    assert!(core_libs.is_enabled("formatting"));
    
    // Test WASM import generation
    let wasm_imports = core_libs.get_wasm_imports();
    assert!(!wasm_imports.is_empty());
    assert!(wasm_imports.len() > 20); // Should have all function imports
    
    // Check that specific imports exist
    let has_math_abs = wasm_imports.iter().any(|import| import.contains("math_abs_i32"));
    let has_string_concat = wasm_imports.iter().any(|import| import.contains("string_concat"));
    let has_format_date = wasm_imports.iter().any(|import| import.contains("format_date"));
    
    assert!(has_math_abs);
    assert!(has_string_concat);
    assert!(has_format_date);
}

#[test]
fn test_js_runtime_function_generation() {
    let mut core_libs = CoreLibraries::new();
    core_libs.enable("math_utils");
    core_libs.enable("string_utils");
    core_libs.enable("formatting");
    
    let js_functions = core_libs.get_js_runtime_functions();
    assert!(!js_functions.is_empty());
    
    // Check that JavaScript implementations exist
    assert!(js_functions.contains_key("math_abs_i32"));
    assert!(js_functions.contains_key("string_concat"));
    assert!(js_functions.contains_key("format_date"));
    
    // Verify JavaScript functions have proper structure
    let math_abs_js = js_functions.get("math_abs_i32").unwrap();
    assert!(math_abs_js.contains("Math.abs"));
    
    let string_concat_js = js_functions.get("string_concat").unwrap();
    assert!(string_concat_js.contains("TextDecoder"));
    
    let format_date_js = js_functions.get("format_date").unwrap();
    assert!(format_date_js.contains("new Date"));
}

#[test]
fn test_math_constants() {
    let math_utils = MathUtils::new();
    let constants = math_utils.get_constants();
    
    assert_eq!(constants.get("MATH_PI"), Some(&314)); // π * 100
    assert_eq!(constants.get("MATH_E"), Some(&272));  // e * 100
    assert_eq!(constants.get("MATH_SQRT2"), Some(&141)); // √2 * 100
    assert_eq!(constants.get("MATH_SQRT3"), Some(&173)); // √3 * 100
}

#[test]
fn test_formatting_patterns() {
    let formatting_utils = FormattingUtils::new();
    
    // Test date patterns
    let date_patterns = formatting_utils.get_date_patterns();
    assert!(date_patterns.contains_key("MM/dd/yyyy"));
    assert!(date_patterns.contains_key("iso"));
    assert!(date_patterns.contains_key("long"));
    
    // Test decimal patterns
    let decimal_patterns = formatting_utils.get_decimal_patterns();
    assert!(decimal_patterns.contains_key("0.00"));
    assert!(decimal_patterns.contains_key("#,##0.00"));
    assert!(decimal_patterns.contains_key("$0.00"));
    
    // Test number patterns
    let number_patterns = formatting_utils.get_number_patterns();
    assert!(number_patterns.contains_key("#,##0"));
    assert!(number_patterns.contains_key("hex"));
    assert!(number_patterns.contains_key("bin"));
}

#[test]
fn test_wasm_import_format() {
    let math_utils = MathUtils::new();
    let wasm_imports = math_utils.get_wasm_imports();
    
    // Find a specific import to test format
    let abs_import = wasm_imports.iter()
        .find(|import| import.contains("math_abs_i32"))
        .unwrap();
    
    // Check proper WASM import format
    assert!(abs_import.starts_with("(import \"env\" \"math_abs_i32\""));
    assert!(abs_import.contains("(func $math_abs_i32"));
    assert!(abs_import.contains("(param i32)"));
    assert!(abs_import.contains("(result i32)"));
    assert!(abs_import.ends_with(")"));
}

#[test]
fn test_function_descriptions() {
    let core_utils = CoreUtilities::new();
    
    let math_desc = core_utils.get_function_description("math_abs_i32");
    assert!(math_desc.is_some());
    assert!(math_desc.unwrap().contains("Absolute value"));
    
    let string_desc = core_utils.get_function_description("string_concat");
    assert!(string_desc.is_some());
    assert!(string_desc.unwrap().contains("Concatenate"));
    
    let format_desc = core_utils.get_function_description("format_date");
    assert!(format_desc.is_some());
    assert!(format_desc.unwrap().contains("Format date"));
    
    let no_desc = core_utils.get_function_description("nonexistent_function");
    assert!(no_desc.is_none());
}

#[test]
fn test_function_summary() {
    let core_utils = CoreUtilities::new();
    let summary = core_utils.get_function_summary();
    
    assert!(!summary.is_empty());
    
    // Should contain section headers
    assert!(summary.iter().any(|line| line.contains("Math Functions")));
    assert!(summary.iter().any(|line| line.contains("String Functions")));
    assert!(summary.iter().any(|line| line.contains("Formatting Functions")));
    
    // Should contain function descriptions
    assert!(summary.iter().any(|line| line.contains("math_abs_i32")));
    assert!(summary.iter().any(|line| line.contains("string_concat")));
    assert!(summary.iter().any(|line| line.contains("format_date")));
}