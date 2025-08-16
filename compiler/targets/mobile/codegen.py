"""Mobile code generation for Android and iOS projects."""

import os
import json
from pathlib import Path
from typing import Dict, Any, List, Optional
from ...codegen_base import BaseCodeGenerator
from ...ast import Program
from .kotlin_generator import KotlinProjectGenerator
from .swift_generator import SwiftProjectGenerator


class MobileProjectCodegen(BaseCodeGenerator):
    """Code generator for mobile projects - generates complete Android and iOS projects."""
    
    def __init__(self, source_file_path: str = None, project_root: str = None):
        self.source_file_path = source_file_path
        self.project_root = Path(project_root) if project_root else Path.cwd()
        self.kotlin_generator = KotlinProjectGenerator()
        self.swift_generator = SwiftProjectGenerator()
        
        # Load project configuration
        self.config = self._load_project_config()
        
    def _load_project_config(self) -> Dict[str, Any]:
        """Load project configuration from droeconfig.json."""
        config_path = self.project_root / "droeconfig.json"
        default_config = {
            "mobile": {
                "enabled": True,
                "platforms": ["android", "ios"],
                "package": "com.example.app",
                "appName": "MyApp",
                "version": "1.0.0",
                "permissions": {}
            }
        }
        
        if config_path.exists():
            try:
                with open(config_path, 'r') as f:
                    config = json.load(f)
                    # Merge with defaults
                    mobile_config = config.get('mobile', {})
                    default_config['mobile'].update(mobile_config)
                    return config
            except Exception as e:
                print(f"Warning: Error reading droeconfig.json: {e}")
        
        return default_config
    
    def _check_incremental_build(self, output_dir: Path, source_files: List[Path]) -> bool:
        """Check if incremental build is possible."""
        if not output_dir.exists():
            return False
            
        # Check if manifest file exists (indicates previous build)
        manifest_file = output_dir / ".droelang_build_manifest.json"
        if not manifest_file.exists():
            return False
            
        try:
            with open(manifest_file, 'r') as f:
                manifest = json.load(f)
                
            # Check if any source files are newer than the manifest
            manifest_time = manifest.get('build_time', 0)
            for source_file in source_files:
                if source_file.stat().st_mtime > manifest_time:
                    print(f"🔄 Source file {source_file.name} changed, rebuilding...")
                    return False
                    
            # Check if project config changed
            config_file = self.project_root / "droeconfig.json"
            if config_file.exists() and config_file.stat().st_mtime > manifest_time:
                print("🔄 Project configuration changed, rebuilding...")
                return False
                
            print("✅ No changes detected, using existing build")
            return True
            
        except Exception as e:
            print(f"Warning: Could not check incremental build: {e}")
            return False
    
    def _create_build_manifest(self, output_dir: Path, source_files: List[Path]):
        """Create build manifest for incremental builds."""
        import time
        
        manifest = {
            "build_time": time.time(),
            "source_files": [str(f) for f in source_files],
            "config_file": str(self.project_root / "droeconfig.json"),
            "platforms": self.config.get('mobile', {}).get('platforms', ['android', 'ios']),
            "version": self.config.get('mobile', {}).get('version', '1.0.0')
        }
        
        manifest_file = output_dir / ".droelang_build_manifest.json"
        with open(manifest_file, 'w') as f:
            json.dump(manifest, f, indent=2)
    
    def generate(self, program: Program) -> str:
        """Generate mobile projects for Android and iOS."""
        mobile_config = self.config.get('mobile', {})
        platforms = mobile_config.get('platforms', ['android', 'ios'])
        
        # Determine output directory (use build for development projects)
        build_dir = self.project_root / self.config.get('build', 'build')
        
        # Find all source files for incremental build check
        source_files = []
        if self.source_file_path:
            source_files.append(Path(self.source_file_path))
        
        # Check for incremental build possibility
        if self._check_incremental_build(build_dir, source_files):
            return str(build_dir)
        
        print(f"🏗️  Generating mobile projects for platforms: {', '.join(platforms)}")
        
        generated_projects = []
        
        # Generate Android project
        if 'android' in platforms:
            android_dir = build_dir / 'android'
            print(f"📱 Generating Android project...")
            
            try:
                self.kotlin_generator.generate_project(program, android_dir, mobile_config)
                generated_projects.append('Android')
                print(f"✅ Android project generated: {android_dir}")
            except Exception as e:
                print(f"❌ Android generation failed: {e}")
        
        # Generate iOS project
        if 'ios' in platforms:
            ios_dir = build_dir / 'ios'
            print(f"📱 Generating iOS project...")
            
            try:
                self.swift_generator.generate_project(program, ios_dir, mobile_config)
                generated_projects.append('iOS')
                print(f"✅ iOS project generated: {ios_dir}")
            except Exception as e:
                print(f"❌ iOS generation failed: {e}")
        
        # Create build manifest for future incremental builds
        self._create_build_manifest(build_dir, source_files)
        
        if generated_projects:
            print(f"🎉 Mobile project generation complete!")
            print(f"📁 Generated: {', '.join(generated_projects)}")
            print(f"📂 Build directory: {build_dir}")
            
            # Provide helpful next steps
            print(f"\n🔧 Development:")
            if 'Android' in generated_projects:
                print(f"   • Android: Open {build_dir}/android in Android Studio")
            if 'iOS' in generated_projects:
                print(f"   • iOS: Open {build_dir}/ios/MyApp.xcodeproj in Xcode")
            
            print(f"\n📦 Next steps:")
            print(f"   • droe run - Run app with hot reload")
            print(f"   • droe build - Development build")  
            print(f"   • droe build --release - Create final APK/IPA in dist/")
        
        # Return a special marker to indicate mobile project generation
        # This tells the compiler not to create individual output files
        return f"MOBILE_PROJECT:{build_dir}"
    
    def emit_statement(self, statement):
        """Emit statement code (not used for project generation)."""
        return ""
    
    def emit_expression(self, expression):
        """Emit expression code (not used for project generation)."""
        return ""


class MobileLegacyCodegen(BaseCodeGenerator):
    """Legacy mobile codegen for single-file compilation (used as fallback)."""
    
    def generate(self, program: Program) -> str:
        """Generate a simple mobile project indicator."""
        return """# Mobile Project Generated
# This file indicates a mobile project was generated.
# Check the dist/android and dist/ios folders for the actual projects.

Mobile platforms: Android, iOS
Generated at: """ + str(__import__('datetime').datetime.now())