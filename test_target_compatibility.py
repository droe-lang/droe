#!/usr/bin/env python3
"""Test backward compatibility with @target syntax."""

import os
import sys
from pathlib import Path

# Add compiler to path
sys.path.insert(0, str(Path(__file__).parent))

from test_mobile_compilation import get_target_platforms
from compiler.parser import Parser


def test_target_syntax_compatibility():
    """Test that all target syntax variants work correctly."""
    
    test_cases = [
        # Original @target syntax (should still work)
        ("@target html", {"web"}),
        ("@target web", {"web"}),
        
        # New @metadata syntax 
        ('@metadata(platform="mobile")', {"android", "ios"}),
        ('@metadata(platform="web")', {"web"}),
        ('@metadata(targets="web, android")', {"web", "android"}),
        
        # Individual platform annotations
        ("@platform mobile", {"android", "ios"}),
        ("@targets android, ios", {"android", "ios"}),
        
        # Legacy compatibility
        ("@target wasm", {"wasm"}),
    ]
    
    print("🧪 Testing Target Syntax Compatibility")
    print("=" * 50)
    
    all_passed = True
    
    for i, (metadata_line, expected_platforms) in enumerate(test_cases, 1):
        print(f"\n{i}. Testing: {metadata_line}")
        
        # Create a minimal test program
        test_program = f"""
{metadata_line}

module test_module
  layout TestLayout
    column
      title "Test"
      button "Test Button" action testAction
    end column
  end layout
  
  action testAction
    show "Test action executed"
  end action
end module
"""
        
        try:
            parser = Parser(test_program)
            ast = parser.parse()
            actual_platforms = get_target_platforms(ast)
            
            if actual_platforms == expected_platforms:
                print(f"   ✅ PASS: {actual_platforms}")
            else:
                print(f"   ❌ FAIL: Expected {expected_platforms}, got {actual_platforms}")
                all_passed = False
                
        except Exception as e:
            print(f"   ❌ ERROR: {e}")
            all_passed = False
    
    print("\n" + "=" * 50)
    if all_passed:
        print("🎉 ALL TARGET SYNTAX VARIANTS WORK!")
        print("✅ @target html - SUPPORTED")
        print("✅ @metadata(platform='mobile') - SUPPORTED") 
        print("✅ @targets 'web, android, ios' - SUPPORTED")
    else:
        print("⚠️  Some target syntax variants have issues")
    
    return all_passed


def test_existing_html_example():
    """Test that existing HTML examples still work."""
    
    print("\n🌐 Testing Existing HTML Example")
    print("=" * 30)
    
    # Test the showcase example that uses @target html
    showcase_file = Path("examples/src/showcase_all_components.roe")
    if not showcase_file.exists():
        print("❌ showcase_all_components.roe not found")
        return False
    
    source_code = showcase_file.read_text()
    
    try:
        parser = Parser(source_code)
        ast = parser.parse()
        platforms = get_target_platforms(ast)
        
        print(f"📄 File: showcase_all_components.roe")
        print(f"🎯 Target platforms: {platforms}")
        
        if 'web' in platforms or 'html' in platforms:
            print("✅ @target html works correctly!")
            return True
        else:
            print("❌ @target html not detected properly")
            return False
            
    except Exception as e:
        print(f"❌ Error parsing HTML example: {e}")
        return False


if __name__ == "__main__":
    compatibility_ok = test_target_syntax_compatibility()
    html_example_ok = test_existing_html_example()
    
    print("\n" + "=" * 50)
    print("📋 COMPATIBILITY SUMMARY")
    print("=" * 50)
    
    if compatibility_ok and html_example_ok:
        print("🎉 FULL BACKWARD COMPATIBILITY MAINTAINED")
        print("   • All existing @target html examples work")
        print("   • New @metadata syntax works")
        print("   • No breaking changes!")
    else:
        print("⚠️  COMPATIBILITY ISSUES DETECTED")
        if not compatibility_ok:
            print("   • Target syntax variants have problems")
        if not html_example_ok:
            print("   • Existing HTML examples broken")