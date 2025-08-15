#!/usr/bin/env python3
"""Test script for the consistent mobile DSL syntax."""

import os
import sys
from pathlib import Path

# Add compiler to path
sys.path.insert(0, str(Path(__file__).parent))

from test_mobile_compilation import test_mobile_compilation, get_target_platforms
from compiler.parser import Parser


def test_consistent_mobile_syntax():
    """Test the new consistent mobile syntax."""
    
    # Read the consistent mobile example
    example_file = Path("examples/src/consistent_mobile_demo.droe")
    if not example_file.exists():
        print(f"Error: {example_file} not found")
        return
    
    source_code = example_file.read_text()
    
    # Parse the DSL
    print("🧪 Testing Consistent Mobile DSL Syntax...")
    print("=" * 50)
    parser = Parser(source_code)
    
    try:
        ast = parser.parse()
        print(f"✅ Parsed successfully: {len(ast.statements)} statements")
        
        # Check metadata for target selection
        target_platforms = get_target_platforms(ast)
        print(f"🎯 Target platforms: {', '.join(sorted(target_platforms))}")
        
        # Show some example parsed components
        print("\n📋 Parsed components:")
        layout_count = form_count = action_count = 0
        
        for stmt in ast.statements:
            stmt_type = stmt.__class__.__name__
            if 'Layout' in stmt_type:
                layout_count += 1
            elif 'Form' in stmt_type:
                form_count += 1
            elif 'Action' in stmt_type:
                action_count += 1
        
        print(f"   - {layout_count} layouts")
        print(f"   - {form_count} forms") 
        print(f"   - {action_count} actions")
        
        print("\n✅ Consistent mobile syntax works!")
        
    except Exception as e:
        print(f"❌ Parsing error: {e}")
        print("\nℹ️  The consistent syntax needs parser updates to handle:")
        print("   - Natural language component definitions")
        print("   - Mobile-specific component types") 
        print("   - Extended attribute syntax")


def compare_syntax_styles():
    """Compare different DSL syntax styles."""
    
    print("\n🔍 DSL Syntax Comparison")
    print("=" * 50)
    
    print("\n1️⃣ CURRENT (Colon-based):")
    print("   Layout:")
    print("       Name: \"MainScreen\"") 
    print("       Input: \"Enter name\" [bind: userName]")
    print("       Camera: \"Take Photo\" [action: capture]")
    
    print("\n2️⃣ CONSISTENT (Natural language):")
    print("   layout MainScreen")
    print("       input text placeholder \"Enter name\" bind to userName")
    print("       button \"Take Photo\" type camera action capture")
    
    print("\n3️⃣ VERBOSE (Descriptive):")
    print("   screen MainScreen {")
    print("       text input placeholder \"Enter name\" bind to userName")
    print("       camera button \"Take Photo\" on press capture photo")
    print("   }")
    
    print("\n💡 Recommendation: Use consistent natural language style")
    print("   - Matches existing Roelang patterns (show, set, when)")
    print("   - More readable than colon syntax")
    print("   - Less verbose than full descriptive syntax")


if __name__ == "__main__":
    test_consistent_mobile_syntax()
    compare_syntax_styles()
    
    print("\n" + "=" * 50)
    print("🏁 Testing original mobile demo with fixed metadata...")
    test_mobile_compilation()