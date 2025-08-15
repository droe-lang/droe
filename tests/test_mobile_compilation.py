#!/usr/bin/env python3
"""Test script for mobile compilation targets."""

import os
import sys
from pathlib import Path

# Add compiler to path
sys.path.insert(0, str(Path(__file__).parent))

from compiler.parser import Parser
from compiler.targets.mobile import KotlinGenerator, SwiftGenerator
from compiler.targets.html import HTMLGenerator
from compiler.ast import MetadataAnnotation


def get_target_platforms(ast):
    """Extract target platforms from AST metadata."""
    platforms = set()
    
    # Look for metadata annotations
    for stmt in ast.statements:
        if isinstance(stmt, MetadataAnnotation):
            if stmt.key == 'metadata' and isinstance(stmt.value, dict):
                # Handle @metadata(platform="mobile", ...)
                if 'platform' in stmt.value:
                    platform_value = stmt.value['platform']
                    if platform_value == 'mobile':
                        platforms.update(['android', 'ios'])
                    elif platform_value in ['web', 'html']:
                        platforms.add('web')
                    else:
                        platforms.add(platform_value)
                        
                if 'targets' in stmt.value:
                    # Parse comma-separated targets
                    targets = [t.strip() for t in stmt.value['targets'].split(',')]
                    platforms.update(targets)
                    
            elif stmt.key == 'platform':
                # Handle @platform mobile
                if stmt.value == 'mobile':
                    platforms.update(['android', 'ios'])
                elif stmt.value in ['web', 'html']:
                    platforms.add('web')
                else:
                    platforms.add(stmt.value)
                    
            elif stmt.key == 'targets':
                # Parse comma-separated targets like "web, android, ios"
                targets = [t.strip() for t in stmt.value.split(',')]
                platforms.update(targets)
                
            elif stmt.key == 'target':  # Legacy @target annotation
                if stmt.value in ['html', 'web']:
                    platforms.add('web')
                elif stmt.value == 'wasm':
                    platforms.add('wasm')
                else:
                    platforms.add(stmt.value)
    
    # Default to mobile platforms only if no metadata found and we have mobile components
    if not platforms:
        # Check if the DSL contains mobile-specific components
        has_mobile_components = any(
            hasattr(stmt, 'attributes') and 
            any(getattr(attr, 'name', '') == 'mobile_component' for attr in stmt.attributes)
            for stmt in ast.statements
        )
        
        if has_mobile_components:
            platforms = {'android', 'ios'}
        else:
            platforms = {'web'}  # Conservative default
    
    return platforms


def test_mobile_compilation():
    """Test compiling Droe DSL to mobile targets."""
    
    # Read the example file
    example_file = Path("examples/src/mobile_app_demo.droe")
    if not example_file.exists():
        print(f"Error: {example_file} not found")
        return
    
    source_code = example_file.read_text()
    
    # Parse the DSL
    print("Parsing Droe DSL...")
    parser = Parser(source_code)
    ast = parser.parse()
    print(f"✅ Parsed successfully: {len(ast.statements)} statements")
    
    # Check metadata for target selection
    target_platforms = get_target_platforms(ast)
    print(f"🎯 Target platforms: {', '.join(sorted(target_platforms))}")
    
    # Generate Android/Kotlin code (only if android is in targets)
    if 'android' in target_platforms:
        print("\n📱 Generating Android (Kotlin) code...")
        kotlin_gen = KotlinGenerator()
        output_dir = Path("examples/output/android")
        output_dir.mkdir(parents=True, exist_ok=True)
        
        try:
            kotlin_files = kotlin_gen.generate(ast, str(output_dir))
            print(f"✅ Generated {len(kotlin_files)} Android files:")
            for file_path in kotlin_files:
                rel_path = Path(file_path).relative_to(output_dir)
                print(f"   - {rel_path}")
        except Exception as e:
            print(f"❌ Kotlin generation error: {e}")
    else:
        print("\n⏭️  Skipping Android generation (not in target platforms)")
    
    # Generate iOS/Swift code (only if ios is in targets)
    if 'ios' in target_platforms:
        print("\n📱 Generating iOS (Swift) code...")
        swift_gen = SwiftGenerator()
        output_dir = Path("examples/output/ios")
        output_dir.mkdir(parents=True, exist_ok=True)
        
        try:
            swift_files = swift_gen.generate(ast, str(output_dir))
            print(f"✅ Generated {len(swift_files)} iOS files:")
            for file_path in swift_files:
                rel_path = Path(file_path).relative_to(output_dir)
                print(f"   - {rel_path}")
        except Exception as e:
            print(f"❌ Swift generation error: {e}")
    else:
        print("\n⏭️  Skipping iOS generation (not in target platforms)")
    
    # Generate HTML (only if web is in targets)
    if 'web' in target_platforms or 'html' in target_platforms:
        print("\n🌐 Generating Web (HTML) code...")
        html_gen = HTMLGenerator()
        output_file = Path("examples/output/web/mobile_app_demo.html")
        output_file.parent.mkdir(parents=True, exist_ok=True)
        
        try:
            html_code = html_gen.generate(ast)
            output_file.write_text(html_code)
            print(f"✅ Generated HTML file: {output_file}")
        except Exception as e:
            print(f"❌ HTML generation error: {e}")
    else:
        print("\n⏭️  Skipping Web generation (not in target platforms)")
    
    print("\n✨ Mobile compilation test complete!")
    print("Check the examples/output/ directory for generated code.")


if __name__ == "__main__":
    test_mobile_compilation()