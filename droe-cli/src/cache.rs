//! Cache System - SQLite-based compilation caching for Puck JSON
//! 
//! This module provides functionality for:
//! - Storing Puck JSON compilations in SQLite
//! - Retrieving cached compilations
//! - Hash-based change detection
//! - Project-scoped caching

use anyhow::{Result, Context};
use rusqlite::{Connection, params};
use sha2::{Sha256, Digest};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use home::home_dir;

pub struct PuckCache {
    db_path: PathBuf,
}

#[derive(Debug)]
pub struct CachedCompilation {
    pub id: i64,
    pub project_id: String,
    pub file_path: String,
    pub dsl_source: String,
    pub dsl_hash: String,
    pub puck_json: String,
    pub compiled_at: DateTime<Utc>,
}

impl PuckCache {
    pub fn new() -> Result<Self> {
        let db_path = home_dir()
            .context("Failed to get home directory")?
            .join(".droelang")
            .join("puck_compilations.db");
        
        // Ensure directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let cache = Self { db_path };
        cache.init_database()?;
        Ok(cache)
    }

    fn init_database(&self) -> Result<()> {
        let conn = Connection::open(&self.db_path)?;
        
        // Create table for Puck compilations
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS puck_compilations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id TEXT,
                file_path TEXT NOT NULL,
                dsl_source TEXT NOT NULL,
                dsl_hash TEXT NOT NULL,
                puck_json TEXT NOT NULL,
                compiled_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(project_id, file_path)
            )
            "#,
            [],
        )?;
        
        // Create index for faster lookups
        conn.execute(
            r#"
            CREATE INDEX IF NOT EXISTS idx_puck_project_file 
            ON puck_compilations(project_id, file_path)
            "#,
            [],
        )?;
        
        Ok(())
    }

    pub fn store_compilation(
        &self,
        file_path: &Path,
        dsl_source: &str,
        puck_json: &str,
        project_id: Option<&str>,
    ) -> Result<i64> {
        let dsl_hash = self.calculate_hash(dsl_source);
        let project_id = project_id.unwrap_or("default");
        let file_path_str = file_path.to_string_lossy();
        let now = Utc::now();
        
        let conn = Connection::open(&self.db_path)?;
        
        conn.execute(
            r#"
            INSERT OR REPLACE INTO puck_compilations 
            (project_id, file_path, dsl_source, dsl_hash, puck_json, compiled_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                project_id,
                file_path_str,
                dsl_source,
                dsl_hash,
                puck_json,
                now.format("%Y-%m-%d %H:%M:%S").to_string()
            ],
        )?;
        
        let compilation_id = conn.last_insert_rowid();
        println!("💾 Stored Puck JSON in cache (ID: {})", compilation_id);
        
        Ok(compilation_id)
    }

    pub fn get_compilation(
        &self,
        file_path: &Path,
        project_id: Option<&str>,
    ) -> Result<Option<CachedCompilation>> {
        let project_id = project_id.unwrap_or("default");
        let file_path_str = file_path.to_string_lossy();
        
        let conn = Connection::open(&self.db_path)?;
        
        let mut stmt = conn.prepare(
            r#"
            SELECT id, project_id, file_path, dsl_source, dsl_hash, puck_json, compiled_at
            FROM puck_compilations 
            WHERE project_id = ?1 AND file_path = ?2
            "#,
        )?;
        
        let mut rows = stmt.query_map(params![project_id, file_path_str], |row| {
            Ok(CachedCompilation {
                id: row.get(0)?,
                project_id: row.get(1)?,
                file_path: row.get(2)?,
                dsl_source: row.get(3)?,
                dsl_hash: row.get(4)?,
                puck_json: row.get(5)?,
                compiled_at: DateTime::parse_from_str(&row.get::<_, String>(6)?, "%Y-%m-%d %H:%M:%S")
                    .map_err(|e| rusqlite::Error::InvalidColumnType(6, "datetime".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc),
            })
        })?;
        
        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    pub fn check_cache(
        &self,
        file_path: &Path,
        dsl_source: &str,
        project_id: Option<&str>,
    ) -> Result<(bool, Option<String>)> {
        if let Some(cached) = self.get_compilation(file_path, project_id)? {
            let current_hash = self.calculate_hash(dsl_source);
            
            if cached.dsl_hash == current_hash {
                println!("✅ Using cached Puck JSON (no changes detected)");
                return Ok((true, Some(cached.puck_json)));
            } else {
                println!("🔄 DSL source changed, cache invalid");
            }
        }
        
        Ok((false, None))
    }

    pub fn list_cached_compilations(&self, project_id: Option<&str>) -> Result<Vec<CachedCompilation>> {
        let conn = Connection::open(&self.db_path)?;
        
        let (query, params): (&str, Vec<&dyn rusqlite::ToSql>) = if let Some(pid) = project_id {
            (
                r#"
                SELECT id, project_id, file_path, dsl_source, dsl_hash, puck_json, compiled_at
                FROM puck_compilations 
                WHERE project_id = ?1
                ORDER BY compiled_at DESC
                "#,
                vec![&pid],
            )
        } else {
            (
                r#"
                SELECT id, project_id, file_path, dsl_source, dsl_hash, puck_json, compiled_at
                FROM puck_compilations 
                ORDER BY compiled_at DESC
                "#,
                vec![],
            )
        };
        
        let mut stmt = conn.prepare(query)?;
        let rows = stmt.query_map(params.as_slice(), |row| {
            Ok(CachedCompilation {
                id: row.get(0)?,
                project_id: row.get(1)?,
                file_path: row.get(2)?,
                dsl_source: row.get(3)?,
                dsl_hash: row.get(4)?,
                puck_json: row.get(5)?,
                compiled_at: DateTime::parse_from_str(&row.get::<_, String>(6)?, "%Y-%m-%d %H:%M:%S")
                    .map_err(|e| rusqlite::Error::InvalidColumnType(6, "datetime".to_string(), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc),
            })
        })?;
        
        let mut compilations = Vec::new();
        for row in rows {
            compilations.push(row?);
        }
        
        Ok(compilations)
    }

    pub fn clear_cache(&self, project_id: Option<&str>) -> Result<usize> {
        let conn = Connection::open(&self.db_path)?;
        
        let rows_affected = if let Some(pid) = project_id {
            conn.execute(
                "DELETE FROM puck_compilations WHERE project_id = ?1",
                params![pid],
            )?
        } else {
            conn.execute("DELETE FROM puck_compilations", [])?
        };
        
        if let Some(pid) = project_id {
            println!("🧹 Cleared {} cached compilations for project '{}'", rows_affected, pid);
        } else {
            println!("🧹 Cleared {} cached compilations", rows_affected);
        }
        
        Ok(rows_affected)
    }

    pub fn get_cache_stats(&self) -> Result<CacheStats> {
        let conn = Connection::open(&self.db_path)?;
        
        let total_compilations: i64 = conn.query_row(
            "SELECT COUNT(*) FROM puck_compilations",
            [],
            |row| row.get(0),
        )?;
        
        let total_projects: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT project_id) FROM puck_compilations",
            [],
            |row| row.get(0),
        )?;
        
        let most_recent: Option<DateTime<Utc>> = conn.query_row(
            "SELECT MAX(compiled_at) FROM puck_compilations",
            [],
            |row| {
                let timestamp: Option<String> = row.get(0)?;
                if let Some(ts) = timestamp {
                    Ok(Some(DateTime::parse_from_str(&ts, "%Y-%m-%d %H:%M:%S")
                        .map_err(|_| rusqlite::Error::InvalidColumnType(0, "datetime".to_string(), rusqlite::types::Type::Text))?
                        .with_timezone(&Utc)))
                } else {
                    Ok(None)
                }
            },
        )?;
        
        // Calculate total size of DSL sources and Puck JSON
        let total_size: i64 = conn.query_row(
            "SELECT SUM(LENGTH(dsl_source) + LENGTH(puck_json)) FROM puck_compilations",
            [],
            |row| row.get(0),
        )?;
        
        Ok(CacheStats {
            total_compilations: total_compilations as usize,
            total_projects: total_projects as usize,
            most_recent_compilation: most_recent,
            total_size_bytes: total_size as usize,
        })
    }

    fn calculate_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub total_compilations: usize,
    pub total_projects: usize,
    pub most_recent_compilation: Option<DateTime<Utc>>,
    pub total_size_bytes: usize,
}

impl CacheStats {
    pub fn display(&self) {
        println!("📊 Cache Statistics:");
        println!("  Total compilations: {}", self.total_compilations);
        println!("  Total projects: {}", self.total_projects);
        if let Some(recent) = &self.most_recent_compilation {
            println!("  Most recent: {}", recent.format("%Y-%m-%d %H:%M:%S UTC"));
        } else {
            println!("  Most recent: None");
        }
        println!("  Total size: {:.2} KB", self.total_size_bytes as f64 / 1024.0);
    }
}

// Convenience functions for global cache operations
pub fn init_puck_cache() -> Result<PuckCache> {
    PuckCache::new()
}

pub fn store_puck_compilation(
    file_path: &Path,
    dsl_source: &str,
    puck_json: &str,
    project_id: Option<&str>,
) -> Result<i64> {
    let cache = PuckCache::new()?;
    cache.store_compilation(file_path, dsl_source, puck_json, project_id)
}

pub fn get_puck_compilation(
    file_path: &Path,
    project_id: Option<&str>,
) -> Result<Option<CachedCompilation>> {
    let cache = PuckCache::new()?;
    cache.get_compilation(file_path, project_id)
}

pub fn check_puck_cache(
    file_path: &Path,
    dsl_source: &str,
    project_id: Option<&str>,
) -> Result<(bool, Option<String>)> {
    let cache = PuckCache::new()?;
    cache.check_cache(file_path, dsl_source, project_id)
}