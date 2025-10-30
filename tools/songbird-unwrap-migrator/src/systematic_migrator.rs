//! Systematic Unwrap Migrator - Core Logic
//!
//! Systematically replaces unwrap(), expect(), and panic! patterns with proper
//! Songbird error handling using SongbirdError and SongbirdResult.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use regex::Regex;
use thiserror::Error;
use tracing::{info, error};

#[derive(Error, Debug)]
pub enum MigratorError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
    #[error("Migration error: {message}")]
    Migration { message: String },
}

pub type MigratorResult<T> = Result<T, MigratorError>;

pub struct SystematicUnwrapMigrator {
    error_patterns: HashMap<String, MigrationPattern>,
    files_processed: std::sync::atomic::AtomicU64,
    migrations_applied: std::sync::atomic::AtomicU64,
}

#[derive(Clone)]
pub struct MigrationPattern {
    pub pattern: String,
    pub replacement: String,
    pub error_category: String,
    pub context: String,
    pub songbird_compatible: bool,
}

#[derive(Debug)]
pub struct CodebaseStats {
    pub files_scanned: usize,
    pub total_unwrap_calls: usize,
    pub migrable_patterns: usize,
    pub test_file_patterns: usize,
    pub songbird_compatible: usize,
    pub pattern_categories: HashMap<String, usize>,
}

#[derive(Debug)]
pub struct MigrationResult {
    pub files_processed: usize,
    pub migrations_applied: usize,
    pub failed_files: Vec<(PathBuf, String)>,
    pub execution_time_ms: u64,
}

impl SystematicUnwrapMigrator {
    /// Create a new migrator optimized for Songbird error patterns
    pub fn new_songbird_optimized() -> Self {
        let mut error_patterns = HashMap::with_capacity(20);

        // Configuration patterns
        error_patterns.insert(
            "env_var_unwrap".to_string(),
            MigrationPattern {
                pattern: r#"std::env::var\("([^"]+)"\)\.unwrap\(\)"#.to_string(),
                replacement: r#"std::env::var("$1").map_err(|e| SongbirdError::configuration(format!("Environment variable '{}' not found: {}", "$1", e)))?"#.to_string(),
                error_category: "Configuration".to_string(),
                context: "Environment variable access".to_string(),
                songbird_compatible: true,
            }
        );

        error_patterns.insert(
            "env_var_expect".to_string(),
            MigrationPattern {
                pattern: r#"std::env::var\("([^"]+)"\)\.expect\("([^"]+)"\)"#.to_string(),
                replacement: r#"std::env::var("$1").map_err(|e| SongbirdError::configuration(format!("$2: {}", e)))?"#.to_string(),
                error_category: "Configuration".to_string(),
                context: "Environment variable with expect".to_string(),
                songbird_compatible: true,
            }
        );

        // JSON patterns
        error_patterns.insert(
            "json_from_str_unwrap".to_string(),
            MigrationPattern {
                pattern: r#"serde_json::from_str\(([^)]+)\)\.unwrap\(\)"#.to_string(),
                replacement: r#"serde_json::from_str($1).map_err(|e| SongbirdError::Serialization { format: Some("JSON".to_string()), message: format!("Parsing failed: {}", e), debug_info: None })?"#.to_string(),
                error_category: "Validation".to_string(),
                context: "JSON deserialization".to_string(),
                songbird_compatible: true,
            }
        );

        error_patterns.insert(
            "json_from_str_expect".to_string(),
            MigrationPattern {
                pattern: r#"serde_json::from_str\(([^)]+)\)\.expect\("([^"]+)"\)"#.to_string(),
                replacement: r#"serde_json::from_str($1).map_err(|e| SongbirdError::Serialization { format: Some("JSON".to_string()), message: format!("$2: {}", e), debug_info: None })?"#.to_string(),
                error_category: "Validation".to_string(),
                context: "JSON deserialization with expect".to_string(),
                songbird_compatible: true,
            }
        );

        error_patterns.insert(
            "json_to_string_unwrap".to_string(),
            MigrationPattern {
                pattern: r#"serde_json::to_string\(([^)]+)\)\.unwrap\(\)"#.to_string(),
                replacement: r#"serde_json::to_string($1).map_err(|e| SongbirdError::Serialization { format: Some("JSON".to_string()), message: format!("Serialization failed: {}", e), debug_info: None })?"#.to_string(),
                error_category: "Validation".to_string(),
                context: "JSON serialization".to_string(),
                songbird_compatible: true,
            }
        );

        // Network patterns
        error_patterns.insert(
            "http_send_unwrap".to_string(),
            MigrationPattern {
                pattern: r#"\.send\(\)\.await\.unwrap\(\)"#.to_string(),
                replacement: r#".send().await.map_err(|e| SongbirdError::network(format!("HTTP request failed: {}", e)))?"#.to_string(),
                error_category: "Network".to_string(),
                context: "HTTP request execution".to_string(),
                songbird_compatible: true,
            }
        );

        error_patterns.insert(
            "http_send_expect".to_string(),
            MigrationPattern {
                pattern: r#"\.send\(\)\.await\.expect\("([^"]+)"\)"#.to_string(),
                replacement: r#".send().await.map_err(|e| SongbirdError::network(format!("$1: {}", e)))?"#.to_string(),
                error_category: "Network".to_string(),
                context: "HTTP request with expect".to_string(),
                songbird_compatible: true,
            }
        );

        // File I/O patterns
        error_patterns.insert(
            "file_read_unwrap".to_string(),
            MigrationPattern {
                pattern: r#"fs::read_to_string\(([^)]+)\)\.unwrap\(\)"#.to_string(),
                replacement: r#"fs::read_to_string($1).map_err(|e| SongbirdError::configuration(format!("File read failed: {}", e)))?"#.to_string(),
                error_category: "Storage".to_string(),
                context: "File read operation".to_string(),
                songbird_compatible: true,
            }
        );

        error_patterns.insert(
            "file_write_unwrap".to_string(),
            MigrationPattern {
                pattern: r#"fs::write\(([^,]+),\s*([^)]+)\)\.unwrap\(\)"#.to_string(),
                replacement: r#"fs::write($1, $2).map_err(|e| SongbirdError::configuration(format!("File write failed: {}", e)))?"#.to_string(),
                error_category: "Storage".to_string(),
                context: "File write operation".to_string(),
                songbird_compatible: true,
            }
        );

        // Lock patterns (non-panicking recovery)
        error_patterns.insert(
            "lock_unwrap".to_string(),
            MigrationPattern {
                pattern: r#"\.lock\(\)\.unwrap\(\)"#.to_string(),
                replacement: r#".lock().unwrap_or_else(|poisoned| {
        tracing::warn!("Mutex poisoned, recovering");
        poisoned.into_inner()
    })"#.to_string(),
                error_category: "System".to_string(),
                context: "Mutex lock acquisition".to_string(),
                songbird_compatible: true,
            }
        );

        error_patterns.insert(
            "read_lock_unwrap".to_string(),
            MigrationPattern {
                pattern: r#"\.read\(\)\.unwrap\(\)"#.to_string(),
                replacement: r#".read().unwrap_or_else(|poisoned| {
        tracing::warn!("RwLock poisoned for read, recovering");
        poisoned.into_inner()
    })"#.to_string(),
                error_category: "System".to_string(),
                context: "RwLock read operation".to_string(),
                songbird_compatible: true,
            }
        );

        error_patterns.insert(
            "write_lock_unwrap".to_string(),
            MigrationPattern {
                pattern: r#"\.write\(\)\.unwrap\(\)"#.to_string(),
                replacement: r#".write().unwrap_or_else(|poisoned| {
        tracing::warn!("RwLock poisoned for write, recovering");
        poisoned.into_inner()
    })"#.to_string(),
                error_category: "System".to_string(),
                context: "RwLock write operation".to_string(),
                songbird_compatible: true,
            }
        );

        // Collection patterns
        error_patterns.insert(
            "first_unwrap".to_string(),
            MigrationPattern {
                pattern: r#"\.first\(\)\.unwrap\(\)"#.to_string(),
                replacement: r#".first().ok_or_else(|| SongbirdError::configuration("Collection is empty when accessing first element".to_string()))?"#.to_string(),
                error_category: "Validation".to_string(),
                context: "Collection first element access".to_string(),
                songbird_compatible: true,
            }
        );

        error_patterns.insert(
            "last_unwrap".to_string(),
            MigrationPattern {
                pattern: r#"\.last\(\)\.unwrap\(\)"#.to_string(),
                replacement: r#".last().ok_or_else(|| SongbirdError::configuration("Collection is empty when accessing last element".to_string()))?"#.to_string(),
                error_category: "Validation".to_string(),
                context: "Collection last element access".to_string(),
                songbird_compatible: true,
            }
        );

        // Parsing patterns
        error_patterns.insert(
            "parse_unwrap".to_string(),
            MigrationPattern {
                pattern: r#"\.parse\(\)\.unwrap\(\)"#.to_string(),
                replacement: r#".parse().map_err(|e| SongbirdError::configuration(format!("Parsing failed: {:?}", e)))?"#.to_string(),
                error_category: "Validation".to_string(),
                context: "String parsing operation".to_string(),
                songbird_compatible: true,
            }
        );

        error_patterns.insert(
            "parse_expect".to_string(),
            MigrationPattern {
                pattern: r#"\.parse\(\)\.expect\("([^"]+)"\)"#.to_string(),
                replacement: r#".parse().map_err(|e| SongbirdError::configuration(format!("$1: {:?}", e)))?"#.to_string(),
                error_category: "Validation".to_string(),
                context: "String parsing with expect".to_string(),
                songbird_compatible: true,
            }
        );

        // Iterator patterns
        error_patterns.insert(
            "max_by_unwrap".to_string(),
            MigrationPattern {
                pattern: r#"\.max_by\(([^)]+)\)\.unwrap\(\)"#.to_string(),
                replacement: r#".max_by($1).ok_or_else(|| SongbirdError::configuration("Iterator is empty when finding maximum".to_string()))?"#.to_string(),
                error_category: "Validation".to_string(),
                context: "Iterator max_by operation".to_string(),
                songbird_compatible: true,
            }
        );

        // Generic fallback patterns (lowest priority)
        error_patterns.insert(
            "general_unwrap".to_string(),
            MigrationPattern {
                pattern: r#"\.unwrap\(\)"#.to_string(),
                replacement: r#".map_err(|e| SongbirdError::configuration(format!("TODO: Replace with proper error handling: {}", e)))?"#.to_string(),
                error_category: "System".to_string(),
                context: "General operation".to_string(),
                songbird_compatible: true,
            }
        );

        error_patterns.insert(
            "general_expect".to_string(),
            MigrationPattern {
                pattern: r#"\.expect\("([^"]+)"\)"#.to_string(),
                replacement: r#".map_err(|e| SongbirdError::configuration(format!("$1: {}", e)))?"#.to_string(),
                error_category: "System".to_string(),
                context: "General operation with expect".to_string(),
                songbird_compatible: true,
            }
        );

        Self {
            error_patterns,
            files_processed: std::sync::atomic::AtomicU64::new(0),
            migrations_applied: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Analyze codebase for unwrap/expect patterns
    pub async fn analyze_codebase(
        &self,
        root_path: &Path,
        exclude_tests: bool,
    ) -> MigratorResult<CodebaseStats> {
        let mut stats = CodebaseStats {
            files_scanned: 0,
            total_unwrap_calls: 0,
            migrable_patterns: 0,
            test_file_patterns: 0,
            songbird_compatible: 0,
            pattern_categories: HashMap::new(),
        };

        let mut files_to_process = Vec::new();
        self.collect_rust_files(root_path, &mut files_to_process).await?;

        for file_path in &files_to_process {
            let is_test_file = self.is_test_file(file_path);
            
            if exclude_tests && is_test_file {
                continue;
            }

            stats.files_scanned += 1;
            
            let content = fs::read_to_string(file_path).await?;
            self.analyze_file_content(&content, &mut stats, is_test_file);
        }

        Ok(stats)
    }

    /// Migrate entire codebase
    pub async fn migrate_codebase(
        &self,
        root_path: &Path,
        dry_run: bool,
        exclude_tests: bool,
    ) -> MigratorResult<MigrationResult> {
        let start_time = std::time::Instant::now();
        
        let mut result = MigrationResult {
            files_processed: 0,
            migrations_applied: 0,
            failed_files: Vec::new(),
            execution_time_ms: 0,
        };

        let mut files_to_process = Vec::new();
        self.collect_rust_files(root_path, &mut files_to_process).await?;

        for file_path in files_to_process {
            let is_test_file = self.is_test_file(&file_path);
            
            if exclude_tests && is_test_file {
                continue;
            }

            result.files_processed += 1;
            
            match self.migrate_file(&file_path, dry_run).await {
                Ok(migrations) => {
                    result.migrations_applied += migrations;
                    if migrations > 0 {
                        info!("📄 Migrated {}: {} patterns", file_path.display(), migrations);
                    }
                }
                Err(e) => {
                    error!("❌ Failed to migrate {}: {}", file_path.display(), e);
                    result.failed_files.push((file_path, e.to_string()));
                }
            }
        }

        result.execution_time_ms = start_time.elapsed().as_millis() as u64;
        Ok(result)
    }

    /// Migrate a single file
    pub async fn migrate_file(&self, file_path: &Path, dry_run: bool) -> MigratorResult<usize> {
        let content = fs::read_to_string(file_path).await?;
        let mut modified_content = content.clone();
        let mut migrations_applied = 0;

        // Apply patterns in priority order (specific patterns first)
        for (_name, pattern) in &self.error_patterns {
            if let Ok(regex) = Regex::new(&pattern.pattern) {
                let matches = regex.find_iter(&modified_content).count();
                if matches > 0 {
                    modified_content = regex.replace_all(&modified_content, pattern.replacement.as_str()).to_string();
                    migrations_applied += matches;
                }
            }
        }

        if !dry_run && migrations_applied > 0 {
            fs::write(file_path, modified_content).await?;
        }

        Ok(migrations_applied)
    }

    fn analyze_file_content(&self, content: &str, stats: &mut CodebaseStats, is_test: bool) {
        // Count all unwrap/expect calls
        if let Ok(unwrap_regex) = Regex::new(r"\.unwrap\(\)") {
            stats.total_unwrap_calls += unwrap_regex.find_iter(content).count();
        }
        if let Ok(expect_regex) = Regex::new(r"\.expect\(") {
            stats.total_unwrap_calls += expect_regex.find_iter(content).count();
        }

        // Analyze against patterns
        for (_name, pattern) in &self.error_patterns {
            if let Ok(regex) = Regex::new(&pattern.pattern) {
                let matches = regex.find_iter(content).count();
                if matches > 0 {
                    stats.migrable_patterns += matches;
                    *stats.pattern_categories.entry(pattern.error_category.clone()).or_insert(0) += matches;
                    
                    if pattern.songbird_compatible {
                        stats.songbird_compatible += matches;
                    }
                    
                    if is_test {
                        stats.test_file_patterns += matches;
                    }
                }
            }
        }
    }

    async fn collect_rust_files(&self, root_path: &Path, files: &mut Vec<PathBuf>) -> MigratorResult<()> {
        let mut entries = fs::read_dir(root_path).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_dir() {
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if dir_name == "target" || dir_name.starts_with('.') {
                        continue;
                    }
                }
                
                Box::pin(self.collect_rust_files(&path, files)).await?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                files.push(path);
            }
        }
        
        Ok(())
    }

    fn is_test_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        path_str.contains("/tests/") || path_str.contains("test_") || path_str.ends_with("_test.rs")
    }
}
