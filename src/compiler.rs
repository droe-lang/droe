use anyhow::Result;
use std::path::{Path, PathBuf};
use console::style;

pub async fn compile_file(
    input: &Path,
    output: Option<&PathBuf>,
    target: &str,
    _opt_level: u8,
) -> Result<()> {
    // Read and parse the source file
    let source = std::fs::read_to_string(input)
        .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;

    // Apply target aliasing (droe -> bytecode)
    let resolved_target = crate::cli::resolve_target(&source, target, Some(target));
    
    println!("{} Compiling {} to {}...", 
        style("[INFO]").cyan(), 
        input.display(), 
        resolved_target
    );
    
    let compiler = droe_compiler::Compiler::new();
    let program = compiler.parse(&source)
        .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;
    
    let output_dir = output
.cloned()
        .unwrap_or_else(|| {
            input.parent()
                .unwrap_or(Path::new("."))
                .join("build")
        });
    
    std::fs::create_dir_all(&output_dir)?;
    
    // Use the global compile_to_target function
    let compilation_result = droe_compiler::compile_to_target(&program, &resolved_target, None)
        .map_err(|e| anyhow::anyhow!("Compilation error: {}", e))?;
    
    // Write the compiled output
    let basename = input.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("main");
    let output_file = output_dir.join(format!("{}{}", 
        basename,
        droe_compiler::CompilerTarget::from_str(&resolved_target)
            .map_err(|e| anyhow::anyhow!("Invalid target: {}", e))?
            .file_extension()
    ));
    
    match compilation_result {
        droe_compiler::CompilationResult::Code(code) => {
            if resolved_target == "wasm" {
                // For WASM target, generate both .wat and .wasm files
                let wat_file = output_dir.join(format!("{}.wat", basename));
                let wasm_file = output_dir.join(format!("{}.wasm", basename));
                
                // Write WAT file
                std::fs::write(&wat_file, &code)
                    .map_err(|e| anyhow::anyhow!("Failed to write WAT file: {}", e))?;
                
                // Convert WAT to binary WASM using wat crate
                let wasm_bytes = wat::parse_str(&code)
                    .map_err(|e| anyhow::anyhow!("Failed to convert WAT to WASM: {}", e))?;
                
                std::fs::write(&wasm_file, wasm_bytes)
                    .map_err(|e| anyhow::anyhow!("Failed to write WASM file: {}", e))?;
                
                println!("✅ Generated both {} and {}", wat_file.display(), wasm_file.display());
            } else {
                std::fs::write(&output_file, code)
                    .map_err(|e| anyhow::anyhow!("Failed to write output: {}", e))?;
            }
        }
        droe_compiler::CompilationResult::Project(project) => {
            // For project outputs, create the structure
            for (file_path, content) in project.files {
                let full_path = output_dir.join(file_path);
                if let Some(parent) = full_path.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| anyhow::anyhow!("Failed to create directory: {}", e))?;
                }
                std::fs::write(full_path, content)
                    .map_err(|e| anyhow::anyhow!("Failed to write file: {}", e))?;
            }
        }
    }
    
    let result = output_file;
    
    println!("{} Compilation completed! Output: {}", 
        style("[SUCCESS]").green(),
        result.display()
    );
    
    Ok(())
}

pub async fn reverse_compile(input: &Path, output: Option<&PathBuf>) -> Result<()> {
    println!("{} Converting {} back to DSL...", 
        style("[INFO]").cyan(), 
        input.display()
    );
    
    let output_path = output
.cloned()
        .unwrap_or_else(|| input.with_extension("droe"));
    
    // Use droe-compiler's reverse compilation feature
    droe_compiler::reverse_compile_puck(input, &output_path)
        .map_err(|e| anyhow::anyhow!("Reverse compilation error: {}", e))?;
    
    println!("{} Reverse compilation completed! Output: {}", 
        style("[SUCCESS]").green(),
        output_path.display()
    );
    
    Ok(())
}

pub async fn generate_framework(framework: &str, output: Option<&PathBuf>) -> Result<()> {
    println!("{} Generating {} framework code...", 
        style("[INFO]").cyan(), 
        framework
    );
    
    let output_dir = output
.cloned()
        .unwrap_or_else(|| PathBuf::from(format!("{}-project", framework)));
    
    std::fs::create_dir_all(&output_dir)?;
    
    // Use droe-compiler for framework generation
    match framework {
        "spring" => droe_compiler::generate_spring_project(&output_dir)
            .map_err(|e| anyhow::anyhow!("Spring generation error: {}", e))?,
        "fastapi" => droe_compiler::generate_fastapi_project(&output_dir)
            .map_err(|e| anyhow::anyhow!("FastAPI generation error: {}", e))?,
        "fiber" => droe_compiler::generate_fiber_project(&output_dir)
            .map_err(|e| anyhow::anyhow!("Fiber generation error: {}", e))?,
        "android" => droe_compiler::generate_android_project(&output_dir)
            .map_err(|e| anyhow::anyhow!("Android generation error: {}", e))?,
        "ios" => droe_compiler::generate_ios_project(&output_dir)
            .map_err(|e| anyhow::anyhow!("iOS generation error: {}", e))?,
        _ => {
            return Err(anyhow::anyhow!("Unsupported framework: {}", framework));
        }
    }
    
    println!("{} Framework generation completed! Output: {}", 
        style("[SUCCESS]").green(),
        output_dir.display()
    );
    
    Ok(())
}