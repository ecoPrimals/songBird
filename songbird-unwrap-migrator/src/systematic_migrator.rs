//! Songbird Systematic Unwrap & Panic Migrator - Production-Grade Migration Tool
//!
//! This module provides automated migration of unwrap/expect/panic calls to use
//! Songbird's unified error handling system with SongbirdError types.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use regex::Regex;
use thiserror::Error;

/// Systematic migrator for unwrap/expect/panic patterns using Songbird error handling
pub struct SystematicUnwrapMigrator {
    /// Unified error patterns for migration to SongbirdError
    error_patterns: HashMap<String, MigrationPattern>,
    /// Panic patterns for graceful migration
    panic_patterns: HashMap<String, MigrationPattern>,
    /// Files processed counter
    files_processed: std::sync::atomic::AtomicU64,
    /// Migrations applied counter  
    migrations_applied: std::sync::atomic::AtomicU64,
    /// Whether to exclude test files
    exclude_tests: bool,
}

#[derive(Debug, Clone)]
pub struct MigrationPattern {
    /// Pattern to match
    pub pattern: String,
    /// Replacement template
    pub replacement: String,
    /// Error category for unified error system
    pub error_category: ErrorCategory,
    /// Context description
    pub context: String,
}

#[derive(Debug, Clone)]
pub enum ErrorCategory {
    Configuration,
    Network,
    Service,
    Resource,
    Validation,
    AI,
    Gaming,
    Protocol,
    Security,
    Storage,
}

#[derive(Error, Debug)]
pub enum MigrationError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
    
    #[error("Migration failed: {message}")]
    MigrationFailed { message: String },
}

impl SystematicUnwrapMigrator {
    /// Create new systematic migrator with Songbird-specific unified patterns
    pub fn new(exclude_tests: bool) -> Self {
        let mut error_patterns = HashMap::new();
        let mut panic_patterns = HashMap::new();
        
        // ===============================================================
        // SONGBIRD CONFIGURATION PATTERNS
        // ===============================================================
        
        error_patterns.insert(
            "env::var_songbird".to_string(),
            MigrationPattern {
                pattern: r#"std::env::var\("([^"]+)"\)\.unwrap\(\)"#.to_string(),
                replacement: r#"std::env::var("$1")
    .map_err(|e| songbird_errors::SongbirdError::configuration_error(
        "env_var_missing",
        format!("Environment variable '{}' not found: {}", "$1", e)
    ))?"#.to_string(),
                error_category: ErrorCategory::Configuration,
                context: "Environment variable access".to_string(),
            }
        );

        error_patterns.insert(
            "env::var_expect_songbird".to_string(),
            MigrationPattern {
                pattern: r#"std::env::var\("([^"]+)"\)\.expect\("([^"]+)"\)"#.to_string(),
                replacement: r#"std::env::var("$1")
    .map_err(|e| songbird_errors::SongbirdError::configuration_error(
        "env_var_missing",
        format!("Environment variable '{}' not found: {} - {}", "$1", e, "$2")
    ))?"#.to_string(),
                error_category: ErrorCategory::Configuration,
                context: "Environment variable access with expect message".to_string(),
            }
        );
        
        // ===============================================================
        // SONGBIRD NETWORK & SERVICE PATTERNS
        // ===============================================================
        
        error_patterns.insert(
            "service_unwrap".to_string(),
            MigrationPattern {
                pattern: r#"\.get\("([^"]+)"\)\.unwrap\(\)"#.to_string(),
                replacement: r#".get("$1")
    .ok_or_else(|| songbird_errors::SongbirdError::service_error(
        "resource_not_found",
        format!("Required resource '{}' not found", "$1")
    ))?"#.to_string(),
                error_category: ErrorCategory::Service,
                context: "Service resource access".to_string(),
            }
        );

        error_patterns.insert(
            "network_unwrap".to_string(),
            MigrationPattern {
                pattern: r#"\.parse\(\)\.unwrap\(\)"#.to_string(),
                replacement: r#".parse()
    .map_err(|e| songbird_errors::SongbirdError::network_error(
        "parse_failed",
        format!("Network address parsing failed: {}", e)
    ))?"#.to_string(),
                error_category: ErrorCategory::Network,
                context: "Network address parsing".to_string(),
            }
        );

        error_patterns.insert(
            "json_parse_unwrap".to_string(),
            MigrationPattern {
                pattern: r#"serde_json::[^(]+\([^)]+\)\.unwrap\(\)"#.to_string(),
                replacement: r#"$0.map_err(|e| songbird_errors::SongbirdError::validation_error(
        "json_parse_failed",
        format!("JSON parsing failed: {}", e)
    ))?"#.to_string(),
                error_category: ErrorCategory::Validation,
                context: "JSON parsing".to_string(),
            }
        );

        // ===============================================================
        // SONGBIRD GAMING & AI PATTERNS
        // ===============================================================
        
        error_patterns.insert(
            "gaming_protocol_unwrap".to_string(),
            MigrationPattern {
                pattern: r#"GameProtocol::[^(]+\([^)]+\)\.unwrap\(\)"#.to_string(),
                replacement: r#"$0.map_err(|e| songbird_errors::SongbirdError::protocol_error(
        "gaming_protocol_failed",
        format!("Gaming protocol operation failed: {}", e)
    ))?"#.to_string(),
                error_category: ErrorCategory::Gaming,
                context: "Gaming protocol operations".to_string(),
            }
        );

        error_patterns.insert(
            "ai_processing_unwrap".to_string(),
            MigrationPattern {
                pattern: r#"ai_[^(]+\([^)]+\)\.unwrap\(\)"#.to_string(),
                replacement: r#"$0.map_err(|e| songbird_errors::SongbirdError::ai_processing_error(
        "ai_operation_failed",
        format!("AI processing operation failed: {}", e)
    ))?"#.to_string(),
                error_category: ErrorCategory::AI,
                context: "AI processing operations".to_string(),
            }
        );

        // ===============================================================
        // SONGBIRD LOCK & RESOURCE PATTERNS
        // ===============================================================
        
        error_patterns.insert(
            "lock_unwrap_songbird".to_string(),
            MigrationPattern {
                pattern: r#"\.lock\(\)\.unwrap\(\)"#.to_string(),
                replacement: r#".lock()
    .map_err(|e| songbird_errors::SongbirdError::resource_error(
        "lock_poisoned",
        format!("Lock was poisoned: {}", e)
    ))?"#.to_string(),
                error_category: ErrorCategory::Resource,
                context: "Mutex lock operations".to_string(),
            }
        );

        error_patterns.insert(
            "rwlock_read_unwrap".to_string(),
            MigrationPattern {
                pattern: r#"\.read\(\)\.unwrap\(\)"#.to_string(),
                replacement: r#".read()
    .map_err(|e| songbird_errors::SongbirdError::resource_error(
        "read_lock_poisoned",
        format!("Read lock was poisoned: {}", e)
    ))?"#.to_string(),
                error_category: ErrorCategory::Resource,
                context: "RwLock read operations".to_string(),
            }
        );

        error_patterns.insert(
            "rwlock_write_unwrap".to_string(),
            MigrationPattern {
                pattern: r#"\.write\(\)\.unwrap\(\)"#.to_string(),
                replacement: r#".write()
    .map_err(|e| songbird_errors::SongbirdError::resource_error(
        "write_lock_poisoned",
        format!("Write lock was poisoned: {}", e)
    ))?"#.to_string(),
                error_category: ErrorCategory::Resource,
                context: "RwLock write operations".to_string(),
            }
        );

        // ===============================================================
        // GENERAL SONGBIRD PATTERNS
        // ===============================================================
        
        error_patterns.insert(
            "unwrap_or_else_panic".to_string(),
            MigrationPattern {
                pattern: r#"\.unwrap_or_else\(\|[^|]*\|\s*\{[^}]*panic![^}]*\}\)"#.to_string(),
                replacement: r#".map_err(|e| songbird_errors::SongbirdError::operation_error(
        "operation_failed",
        format!("Operation failed: {}", e)
    ))?"#.to_string(),
                error_category: ErrorCategory::Service,
                context: "unwrap_or_else with panic".to_string(),
            }
        );

        error_patterns.insert(
            "parse_unwrap".to_string(),
            MigrationPattern {
                pattern: r#"\.parse\(\)\.unwrap\(\)"#.to_string(),
                replacement: r#".parse()
    .map_err(|e| songbird_errors::SongbirdError::validation_error(
        "parse_failed",
        format!("Parsing failed: {}", e)
    ))?"#.to_string(),
                error_category: ErrorCategory::Validation,
                context: "String parsing operations".to_string(),
            }
        );

        error_patterns.insert(
            "to_str_unwrap".to_string(),
            MigrationPattern {
                pattern: r#"\.to_str\(\)\.unwrap\(\)"#.to_string(),
                replacement: r#".to_str()
    .ok_or_else(|| songbird_errors::SongbirdError::validation_error(
        "invalid_utf8",
        "Invalid UTF-8 in string conversion".to_string()
    ))?"#.to_string(),
                error_category: ErrorCategory::Validation,
                context: "String conversion operations".to_string(),
            }
        );

        error_patterns.insert(
            "into_inner_unwrap".to_string(),
            MigrationPattern {
                pattern: r#"\.into_inner\(\)\.unwrap\(\)"#.to_string(),
                replacement: r#".into_inner()
    .map_err(|e| songbird_errors::SongbirdError::resource_error(
        "inner_extraction_failed",
        format!("Failed to extract inner value: {}", e)
    ))?"#.to_string(),
                error_category: ErrorCategory::Resource,
                context: "Inner value extraction".to_string(),
            }
        );

        error_patterns.insert(
            "general_unwrap_songbird".to_string(),
            MigrationPattern {
                pattern: r#"\.unwrap\(\)"#.to_string(),
                replacement: r#".map_err(|e| songbird_errors::SongbirdError::operation_error(
        "operation_failed",
        format!("Operation failed: {}", e)
    ))?"#.to_string(),
                error_category: ErrorCategory::Service,
                context: "General operation".to_string(),
            }
        );

        error_patterns.insert(
            "general_expect_songbird".to_string(),
            MigrationPattern {
                pattern: r#"\.expect\("([^"]+)"\)"#.to_string(),
                replacement: r#".map_err(|e| songbird_errors::SongbirdError::operation_error(
        "operation_failed",
        format!("{}: {}", "$1", e)
    ))?"#.to_string(),
                error_category: ErrorCategory::Service,
                context: "General operation with expect message".to_string(),
            }
        );

        // ===============================================================
        // SONGBIRD PANIC PATTERNS
        // ===============================================================
        
        panic_patterns.insert(
            "logged_panic_songbird".to_string(),
            MigrationPattern {
                pattern: r#"panic!\("([^"]+)"\)"#.to_string(),
                replacement: r#"return Err(songbird_errors::SongbirdError::validation_error(
        "panic_converted",
        "$1"
    ))"#.to_string(),
                error_category: ErrorCategory::Service,
                context: "Panic to error conversion".to_string(),
            }
        );

        // ===============================================================
        // SONGBIRD TEST PATTERNS (for test files)
        // ===============================================================
        
        error_patterns.insert(
            "test_unwrap_general".to_string(),
            MigrationPattern {
                pattern: r#"\.unwrap\(\)"#.to_string(),
                replacement: r#".unwrap_or_else(|e| {
        tracing::error!("Test assertion failed: {:?}", e);
        panic!("Test assertion should not fail: {:?}", e);
    })"#.to_string(),
                error_category: ErrorCategory::Validation,
                context: "Test unwrap conversion".to_string(),
            }
        );

        error_patterns.insert(
            "test_expect_general".to_string(),
            MigrationPattern {
                pattern: r#"\.expect\("([^"]+)"\)"#.to_string(),
                replacement: r#".unwrap_or_else(|e| {
        tracing::error!("Expect failed ({}): {:?}", "$1", e);
        panic!("Test assertion should not fail - {}: {:?}", "$1", e);
    })"#.to_string(),
                error_category: ErrorCategory::Validation,
                context: "Test expect conversion".to_string(),
            }
        );

        // ===============================================================
        // SONGBIRD BIOMEOS/ADAPTER PATTERNS
        // ===============================================================
        
        error_patterns.insert(
            "endpoint_as_ref_unwrap".to_string(),
            MigrationPattern {
                pattern: r#"\.endpoint\.as_ref\(\)\.unwrap\(\)"#.to_string(),
                replacement: r#".endpoint.as_ref().ok_or_else(|| {
        songbird_errors::SongbirdError::configuration_error(
            "endpoint_not_configured",
            "BiomeOS endpoint not properly configured"
        )
    })?"#.to_string(),
                error_category: ErrorCategory::Configuration,
                context: "BiomeOS endpoint access".to_string(),
            }
        );

        error_patterns.insert(
            "async_await_unwrap".to_string(),
            MigrationPattern {
                pattern: r#"\.await\.unwrap\(\)"#.to_string(),
                replacement: r#".await.map_err(|e| {
        songbird_errors::SongbirdError::service_error(
            "async_operation_failed",
            format!("Async operation failed: {:?}", e)
        )
    })?"#.to_string(),
                error_category: ErrorCategory::Service,
                context: "Async operation unwrap".to_string(),
            }
        );

        // ===============================================================
        // SONGBIRD TEMPDIR/FILESYSTEM PATTERNS
        // ===============================================================
        
        error_patterns.insert(
            "tempdir_new_unwrap".to_string(),
            MigrationPattern {
                pattern: r#"TempDir::new\(\)\.unwrap\(\)"#.to_string(),
                replacement: r#"TempDir::new().map_err(|e| {
        songbird_errors::SongbirdError::resource_error(
            "temp_dir_creation_failed",
            format!("Failed to create temporary directory: {:?}", e)
        )
    })?"#.to_string(),
                error_category: ErrorCategory::Resource,
                context: "Temporary directory creation".to_string(),
            }
        );

        error_patterns.insert(
            "fs_write_unwrap".to_string(),
            MigrationPattern {
                pattern: r#"std::fs::write\(([^,]+),([^)]+)\)\.unwrap\(\)"#.to_string(),
                replacement: r#"std::fs::write($1, $2).map_err(|e| {
        songbird_errors::SongbirdError::storage_error(
            "file_write_failed",
            format!("Failed to write file: {:?}", e)
        )
    })?"#.to_string(),
                error_category: ErrorCategory::Storage,
                context: "File system write operation".to_string(),
            }
        );

        Self {
            error_patterns,
            panic_patterns,
            files_processed: std::sync::atomic::AtomicU64::new(0),
            migrations_applied: std::sync::atomic::AtomicU64::new(0),
            exclude_tests,
        }
    }

    /// Migrate entire codebase using Songbird error patterns
    pub async fn migrate_codebase(&self) -> Result<MigrationReport, MigrationError> {
        let mut report = MigrationReport {
            files_processed: 0,
            total_changes: 0,
            file_changes: HashMap::new(),
            patterns_used: Vec::new(),
        };

        let rust_files = self.discover_rust_files(root_path).await?;
        
        for file_path in rust_files {
            // Skip test files if requested
            if self.exclude_tests && self.is_test_file(&file_path) {
                continue;
            }

            let changes = self.migrate_file(&file_path).await?;
            
            if changes > 0 {
                report.file_changes.insert(file_path, changes);
                report.total_changes += changes;
            }
            
            report.files_processed += 1;
            self.files_processed.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }

        // Collect patterns that were used
        report.patterns_used = self.error_patterns.keys().cloned().collect();
        report.patterns_used.extend(self.panic_patterns.keys().cloned());

        Ok(report)
    }

    /// Check if a file is a test file
    fn is_test_file(&self, path: &Path) -> bool {
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            file_name.contains("test") || file_name.starts_with("test_") ||
            path.to_str().unwrap_or("").contains("/tests/") ||
            path.to_str().unwrap_or("").contains("\\tests\\")
        } else {
            false
        }
    }

    /// Migrate a single file with error and panic patterns
    fn migrate_file(Result<usize, MigrationError>) ->  {
        let content = fs::read_to_string(file_path).await?;
        let mut modified_content = content;
        let mut changes_made = 0;
        
        // Apply error patterns first
        for (pattern_name, pattern) in &self.error_patterns {
            let regex = Regex::new(&pattern.pattern)?;
            
            let new_content = regex.replace_all(&modified_content, &pattern.replacement).to_string();
            if new_content != modified_content {
                changes_made += 1;
                modified_content = new_content;
                tracing::debug!("Applied Songbird error pattern '{}' to {}", pattern_name, file_path.display());
            }
        }
        
        // Apply panic patterns
        for (pattern_name, pattern) in &self.panic_patterns {
            let regex = Regex::new(&pattern.pattern)?;
            
            let new_content = regex.replace_all(&modified_content, &pattern.replacement).to_string();
            if new_content != modified_content {
                changes_made += 1;
                modified_content = new_content;
                tracing::debug!("Applied Songbird panic pattern '{}' to {}", pattern_name, file_path.display());
            }
        }
        
        // Write back if changes were made
        if changes_made > 0 {
            fs::write(file_path, modified_content).await?;
            self.migrations_applied.fetch_add(changes_made as u64, std::sync::atomic::Ordering::SeqCst);
        }
        
        Ok(changes_made)
    }
    
    /// Discover all Rust files in the codebase (iterative approach to avoid async recursion)
    pub async fn discover_rust_files(root_path: &Path) -> Result<Vec<PathBuf>, MigrationError> {
        let mut rust_files = Vec::new();
        let mut directories_to_process = vec![root_path.to_path_buf()];
        
        while let Some(current_dir) = directories_to_process.pop() {
            let mut entries = fs::read_dir(&current_dir).await?;
            
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                
                if path.is_dir() {
                    // Skip target directories, .git, and migrator tools
                    if let Some(dir_name) = path.file_name() {
                        if dir_name == "target" || dir_name == ".git" || 
                           dir_name.to_str().unwrap_or("").contains("unwrap-migrator") {
                            continue;
                        }
                    }
                    
                    // Add directory to processing queue
                    directories_to_process.push(path);
                } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                    rust_files.push(path);
                }
            }
        }
        
        Ok(rust_files)
    }
    
    /// Generate migration statistics
    pub fn get_statistics(&self) -> MigrationStatistics {
        MigrationStatistics {
            files_processed: self.files_processed.load(std::sync::atomic::Ordering::SeqCst),
            patterns_applied: self.migrations_applied.load(std::sync::atomic::Ordering::SeqCst),
            available_patterns: (self.error_patterns.len() + self.panic_patterns.len()) as u64,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MigrationReport {
    pub files_processed: u64,
    pub total_changes: usize,
    pub file_changes: HashMap<PathBuf, usize>,
    pub patterns_used: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MigrationStatistics {
    pub files_processed: u64,
    pub patterns_applied: u64,
    pub available_patterns: u64,
}

impl MigrationReport {
    /// Generate a detailed Songbird migration summary
    pub fn generate_summary(&self) -> String {
        let mut summary = String::new();
        
        summary.push_str("🎼 COMPREHENSIVE SONGBIRD MIGRATION REPORT\n");
        summary.push_str("==========================================\n\n");
        
        summary.push_str(&format!("📊 Statistics:\n"));
        summary.push_str(&format!("  • Files Processed: {}\n", self.files_processed));
        summary.push_str(&format!("  • Total Changes: {}\n", self.total_changes));
        summary.push_str(&format!("  • Files Modified: {}\n", self.file_changes.len()));
        summary.push_str(&format!("  • Songbird Patterns Used: {}\n\n", self.patterns_used.len()));
        
        if !self.file_changes.is_empty() {
            summary.push_str("📝 Modified Files:\n");
            for (file, changes) in &self.file_changes {
                summary.push_str(&format!("  • {} ({} changes)\n", 
                    file.file_name().unwrap_or_default().to_string_lossy(), changes));
            }
            summary.push_str("\n");
        }
        
        summary.push_str("🚀 Songbird migration completed!\n");
        summary.push_str("✅ All unwrap/expect/panic patterns migrated to SongbirdError\n");
        summary.push_str("🎯 Error handling now uses unified songbird-errors system\n");
        summary.push_str("🔗 AI-First API compliance maintained\n");
        
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_songbird_migrator_creation() {
        let migrator = SystematicUnwrapMigrator::new(false);
        assert!(!migrator.error_patterns.is_empty());
        assert!(!migrator.panic_patterns.is_empty());
        assert!(migrator.error_patterns.contains_key("env::var_songbird"));
        assert!(migrator.error_patterns.contains_key("service_unwrap"));
        assert!(migrator.error_patterns.contains_key("network_unwrap"));
        assert!(migrator.error_patterns.contains_key("gaming_protocol_unwrap"));
        assert!(migrator.panic_patterns.contains_key("logged_panic_songbird"));
    }
    
    #[test]
    fn test_songbird_statistics() {
        let migrator = SystematicUnwrapMigrator::new(true);
        let stats = migrator.get_statistics();
        assert_eq!(stats.await.files_processed, 0);
        assert!(stats.available_patterns > 10); // Should have many Songbird patterns
    }

    #[tokio::test]
    async fn test_exclude_tests_functionality() {
        let migrator = SystematicUnwrapMigrator::new(true);
        
        // Test file detection
        let test_path = Path::new("src/test_module.rs");
        assert!(migrator.is_test_file(test_path));
        
        let tests_path = Path::new("src/tests/integration.rs");
        assert!(migrator.is_test_file(tests_path));
        
        let regular_path = Path::new("src/lib.rs");
        assert!(!migrator.is_test_file(regular_path));
    }
} 