//! Modernized Unwrap Migrator - Songbird 2025 Edition
//!
//! Enhanced migrator targeting the remaining 277 unwrap() calls identified in our
//! comprehensive codebase analysis. Focuses on production code first, then tests.
//! 
//! NEW: Added canonical modernization patterns for fixing compilation errors,
//! unifying imports, and eliminating deprecated patterns.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tokio::fs;
use regex::Regex;
use tracing::{info, error, debug, warn};
use serde::Serialize;

#[derive(Debug)]
pub struct ModernizedMigrator {
    /// Migration patterns organized by priority and context
    patterns: Vec<MigrationPattern>,
    /// Files to exclude from migration
    exclusions: HashSet<PathBuf>,
    /// Migration statistics
    pub stats: MigrationStats,
    /// Dry run mode (don't actually modify files)
    dry_run: bool,
    /// Target directories (empty = all)
    target_dirs: Vec<PathBuf>,
    /// Canonical patterns for modernization
    canonical_patterns: Vec<CanonicalPattern>,
}

#[derive(Debug, Clone)]
pub struct MigrationPattern {
    /// Unique identifier for this pattern
    pub id: String,
    /// Regex pattern to match
    pub pattern: Regex,
    /// Replacement template
    pub replacement: String,
    /// Context description
    pub context: String,
    /// Priority (higher = applied first)
    pub priority: u32,
    /// Category for statistics
    pub category: String,
    /// Whether this pattern requires manual review
    pub requires_review: bool,
}

#[derive(Debug, Clone)]
pub struct CanonicalPattern {
    /// Pattern identifier
    pub id: String,
    /// What to find (regex)
    pub find: Regex,
    /// What to replace with
    pub replace: String,
    /// Description of the modernization
    pub description: String,
    /// File patterns this applies to
    pub file_patterns: Vec<String>,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct MigrationStats {
    pub files_scanned: usize,
    pub files_modified: usize,
    pub total_replacements: usize,
    pub patterns_applied: HashMap<String, usize>,
    pub categories: HashMap<String, usize>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub manual_review_needed: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct MigrationReport {
    pub stats: MigrationStats,
    pub files_modified: Vec<PathBuf>,
    pub remaining_unwraps: Vec<UnwrapLocation>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct UnwrapLocation {
    pub file: PathBuf,
    pub line: usize,
    pub context: String,
    pub pattern: String,
}

impl ModernizedMigrator {
    /// Create a new modernized migrator with 2025 patterns
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut migrator = Self {
            patterns: Vec::new(),
            exclusions: HashSet::new(),
            stats: MigrationStats::default(),
            dry_run: false,
            target_dirs: Vec::new(),
            canonical_patterns: Vec::new(),
        };

        migrator.initialize_patterns()?;
        migrator.initialize_canonical_patterns()?;
        migrator.initialize_exclusions();
        migrator.finalize_patterns();
        
        Ok(migrator)
    }

    /// Set dry run mode
    pub fn set_dry_run(&mut self, dry_run: bool) {
        self.dry_run = dry_run;
    }

    /// Set target directories
    pub fn set_target_dirs(&mut self, dirs: Vec<PathBuf>) {
        self.target_dirs = dirs;
    }

    /// Add a migration pattern
    fn add_pattern(&mut self, pattern: MigrationPattern) {
        self.patterns.push(pattern);
    }

    /// Initialize comprehensive migration patterns based on our analysis
    fn initialize_patterns(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // High priority patterns - Production critical
        self.add_pattern(MigrationPattern {
            id: "parse_network_addr".to_string(),
            pattern: Regex::new(r#""([^"]+)"\.parse\(\)\.unwrap\(\)"#)?,
            replacement: r#""$1".parse()
    .map_err(|e| SongbirdError::network_error(&format!("Invalid address '{}': {}", "$1", e)))?"#.to_string(),
            context: "Network address parsing".to_string(),
            priority: 100,
            category: "network".to_string(),
            requires_review: false,
        });

        // Enhanced parsing patterns
        self.add_pattern(MigrationPattern {
            id: "parse_ip_addr".to_string(),
            pattern: Regex::new(r#"\.parse::<IpAddr>\(\)\.unwrap\(\)"#)?,
            replacement: r#".parse::<IpAddr>()
    .map_err(|e| SongbirdError::network_error(&format!("Invalid IP address: {}", e)))?"#.to_string(),
            context: "IP address parsing".to_string(),
            priority: 95,
            category: "network".to_string(),
            requires_review: false,
        });

        self.add_pattern(MigrationPattern {
            id: "parse_generic".to_string(),
            pattern: Regex::new(r#"\.parse\(\)\.unwrap\(\)"#)?,
            replacement: r#".parse()
    .map_err(|e| SongbirdError::validation_error(&format!("Parse failed: {}", e)))?"#.to_string(),
            context: "Generic parsing".to_string(),
            priority: 90,
            category: "parsing".to_string(),
            requires_review: false,
        });

        // Configuration patterns
        self.add_pattern(MigrationPattern {
            id: "env_var".to_string(),
            pattern: Regex::new(r#"std::env::var\("([^"]+)"\)\.unwrap\(\)"#)?,
            replacement: r#"std::env::var("$1")
    .map_err(|_| SongbirdError::configuration_error(&format!("Environment variable '{}' not found", "$1")))?"#.to_string(),
            context: "Environment variable access".to_string(),
            priority: 95,
            category: "configuration".to_string(),
            requires_review: false,
        });

        // File operations
        self.add_pattern(MigrationPattern {
            id: "fs_write".to_string(),
            pattern: Regex::new(r#"std::fs::write\(([^)]+)\)\.unwrap\(\)"#)?,
            replacement: r#"std::fs::write($1)
    .map_err(|e| SongbirdError::io_error(&format!("Write failed: {}", e)))?"#.to_string(),
            context: "File write operations".to_string(),
            priority: 85,
            category: "io".to_string(),
            requires_review: false,
        });

        self.add_pattern(MigrationPattern {
            id: "fs_read".to_string(),
            pattern: Regex::new(r#"std::fs::read_to_string\(([^)]+)\)\.unwrap\(\)"#)?,
            replacement: r#"std::fs::read_to_string($1)
    .map_err(|e| SongbirdError::io_error(&format!("Read failed: {}", e)))?"#.to_string(),
            context: "File read operations".to_string(),
            priority: 85,
            category: "io".to_string(),
            requires_review: false,
        });

        // JSON/TOML serialization
        self.add_pattern(MigrationPattern {
            id: "json_serialize".to_string(),
            pattern: Regex::new(r#"serde_json::to_string\(([^)]+)\)\.unwrap\(\)"#)?,
            replacement: r#"serde_json::to_string($1)
    .map_err(|e| SongbirdError::serialization_error(&format!("JSON serialization failed: {}", e)))?"#.to_string(),
            context: "JSON serialization".to_string(),
            priority: 80,
            category: "serialization".to_string(),
            requires_review: false,
        });

        self.add_pattern(MigrationPattern {
            id: "json_deserialize".to_string(),
            pattern: Regex::new(r#"serde_json::from_str\(([^)]+)\)\.unwrap\(\)"#)?,
            replacement: r#"serde_json::from_str($1)
    .map_err(|e| SongbirdError::serialization_error(&format!("JSON deserialization failed: {}", e)))?"#.to_string(),
            context: "JSON deserialization".to_string(),
            priority: 80,
            category: "serialization".to_string(),
            requires_review: false,
        });

        self.add_pattern(MigrationPattern {
            id: "toml_serialize".to_string(),
            pattern: Regex::new(r#"toml::to_string\(([^)]+)\)\.unwrap\(\)"#)?,
            replacement: r#"toml::to_string($1)
    .map_err(|e| SongbirdError::serialization_error(&format!("TOML serialization failed: {}", e)))?"#.to_string(),
            context: "TOML serialization".to_string(),
            priority: 80,
            category: "serialization".to_string(),
            requires_review: false,
        });

        self.add_pattern(MigrationPattern {
            id: "toml_deserialize".to_string(),
            pattern: Regex::new(r#"toml::from_str\(([^)]+)\)\.unwrap\(\)"#)?,
            replacement: r#"toml::from_str($1)
    .map_err(|e| SongbirdError::serialization_error(&format!("TOML deserialization failed: {}", e)))?"#.to_string(),
            context: "TOML deserialization".to_string(),
            priority: 80,
            category: "serialization".to_string(),
            requires_review: false,
        });

        // Thread operations
        self.add_pattern(MigrationPattern {
            id: "thread_join".to_string(),
            pattern: Regex::new(r#"\.join\(\)\.unwrap\(\)"#)?,
            replacement: r#".join()
    .map_err(|e| SongbirdError::runtime_error(&format!("Thread join failed: {:?}", e)))?"#.to_string(),
            context: "Thread join operations".to_string(),
            priority: 75,
            category: "threading".to_string(),
            requires_review: false,
        });

        // Mutex operations
        self.add_pattern(MigrationPattern {
            id: "mutex_lock".to_string(),
            pattern: Regex::new(r#"\.lock\(\)\.unwrap\(\)"#)?,
            replacement: r#".lock()
    .map_err(|e| SongbirdError::runtime_error(&format!("Lock acquisition failed: {}", e)))?"#.to_string(),
            context: "Mutex lock operations".to_string(),
            priority: 75,
            category: "threading".to_string(),
            requires_review: false,
        });

        // Enhanced test patterns - handle specific test scenarios
        self.add_pattern(MigrationPattern {
            id: "test_orchestrator_new".to_string(),
            pattern: Regex::new(r#"Orchestrator::new\(([^)]+)\)\.unwrap\(\)"#)?,
            replacement: r#"Orchestrator::new($1)
    .expect("Test orchestrator should initialize successfully")"#.to_string(),
            context: "Test orchestrator initialization".to_string(),
            priority: 70,
            category: "testing".to_string(),
            requires_review: false,
        });

        self.add_pattern(MigrationPattern {
            id: "test_config_creation".to_string(),
            pattern: Regex::new(r#"SongbirdConfig::from_file\(([^)]+)\)\.unwrap\(\)"#)?,
            replacement: r#"SongbirdConfig::from_file($1)
    .expect("Test config file should load successfully")"#.to_string(),
            context: "Test config loading".to_string(),
            priority: 70,
            category: "testing".to_string(),
            requires_review: false,
        });

        self.add_pattern(MigrationPattern {
            id: "test_temp_dir".to_string(),
            pattern: Regex::new(r#"TempDir::new\(\)\.unwrap\(\)"#)?,
            replacement: r#"TempDir::new()
    .expect("Test temp directory should be created successfully")"#.to_string(),
            context: "Test temp directory creation".to_string(),
            priority: 70,
            category: "testing".to_string(),
            requires_review: false,
        });

        self.add_pattern(MigrationPattern {
            id: "test_registry_get".to_string(),
            pattern: Regex::new(r#"registry\.get_primal\(([^)]+)\)\.unwrap\(\)"#)?,
            replacement: r#"registry.get_primal($1)
    .expect("Test primal should be registered")"#.to_string(),
            context: "Test registry access".to_string(),
            priority: 70,
            category: "testing".to_string(),
            requires_review: false,
        });

        // Generic test unwrap - convert to expect with context
        self.add_pattern(MigrationPattern {
            id: "test_unwrap_simple".to_string(),
            pattern: Regex::new(r#"\.unwrap\(\);"#)?,
            replacement: r#".expect("Test operation should succeed");"#.to_string(),
            context: "Simple test unwrap".to_string(),
            priority: 60,
            category: "testing".to_string(),
            requires_review: false,
        });

        // Collection operations
        self.add_pattern(MigrationPattern {
            id: "vec_max_min".to_string(),
            pattern: Regex::new(r#"\.(max|min)\(\)\.unwrap\(\)"#)?,
            replacement: r#".$1()
    .ok_or_else(|| SongbirdError::runtime_error("Collection is empty"))?"#.to_string(),
            context: "Collection max/min operations".to_string(),
            priority: 65,
            category: "collections".to_string(),
            requires_review: false,
        });

        // Enhanced generic unwrap pattern for remaining cases
        self.add_pattern(MigrationPattern {
            id: "generic_unwrap_enhanced".to_string(),
            pattern: Regex::new(r#"([a-zA-Z_][a-zA-Z0-9_]*(?:\([^)]*\))?)\.unwrap\(\)"#)?,
            replacement: r#"$1
    .map_err(|e| SongbirdError::runtime_error(&format!("Operation failed: {:?}", e)))?"#.to_string(),
            context: "Enhanced generic unwrap".to_string(),
            priority: 30,
            category: "generic".to_string(),
            requires_review: true,
        });

        Ok(())
    }

    /// Initialize canonical modernization patterns
    fn initialize_canonical_patterns(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Fix import consolidation issues
        self.canonical_patterns.push(CanonicalPattern {
            id: "unify_songbird_response_imports".to_string(),
            find: Regex::new(r"use songbird_canonical::\{SongbirdResponse, SongbirdResult\};")?,
            replace: "use songbird_errors::{SongbirdResponse, SongbirdResult, SongbirdError};".to_string(),
            description: "Unify SongbirdResponse imports to canonical location".to_string(),
            file_patterns: vec!["*.rs".to_string()],
        });

        // Fix async method calls
        self.canonical_patterns.push(CanonicalPattern {
            id: "fix_missing_await".to_string(),
            find: Regex::new(r"assert_eq!\(stats\.(\w+), (\d+)\);")?,
            replace: "assert_eq!(stats.await.$1, $2);".to_string(),
            description: "Add missing .await for async calls".to_string(),
            file_patterns: vec!["*test*.rs".to_string()],
        });

        // Fix NetworkConfig ambiguity
        self.canonical_patterns.push(CanonicalPattern {
            id: "fix_network_config_ambiguity".to_string(),
            find: Regex::new(r"let network_config = NetworkConfig::default\(\);")?,
            replace: "let network_config = songbird_config::NetworkConfig::default();".to_string(),
            description: "Resolve NetworkConfig ambiguity".to_string(),
            file_patterns: vec!["*test*.rs".to_string()],
        });

        // Fix correlation_id type mismatches
        self.canonical_patterns.push(CanonicalPattern {
            id: "fix_correlation_id_types".to_string(),
            find: Regex::new(r#"correlation_id: "([^"]+)","#)?,
            replace: r#"correlation_id: Some("$1".to_string()),"#.to_string(),
            description: "Fix correlation_id type from &str to Option<String>".to_string(),
            file_patterns: vec!["*test*.rs".to_string()],
        });

        // Remove unused async keywords
        self.canonical_patterns.push(CanonicalPattern {
            id: "remove_unused_async".to_string(),
            find: Regex::new(r"async fn (\w+)\([^)]*\) -> ([^{]+) \{")?,
            replace: "fn $1($2) -> $3 {".to_string(),
            description: "Remove unused async keywords from functions".to_string(),
            file_patterns: vec!["*.rs".to_string()],
        });

        // Fix panic sources with proper error handling
        self.canonical_patterns.push(CanonicalPattern {
            id: "fix_production_unwraps".to_string(),
            find: Regex::new(r"\.unwrap\(\)")?,
            replace: ".map_err(|e| SongbirdError::internal(format!(\"Operation failed: {:?}\", e)))?".to_string(),
            description: "Replace unwrap() with proper error handling".to_string(),
            file_patterns: vec!["src/**/*.rs".to_string(), "crates/**/src/**/*.rs".to_string()],
        });

        // Fix hardcoded IP addresses
        self.canonical_patterns.push(CanonicalPattern {
            id: "fix_hardcoded_ips".to_string(),
            find: Regex::new(r#""127\.0\.0\.1""#)?,
            replace: r#"&get_bind_address()"#.to_string(),
            description: "Replace hardcoded IP addresses with configuration".to_string(),
            file_patterns: vec!["*.rs".to_string()],
        });

        // Fix hardcoded ports
        self.canonical_patterns.push(CanonicalPattern {
            id: "fix_hardcoded_ports".to_string(),
            find: Regex::new(r":{}")?,
            replace: ":{}".to_string(),
            description: "Replace hardcoded ports with configuration placeholders".to_string(),
            file_patterns: vec!["*.rs".to_string()],
        });

        Ok(())
    }

    /// Initialize files and patterns to exclude from migration
    fn initialize_exclusions(&mut self) {
        // Exclude generated files and dependencies
        self.exclusions.insert(PathBuf::from("target"));
        self.exclusions.insert(PathBuf::from("Cargo.lock"));
        
        // Exclude specific problematic files that should be manually reviewed
        self.exclusions.insert(PathBuf::from("songbird-unwrap-migrator"));
        self.exclusions.insert(PathBuf::from("examples/legacy"));
        
        // Exclude files that are known to have complex unwrap patterns
        self.exclusions.insert(PathBuf::from("benches/unsafe_patterns.rs"));
        
        info!("Initialized {} exclusion patterns", self.exclusions.len());
    }

    /// Finalize patterns (sort by priority and log statistics)
    fn finalize_patterns(&mut self) {
        // Sort patterns by priority (highest first)
        self.patterns.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        info!("Initialized {} migration patterns", self.patterns.len());
        
        // Log pattern distribution by category
        let mut categories: HashMap<String, usize> = HashMap::new();
        for pattern in &self.patterns {
            *categories.entry(pattern.category.clone()).or_insert(0) += 1;
        }
        
        for (category, count) in categories {
            debug!("  {}: {} patterns", category, count);
        }
    }

    /// Run the standard migration process (for backward compatibility)
    pub async fn run_migration(&self) -> Result<MigrationReport, Box<dyn std::error::Error>> {
        info!("🚀 Starting unwrap migration process...");
        
        self.stats = MigrationStats::default();
        let current_dir = std::env::current_dir()?;
        
        let report = self.migrate_directory(&current_dir).await?;
        
        info!("✅ Unwrap migration complete!");
        Ok(report)
    }

    /// Migrate a directory (main entry point)
    pub async fn migrate_directory(&self, dir: &Path) -> Result<MigrationReport, Box<dyn std::error::Error>> {
        info!("📂 Processing directory: {}", dir.display());
        
        let rust_files = self.find_rust_files(dir).await?;
        self.stats.files_scanned = rust_files.len();
        
        let mut files_modified = Vec::new();
        let mut remaining_unwraps = Vec::new();
        
        for file_path in rust_files {
            if self.should_skip_file(&file_path) {
                debug!("⏭️  Skipping file: {}", file_path.display());
                continue;
            }
            
            match self.migrate_file(&file_path).await {
                Ok((modified, mut unwraps)) => {
                    if modified {
                        files_modified.push(file_path);
                        self.stats.files_modified += 1;
                    }
                    remaining_unwraps.append(&mut unwraps);
                }
                Err(e) => {
                    let error_msg = format!("Failed to migrate {}: {}", file_path.display(), e);
                    error!("{}", error_msg);
                    self.stats.errors.push(error_msg);
                }
            }
        }
        
        let recommendations = self.generate_recommendations();
        
        Ok(MigrationReport {
            stats: self.stats.clone(),
            files_modified,
            remaining_unwraps,
            recommendations,
        })
    }

    /// Find all Rust files in a directory (using Box::pin for recursion)
    async fn find_rust_files(&self, dir: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let mut files = Vec::new();
        self.find_rust_files_recursive(dir, &mut files).await?;
        Ok(files)
    }

    /// Recursively find Rust files
    fn find_rust_files_recursive(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
        let mut entries = fs::read_dir(dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_dir() {
                // Skip target and hidden directories
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if !name.starts_with('.') && name != "target" {
                        Box::pin(self.find_rust_files_recursive(&path, files)).await?;
                    }
                }
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                files.push(path);
            }
        }
        
        Ok(())
    }

    /// Check if a file should be skipped
    fn should_skip_file(&self, path: &Path) -> bool {
        // Check if any part of the path matches exclusions
        for exclusion in &self.exclusions {
            if path.starts_with(exclusion) || 
               path.to_string_lossy().contains(&exclusion.to_string_lossy().to_string()) {
                return true;
            }
        }

        // If target directories are specified, only process those
        if !self.target_dirs.is_empty() {
            return !self.target_dirs.iter().any(|target| path.starts_with(target));
        }

        false
    }

    /// Migrate a single file
    async fn migrate_file(&self, file_path: &Path) -> Result<(bool, Vec<UnwrapLocation>), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path).await?;
        let modified_content = content.clone();
        let mut modifications_made = false;
        let mut remaining_unwraps = Vec::new();

        let mut current_content = modified_content;
        
        // Apply patterns in priority order
        for pattern in &self.patterns.clone() {
            // Clone current content to avoid borrow checker issues
            let content_for_matching = current_content.clone();
            let matches: Vec<_> = pattern.pattern.find_iter(&content_for_matching).collect();
            
            if !matches.is_empty() {
                debug!("Applying pattern '{}' to {}: {} matches", 
                       pattern.id, file_path.display(), matches.len());
                
                if pattern.requires_review {
                    // Don't auto-apply patterns that require review
                    for m in &matches {
                        let line_num = content_for_matching[..m.start()].lines().count() + 1;
                        remaining_unwraps.push(UnwrapLocation {
                            file: file_path.to_path_buf(),
                            line: line_num,
                            context: pattern.context.clone(),
                            pattern: m.as_str().to_string(),
                        });
                    }
                    
                    self.stats.manual_review_needed.push(format!(
                        "{}:{} - {} ({})", 
                        file_path.display(), 
                        "multiple", 
                        pattern.context,
                        matches.len()
                    ));
                } else {
                    // Apply the replacement
                    current_content = pattern.pattern.replace_all(&current_content, &pattern.replacement).to_string();
                    modifications_made = true;
                    
                    let count = matches.len();
                    self.stats.total_replacements += count;
                    *self.stats.patterns_applied.entry(pattern.id.clone()).or_insert(0) += count;
                    *self.stats.categories.entry(pattern.category.clone()).or_insert(0) += count;
                }
            }
        }

        // Write back the modified content if changes were made and not in dry run
        if modifications_made && !self.dry_run {
            fs::write(file_path, current_content).await?;
            info!("Modified file: {}", file_path.display());
        } else if modifications_made {
            info!("Would modify file (dry run): {}", file_path.display());
        }

        Ok((modifications_made, remaining_unwraps))
    }

    /// Apply compilation fixes to problematic files
    async fn apply_compilation_fixes(&self) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        info!("🔧 Applying compilation fixes...");
        let mut fixed_files = Vec::new();
        
        // Find files with compilation issues
        let problematic_files = self.find_files_with_compilation_issues().await?;
        
        for file_path in problematic_files {
            if self.fix_compilation_errors(&file_path).await? {
                fixed_files.push(file_path);
            }
        }
        
        Ok(fixed_files)
    }

    /// Fix test compilation errors
    async fn fix_test_compilation_errors(&self, file_path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        let mut modified = false;
        let mut lines: Vec<String> = content.lines().map(String::from).collect();
        
        for line in &mut lines {
            // Fix common test compilation patterns
            if line.contains("expect(\"Test operation should succeed\")") {
                *line = line.replace("expect(\"Test operation should succeed\")", "map_err(|e| format!(\"Test error: {}\", e))?");
                modified = true;
            }
        }
        
        if modified {
            fs::write(file_path, lines.join("\n"))?;
        }
        
        Ok(modified)
    }

    /// Apply canonical patterns to the codebase
    async fn apply_canonical_patterns(&self) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        info!("🎯 Applying canonical patterns...");
        let mut updated_files = Vec::new();
        
        // Apply patterns to all Rust files
        let files = self.find_rust_files(Path::new("crates/")).await?;
        
        for file_path in files {
            if self.apply_canonical_patterns_to_file(&file_path).await? {
                updated_files.push(file_path);
            }
        }
        
        Ok(updated_files)
    }

    /// Clean up fragments and deprecations
    async fn cleanup_fragments_and_deprecations(&self) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        info!("🧹 Cleaning fragments and deprecations...");
        let mut cleaned_files = Vec::new();
        
        // Find files with fragments
        let fragment_files = self.find_files_with_fragments().await?;
        
        for file_path in fragment_files {
            if self.clean_file_fragments(&file_path).await? {
                cleaned_files.push(file_path);
                info!("🧹 Cleaned fragments in: {}", file_path.display());
            }
        }
        
        Ok(cleaned_files)
    }

    /// Clean fragments and deprecated patterns from a file
    async fn clean_file_fragments(&self, file_path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        let mut modified = false;
        let mut lines: Vec<String> = content.lines().map(String::from).collect();
        
        for line in &mut lines {
            // Remove deprecated comments
            if line.contains("// DEPRECATED:") || line.contains("// TODO: Remove deprecated") {
                line.clear();
                modified = true;
            }
            
            // Clean up fragment patterns
            if line.contains("_fragment") || line.contains("Fragment") {
                *line = line.replace("_fragment", "").replace("Fragment", "Canonical");
                modified = true;
            }
        }
        
        if modified {
            fs::write(file_path, lines.join("\n"))?;
        }
        
        Ok(modified)
    }

    /// Find files matching specific patterns
    async fn find_files_matching(&self, pattern: &str) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let mut matching_files = Vec::new();
        let files = self.find_rust_files(Path::new("crates/")).await?;
        
        for file_path in files {
            let content = fs::read_to_string(&file_path)?;
            if content.contains(pattern) {
                matching_files.push(file_path);
            }
        }
        
        Ok(matching_files)
    }

    /// Find files for a specific pattern
    async fn find_files_for_pattern(&self, pattern: &MigrationPattern) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        self.find_files_matching(&pattern.search_pattern).await
    }

    /// Apply pattern to a specific file
    async fn apply_pattern_to_file(&self, file_path: &Path, pattern: &MigrationPattern) -> Result<bool, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        let new_content = content.replace(&pattern.search_pattern, &pattern.replacement);
        
        if content != new_content {
            fs::write(file_path, new_content)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Scan for remaining unwrap() calls
    async fn scan_remaining_unwraps(&self) -> Result<Vec<UnwrapLocation>, Box<dyn std::error::Error>> {
        let mut unwrap_locations = Vec::new();
        let files = self.find_rust_files(Path::new("crates/")).await?;
        
        for file_path in files {
            let content = fs::read_to_string(&file_path)?;
            for (line_num, line) in content.lines().enumerate() {
                if line.contains(".unwrap()") || line.contains(".expect(") {
                    unwrap_locations.push(UnwrapLocation {
                        file: file_path.clone(),
                        line: line_num + 1,
                        context: line.trim().to_string(),
                        pattern_type: if line.contains(".unwrap()") { "unwrap" } else { "expect" }.to_string(),
                    });
                }
            }
        }
        
        Ok(unwrap_locations)
    }

    /// Generate recommendations based on analysis
    fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if self.stats.total_replacements > 0 {
            recommendations.push(format!("✅ Successfully applied {} modernization changes", self.stats.total_replacements));
        }
        
        recommendations.push("🔧 Run 'cargo fmt' to ensure consistent formatting".to_string());
        recommendations.push("🧪 Run 'cargo test' to verify all changes work correctly".to_string());
        recommendations.push("📋 Run 'cargo clippy' to check for additional issues".to_string());
        
        if self.stats.errors.len() > 0 {
            recommendations.push(format!("⚠️  {} errors encountered - manual review needed", self.stats.errors.len()));
        }
        
        recommendations
    }

    /// Generate a summary of the migration process
    pub fn generate_summary(&self) -> String {
        format!(
            "📊 Migration Summary:\n\
             Files scanned: {}\n\
             Files modified: {}\n\
             Total replacements: {}\n\
             Errors: {}\n\
             Warnings: {}",
            self.stats.files_scanned,
            self.stats.files_modified,
            self.stats.total_replacements,
            self.stats.errors.len(),
            self.stats.warnings.len()
        )
    }

    /// Run canonical modernization on the entire codebase
    pub async fn run_canonical_modernization() -> Result<MigrationReport, Box<dyn std::error::Error>> {
        info!("🚀 Starting canonical modernization...");
        
        let migrator = ModernizedMigrator::new()?;
        let report = migrator.run_migration().await?;
        
        info!("✅ Canonical modernization completed!");
        info!("📊 Migration Report:");
        info!("  Files processed: {}", report.files_processed);
        info!("  Patterns applied: {}", report.total_patterns_applied);
        info!("  Compilation fixes: {}", report.compilation_fixes);
        
        Ok(report)
    }
} 