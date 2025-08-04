"""Main compiler entry point for Roe DSL."""

import sys
import os
from typing import Optional
from .parser import parse, ParseError
from .codegen_wat import generate_wat, CodeGenError
from .module_resolver import ModuleResolver, ModuleResolutionError


class CompilerError(Exception):
    """General compiler error."""
    pass


def compile(source: str, file_path: Optional[str] = None) -> str:
    """
    Compile Roe DSL source code to WebAssembly Text format.
    
    Args:
        source: Roe DSL source code
        file_path: Optional path to source file (for module resolution)
        
    Returns:
        WAT (WebAssembly Text) string
        
    Raises:
        CompilerError: If compilation fails
    """
    try:
        # Parse source to AST
        ast = parse(source)
        
        # Resolve includes if file path is provided
        if file_path:
            resolver = ModuleResolver()
            ast = resolver.resolve_includes(ast, file_path)
        
        # Generate WAT from AST
        wat = generate_wat(ast)
        
        return wat
        
    except ParseError as e:
        raise CompilerError(f"Parse error: {str(e)}")
    except ModuleResolutionError as e:
        raise CompilerError(f"Module resolution error: {str(e)}")
    except CodeGenError as e:
        raise CompilerError(f"Code generation error: {str(e)}")
    except Exception as e:
        raise CompilerError(f"Unexpected error: {str(e)}")


def compile_file(input_path: str, output_path: Optional[str] = None) -> str:
    """
    Compile a Roe DSL file to WAT.
    
    Args:
        input_path: Path to .roe file
        output_path: Optional output path for .wat file
        
    Returns:
        Output file path
        
    Raises:
        CompilerError: If compilation fails
    """
    # Read source file
    try:
        with open(input_path, 'r') as f:
            source = f.read()
    except IOError as e:
        raise CompilerError(f"Failed to read input file: {str(e)}")
    
    # Compile to WAT (pass file path for module resolution)
    wat = compile(source, input_path)
    
    # Determine output path
    if output_path is None:
        base_name = os.path.splitext(input_path)[0]
        output_path = f"{base_name}.wat"
    
    # Write output file
    try:
        with open(output_path, 'w') as f:
            f.write(wat)
    except IOError as e:
        raise CompilerError(f"Failed to write output file: {str(e)}")
    
    return output_path


def main():
    """CLI entry point."""
    if len(sys.argv) < 2:
        print("Usage: python -m compiler.compiler <input.roe> [output.wat]")
        sys.exit(1)
    
    input_file = sys.argv[1]
    output_file = sys.argv[2] if len(sys.argv) > 2 else None
    
    try:
        output_path = compile_file(input_file, output_file)
        print(f"Successfully compiled to: {output_path}")
    
    except CompilerError as e:
        print(f"Compilation failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()