"""Target factory for Roelang compiler.

This module provides a factory system for creating target-specific
code generators and managing compilation targets.
"""

from typing import Dict, Type, Optional, List
from abc import ABC, abstractmethod
from .codegen_base import BaseCodeGenerator
from .ast import Program


class CompilerTarget(ABC):
    """Abstract base class for compilation targets."""
    
    def __init__(self, name: str, file_extension: str, description: str):
        self.name = name
        self.file_extension = file_extension
        self.description = description
    
    @abstractmethod
    def create_codegen(self, source_file_path: str = None, is_main_file: bool = False) -> BaseCodeGenerator:
        """Create a code generator for this target."""
        pass
    
    @abstractmethod
    def get_runtime_files(self) -> List[str]:
        """Get list of runtime files needed for this target."""
        pass
    
    @abstractmethod
    def get_dependencies(self) -> List[str]:
        """Get list of external dependencies for this target."""
        pass


class WASMTarget(CompilerTarget):
    """WebAssembly compilation target."""
    
    def __init__(self):
        super().__init__("wasm", ".wasm", "WebAssembly binary format")
    
    def create_codegen(self, source_file_path: str = None, is_main_file: bool = False) -> BaseCodeGenerator:
        from .targets.wasm.codegen import WATCodeGenerator
        return WATCodeGenerator()
    
    def get_runtime_files(self) -> List[str]:
        return ["run.js"]
    
    def get_dependencies(self) -> List[str]:
        return ["node", "wat2wasm"]


class PythonTarget(CompilerTarget):
    """Python compilation target."""
    
    def __init__(self):
        super().__init__("python", ".py", "Python source code")
    
    def create_codegen(self, source_file_path: str = None, is_main_file: bool = False) -> BaseCodeGenerator:
        from .targets.python.codegen import PythonCodeGenerator
        return PythonCodeGenerator()
    
    def get_runtime_files(self) -> List[str]:
        return []  # No runtime files needed - using inline code generation
    
    def get_dependencies(self) -> List[str]:
        return ["python3"]


class JavaTarget(CompilerTarget):
    """Java compilation target."""
    
    def __init__(self):
        super().__init__("java", ".java", "Java source code")
    
    def create_codegen(self, source_file_path: str = None, is_main_file: bool = False) -> BaseCodeGenerator:
        from .targets.java.codegen import JavaCodeGenerator
        return JavaCodeGenerator(source_file_path, is_main_file)
    
    def get_runtime_files(self) -> List[str]:
        return []  # No runtime files needed - using inline code generation
    
    def get_dependencies(self) -> List[str]:
        return ["javac", "java"]


class HTMLTarget(CompilerTarget):
    """HTML/JavaScript compilation target."""
    
    def __init__(self):
        super().__init__("html", ".html", "HTML with embedded JavaScript")
    
    def create_codegen(self, source_file_path: str = None, is_main_file: bool = False) -> BaseCodeGenerator:
        from .targets.html.codegen import HTMLCodeGenerator
        return HTMLCodeGenerator()
    
    def get_runtime_files(self) -> List[str]:
        return ["roelang.js", "styles.css"]
    
    def get_dependencies(self) -> List[str]:
        return []  # Runs in browser, no external deps


class KotlinTarget(CompilerTarget):
    """Kotlin compilation target."""
    
    def __init__(self):
        super().__init__("kotlin", ".kt", "Kotlin source code")
    
    def create_codegen(self, source_file_path: str = None, is_main_file: bool = False) -> BaseCodeGenerator:
        from .targets.kotlin.codegen import KotlinCodeGenerator
        return KotlinCodeGenerator()
    
    def get_runtime_files(self) -> List[str]:
        return ["RoelangRuntime.kt"]
    
    def get_dependencies(self) -> List[str]:
        return ["kotlinc", "kotlin"]


class SwiftTarget(CompilerTarget):
    """Swift compilation target."""
    
    def __init__(self):
        super().__init__("swift", ".swift", "Swift source code")
    
    def create_codegen(self, source_file_path: str = None, is_main_file: bool = False) -> BaseCodeGenerator:
        from .targets.swift.codegen import SwiftCodeGenerator
        return SwiftCodeGenerator()
    
    def get_runtime_files(self) -> List[str]:
        return ["RoelangRuntime.swift"]
    
    def get_dependencies(self) -> List[str]:
        return ["swift", "swiftc"]


class GoTarget(CompilerTarget):
    """Go compilation target."""
    
    def __init__(self):
        super().__init__("go", ".go", "Go source code")
    
    def create_codegen(self, source_file_path: str = None, is_main_file: bool = False) -> BaseCodeGenerator:
        from .targets.go.codegen import GoCodeGenerator
        return GoCodeGenerator()
    
    def get_runtime_files(self) -> List[str]:
        return []  # No runtime files needed - using inline code generation
    
    def get_dependencies(self) -> List[str]:
        return ["go"]


class NodeTarget(CompilerTarget):
    """Node.js compilation target."""
    
    def __init__(self):
        super().__init__("node", ".js", "Node.js JavaScript code")
    
    def create_codegen(self, source_file_path: str = None, is_main_file: bool = False) -> BaseCodeGenerator:
        from .targets.node.codegen import NodeCodeGenerator
        return NodeCodeGenerator()
    
    def get_runtime_files(self) -> List[str]:
        return []  # No runtime files needed - using inline code generation
    
    def get_dependencies(self) -> List[str]:
        return ["node", "npm"]


class BytecodeTarget(CompilerTarget):
    """Bytecode compilation target for Roe VM."""
    
    def __init__(self):
        super().__init__("bytecode", ".roebc", "Roe VM bytecode format")
    
    def create_codegen(self, source_file_path: str = None, is_main_file: bool = False) -> BaseCodeGenerator:
        from .targets.bytecode.codegen import BytecodeGenerator
        return BytecodeGenerator()
    
    def get_runtime_files(self) -> List[str]:
        return []  # VM is bundled separately
    
    def get_dependencies(self) -> List[str]:
        return ["roevm"]  # Requires the Roe VM


class TargetFactory:
    """Factory for creating compilation targets."""
    
    def __init__(self):
        self._targets: Dict[str, Type[CompilerTarget]] = {
            "wasm": WASMTarget,
            "python": PythonTarget,
            "java": JavaTarget,
            "html": HTMLTarget,
            "kotlin": KotlinTarget,
            "swift": SwiftTarget,
            "go": GoTarget,
            "node": NodeTarget,
            "bytecode": BytecodeTarget
        }
    
    def get_available_targets(self) -> List[str]:
        """Get list of available compilation targets."""
        return list(self._targets.keys())
    
    def create_target(self, target_name: str) -> CompilerTarget:
        """Create a compilation target by name."""
        if target_name not in self._targets:
            available = ", ".join(self.get_available_targets())
            raise ValueError(f"Unknown target '{target_name}'. Available: {available}")
        
        return self._targets[target_name]()
    
    def register_target(self, name: str, target_class: Type[CompilerTarget]):
        """Register a new compilation target."""
        self._targets[name] = target_class
    
    def get_target_info(self, target_name: str) -> Dict[str, any]:
        """Get information about a compilation target."""
        target = self.create_target(target_name)
        return {
            "name": target.name,
            "file_extension": target.file_extension,
            "description": target.description,
            "runtime_files": target.get_runtime_files(),
            "dependencies": target.get_dependencies()
        }


# Global factory instance
target_factory = TargetFactory()


def compile_to_target(program: Program, target_name: str, source_file_path: str = None, is_main_file: bool = False) -> str:
    """Compile a program to a specific target."""
    target = target_factory.create_target(target_name)
    codegen = target.create_codegen(source_file_path, is_main_file)
    return codegen.generate(program)


def get_target_codegen(target_name: str, source_file_path: str = None, is_main_file: bool = False) -> BaseCodeGenerator:
    """Get a code generator for a specific target."""
    target = target_factory.create_target(target_name)
    return target.create_codegen(source_file_path, is_main_file)