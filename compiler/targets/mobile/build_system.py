"""Mobile build system for creating APKs and IPAs."""

import json
import subprocess
from pathlib import Path
from typing import Dict, Any, List, Optional


class MobileBuildSystem:
    """Handles building mobile projects into distributable artifacts."""
    
    def __init__(self, project_root: Path):
        self.project_root = project_root
        self.config = self._load_project_config()
        
    def _load_project_config(self) -> Dict[str, Any]:
        """Load project configuration."""
        config_path = self.project_root / "droeconfig.json"
        if config_path.exists():
            with open(config_path, 'r') as f:
                return json.load(f)
        return {}
    
    def build_mobile_projects(self, release: bool = False) -> Dict[str, str]:
        """Build mobile projects into distributable artifacts."""
        mobile_config = self.config.get('mobile', {})
        platforms = mobile_config.get('platforms', ['android', 'ios'])
        
        build_dir = self.project_root / self.config.get('build', 'build')
        dist_dir = self.project_root / self.config.get('dist', 'dist')
        
        results = {}
        
        if release:
            print(f"🚀 Creating release builds...")
            output_dir = dist_dir
            # Ensure dist directory exists
            dist_dir.mkdir(parents=True, exist_ok=True)
        else:
            print(f"🔨 Creating development builds...")
            output_dir = build_dir
        
        # Build Android
        if 'android' in platforms:
            android_result = self._build_android(build_dir, output_dir, release)
            if android_result:
                results['android'] = android_result
        
        # Build iOS
        if 'ios' in platforms:
            ios_result = self._build_ios(build_dir, output_dir, release)
            if ios_result:
                results['ios'] = ios_result
                
        return results
    
    def _build_android(self, build_dir: Path, output_dir: Path, release: bool) -> Optional[str]:
        """Build Android project."""
        android_project = build_dir / 'android'
        
        if not android_project.exists():
            print(f"❌ Android project not found in {android_project}")
            print(f"   Run 'droe compile' first to generate projects")
            return None
        
        print(f"📱 Building Android project...")
        
        try:
            # Change to Android project directory
            original_cwd = Path.cwd()
            
            try:
                import os
                os.chdir(android_project)
                
                if release:
                    # Build release APK
                    result = subprocess.run(
                        ['./gradlew', 'assembleRelease'], 
                        capture_output=True, 
                        text=True,
                        timeout=300  # 5 minute timeout
                    )
                    
                    if result.returncode == 0:
                        # Find the generated APK
                        apk_path = android_project / 'app/build/outputs/apk/release/app-release.apk'
                        if apk_path.exists():
                            # Copy to dist directory
                            dest_apk = output_dir / 'PhotoShare.apk'
                            import shutil
                            shutil.copy2(apk_path, dest_apk)
                            print(f"✅ Android APK created: {dest_apk}")
                            return str(dest_apk)
                        else:
                            print(f"❌ APK not found at expected location: {apk_path}")
                    else:
                        print(f"❌ Android build failed:")
                        print(result.stderr)
                else:
                    # Development build - just validate project
                    result = subprocess.run(
                        ['./gradlew', 'build'], 
                        capture_output=True, 
                        text=True,
                        timeout=180  # 3 minute timeout
                    )
                    
                    if result.returncode == 0:
                        print(f"✅ Android development build successful")
                        return str(android_project)
                    else:
                        print(f"❌ Android build failed:")
                        print(result.stderr)
                        
            finally:
                os.chdir(original_cwd)
                
        except subprocess.TimeoutExpired:
            print(f"❌ Android build timed out")
        except FileNotFoundError:
            print(f"❌ Gradle not found. Make sure Android SDK is installed.")
            print(f"   • Install Android Studio")
            print(f"   • Add gradle to PATH")
        except Exception as e:
            print(f"❌ Android build error: {e}")
            
        return None
    
    def _build_ios(self, build_dir: Path, output_dir: Path, release: bool) -> Optional[str]:
        """Build iOS project."""
        ios_project = build_dir / 'ios'
        
        if not ios_project.exists():
            print(f"❌ iOS project not found in {ios_project}")
            print(f"   Run 'droe compile' first to generate projects")
            return None
        
        print(f"📱 Building iOS project...")
        
        try:
            # Find .xcodeproj file
            xcodeproj_files = list(ios_project.glob('*.xcodeproj'))
            if not xcodeproj_files:
                print(f"❌ No .xcodeproj file found in {ios_project}")
                return None
                
            xcodeproj = xcodeproj_files[0]
            
            if release:
                # Build release IPA using xcodebuild
                result = subprocess.run([
                    'xcodebuild', 
                    '-project', str(xcodeproj),
                    '-scheme', xcodeproj.stem,
                    '-configuration', 'Release',
                    '-archivePath', str(output_dir / f'{xcodeproj.stem}.xcarchive'),
                    'archive'
                ], capture_output=True, text=True, timeout=300)
                
                if result.returncode == 0:
                    # Export IPA
                    archive_path = output_dir / f'{xcodeproj.stem}.xcarchive'
                    ipa_path = output_dir / f'{xcodeproj.stem}.ipa'
                    
                    # Create export options plist
                    export_plist = output_dir / 'ExportOptions.plist'
                    plist_content = """<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>method</key>
    <string>development</string>
    <key>teamID</key>
    <string>YOUR_TEAM_ID</string>
</dict>
</plist>"""
                    with open(export_plist, 'w') as f:
                        f.write(plist_content)
                    
                    export_result = subprocess.run([
                        'xcodebuild',
                        '-exportArchive',
                        '-archivePath', str(archive_path),
                        '-exportPath', str(output_dir),
                        '-exportOptionsPlist', str(export_plist)
                    ], capture_output=True, text=True)
                    
                    if export_result.returncode == 0:
                        print(f"✅ iOS IPA created: {ipa_path}")
                        return str(ipa_path)
                    else:
                        print(f"❌ iOS export failed:")
                        print(export_result.stderr)
                else:
                    print(f"❌ iOS build failed:")
                    print(result.stderr)
            else:
                # Development build - just validate project
                result = subprocess.run([
                    'xcodebuild', 
                    '-project', str(xcodeproj),
                    '-scheme', xcodeproj.stem,
                    'build'
                ], capture_output=True, text=True, timeout=180)
                
                if result.returncode == 0:
                    print(f"✅ iOS development build successful")
                    return str(ios_project)
                else:
                    print(f"❌ iOS build failed:")
                    print(result.stderr)
                    
        except subprocess.TimeoutExpired:
            print(f"❌ iOS build timed out")
        except FileNotFoundError:
            print(f"❌ Xcode not found. Make sure Xcode is installed.")
            print(f"   • Install Xcode from App Store")
            print(f"   • Run 'xcode-select --install'")
        except Exception as e:
            print(f"❌ iOS build error: {e}")
            
        return None
    
    def run_mobile_app(self, platform: str = None) -> bool:
        """Run mobile app with hot reload support."""
        mobile_config = self.config.get('mobile', {})
        platforms = mobile_config.get('platforms', ['android', 'ios'])
        
        if platform:
            platforms = [platform] if platform in platforms else []
        
        build_dir = self.project_root / self.config.get('build', 'build')
        success = False
        
        for platform in platforms:
            if platform == 'android':
                success = self._run_android(build_dir) or success
            elif platform == 'ios':
                success = self._run_ios(build_dir) or success
                
        return success
    
    def _run_android(self, build_dir: Path) -> bool:
        """Run Android app."""
        android_project = build_dir / 'android'
        
        if not android_project.exists():
            print(f"❌ Android project not found. Run 'droe compile' first.")
            return False
        
        print(f"🚀 Starting Android app...")
        
        try:
            import os
            original_cwd = Path.cwd()
            os.chdir(android_project)
            
            # Install and run on connected device/emulator
            result = subprocess.run([
                './gradlew', 'installDebug'
            ], capture_output=True, text=True)
            
            os.chdir(original_cwd)
            
            if result.returncode == 0:
                print(f"✅ Android app installed and running")
                print(f"📱 Check your Android device/emulator")
                return True
            else:
                print(f"❌ Failed to run Android app:")
                print(result.stderr)
                print(f"💡 Make sure an Android device/emulator is connected")
                
        except Exception as e:
            print(f"❌ Error running Android app: {e}")
            
        return False
    
    def _run_ios(self, build_dir: Path) -> bool:
        """Run iOS app."""
        ios_project = build_dir / 'ios'
        
        if not ios_project.exists():
            print(f"❌ iOS project not found. Run 'droe compile' first.")
            return False
        
        print(f"🚀 Starting iOS app...")
        
        try:
            xcodeproj_files = list(ios_project.glob('*.xcodeproj'))
            if not xcodeproj_files:
                print(f"❌ No .xcodeproj file found")
                return False
                
            xcodeproj = xcodeproj_files[0]
            
            # Build and run on simulator
            result = subprocess.run([
                'xcodebuild',
                '-project', str(xcodeproj),
                '-scheme', xcodeproj.stem,
                '-destination', 'platform=iOS Simulator,name=iPhone 15',
                'build'
            ], capture_output=True, text=True)
            
            if result.returncode == 0:
                print(f"✅ iOS app running on simulator")
                return True
            else:
                print(f"❌ Failed to run iOS app:")
                print(result.stderr)
                print(f"💡 Make sure iOS Simulator is available")
                
        except Exception as e:
            print(f"❌ Error running iOS app: {e}")
            
        return False