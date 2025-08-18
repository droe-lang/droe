//! Utility Functions - Common helper functions
//! 
//! This module provides utility functions for:
//! - File system operations
//! - String manipulation
//! - Path handling
//! - Validation helpers

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

/// Validate project name according to naming conventions
pub fn validate_project_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Project name cannot be empty".to_string());
    }

    if name.len() > 100 {
        return Err("Project name is too long (max 100 characters)".to_string());
    }

    // Check for valid characters
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.') {
        return Err("Project name should only contain letters, numbers, hyphens, underscores, and dots".to_string());
    }

    // Cannot start with dot or hyphen
    if name.starts_with('.') || name.starts_with('-') {
        return Err("Project name cannot start with '.' or '-'".to_string());
    }

    // Reserved names
    let reserved_names = [
        "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5",
        "com6", "com7", "com8", "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5",
        "lpt6", "lpt7", "lpt8", "lpt9", "node_modules", "build", "dist"
    ];

    if reserved_names.contains(&name.to_lowercase().as_str()) {
        return Err(format!("'{}' is a reserved name", name));
    }

    Ok(())
}

/// Validate package name for Java projects
pub fn validate_java_package_name(package: &str) -> Result<(), String> {
    if package.is_empty() {
        return Err("Package name cannot be empty".to_string());
    }

    let parts: Vec<&str> = package.split('.').collect();
    
    if parts.len() < 2 {
        return Err("Package name must have at least two parts (e.g., com.example)".to_string());
    }

    for part in parts {
        if part.is_empty() {
            return Err("Package name parts cannot be empty".to_string());
        }

        if !part.chars().next().unwrap().is_ascii_lowercase() {
            return Err("Package name parts must start with lowercase letter".to_string());
        }

        if !part.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err("Package name parts can only contain letters and numbers".to_string());
        }

        // Java keywords
        let java_keywords = [
            "abstract", "assert", "boolean", "break", "byte", "case", "catch", "char",
            "class", "const", "continue", "default", "do", "double", "else", "enum",
            "extends", "false", "final", "finally", "float", "for", "goto", "if",
            "implements", "import", "instanceof", "int", "interface", "long", "native",
            "new", "null", "package", "private", "protected", "public", "return",
            "short", "static", "strictfp", "super", "switch", "synchronized", "this",
            "throw", "throws", "transient", "true", "try", "void", "volatile", "while"
        ];

        if java_keywords.contains(&part) {
            return Err(format!("'{}' is a Java keyword and cannot be used in package name", part));
        }
    }

    Ok(())
}

/// Convert project name to various naming conventions
pub fn convert_name_formats(name: &str) -> HashMap<String, String> {
    let mut formats = HashMap::new();
    
    // kebab-case (original if already in this format)
    formats.insert("kebab".to_string(), name.to_lowercase().replace('_', "-"));
    
    // snake_case
    formats.insert("snake".to_string(), name.to_lowercase().replace('-', "_"));
    
    // camelCase
    let camel = name.split(|c| c == '-' || c == '_')
        .enumerate()
        .map(|(i, word)| {
            if i == 0 {
                word.to_lowercase()
            } else {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                }
            }
        })
        .collect::<Vec<_>>()
        .join("");
    formats.insert("camel".to_string(), camel);
    
    // PascalCase
    let pascal = name.split(|c| c == '-' || c == '_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
            }
        })
        .collect::<Vec<_>>()
        .join("");
    formats.insert("pascal".to_string(), pascal);
    
    // SCREAMING_SNAKE_CASE
    formats.insert("screaming".to_string(), name.to_uppercase().replace('-', "_"));
    
    formats
}

/// Ensure directory exists and is writable
pub fn ensure_directory_writable(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }

    if !path.is_dir() {
        anyhow::bail!("Path exists but is not a directory: {}", path.display());
    }

    // Test write permissions by creating a temporary file
    let test_file = path.join(".droe_write_test");
    match fs::write(&test_file, "test") {
        Ok(_) => {
            let _ = fs::remove_file(&test_file);
            Ok(())
        }
        Err(e) => anyhow::bail!("Directory is not writable: {} ({})", path.display(), e),
    }
}

/// Copy directory recursively with progress callback
pub fn copy_directory_with_progress<F>(
    source: &Path,
    destination: &Path,
    mut progress_callback: F,
) -> Result<()>
where
    F: FnMut(&Path, usize, usize),
{
    if !source.exists() {
        anyhow::bail!("Source directory does not exist: {}", source.display());
    }

    // Count total files first
    let total_files = count_files_recursive(source)?;
    let mut processed_files = 0;

    copy_directory_recursive(source, destination, &mut processed_files, total_files, &mut progress_callback)?;
    
    Ok(())
}

fn copy_directory_recursive<F>(
    source: &Path,
    destination: &Path,
    processed_files: &mut usize,
    total_files: usize,
    progress_callback: &mut F,
) -> Result<()>
where
    F: FnMut(&Path, usize, usize),
{
    fs::create_dir_all(destination)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let dest_path = destination.join(entry.file_name());

        if source_path.is_dir() {
            copy_directory_recursive(&source_path, &dest_path, processed_files, total_files, progress_callback)?;
        } else {
            fs::copy(&source_path, &dest_path)?;
            *processed_files += 1;
            progress_callback(&source_path, *processed_files, total_files);
        }
    }

    Ok(())
}

fn count_files_recursive(path: &Path) -> Result<usize> {
    let mut count = 0;
    
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            count += count_files_recursive(&path)?;
        } else {
            count += 1;
        }
    }
    
    Ok(count)
}

/// Format file size in human-readable format
pub fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Calculate directory size recursively
pub fn calculate_directory_size(path: &Path) -> Result<u64> {
    let mut total_size = 0;

    if path.is_file() {
        return Ok(path.metadata()?.len());
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            total_size += calculate_directory_size(&path)?;
        } else {
            total_size += path.metadata()?.len();
        }
    }

    Ok(total_size)
}

/// Find files matching a pattern in a directory
pub fn find_files_by_pattern(directory: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
    let mut matching_files = Vec::new();
    
    if !directory.exists() {
        return Ok(matching_files);
    }

    let glob_pattern = glob::Pattern::new(pattern)?;

    for entry in walkdir::WalkDir::new(directory) {
        let entry = entry?;
        if entry.file_type().is_file() {
            if let Some(file_name) = entry.file_name().to_str() {
                if glob_pattern.matches(file_name) {
                    matching_files.push(entry.path().to_path_buf());
                }
            }
        }
    }

    matching_files.sort();
    Ok(matching_files)
}

/// Extract file extension safely
pub fn get_file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}

/// Check if a path is a hidden file or directory
pub fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with('.'))
        .unwrap_or(false)
}

/// Create a backup of a file before modifying it
pub fn backup_file(file_path: &Path) -> Result<PathBuf> {
    if !file_path.exists() {
        anyhow::bail!("File does not exist: {}", file_path.display());
    }

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_path = if let Some(parent) = file_path.parent() {
        let file_name = file_path.file_name().unwrap().to_str().unwrap();
        parent.join(format!("{}.backup.{}", file_name, timestamp))
    } else {
        PathBuf::from(format!("{}.backup.{}", file_path.display(), timestamp))
    };

    fs::copy(file_path, &backup_path)?;
    println!("📄 Created backup: {}", backup_path.display());
    
    Ok(backup_path)
}

/// Restore a file from backup
pub fn restore_from_backup(backup_path: &Path, original_path: &Path) -> Result<()> {
    if !backup_path.exists() {
        anyhow::bail!("Backup file does not exist: {}", backup_path.display());
    }

    fs::copy(backup_path, original_path)?;
    println!("🔄 Restored from backup: {} -> {}", backup_path.display(), original_path.display());
    
    Ok(())
}

/// Generate a unique filename if the target already exists
pub fn generate_unique_filename(base_path: &Path) -> PathBuf {
    if !base_path.exists() {
        return base_path.to_path_buf();
    }

    let parent = base_path.parent().unwrap_or_else(|| Path::new("."));
    let stem = base_path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
    let extension = base_path.extension().and_then(|s| s.to_str()).unwrap_or("");
    
    for i in 1..1000 {
        let new_name = if extension.is_empty() {
            format!("{}_{}", stem, i)
        } else {
            format!("{}_{}.{}", stem, i, extension)
        };
        
        let new_path = parent.join(new_name);
        if !new_path.exists() {
            return new_path;
        }
    }

    // Fallback with timestamp
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let new_name = if extension.is_empty() {
        format!("{}_{}", stem, timestamp)
    } else {
        format!("{}_{}.{}", stem, timestamp, extension)
    };
    
    parent.join(new_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_project_name() {
        assert!(validate_project_name("valid-project").is_ok());
        assert!(validate_project_name("valid_project").is_ok());
        assert!(validate_project_name("ValidProject123").is_ok());
        
        assert!(validate_project_name("").is_err());
        assert!(validate_project_name(".invalid").is_err());
        assert!(validate_project_name("-invalid").is_err());
        assert!(validate_project_name("invalid/name").is_err());
        assert!(validate_project_name("con").is_err());
    }

    #[test]
    fn test_validate_java_package_name() {
        assert!(validate_java_package_name("com.example").is_ok());
        assert!(validate_java_package_name("org.springframework.boot").is_ok());
        
        assert!(validate_java_package_name("").is_err());
        assert!(validate_java_package_name("single").is_err());
        assert!(validate_java_package_name("com.").is_err());
        assert!(validate_java_package_name("com.class").is_err());
        assert!(validate_java_package_name("Com.Example").is_err());
    }

    #[test]
    fn test_convert_name_formats() {
        let formats = convert_name_formats("my-project");
        
        assert_eq!(formats.get("kebab"), Some(&"my-project".to_string()));
        assert_eq!(formats.get("snake"), Some(&"my_project".to_string()));
        assert_eq!(formats.get("camel"), Some(&"myProject".to_string()));
        assert_eq!(formats.get("pascal"), Some(&"MyProject".to_string()));
        assert_eq!(formats.get("screaming"), Some(&"MY_PROJECT".to_string()));
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1048576), "1.0 MB");
    }

    #[test]
    fn test_get_file_extension() {
        assert_eq!(get_file_extension(Path::new("file.txt")), Some("txt".to_string()));
        assert_eq!(get_file_extension(Path::new("file.TAR.GZ")), Some("gz".to_string()));
        assert_eq!(get_file_extension(Path::new("file")), None);
    }

    #[test]
    fn test_is_hidden() {
        assert!(is_hidden(Path::new(".hidden")));
        assert!(is_hidden(Path::new("dir/.hidden")));
        assert!(!is_hidden(Path::new("visible")));
        assert!(!is_hidden(Path::new("dir/visible")));
    }
}