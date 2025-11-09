//! Tests for Migration Tools
//!
//! Comprehensive test coverage for canonical pattern migration utilities.

use super::migration::*;
use std::path::Path;

// ============================================================================
// CanonicalMigrator Tests
// ============================================================================

#[test]
fn test_canonical_migrator_new() {
    let migrator = CanonicalMigrator::new();
    // Verify migrator can be created
    let _ = migrator;
}

#[test]
fn test_canonical_migrator_default() {
    let migrator = CanonicalMigrator::default();
    // Verify default constructor works
    let _ = migrator;
}

#[test]
fn test_analyze_codebase() {
    let path = Path::new("/tmp");
    let report = CanonicalMigrator::analyze_codebase(path);

    assert_eq!(report.files_analyzed, 0);
    assert!(report.patterns_found.is_empty());
    assert!(report.suggested_changes.is_empty());
    assert_eq!(report.estimated_effort_hours, 0);
}

#[test]
fn test_migrate_file_no_changes() {
    let migrator = CanonicalMigrator::new();
    let content = "fn hello() { println!(\"Hello\"); }";
    let path = Path::new("test.rs");

    let result = migrator.migrate_file(path, content);

    assert_eq!(result.original_content, content);
    assert_eq!(result.migrated_content, content);
    assert!(result.changes_made.is_empty());
    assert_eq!(result.compilation_status, CompilationStatus::Unknown);
}

#[test]
fn test_migrate_file_return_type_change() {
    let migrator = CanonicalMigrator::new();
    let content = "fn get_data() -> SongbirdResult<T>> { }";
    let path = Path::new("test.rs");

    let result = migrator.migrate_file(path, content);

    assert_eq!(result.original_content, content);
    assert!(result.migrated_content.contains("SongbirdResult<T>"));
    assert!(!result.migrated_content.contains("SongbirdResult<T>>"));
    assert_eq!(result.changes_made.len(), 1);
    assert_eq!(result.changes_made[0].change_type, ChangeType::ReturnType);
}

#[test]
fn test_migrate_file_error_pattern_change() {
    let migrator = CanonicalMigrator::new();
    let content = "service_error!(\"error\")";
    let path = Path::new("test.rs");

    let result = migrator.migrate_file(path, content);

    assert!(result.migrated_content.contains("SongbirdError::service_error("));
    assert_eq!(result.changes_made.len(), 1);
    assert_eq!(result.changes_made[0].change_type, ChangeType::ErrorHandling);
}

#[test]
fn test_migrate_file_ok_unit_change() {
    let migrator = CanonicalMigrator::new();
    let content = "return Ok(());";
    let path = Path::new("test.rs");

    let result = migrator.migrate_file(path, content);

    assert!(result.migrated_content.contains("Ok(SongbirdResult::unit()"));
    assert_eq!(result.changes_made.len(), 1);
    assert_eq!(result.changes_made[0].change_type, ChangeType::ErrorHandling);
}

#[test]
fn test_migrate_file_config_field_changes() {
    let migrator = CanonicalMigrator::new();
    let content = "config.enable_connection_reuse = true; config.max_batch_size = 100;";
    let path = Path::new("test.rs");

    let result = migrator.migrate_file(path, content);

    assert!(result.migrated_content.contains("enable_async_batching"));
    assert!(result.migrated_content.contains("batch_size"));
    assert!(!result.migrated_content.contains("enable_connection_reuse"));
    assert!(!result.migrated_content.contains("max_batch_size"));
    assert_eq!(result.changes_made.len(), 2);
    assert!(result.changes_made.iter().any(|c| c.change_type == ChangeType::ConfigField));
}

#[test]
fn test_migrate_file_multiple_changes() {
    let migrator = CanonicalMigrator::new();
    let content = "fn test() -> SongbirdResult<T>> { return Ok(()); }";
    let path = Path::new("test.rs");

    let result = migrator.migrate_file(path, content);

    assert!(result.changes_made.len() >= 2);
    assert!(result.migrated_content.contains("SongbirdResult<T>"));
    assert!(result.migrated_content.contains("Ok(SongbirdResult::unit()"));
}

// ============================================================================
// MigrationReport Tests
// ============================================================================

#[test]
fn test_migration_report_creation() {
    let report = MigrationReport {
        files_analyzed: 10,
        patterns_found: std::collections::HashMap::new(),
        suggested_changes: Vec::new(),
        estimated_effort_hours: 5,
    };

    assert_eq!(report.files_analyzed, 10);
    assert_eq!(report.estimated_effort_hours, 5);
}

#[test]
fn test_migration_report_with_patterns() {
    let mut patterns = std::collections::HashMap::new();
    patterns.insert("SongbirdResult".to_string(), 42);
    patterns.insert("service_error".to_string(), 15);

    let report = MigrationReport {
        files_analyzed: 10,
        patterns_found: patterns.clone(),
        suggested_changes: Vec::new(),
        estimated_effort_hours: 5,
    };

    assert_eq!(report.patterns_found.len(), 2);
    assert_eq!(report.patterns_found.get("SongbirdResult"), Some(&42));
    assert_eq!(report.patterns_found.get("service_error"), Some(&15));
}

#[test]
fn test_migration_report_with_suggestions() {
    let suggestion = SuggestedChange {
        file_path: "src/main.rs".to_string(),
        line_number: 42,
        description: "Update return type".to_string(),
        suggested_replacement: Some("SongbirdResult<T>".to_string()),
        priority: 8,
    };

    let report = MigrationReport {
        files_analyzed: 1,
        patterns_found: std::collections::HashMap::new(),
        suggested_changes: vec![suggestion],
        estimated_effort_hours: 2,
    };

    assert_eq!(report.suggested_changes.len(), 1);
    assert_eq!(report.suggested_changes[0].priority, 8);
}

// ============================================================================
// MigrationResult Tests
// ============================================================================

#[test]
fn test_migration_result_creation() {
    let result = MigrationResult {
        original_content: "old".to_string(),
        migrated_content: "new".to_string(),
        changes_made: Vec::new(),
        compilation_status: CompilationStatus::Success,
    };

    assert_eq!(result.original_content, "old");
    assert_eq!(result.migrated_content, "new");
    assert_eq!(result.compilation_status, CompilationStatus::Success);
}

#[test]
fn test_migration_result_with_changes() {
    let change = MigrationChange {
        change_type: ChangeType::ReturnType,
        old_pattern: "Old".to_string(),
        new_pattern: "New".to_string(),
        line_number: Some(10),
    };

    let result = MigrationResult {
        original_content: "content".to_string(),
        migrated_content: "migrated".to_string(),
        changes_made: vec![change],
        compilation_status: CompilationStatus::NotTested,
    };

    assert_eq!(result.changes_made.len(), 1);
    assert_eq!(result.changes_made[0].line_number, Some(10));
}

// ============================================================================
// MigrationChange Tests
// ============================================================================

#[test]
fn test_migration_change_creation() {
    let change = MigrationChange {
        change_type: ChangeType::ReturnType,
        old_pattern: "OldType".to_string(),
        new_pattern: "NewType".to_string(),
        line_number: Some(42),
    };

    assert_eq!(change.change_type, ChangeType::ReturnType);
    assert_eq!(change.old_pattern, "OldType");
    assert_eq!(change.new_pattern, "NewType");
    assert_eq!(change.line_number, Some(42));
}

#[test]
fn test_migration_change_without_line_number() {
    let change = MigrationChange {
        change_type: ChangeType::ErrorHandling,
        old_pattern: "old".to_string(),
        new_pattern: "new".to_string(),
        line_number: None,
    };

    assert!(change.line_number.is_none());
}

// ============================================================================
// ChangeType Tests
// ============================================================================

#[test]
fn test_change_type_variants() {
    let return_type = ChangeType::ReturnType;
    let error_handling = ChangeType::ErrorHandling;
    let config_field = ChangeType::ConfigField;
    let import = ChangeType::Import;
    let function_sig = ChangeType::FunctionSignature;

    assert_eq!(return_type, ChangeType::ReturnType);
    assert_eq!(error_handling, ChangeType::ErrorHandling);
    assert_eq!(config_field, ChangeType::ConfigField);
    assert_eq!(import, ChangeType::Import);
    assert_eq!(function_sig, ChangeType::FunctionSignature);
}

#[test]
fn test_change_type_equality() {
    assert_eq!(ChangeType::ReturnType, ChangeType::ReturnType);
    assert_ne!(ChangeType::ReturnType, ChangeType::ErrorHandling);
}

// ============================================================================
// SuggestedChange Tests
// ============================================================================

#[test]
fn test_suggested_change_creation() {
    let change = SuggestedChange {
        file_path: "src/lib.rs".to_string(),
        line_number: 100,
        description: "Update error handling".to_string(),
        suggested_replacement: Some("new_code".to_string()),
        priority: 9,
    };

    assert_eq!(change.file_path, "src/lib.rs");
    assert_eq!(change.line_number, 100);
    assert_eq!(change.priority, 9);
    assert!(change.suggested_replacement.is_some());
}

#[test]
fn test_suggested_change_without_replacement() {
    let change = SuggestedChange {
        file_path: "src/main.rs".to_string(),
        line_number: 50,
        description: "Manual review needed".to_string(),
        suggested_replacement: None,
        priority: 5,
    };

    assert!(change.suggested_replacement.is_none());
    assert_eq!(change.priority, 5);
}

#[test]
fn test_suggested_change_priority_ranges() {
    let low = SuggestedChange {
        file_path: "test.rs".to_string(),
        line_number: 1,
        description: "Low priority".to_string(),
        suggested_replacement: None,
        priority: 1,
    };

    let high = SuggestedChange {
        file_path: "test.rs".to_string(),
        line_number: 1,
        description: "High priority".to_string(),
        suggested_replacement: None,
        priority: 10,
    };

    assert_eq!(low.priority, 1);
    assert_eq!(high.priority, 10);
}

// ============================================================================
// CompilationStatus Tests
// ============================================================================

#[test]
fn test_compilation_status_variants() {
    let success = CompilationStatus::Success;
    let failed = CompilationStatus::Failed;
    let unknown = CompilationStatus::Unknown;
    let not_tested = CompilationStatus::NotTested;

    assert_eq!(success, CompilationStatus::Success);
    assert_eq!(failed, CompilationStatus::Failed);
    assert_eq!(unknown, CompilationStatus::Unknown);
    assert_eq!(not_tested, CompilationStatus::NotTested);
}

#[test]
fn test_compilation_status_equality() {
    assert_eq!(CompilationStatus::Success, CompilationStatus::Success);
    assert_ne!(CompilationStatus::Success, CompilationStatus::Failed);
    assert_ne!(CompilationStatus::Unknown, CompilationStatus::NotTested);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_full_migration_workflow() {
    let migrator = CanonicalMigrator::new();
    let original = r#"
        fn process() -> SongbirdResult<T>> {
            if error {
                return service_error!("failed");
            }
            return Ok(());
        }
    "#;
    let path = Path::new("workflow.rs");

    let result = migrator.migrate_file(path, original);

    assert!(!result.changes_made.is_empty());
    assert!(result.migrated_content.contains("SongbirdError::service_error"));
    assert!(result.migrated_content.contains("Ok(SongbirdResult::unit()"));
    assert_eq!(result.original_content, original);
}

#[test]
fn test_migration_report_complete_workflow() {
    let path = Path::new("src/");
    let report = CanonicalMigrator::analyze_codebase(path);

    // Verify report structure is complete
    assert_eq!(report.files_analyzed, 0);
    assert!(report.patterns_found.is_empty());
    assert!(report.suggested_changes.is_empty());
}

#[test]
fn test_complex_config_migration() {
    let migrator = CanonicalMigrator::new();
    let content = r"
        let config = Config {
            enable_connection_reuse: true,
            max_batch_size: 100,
            batch_timeout: 5000,
        };
    ";
    let path = Path::new("config.rs");

    let result = migrator.migrate_file(path, content);

    assert!(result.migrated_content.contains("enable_async_batching"));
    assert!(result.migrated_content.contains("batch_size"));
    assert!(result.migrated_content.contains("batch_timeout_ms"));
    assert_eq!(result.changes_made.len(), 3);
}
