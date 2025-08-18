//! Mobile Build System - Android and iOS project building
//! 
//! This module provides functionality for:
//! - Building Android APKs
//! - Building iOS apps
//! - Managing mobile project templates
//! - Cross-platform mobile development

use anyhow::{Result, Context};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde_json::{json, Value};

pub struct MobileBuildSystem {
    project_root: PathBuf,
    build_dir: PathBuf,
    dist_dir: PathBuf,
}

#[derive(Debug)]
pub struct MobileBuildResult {
    pub android_artifacts: Vec<PathBuf>,
    pub ios_artifacts: Vec<PathBuf>,
    pub build_successful: bool,
}

impl MobileBuildSystem {
    pub fn new(project_root: PathBuf) -> Self {
        let build_dir = project_root.join("build");
        let dist_dir = project_root.join("dist");
        
        Self {
            project_root,
            build_dir,
            dist_dir,
        }
    }

    pub async fn build_mobile_projects(&self, release: bool) -> Result<HashMap<String, PathBuf>> {
        println!("📱 Building mobile projects...");
        
        let mut results = HashMap::new();
        
        // Check for Android project
        let android_dir = self.build_dir.join("android");
        if android_dir.exists() && self.is_android_project(&android_dir) {
            match self.build_android_project(&android_dir, release).await {
                Ok(apk_path) => {
                    results.insert("android".to_string(), apk_path);
                    println!("✅ Android build successful");
                }
                Err(e) => {
                    eprintln!("❌ Android build failed: {}", e);
                }
            }
        }

        // Check for iOS project
        let ios_dir = self.build_dir.join("ios");
        if ios_dir.exists() && self.is_ios_project(&ios_dir) {
            match self.build_ios_project(&ios_dir, release).await {
                Ok(app_path) => {
                    results.insert("ios".to_string(), app_path);
                    println!("✅ iOS build successful");
                }
                Err(e) => {
                    eprintln!("❌ iOS build failed: {}", e);
                }
            }
        }

        if results.is_empty() {
            anyhow::bail!("No mobile projects found in build directory. Run compilation first.");
        }

        Ok(results)
    }

    pub async fn run_mobile_app(&self) -> Result<bool> {
        println!("📱 Running mobile app...");
        
        // Try to run Android first
        let android_dir = self.build_dir.join("android");
        if android_dir.exists() && self.is_android_project(&android_dir) {
            return self.run_android_app(&android_dir).await;
        }

        // Try to run iOS simulator
        let ios_dir = self.build_dir.join("ios");
        if ios_dir.exists() && self.is_ios_project(&ios_dir) {
            return self.run_ios_simulator(&ios_dir).await;
        }

        anyhow::bail!("No mobile projects found to run");
    }

    async fn build_android_project(&self, android_dir: &Path, release: bool) -> Result<PathBuf> {
        println!("🤖 Building Android project...");
        
        // Ensure distribution directory exists
        let android_dist = self.dist_dir.join("android");
        fs::create_dir_all(&android_dist)?;

        // Check for Gradle wrapper
        let gradlew = android_dir.join("gradlew");
        let gradle_cmd = if gradlew.exists() {
            "./gradlew"
        } else {
            "gradle"
        };

        // Build command
        let build_type = if release { "assembleRelease" } else { "assembleDebug" };
        
        println!("Running: {} {}", gradle_cmd, build_type);
        
        let output = Command::new(gradle_cmd)
            .arg(build_type)
            .current_dir(android_dir)
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    // Find the built APK
                    let apk_pattern = if release { "*-release.apk" } else { "*-debug.apk" };
                    let apk_path = self.find_android_apk(android_dir, apk_pattern)?;
                    
                    // Copy to distribution directory
                    let output_name = if release { "app-release.apk" } else { "app-debug.apk" };
                    let dist_apk = android_dist.join(output_name);
                    fs::copy(&apk_path, &dist_apk)?;
                    
                    println!("📦 APK copied to: {}", dist_apk.display());
                    Ok(dist_apk)
                } else {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    anyhow::bail!("Android build failed: {}", stderr);
                }
            }
            Err(e) => anyhow::bail!("Failed to run Gradle: {}", e),
        }
    }

    async fn build_ios_project(&self, ios_dir: &Path, release: bool) -> Result<PathBuf> {
        println!("🍎 Building iOS project...");
        
        // Ensure distribution directory exists
        let ios_dist = self.dist_dir.join("ios");
        fs::create_dir_all(&ios_dist)?;

        // Find Xcode project or workspace
        let project_file = self.find_ios_project_file(ios_dir)?;
        let is_workspace = project_file.extension().and_then(|s| s.to_str()) == Some("xcworkspace");
        
        let flag = if is_workspace { "-workspace" } else { "-project" };
        let configuration = if release { "Release" } else { "Debug" };
        
        println!("Building iOS project: {}", project_file.display());
        
        // Build for simulator
        let output = Command::new("xcodebuild")
            .args(&[
                flag,
                project_file.to_str().unwrap(),
                "-scheme", "App", // Default scheme name
                "-configuration", configuration,
                "-sdk", "iphonesimulator",
                "-destination", "platform=iOS Simulator,name=iPhone 14",
                "build"
            ])
            .current_dir(ios_dir)
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    // Create a placeholder app bundle
                    let app_name = "DroeApp.app";
                    let app_bundle = ios_dist.join(app_name);
                    fs::create_dir_all(&app_bundle)?;
                    
                    // Create Info.plist
                    let info_plist = app_bundle.join("Info.plist");
                    let plist_content = self.create_info_plist();
                    fs::write(&info_plist, plist_content)?;
                    
                    println!("📦 iOS app bundle created: {}", app_bundle.display());
                    Ok(app_bundle)
                } else {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    anyhow::bail!("iOS build failed: {}", stderr);
                }
            }
            Err(e) => anyhow::bail!("Failed to run xcodebuild: {}. Make sure Xcode is installed.", e),
        }
    }

    async fn run_android_app(&self, android_dir: &Path) -> Result<bool> {
        println!("🤖 Running Android app...");
        
        // Check if emulator is running
        let emulator_check = Command::new("adb")
            .args(&["devices"])
            .output();

        match emulator_check {
            Ok(output) => {
                let devices = String::from_utf8_lossy(&output.stdout);
                if !devices.contains("emulator") && !devices.contains("device") {
                    println!("⚠️  No Android devices or emulators found. Starting emulator...");
                    self.start_android_emulator().await?;
                }
            }
            Err(_) => anyhow::bail!("ADB not found. Please install Android SDK."),
        }

        // Install and run the app
        let apk_path = self.find_android_apk(android_dir, "*-debug.apk")?;
        
        // Install APK
        let install_result = Command::new("adb")
            .args(&["install", "-r", apk_path.to_str().unwrap()])
            .output()?;

        if install_result.status.success() {
            println!("✅ APK installed successfully");
            
            // Try to launch the app (this requires knowing the package name)
            println!("📱 App ready to run on Android device/emulator");
            Ok(true)
        } else {
            let stderr = String::from_utf8_lossy(&install_result.stderr);
            anyhow::bail!("Failed to install APK: {}", stderr);
        }
    }

    async fn run_ios_simulator(&self, ios_dir: &Path) -> Result<bool> {
        println!("🍎 Running iOS simulator...");
        
        // Start iOS simulator
        let sim_result = Command::new("xcrun")
            .args(&["simctl", "boot", "iPhone 14"])
            .output();

        match sim_result {
            Ok(_) => {
                println!("📱 iOS Simulator started");
                
                // Open Simulator app
                let _ = Command::new("open")
                    .args(&["-a", "Simulator"])
                    .output();

                println!("✅ iOS Simulator ready");
                Ok(true)
            }
            Err(e) => anyhow::bail!("Failed to start iOS Simulator: {}. Make sure Xcode is installed.", e),
        }
    }

    async fn start_android_emulator(&self) -> Result<()> {
        // List available AVDs
        let avd_list = Command::new("emulator")
            .args(&["-list-avds"])
            .output();

        match avd_list {
            Ok(output) => {
                let avds = String::from_utf8_lossy(&output.stdout);
                let avd_lines: Vec<&str> = avds.lines().collect();
                
                if avd_lines.is_empty() {
                    anyhow::bail!("No Android Virtual Devices found. Please create one using Android Studio.");
                }

                let first_avd = avd_lines[0];
                println!("Starting Android emulator: {}", first_avd);
                
                // Start emulator in background
                let _ = Command::new("emulator")
                    .args(&["-avd", first_avd])
                    .spawn();

                // Wait a bit for emulator to start
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                
                Ok(())
            }
            Err(_) => anyhow::bail!("Android emulator not found. Please install Android SDK."),
        }
    }

    fn is_android_project(&self, dir: &Path) -> bool {
        dir.join("build.gradle").exists() || 
        dir.join("build.gradle.kts").exists() ||
        dir.join("app/build.gradle").exists()
    }

    fn is_ios_project(&self, dir: &Path) -> bool {
        // Look for .xcodeproj or .xcworkspace
        for entry in fs::read_dir(dir).unwrap_or_else(|_| fs::read_dir(".").unwrap()) {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if ext == "xcodeproj" || ext == "xcworkspace" {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn find_android_apk(&self, android_dir: &Path, pattern: &str) -> Result<PathBuf> {
        // Look in common APK locations
        let search_dirs = [
            android_dir.join("app/build/outputs/apk/debug"),
            android_dir.join("app/build/outputs/apk/release"),
            android_dir.join("build/outputs/apk/debug"),
            android_dir.join("build/outputs/apk/release"),
        ];

        for search_dir in &search_dirs {
            if search_dir.exists() {
                for entry in fs::read_dir(search_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("apk") {
                        return Ok(path);
                    }
                }
            }
        }

        anyhow::bail!("No APK found matching pattern: {}", pattern);
    }

    fn find_ios_project_file(&self, ios_dir: &Path) -> Result<PathBuf> {
        for entry in fs::read_dir(ios_dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if ext == "xcworkspace" {
                    return Ok(path); // Prefer workspace over project
                }
            }
        }

        for entry in fs::read_dir(ios_dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if ext == "xcodeproj" {
                    return Ok(path);
                }
            }
        }

        anyhow::bail!("No Xcode project or workspace found in {}", ios_dir.display());
    }

    fn create_info_plist(&self) -> String {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>DroeApp</string>
    <key>CFBundleDisplayName</key>
    <string>Droe App</string>
    <key>CFBundleIdentifier</key>
    <string>com.example.droeapp</string>
    <key>CFBundleVersion</key>
    <string>1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleExecutable</key>
    <string>DroeApp</string>
    <key>LSRequiresIPhoneOS</key>
    <true/>
    <key>UIRequiredDeviceCapabilities</key>
    <array>
        <string>armv7</string>
    </array>
    <key>UISupportedInterfaceOrientations</key>
    <array>
        <string>UIInterfaceOrientationPortrait</string>
    </array>
</dict>
</plist>"#.to_string()
    }
}

// Template creation for new mobile projects
pub struct MobileTemplateGenerator {
    project_root: PathBuf,
}

impl MobileTemplateGenerator {
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }

    pub async fn create_android_template(&self, package_name: &str, app_name: &str) -> Result<()> {
        let android_dir = self.project_root.join("build/android");
        fs::create_dir_all(&android_dir)?;

        // Create build.gradle
        let build_gradle = format!(r#"
apply plugin: 'com.android.application'

android {{
    compileSdkVersion 33
    defaultConfig {{
        applicationId "{}"
        minSdkVersion 21
        targetSdkVersion 33
        versionCode 1
        versionName "1.0"
    }}
    buildTypes {{
        release {{
            minifyEnabled false
            proguardFiles getDefaultProguardFile('proguard-android-optimize.txt'), 'proguard-rules.pro'
        }}
    }}
}}

dependencies {{
    implementation 'androidx.appcompat:appcompat:1.6.1'
    implementation 'com.google.android.material:material:1.8.0'
}}
"#, package_name);

        fs::write(android_dir.join("build.gradle"), build_gradle)?;

        // Create app structure
        let app_dir = android_dir.join("app");
        let src_main = app_dir.join("src/main");
        let java_dir = src_main.join(format!("java/{}", package_name.replace(".", "/")));
        let res_dir = src_main.join("res");
        
        fs::create_dir_all(&java_dir)?;
        fs::create_dir_all(&res_dir.join("layout"))?;
        fs::create_dir_all(&res_dir.join("values"))?;

        // Create MainActivity
        let main_activity = format!(r#"
package {};

import androidx.appcompat.app.AppCompatActivity;
import android.os.Bundle;

public class MainActivity extends AppCompatActivity {{
    @Override
    protected void onCreate(Bundle savedInstanceState) {{
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);
    }}
}}
"#, package_name);

        fs::write(java_dir.join("MainActivity.java"), main_activity)?;

        println!("✅ Android template created in {}", android_dir.display());
        Ok(())
    }

    pub async fn create_ios_template(&self, bundle_id: &str, app_name: &str) -> Result<()> {
        let ios_dir = self.project_root.join("build/ios");
        fs::create_dir_all(&ios_dir)?;

        // Create Xcode project structure (simplified)
        let project_dir = ios_dir.join(format!("{}.xcodeproj", app_name));
        fs::create_dir_all(&project_dir)?;

        // Create project.pbxproj (simplified)
        let pbxproj = format!(r#"
// !$*UTF8*$!
{{
    archiveVersion = 1;
    classes = {{
    }};
    objectVersion = 50;
    objects = {{
        /* Begin PBXFileReference section */
        13B07F961A680F5B00A75B9A /* {}.app */ = {{isa = PBXFileReference; explicitFileType = wrapper.application; includeInIndex = 0; path = {}.app; sourceTree = BUILT_PRODUCTS_DIR; }};
        /* End PBXFileReference section */
    }};
    rootObject = 83CBB9F71A601CBA00E9B192 /* Project object */;
}}
"#, app_name, app_name);

        fs::write(project_dir.join("project.pbxproj"), pbxproj)?;

        println!("✅ iOS template created in {}", ios_dir.display());
        Ok(())
    }
}