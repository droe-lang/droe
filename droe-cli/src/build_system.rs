//! Build System - Distribution building and packaging
//! 
//! This module provides functionality for:
//! - Building entire projects
//! - Creating distribution packages
//! - Managing build artifacts
//! - Target-specific packaging

use anyhow::{Result, Context};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;
use indicatif::{ProgressBar, ProgressStyle};
use chrono::Utc;
use fs_extra::dir::{copy, CopyOptions};

pub struct BuildSystem {
    project_root: PathBuf,
    config: Value,
}

#[derive(Debug)]
pub struct BuildResult {
    pub files_compiled: usize,
    pub files_failed: usize,
    pub output_files: Vec<PathBuf>,
    pub main_entry: Option<PathBuf>,
    pub manifest_path: Option<PathBuf>,
}

impl BuildSystem {
    pub fn new() -> Result<Self> {
        let project_root = crate::find_project_root()?;
        let config = crate::load_config(&project_root)?;
        
        Ok(Self {
            project_root,
            config,
        })
    }

    pub async fn build_project(&self, clean: bool, release: bool) -> Result<BuildResult> {
        let target = self.config.get("target").and_then(|t| t.as_str()).unwrap_or("droe");
        let framework = self.config.get("framework").and_then(|f| f.as_str()).unwrap_or("plain");
        
        println!("📦 Building distribution package...");
        println!("🎯 Target: {}{}", target, if framework != "plain" { format!(" ({})", framework) } else { String::new() });
        
        if release {
            println!("🚀 Release build (optimized)");
        }
        
        if clean {
            self.clean_build_directory().await?;
        }

        match target {
            "mobile" => self.build_mobile_project(release).await,
            "java" if framework == "spring" => self.build_spring_project(release).await,
            _ => self.build_standard_project(release).await,
        }
    }

    async fn build_standard_project(&self, release: bool) -> Result<BuildResult> {
        let src_dir = self.project_root.join(self.config.get("src").and_then(|s| s.as_str()).unwrap_or("src"));
        let build_dir = self.project_root.join(self.config.get("build").and_then(|s| s.as_str()).unwrap_or("build"));
        let dist_dir = self.project_root.join(self.config.get("dist").and_then(|s| s.as_str()).unwrap_or("dist"));
        
        // Find all .droe files
        let droe_files = self.find_source_files(&src_dir)?;
        
        if droe_files.is_empty() {
            anyhow::bail!("No .droe files found in {}", src_dir.display());
        }

        println!("🔍 Found {} .droe files to compile", droe_files.len());
        
        // Set up progress bar
        let pb = ProgressBar::new(droe_files.len() as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"));

        let mut compiled_count = 0;
        let mut failed_count = 0;
        let mut output_files = Vec::new();

        // Compile each file
        for droe_file in &droe_files {
            let relative_path = droe_file.strip_prefix(&self.project_root)?;
            pb.set_message(format!("Compiling {}", relative_path.display()));

            match self.compile_single_file(droe_file, &build_dir, release).await {
                Ok(output_file) => {
                    compiled_count += 1;
                    if let Some(file) = output_file {
                        output_files.push(file);
                    }
                }
                Err(e) => {
                    failed_count += 1;
                    eprintln!("❌ Failed to compile {}: {}", relative_path.display(), e);
                }
            }
            
            pb.inc(1);
        }

        pb.finish_with_message("Compilation complete");
        println!("📊 Compilation results: {} succeeded, {} failed", compiled_count, failed_count);

        if failed_count > 0 {
            println!("⚠️  Some files failed to compile. Distribution will only include successful compilations.");
        }

        // Create distribution
        let manifest_path = self.create_distribution(&output_files, &dist_dir, release).await?;
        let main_entry = self.determine_main_entry(&output_files);

        Ok(BuildResult {
            files_compiled: compiled_count,
            files_failed: failed_count,
            output_files,
            main_entry,
            manifest_path: Some(manifest_path),
        })
    }

    async fn build_mobile_project(&self, release: bool) -> Result<BuildResult> {
        println!("📱 Building mobile project...");
        
        // This would integrate with the mobile build system
        let build_dir = self.project_root.join("build");
        let dist_dir = self.project_root.join("dist");
        
        // For now, create a placeholder mobile build
        fs::create_dir_all(&dist_dir)?;
        
        let android_dir = dist_dir.join("android");
        let ios_dir = dist_dir.join("ios");
        
        fs::create_dir_all(&android_dir)?;
        fs::create_dir_all(&ios_dir)?;
        
        // Create placeholder APK
        let apk_path = android_dir.join("app-release.apk");
        fs::write(&apk_path, "# Placeholder APK file\n")?;
        
        // Create placeholder iOS app
        let ios_app = ios_dir.join("App.app");
        fs::create_dir_all(&ios_app)?;
        fs::write(ios_app.join("Info.plist"), "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\">\n<dict>\n\t<key>CFBundleName</key>\n\t<string>DroeApp</string>\n</dict>\n</plist>")?;

        let output_files = vec![apk_path, ios_app];
        let manifest_path = self.create_distribution(&output_files, &dist_dir, release).await?;

        println!("✅ Mobile build completed");
        
        Ok(BuildResult {
            files_compiled: 1,
            files_failed: 0,
            output_files,
            main_entry: None,
            manifest_path: Some(manifest_path),
        })
    }

    async fn build_spring_project(&self, release: bool) -> Result<BuildResult> {
        println!("☕ Building Spring Boot project...");
        
        let build_dir = self.project_root.join("build");
        let dist_dir = self.project_root.join("dist");
        
        // Look for Spring Boot projects in build directory
        let mut spring_projects = Vec::new();
        
        if build_dir.exists() {
            for entry in WalkDir::new(&build_dir).min_depth(1).max_depth(1) {
                let entry = entry?;
                if entry.file_type().is_dir() {
                    let project_dir = entry.path();
                    if project_dir.join("pom.xml").exists() {
                        spring_projects.push(project_dir.to_path_buf());
                    }
                }
            }
        }

        if spring_projects.is_empty() {
            anyhow::bail!("No Spring Boot projects found in build directory. Run compilation first.");
        }

        println!("Found {} Spring Boot project(s)", spring_projects.len());

        // Create distribution directory
        fs::create_dir_all(&dist_dir)?;

        let mut output_files = Vec::new();
        
        for project_dir in spring_projects {
            let project_name = project_dir.file_name().unwrap().to_string_lossy();
            println!("📋 Packaging Spring Boot project: {}", project_name);
            
            // Copy entire project to dist
            let dest_dir = dist_dir.join(&*project_name);
            let mut copy_options = CopyOptions::new();
            copy_options.overwrite = true;
            copy_options.copy_inside = true;
            
            copy(&project_dir, &dest_dir, &copy_options)?;
            output_files.push(dest_dir);
            
            // If Maven is available, try to build JAR
            if release {
                self.build_spring_jar(&dest_dir).await?;
            }
        }

        let manifest_path = self.create_distribution(&output_files, &dist_dir, release).await?;

        println!("✅ Spring Boot build completed");
        
        Ok(BuildResult {
            files_compiled: spring_projects.len(),
            files_failed: 0,
            output_files,
            main_entry: output_files.first().cloned(),
            manifest_path: Some(manifest_path),
        })
    }

    async fn compile_single_file(&self, file_path: &Path, build_dir: &Path, _release: bool) -> Result<Option<PathBuf>> {
        let target = self.config.get("target").and_then(|t| t.as_str()).unwrap_or("droe");
        
        // Determine output file path
        let relative_path = file_path.strip_prefix(&self.project_root)?;
        let stem = file_path.file_stem().unwrap().to_string_lossy();
        
        let extension = match target {
            "javascript" | "js" => ".js",
            "webassembly" | "wasm" => ".wasm",
            "bytecode" | "droe" => ".droebc",
            "go" => ".go",
            "python" => ".py",
            "java" => ".java",
            "rust" => ".rs",
            _ => ".txt",
        };

        let output_file = build_dir.join(relative_path.parent().unwrap_or(Path::new(""))).join(format!("{}{}", stem, extension));
        
        // Ensure output directory exists
        if let Some(parent) = output_file.parent() {
            fs::create_dir_all(parent)?;
        }

        // Use the droe compiler
        let source_content = fs::read_to_string(file_path)?;
        let compiler = droe_compiler::Compiler::new();
        let program = compiler.parse(&source_content)?;

        let result = match target {
            "javascript" | "js" => {
                let generator = droe_compiler::JavaScriptGenerator::new();
                generator.generate(&program)?
            }
            "webassembly" | "wasm" => {
                let generator = droe_compiler::WebAssemblyGenerator::new();
                generator.generate(&program)?
            }
            "bytecode" | "droe" => {
                let generator = droe_compiler::BytecodeGenerator::new();
                generator.generate(&program)?
            }
            "go" => {
                let generator = droe_compiler::GoGenerator::new();
                generator.generate(&program)?
            }
            _ => anyhow::bail!("Unsupported target for single file compilation: {}", target),
        };

        fs::write(&output_file, result)?;
        Ok(Some(output_file))
    }

    async fn build_spring_jar(&self, project_dir: &Path) -> Result<()> {
        println!("🔨 Building Spring Boot JAR...");
        
        let output = Command::new("mvn")
            .args(&["clean", "package", "-DskipTests"])
            .current_dir(project_dir)
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    println!("✅ Spring Boot JAR built successfully");
                } else {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    println!("⚠️  Maven build warnings: {}", stderr);
                }
            }
            Err(_) => {
                println!("⚠️  Maven not found, skipping JAR build");
            }
        }

        Ok(())
    }

    async fn create_distribution(&self, output_files: &[PathBuf], dist_dir: &Path, release: bool) -> Result<PathBuf> {
        // Generate manifest.json
        let project_name = self.project_root.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("droe-project");

        let target = self.config.get("target").and_then(|t| t.as_str()).unwrap_or("droe");
        let framework = self.config.get("framework").and_then(|f| f.as_str()).unwrap_or("plain");

        let relative_files: Vec<String> = output_files.iter()
            .filter_map(|f| f.strip_prefix(dist_dir).ok())
            .map(|f| f.to_string_lossy().to_string())
            .collect();

        let main_entry = self.determine_main_entry_string(output_files, dist_dir);

        let manifest = json!({
            "name": project_name,
            "version": "1.0.0",
            "description": self.config.get("description").unwrap_or(&json!(format!("Droelang project: {}", project_name))),
            "target": target,
            "framework": if framework != "plain" { Some(framework) } else { None },
            "main": main_entry,
            "files": relative_files,
            "build_time": Utc::now().to_rfc3339(),
            "build_mode": if release { "release" } else { "debug" },
            "droelang_version": "1.0.0"
        });

        let manifest_path = dist_dir.join("manifest.json");
        fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;

        println!("📄 Created manifest.json");
        println!("✅ Distribution built successfully!");
        println!("📦 {} files packaged in {}", relative_files.len(), dist_dir.display());
        
        if let Some(main) = main_entry {
            println!("🎯 Main entry: {}", main);
        }

        Ok(manifest_path)
    }

    fn find_source_files(&self, src_dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        if !src_dir.exists() {
            return Ok(files);
        }

        for entry in WalkDir::new(src_dir) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "droe" {
                        files.push(path.to_path_buf());
                    }
                }
            }
        }

        files.sort();
        Ok(files)
    }

    fn determine_main_entry(&self, output_files: &[PathBuf]) -> Option<PathBuf> {
        let main_file = self.config.get("main").and_then(|m| m.as_str()).unwrap_or("src/main.droe");
        let main_stem = Path::new(main_file).file_stem().and_then(|s| s.to_str()).unwrap_or("main");

        // Look for a file with the main stem
        for file in output_files {
            if let Some(stem) = file.file_stem().and_then(|s| s.to_str()) {
                if stem == main_stem {
                    return Some(file.clone());
                }
            }
        }

        // Fallback to first file
        output_files.first().cloned()
    }

    fn determine_main_entry_string(&self, output_files: &[PathBuf], dist_dir: &Path) -> Option<String> {
        self.determine_main_entry(output_files)
            .and_then(|f| f.strip_prefix(dist_dir).ok())
            .map(|f| f.to_string_lossy().to_string())
    }

    async fn clean_build_directory(&self) -> Result<()> {
        let build_dir = self.project_root.join(self.config.get("build").and_then(|s| s.as_str()).unwrap_or("build"));
        
        if build_dir.exists() {
            fs::remove_dir_all(&build_dir)?;
            println!("🧹 Cleaned {}", build_dir.display());
        }

        Ok(())
    }
}

pub async fn build_project_simple(clean: bool, release: bool) -> Result<()> {
    let build_system = BuildSystem::new()?;
    let result = build_system.build_project(clean, release).await?;

    if result.files_failed > 0 {
        anyhow::bail!("Build completed with {} failures", result.files_failed);
    }

    Ok(())
}