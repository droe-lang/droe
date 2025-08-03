from .compiler import compile_file, compile, CompilerError


def compile_roe_to_wat(input_path, output_path):
    """
    Compile a Roe DSL file to WebAssembly Text format.
    
    This is a wrapper around the new AST-based compiler for backward compatibility.
    """
    input_path = str(input_path)
    output_path = str(output_path)
    
    try:
        # First, try to compile with the new compiler
        compile_file(input_path, output_path)
        print(f"✅ Compiled {input_path} to {output_path}")
        
    except CompilerError:
        # Fall back to legacy compiler for old "Display" syntax
        with open(input_path, "r") as f:
            content = f.read().strip()
        
        if content.startswith("Display "):
            # Convert old syntax to new syntax
            message = content[len("Display "):]
            new_content = f'display "{message}"'
            
            try:
                wat = compile(new_content)
                
                # Adjust WAT to match old format (using print instead of display)
                wat = wat.replace('"env" "display"', '"env" "print"')
                wat = wat.replace('$display (param i32)', '$print (param i32 i32)')
                wat = wat.replace('call $display', f'i32.const {len(message)}\n    call $print')
                
                with open(output_path, "w") as f:
                    f.write(wat)
                
                print(f"✅ Compiled {input_path} to {output_path} (legacy syntax)")
                
            except Exception as e:
                raise ValueError(f"Compilation failed: {str(e)}")
        else:
            raise
