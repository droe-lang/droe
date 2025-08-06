"""Main compiler entry point for Roe DSL."""

import sys
import os
from typing import Optional
from .parser import parse, ParseError
from .target_factory import target_factory, compile_to_target
from .codegen_base import CodeGenError
from .module_resolver import ModuleResolver, ModuleResolutionError
from .type_checker import TypeChecker, TypeCheckError


class CompilerError(Exception):
    """General compiler error."""
    pass


def get_metadata_value(ast, key: str) -> Optional[str]:
    """
    Get the value of a metadata annotation from the AST.
    
    Args:
        ast: Parsed AST Program node
        key: Metadata key to search for (e.g., "target", "name", "description")
        
    Returns:
        String value if found, None otherwise
    """
    if hasattr(ast, 'metadata') and ast.metadata:
        for annotation in ast.metadata:
            if annotation.key == key:
                return annotation.value
    return None


def get_target_from_source(source: str, default_target: str = "wasm") -> str:
    """
    Determine the target from DSL source metadata.
    
    Args:
        source: Roe DSL source code
        default_target: Default target if no metadata found
        
    Returns:
        Target string (metadata target or default)
    """
    try:
        ast = parse(source)
        metadata_target = get_metadata_value(ast, "target")
        return metadata_target if metadata_target else default_target
    except:
        return default_target


def compile(source: str, file_path: Optional[str] = None, target: str = "wasm") -> str:
    """
    Compile Roe DSL source code to specified target format.
    
    Args:
        source: Roe DSL source code
        file_path: Optional path to source file (for module resolution)
        target: Compilation target (wasm, python, java, go, node, html, kotlin, swift)
        
    Returns:
        Generated code string in target format
        
    Raises:
        CompilerError: If compilation fails
    """
    try:
        # Parse source to AST
        ast = parse(source)
        
        # Note: Target resolution is now handled by the CLI to ensure proper priority
        # DSL @target metadata should be resolved before calling this function
        
        # Resolve includes if file path is provided
        if file_path:
            resolver = ModuleResolver()
            ast = resolver.resolve_includes(ast, file_path)
        
        # Perform strong type checking
        type_checker = TypeChecker()
        type_checker.check_program(ast)
        
        # Generate code from AST using specified target
        generated_code = compile_to_target(ast, target, file_path)
        
        return generated_code
        
    except ParseError as e:
        raise CompilerError(f"Parse error: {str(e)}")
    except TypeCheckError as e:
        raise CompilerError(f"Type checking error: {str(e)}")
    except ModuleResolutionError as e:
        raise CompilerError(f"Module resolution error: {str(e)}")
    except CodeGenError as e:
        raise CompilerError(f"Code generation error: {str(e)}")
    except Exception as e:
        raise CompilerError(f"Unexpected error: {str(e)}")


def compile_file(input_path: str, output_path: Optional[str] = None, target: str = "wasm") -> str:
    """
    Compile a Roe DSL file to specified target.
    
    Args:
        input_path: Path to .roe file
        output_path: Optional output path for generated file
        target: Compilation target (wasm, python, java, go, node, html, kotlin, swift)
        
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
    
    # Compile to target (pass file path for module resolution)
    generated_code = compile(source, input_path, target)
    
    # Determine output path
    if output_path is None:
        base_name = os.path.splitext(input_path)[0]
        # Get appropriate file extension for target
        target_info = target_factory.get_target_info(target)
        file_extension = target_info['file_extension']
        output_path = f"{base_name}{file_extension}"
    
    # Write output file
    try:
        with open(output_path, 'w') as f:
            f.write(generated_code)
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