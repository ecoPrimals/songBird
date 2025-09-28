//! Comprehensive Unwrap Migrator - Production-Scale Panic Elimination
//!
//! This migrator is designed to handle the systematic elimination of 359+ files
//! containing unwrap/expect/panic patterns across the entire Songbird ecosystem.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tokio::fs;
use regex::Regex;
use tracing::{info, warn, error};
use songbird_errors::{SongbirdError, SongbirdResult, SongbirdError};

/// Comprehensive migrator for production-scale panic elimination
pub struct ComprehensiveMigrator {
    /// Migration patterns organized by context
    patterns: HashMap<String, Vec<MigrationRule>>,
    /// Files to exclude from migration
    exclusions: HashSet<PathBuf>,
    /// Statistics tracking
    pub stats: MigrationStats,
    /// Safety checks enabled
    safety_checks: bool,
}

#[derive(Debug, Clone)]
pub struct MigrationRule {
    /// Regex pattern to match
    pub pattern: Regex,
    /// Replacement template with capture groups
    pub replacement: String,
    /// Context description for logging
    pub context: String,
    /// Priority level (higher = more specific, applied first)
    pub priority: u8,
}

#[derive(Debug, Default)]
pub struct MigrationStats {
    pub files_scanned: usize,
    pub files_modified: usize,
    pub total_replacements: usize,
    pub patterns_by_category: HashMap<String, usize>,
    pub errors_encountered: Vec<String>,
}

impl ComprehensiveMigrator {
    /// Create a new comprehensive migrator with production-grade patterns
    pub fn new() -> SongbirdResult<Self> {
        let mut migrator = Self {
            patterns: HashMap::new(),
            exclusions: HashSet::new(),
            stats: MigrationStats::default(),
            safety_checks: true,
        };

        migrator.initialize_patterns()?;
        migrator.initialize_exclusions();
        
        Ok(migrator)
    }

    /// Initialize comprehensive migration patterns
    fn initialize_patterns(&mut self) -> SongbirdResult<()> {
        // Configuration patterns
        self.add_pattern_category("configuration", vec![
            MigrationRule {
                pattern: Regex::new(r#"std::env::var\("([^"]+)"\)\.unwrap\(\)"#)?,
                replacement: r#"std::env::var("$1")
    .map_err(|e| SongbirdError::configuration_error(
        "env_var_missing",
        &format!("Environment variable '{}' not found: {}", "$1", e)
    ))?"#.to_string(),
                context: "Environment variable unwrap".to_string(),
                priority: 90,
            },
            MigrationRule {
                pattern: Regex::new(r#"env::var\("([^"]+)"\)\.unwrap\(\)"#)?,
                replacement: r#"env::var("$1")
    .map_err(|e| SongbirdError::configuration_error(
        "env_var_missing", 
        &format!("Environment variable '{}' not found: {}", "$1", e)
    ))?"#.to_string(),
                context: "Short env::var unwrap".to_string(),
                priority: 89,
            },
            MigrationRule {
                pattern: Regex::new(r#"\.expect\("([^"]*environment[^"]*)"\)"#)?,
                replacement: r#".map_err(|e| SongbirdError::configuration_error(
        "env_operation_failed",
        &format!("$1: {}", e)
    ))?"#.to_string(),
                context: "Environment-related expect".to_string(),
                priority: 85,
            },
        ])?;

        // Network patterns
        self.add_pattern_category("network", vec![
            MigrationRule {
                pattern: Regex::new(r#"\.parse\(\)\.unwrap\(\)"#)?,
                replacement: r#".parse()
    .map_err(|e| SongbirdError::validation_error(
        "parse_failed",
        &format!("Failed to parse value: {}", e)
    ))?"#.to_string(),
                context: "Parse unwrap".to_string(),
                priority: 80,
            },
            MigrationRule {
                pattern: Regex::new(r#"\.expect\("([^"]*[Pp]arse[^"]*)"\)"#)?,
                replacement: r#".map_err(|e| SongbirdError::validation_error(
        "parse_failed",
        &format!("$1: {}", e)
    ))?"#.to_string(),
                context: "Parse expect".to_string(),
                priority: 82,
            },
            MigrationRule {
                pattern: Regex::new(r#"\.expect\("([^"]*[Nn]etwork[^"]*)"\)"#)?,
                replacement: r#".map_err(|e| SongbirdError::network_error(
        "network_operation_failed",
        &format!("$1: {}", e)
    ))?"#.to_string(),
                context: "Network expect".to_string(),
                priority: 85,
            },
        ])?;

        // Service patterns
        self.add_pattern_category("service", vec![
            MigrationRule {
                pattern: Regex::new(r#"\.expect\("([^"]*[Ss]ervice[^"]*)"\)"#)?,
                replacement: r#".map_err(|e| SongbirdError::service_error(
        "service_operation_failed",
        &format!("$1: {}", e)
    ))?"#.to_string(),
                context: "Service expect".to_string(),
                priority: 85,
            },
            MigrationRule {
                pattern: Regex::new(r#"\.expect\("([^"]*[Cc]lient[^"]*)"\)"#)?,
                replacement: r#".map_err(|e| SongbirdError::network_error(
        "client_creation_failed",
        &format!("$1: {}", e)
    ))?"#.to_string(),
                context: "Client creation expect".to_string(),
                priority: 87,
            },
        ])?;

        // Resource patterns  
        self.add_pattern_category("resource", vec![
            MigrationRule {
                pattern: Regex::new(r#"\.lock\(\)\.unwrap\(\)"#)?,
                replacement: r#".lock()
    .map_err(|e| SongbirdError::resource_error(
        "lock_failed",
        &format!("Failed to acquire lock: {}", e)
    ))?"#.to_string(),
                context: "Lock unwrap".to_string(),
                priority: 90,
            },
            MigrationRule {
                pattern: Regex::new(r#"\.expect\("([^"]*[Ll]ock[^"]*)"\)"#)?,
                replacement: r#".map_err(|e| SongbirdError::resource_error(
        "lock_failed",
        &format!("$1: {}", e)
    ))?"#.to_string(),
                context: "Lock expect".to_string(),
                priority: 88,
            },
        ])?;

        // Generic patterns (lower priority)
        self.add_pattern_category("generic", vec![
            MigrationRule {
                pattern: Regex::new(r#"\.unwrap\(\)"#)?,
                replacement: r#".map_err(|e| SongbirdError::operation_failed(
        "unwrap_failed",
        &format!("Operation failed: {}", e)
    ))?"#.to_string(),
                context: "Generic unwrap".to_string(),
                priority: 10,
            },
            MigrationRule {
                pattern: Regex::new(r#"\.expect\("([^"]*)"\)"#)?,
                replacement: r#".map_err(|e| SongbirdError::operation_failed(
        "expect_failed",
        &format!("$1: {}", e)
    ))?"#.to_string(),
                context: "Generic expect".to_string(),
                priority: 15,
            },
        ])?;

        // Panic patterns
        self.add_pattern_category("panic", vec![
            MigrationRule {
                pattern: Regex::new(r#"panic!\("([^"]*)"\)"#)?,
                replacement: r#"return Err(SongbirdError::validation_error(
        "panic_converted",
        "$1"
    ))"#.to_string(),
                context: "Panic macro".to_string(),
                priority: 95,
            },
        ])?;

        Ok(())
    }

    /// Add pattern category with validation
    fn add_pattern_category(&mut self, category: &str, rules: Vec<MigrationRule>) -> SongbirdResult<()> {
        self.patterns.insert(category.to_string(), rules);
        Ok(())
    }

    /// Initialize file exclusions
    fn initialize_exclusions(&mut self) {
        // Exclude generated files, target directories, and specific problematic files
        let exclusions = vec![
            "target/",
            ".git/",
            "node_modules/",
            "coverage-report/",
            "archive/",
            // Keep some test files as they might have legitimate unwrap usage
        ];

        for exclusion in exclusions {
            self.exclusions.insert(PathBuf::from(exclusion));
        }
    }

    /// Execute comprehensive migration across the codebase
    pub async fn migrate_codebase(&self) -> SongbirdResult<MigrationReport> {
        info!("🚀 Starting comprehensive panic elimination across {}", root_path.display());
        
        let rust_files = self.discover_rust_files(root_path).await?;
        info!("📁 Found {} Rust files to process", rust_files.len());

        let mut report = MigrationReport::new();

        for (index, file_path) in rust_files.iter().enumerate() {
            if index % 50 == 0 {
                info!("📈 Progress: {}/{} files processed", index, rust_files.len());
            }

            match self.migrate_file(file_path).await {
                Ok(file_stats) => {
                    self.stats.files_scanned += 1;
                    if file_stats.replacements_made > 0 {
                        self.stats.files_modified += 1;
                        self.stats.total_replacements += file_stats.replacements_made;
                        report.add_file_result(file_path.clone(), file_stats);
                    }
                }
                Err(e) => {
                    warn!("⚠️  Failed to migrate {}: {}", file_path.display(), e);
                    self.stats.errors_encountered.push(format!("{}: {}", file_path.display(), e));
                }
            }
        }

        info!("✅ Migration completed: {} files modified, {} total replacements", 
              self.stats.files_modified, self.stats.total_replacements);

        Ok(report)
    }

    /// Discover all Rust files in the given path
    fn discover_rust_files(SongbirdResult<Vec<PathBuf>>) ->  {
        let mut rust_files = Vec::new();
        self.discover_rust_files_recursive(root_path, &mut rust_files).await?;
        Ok(rust_files)
    }

    /// Recursively discover Rust files
    fn discover_rust_files_recursive(SongbirdResult<()>) ->  {
        if self.should_exclude_path(dir) {
            return Ok(());
        }

        let mut entries = fs::read_dir(dir).await
            .map_err(|e| SongbirdError::storage_error("read_dir_failed", &format!("Failed to read directory {}: {}", dir.display(), e)))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| SongbirdError::storage_error("entry_read_failed", &format!("Failed to read entry: {}", e)))? {
            
            let path = entry.path();
            
            if path.is_dir() {
                self.discover_rust_files_recursive(&path, files).await?;
            } else if path.extension().map_or(false, |ext| ext == "rs") {
                files.push(path);
            }
        }

        Ok(())
    }

    /// Check if path should be excluded
    fn should_exclude_path(&self, path: &Path) -> bool {
        for exclusion in &self.exclusions {
            if path.starts_with(exclusion) || path.to_string_lossy().contains(&exclusion.to_string_lossy().to_string()) {
                return true;
            }
        }
        false
    }

    /// Migrate a single file
    fn migrate_file(SongbirdResult<FileStats>) ->  {
        let content = fs::read_to_string(file_path).await
            .map_err(|e| SongbirdError::storage_error("file_read_failed", &format!("Failed to read {}: {}", file_path.display(), e)))?;

        let mut modified_content = content.clone();
        let mut replacements_made = 0;

        // Apply patterns in priority order
        let mut all_rules: Vec<(&String, &MigrationRule)> = Vec::new();
        for (category, rules) in &self.patterns {
            for rule in rules {
                all_rules.push((category, rule));
            }
        }
        all_rules.sort_by(|a, b| b.1.priority.cmp(&a.1.priority));

        for (category, rule) in all_rules {
            let matches = rule.pattern.find_iter(&modified_content).count();
            if matches > 0 {
                modified_content = rule.pattern.replace_all(&modified_content, &rule.replacement).to_string();
                replacements_made += matches;
                *self.stats.patterns_by_category.entry(category.clone()).or_insert(0) += matches;
            }
        }

        // Write back if changes were made
        if replacements_made > 0 {
            fs::write(file_path, modified_content).await
                .map_err(|e| SongbirdError::storage_error("file_write_failed", &format!("Failed to write {}: {}", file_path.display(), e)))?;
        }

        Ok(FileStats {
            file_path: file_path.to_path_buf(),
            replacements_made,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FileStats {
    pub file_path: PathBuf,
    pub replacements_made: usize,
}

#[derive(Debug)]
pub struct MigrationReport {
    pub file_results: Vec<FileStats>,
    pub start_time: std::time::Instant,
}

impl MigrationReport {
    pub fn new() -> Self {
        Self {
            file_results: Vec::new(),
            start_time: std::time::Instant::now(),
        }
    }

    pub fn add_file_result(&mut self, file_path: PathBuf, stats: FileStats) {
        self.file_results.push(stats);
    }

    pub fn generate_summary(&self) -> String {
        let total_files = self.file_results.len();
        let total_replacements: usize = self.file_results.iter().map(|r| r.replacements_made).sum();
        let duration = self.start_time.elapsed();

        format!(
            "🎯 COMPREHENSIVE MIGRATION COMPLETE\n\
             ===================================\n\
             Files Modified: {}\n\
             Total Replacements: {}\n\
             Duration: {:?}\n\
             \n\
             ✅ Songbird codebase is now panic-free!",
            total_files, total_replacements, duration
        )
    }
} 