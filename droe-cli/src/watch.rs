//! File Watcher - Watch mode functionality for file changes
//! 
//! This module provides functionality for:
//! - Watching files for changes
//! - Debounced event handling
//! - Cross-platform file system monitoring
//! - Callback-based event handling

use anyhow::Result;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use console::style;

pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    receiver: Receiver<notify::Result<Event>>,
}

impl FileWatcher {
    pub fn new() -> Result<Self> {
        let (tx, receiver) = mpsc::channel();
        
        let watcher = RecommendedWatcher::new(
            move |res| {
                if let Err(e) = tx.send(res) {
                    eprintln!("Failed to send watch event: {}", e);
                }
            },
            Config::default(),
        )?;

        Ok(Self {
            _watcher: watcher,
            receiver,
        })
    }

    pub async fn watch_file<F>(&mut self, file_path: &str, mut callback: F) -> Result<()>
    where
        F: FnMut(&Path) + Send + 'static,
    {
        let path = PathBuf::from(file_path);
        if !path.exists() {
            anyhow::bail!("File {} not found", file_path);
        }

        // Watch the parent directory to catch file modifications
        let watch_path = if path.is_file() {
            path.parent().unwrap_or(&path)
        } else {
            &path
        };

        self._watcher.watch(watch_path, RecursiveMode::NonRecursive)?;
        
        println!("{} Watching {} for changes...", 
                 style("👀").cyan(), 
                 style(file_path).yellow());
        println!("{} Press Ctrl+C to stop watching", style("ℹ️").blue());

        let target_file = path.file_name();
        let mut last_event_time = Instant::now();
        let debounce_duration = Duration::from_millis(500);

        // Initial run
        callback(&path);

        loop {
            match self.receiver.try_recv() {
                Ok(Ok(event)) => {
                    if let Some(target) = target_file {
                        if self.should_handle_event(&event, target) {
                            let now = Instant::now();
                            if now.duration_since(last_event_time) >= debounce_duration {
                                last_event_time = now;
                                
                                println!("\n{} Change detected at {}", 
                                         style("📝").cyan(),
                                         style(chrono::Local::now().format("%H:%M:%S")).dim());
                                
                                callback(&path);
                                
                                println!("{} Watching for changes...", style("👀").cyan());
                            }
                        }
                    }
                }
                Ok(Err(e)) => {
                    eprintln!("Watch error: {}", e);
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // No events, continue watching
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    anyhow::bail!("Watch channel disconnected");
                }
            }
            
            // Small delay to prevent busy waiting
            sleep(Duration::from_millis(100)).await;
        }
    }

    pub async fn watch_directory<F>(&mut self, dir_path: &str, pattern: &str, mut callback: F) -> Result<()>
    where
        F: FnMut(&Path) + Send + 'static,
    {
        let path = PathBuf::from(dir_path);
        if !path.exists() || !path.is_dir() {
            anyhow::bail!("Directory {} not found", dir_path);
        }

        self._watcher.watch(&path, RecursiveMode::Recursive)?;
        
        println!("{} Watching {} (pattern: {}) for changes...", 
                 style("👀").cyan(), 
                 style(dir_path).yellow(),
                 style(pattern).yellow());
        println!("{} Press Ctrl+C to stop watching", style("ℹ️").blue());

        let mut last_event_time = Instant::now();
        let debounce_duration = Duration::from_millis(500);

        loop {
            match self.receiver.try_recv() {
                Ok(Ok(event)) => {
                    if self.should_handle_directory_event(&event, pattern) {
                        let now = Instant::now();
                        if now.duration_since(last_event_time) >= debounce_duration {
                            last_event_time = now;
                            
                            if let Some(changed_path) = self.get_changed_path(&event) {
                                println!("\n{} Change detected: {} at {}", 
                                         style("📝").cyan(),
                                         style(changed_path.display()).yellow(),
                                         style(chrono::Local::now().format("%H:%M:%S")).dim());
                                
                                callback(&changed_path);
                                
                                println!("{} Watching for changes...", style("👀").cyan());
                            }
                        }
                    }
                }
                Ok(Err(e)) => {
                    eprintln!("Watch error: {}", e);
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // No events, continue watching
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    anyhow::bail!("Watch channel disconnected");
                }
            }
            
            // Small delay to prevent busy waiting
            sleep(Duration::from_millis(100)).await;
        }
    }

    fn should_handle_event(&self, event: &Event, target_file: &std::ffi::OsStr) -> bool {
        match &event.kind {
            EventKind::Modify(_) | EventKind::Create(_) => {
                event.paths.iter().any(|p| {
                    p.file_name() == Some(target_file)
                })
            }
            _ => false,
        }
    }

    fn should_handle_directory_event(&self, event: &Event, pattern: &str) -> bool {
        match &event.kind {
            EventKind::Modify(_) | EventKind::Create(_) => {
                event.paths.iter().any(|p| {
                    if let Some(file_name) = p.file_name().and_then(|n| n.to_str()) {
                        self.matches_pattern(file_name, pattern)
                    } else {
                        false
                    }
                })
            }
            _ => false,
        }
    }

    fn matches_pattern(&self, file_name: &str, pattern: &str) -> bool {
        // Simple pattern matching - support *.ext patterns
        if pattern.starts_with("*.") {
            let extension = &pattern[2..];
            file_name.ends_with(extension)
        } else if pattern.contains("*") {
            // More complex glob patterns could be implemented here
            glob::Pattern::new(pattern)
                .map(|p| p.matches(file_name))
                .unwrap_or(false)
        } else {
            file_name == pattern
        }
    }

    fn get_changed_path(&self, event: &Event) -> Option<PathBuf> {
        event.paths.first().cloned()
    }
}

pub struct MultiFileWatcher {
    watchers: Vec<FileWatcher>,
}

impl MultiFileWatcher {
    pub fn new() -> Self {
        Self {
            watchers: Vec::new(),
        }
    }

    pub async fn watch_files<F>(&mut self, file_paths: &[&str], callback: F) -> Result<()>
    where
        F: Fn(&Path) + Send + Clone + 'static,
    {
        println!("{} Watching {} files for changes...", 
                 style("👀").cyan(), 
                 style(file_paths.len()).yellow());

        let handles: Vec<_> = file_paths.iter().map(|&file_path| {
            let callback = callback.clone();
            let file_path = file_path.to_string();
            
            tokio::spawn(async move {
                let mut watcher = FileWatcher::new().unwrap();
                watcher.watch_file(&file_path, move |path| callback(path)).await
            })
        }).collect();

        // Wait for all watchers (this will run indefinitely until interrupted)
        for handle in handles {
            if let Err(e) = handle.await {
                eprintln!("Watcher task failed: {}", e);
            }
        }

        Ok(())
    }
}

pub async fn watch_project_files<F>(project_root: &Path, mut callback: F) -> Result<()>
where
    F: FnMut(&Path) + Send + 'static,
{
    let mut watcher = FileWatcher::new()?;
    let src_dir = project_root.join("src");
    
    if !src_dir.exists() {
        anyhow::bail!("Source directory not found: {}", src_dir.display());
    }

    watcher.watch_directory(
        src_dir.to_str().unwrap(),
        "*.droe",
        callback,
    ).await
}

// Convenience function for watching a single file with async callback
pub async fn watch_file_async<F, Fut>(file_path: &str, callback: F) -> Result<()>
where
    F: Fn(&Path) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send,
{
    let mut watcher = FileWatcher::new()?;
    let callback = std::sync::Arc::new(callback);
    
    watcher.watch_file(file_path, move |path| {
        let callback = callback.clone();
        let path = path.to_owned();
        tokio::spawn(async move {
            callback(&path).await;
        });
    }).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_pattern_matching() {
        let watcher = FileWatcher::new().unwrap();
        
        assert!(watcher.matches_pattern("test.droe", "*.droe"));
        assert!(watcher.matches_pattern("main.rs", "*.rs"));
        assert!(!watcher.matches_pattern("test.droe", "*.rs"));
        assert!(watcher.matches_pattern("exact.txt", "exact.txt"));
    }

    #[tokio::test]
    async fn test_file_watcher_creation() {
        let watcher = FileWatcher::new();
        assert!(watcher.is_ok());
    }
}