// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Automated migration tools for converting to canonical patterns

use std::collections::HashMap;
use std::path::Path;

// Note: SongbirdResult moved to songbird-errors to break circular dependency
// Import it directly when needed: use songbird_types::responses::SongbirdResult;
/// Migration tool for converting existing code to canonical patterns
#[expect(
    clippy::struct_field_names,
    reason = "intentional pattern; clippy false positive for this API"
)]
#[derive(Debug)]
pub struct CanonicalMigrator {
    /// Pattern replacements for return types
    return_type_patterns: HashMap<String, String>,
    /// Pattern replacements for error handling
    error_patterns: HashMap<String, String>,
    /// Pattern replacements for configuration fields
    config_field_patterns: HashMap<String, String>,
}

impl CanonicalMigrator {
    /// Create a new migrator with default patterns
    #[must_use]
    pub fn new() -> Self {
        let mut return_type_patterns = HashMap::new();
        let _ = return_type_patterns
            .insert(String::from("SongbirdResult<T>>"), String::from("SongbirdResult<T>"));
        let _ = return_type_patterns
            .insert(String::from("Result<T, SomeError>"), String::from("SongbirdResult<T>"));
        let _ = return_type_patterns
            .insert(String::from("SongbirdResult<T>"), String::from("SongbirdResult<T>"));

        let mut error_patterns = HashMap::new();
        let _ = error_patterns
            .insert(String::from("service_error!("), String::from("SongbirdError::service_error("));
        let _ =
            error_patterns.insert(String::from("Ok(()"), String::from("Ok(SongbirdResult::unit()"));

        let mut config_field_patterns = HashMap::new();
        let _ = config_field_patterns
            .insert(String::from("enable_connection_reuse"), String::from("enable_async_batching"));
        let _ = config_field_patterns
            .insert(String::from("max_batch_size"), String::from("batch_size"));
        let _ = config_field_patterns
            .insert(String::from("batch_timeout"), String::from("batch_timeout_ms"));

        Self {
            return_type_patterns,
            error_patterns,
            config_field_patterns,
        }
    }

    /// Generate migration report for a codebase
    #[must_use]
    pub fn analyze_codebase(_path: &Path) -> MigrationReport {
        // This would analyze the codebase and generate a report
        // Use canonical migration system
        MigrationReport {
            files_analyzed: 0,
            patterns_found: HashMap::new(),
            suggested_changes: Vec::new(),
            estimated_effort_hours: 0,
        }
    }

    /// Apply automatic migrations to a file
    #[must_use]
    pub fn migrate_file(&self, _file_path: &Path, content: &str) -> MigrationResult {
        let mut migrated_content = content.to_string();
        let mut changes_made = Vec::new();

        // Apply return type migrations
        for (old_pattern, new_pattern) in &self.return_type_patterns {
            if migrated_content.contains(old_pattern) {
                migrated_content = migrated_content.replace(old_pattern, new_pattern);
                changes_made.push(MigrationChange {
                    change_type: ChangeType::ReturnType,
                    old_pattern: old_pattern.clone(),
                    new_pattern: new_pattern.clone(),
                    line_number: None, // Would be populated in real implementation
                });
            }
        }

        // Apply error pattern migrations
        for (old_pattern, new_pattern) in &self.error_patterns {
            if migrated_content.contains(old_pattern) {
                migrated_content = migrated_content.replace(old_pattern, new_pattern);
                changes_made.push(MigrationChange {
                    change_type: ChangeType::ErrorHandling,
                    old_pattern: old_pattern.clone(),
                    new_pattern: new_pattern.clone(),
                    line_number: None,
                });
            }
        }

        // Apply config field migrations
        for (old_pattern, new_pattern) in &self.config_field_patterns {
            if migrated_content.contains(old_pattern) {
                migrated_content = migrated_content.replace(old_pattern, new_pattern);
                changes_made.push(MigrationChange {
                    change_type: ChangeType::ConfigField,
                    old_pattern: old_pattern.clone(),
                    new_pattern: new_pattern.clone(),
                    line_number: None,
                });
            }
        }

        MigrationResult {
            original_content: content.to_string(),
            migrated_content,
            changes_made,
            compilation_status: CompilationStatus::Unknown,
        }
    }
}

impl Default for CanonicalMigrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Report of migration analysis
#[derive(Debug, Clone)]
pub struct MigrationReport {
    /// Number of files analyzed
    pub files_analyzed: usize,
    /// Patterns found and their counts
    pub patterns_found: HashMap<String, usize>,
    /// Suggested changes
    pub suggested_changes: Vec<SuggestedChange>,
    /// Estimated effort in hours
    pub estimated_effort_hours: u32,
}

/// Result of migrating a single file
#[derive(Debug, Clone)]
pub struct MigrationResult {
    /// Original file content
    pub original_content: String,
    /// Migrated file content
    pub migrated_content: String,
    /// Changes that were made
    pub changes_made: Vec<MigrationChange>,
    /// Compilation status after migration
    pub compilation_status: CompilationStatus,
}

/// A single migration change
#[derive(Debug, Clone)]
pub struct MigrationChange {
    /// Type of change
    pub change_type: ChangeType,
    /// Old pattern that was replaced
    pub old_pattern: String,
    /// New pattern that replaced it
    pub new_pattern: String,
    /// Line number where change occurred
    pub line_number: Option<usize>,
}

/// Types of changes that can be made
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    /// Return type changes
    ReturnType,
    /// Error handling changes
    ErrorHandling,
    /// Configuration field changes
    ConfigField,
    /// Import statement changes
    Import,
    /// Function signature changes
    FunctionSignature,
}

/// Suggested change for manual review
#[derive(Debug, Clone)]
pub struct SuggestedChange {
    /// File path
    pub file_path: String,
    /// Line number
    pub line_number: usize,
    /// Description of the change
    pub description: String,
    /// Suggested replacement
    pub suggested_replacement: Option<String>,
    /// Priority (1-10, higher = more important)
    pub priority: u8,
}

/// Compilation status after migration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompilationStatus {
    /// Compilation successful
    Success,
    /// Compilation failed
    Failed,
    /// Compilation status unknown
    Unknown,
    /// Not tested yet
    NotTested,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn migrator_default_matches_new() {
        let a = CanonicalMigrator::new();
        let b = CanonicalMigrator::default();
        let src = "SongbirdResult<T>>";
        assert_eq!(
            a.migrate_file(Path::new("x.rs"), src).migrated_content,
            b.migrate_file(Path::new("x.rs"), src).migrated_content
        );
    }

    #[test]
    fn migrate_file_empty_no_changes() {
        let m = CanonicalMigrator::new();
        let r = m.migrate_file(Path::new("x.rs"), "");
        assert!(r.migrated_content.is_empty());
        assert!(r.changes_made.is_empty());
        assert_eq!(r.original_content, "");
        assert_eq!(r.compilation_status, CompilationStatus::Unknown);
    }

    #[test]
    fn migrate_file_return_type_double_angle_bracket() {
        let m = CanonicalMigrator::new();
        let src = "fn f() -> SongbirdResult<T>> { }";
        let r = m.migrate_file(Path::new("a.rs"), src);
        assert!(r.migrated_content.contains("SongbirdResult<T>"));
        assert!(!r.migrated_content.contains(">>"));
        assert!(r.changes_made.iter().any(|c| c.change_type == ChangeType::ReturnType));
    }

    #[test]
    fn migrate_file_error_pattern() {
        let m = CanonicalMigrator::new();
        let src = r#"fn x() { service_error!("oops"); }"#;
        let r = m.migrate_file(Path::new("b.rs"), src);
        assert!(r.migrated_content.contains("SongbirdError::service_error("));
        assert!(r.changes_made.iter().any(|c| c.change_type == ChangeType::ErrorHandling));
    }

    #[test]
    fn migrate_file_ok_unit() {
        let m = CanonicalMigrator::new();
        let src = "fn y() { Ok(()); }";
        let r = m.migrate_file(Path::new("c.rs"), src);
        assert!(r.migrated_content.contains("Ok(SongbirdResult::unit()"));
    }

    #[test]
    fn migrate_file_config_fields() {
        let m = CanonicalMigrator::new();
        let src = "enable_connection_reuse: true, max_batch_size: 3, batch_timeout: 1";
        let r = m.migrate_file(Path::new("d.rs"), src);
        assert!(r.migrated_content.contains("enable_async_batching"));
        assert!(r.migrated_content.contains("batch_size"));
        assert!(r.migrated_content.contains("batch_timeout_ms"));
        assert_eq!(
            r.changes_made.iter().filter(|c| c.change_type == ChangeType::ConfigField).count(),
            3
        );
    }

    #[test]
    fn migrate_file_idempotent_no_extra_passes() {
        let m = CanonicalMigrator::new();
        let src = "// nothing to migrate";
        let once = m.migrate_file(Path::new("e.rs"), src);
        let twice = m.migrate_file(Path::new("e.rs"), &once.migrated_content);
        assert_eq!(once.migrated_content, twice.migrated_content);
        assert!(twice.changes_made.is_empty());
    }

    #[test]
    fn analyze_codebase_returns_empty_report() {
        let r = CanonicalMigrator::analyze_codebase(Path::new("."));
        assert_eq!(r.files_analyzed, 0);
        assert!(r.patterns_found.is_empty());
        assert!(r.suggested_changes.is_empty());
    }

    #[test]
    fn change_type_and_compilation_status_eq() {
        assert_eq!(ChangeType::ReturnType, ChangeType::ReturnType);
        assert_eq!(CompilationStatus::Unknown, CompilationStatus::Unknown);
    }
}
