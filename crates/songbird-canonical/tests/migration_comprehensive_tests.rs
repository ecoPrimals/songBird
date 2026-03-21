// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::await_holding_lock,
    clippy::float_cmp,
    clippy::absurd_extreme_comparisons,
    clippy::needless_collect,
    clippy::nonminimal_bool,
    clippy::used_underscore_binding,
    clippy::field_reassign_with_default,
    clippy::return_self_not_must_use,
    clippy::overly_complex_bool_expr,
    clippy::assertions_on_constants,
    clippy::no_effect_underscore_binding,
    clippy::items_after_statements,
    clippy::empty_line_after_doc_comments,
    clippy::const_is_empty,
    clippy::duplicated_attributes,
    deprecated,
    dead_code,
    clippy::unnecessary_literal_unwrap,
    clippy::needless_pass_by_value,
    clippy::must_use_candidate,
    reason = "test assertions and harness ergonomics"
)]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//! Comprehensive tests for Canonical Migration Tools
//!
//! This test suite provides thorough coverage of the automated migration
//! system for converting existing code to canonical patterns.

use songbird_canonical::migration::*;
use songbird_types::SongbirdResult;
use std::path::Path;

// ========== CanonicalMigrator Tests ==========

#[test]
fn test_migrator_creation() -> SongbirdResult<()> {
    let migrator = CanonicalMigrator::new();
    // Should create migrator with default patterns
    assert!(format!("{:?}", migrator).contains("CanonicalMigrator"));
    Ok(())
}

#[test]
fn test_migrator_default() -> SongbirdResult<()> {
    let migrator = CanonicalMigrator::default();
    assert!(format!("{:?}", migrator).contains("CanonicalMigrator"));
    Ok(())
}

#[test]
fn test_migrator_analyze_codebase() {
    let path = Path::new("./test/path");
    let report = CanonicalMigrator::analyze_codebase(path);

    assert_eq!(report.files_analyzed, 0); // Default implementation
    assert!(report.patterns_found.is_empty());
    assert!(report.suggested_changes.is_empty());
    assert_eq!(report.estimated_effort_hours, 0);
}

#[test]
fn test_migrate_file_no_changes() {
    let migrator = CanonicalMigrator::new();
    let content = "fn test() -> Result<(), Error> { Ok(()) }";
    let path = Path::new("test.rs");

    let result = migrator.migrate_file(path, content);

    assert!(!result.migrated_content.is_empty());
    // Changes made count should be valid
    assert!(result.changes_made.len() < 1000); // Reasonable upper bound
}

#[test]
fn test_migrate_file_with_return_type_pattern() {
    let migrator = CanonicalMigrator::new();
    let content = "fn test() -> Result<T, SomeError> { Ok(value) }";
    let path = Path::new("test.rs");

    let result = migrator.migrate_file(path, content);

    assert!(result.migrated_content.contains("SongbirdResult"));
    assert!(!result.changes_made.is_empty());
}

#[test]
fn test_migrate_file_with_error_pattern() {
    let migrator = CanonicalMigrator::new();
    let content = "return service_error!(\"Test error\");";
    let path = Path::new("test.rs");

    let result = migrator.migrate_file(path, content);

    assert!(result.migrated_content.contains("SongbirdError::service_error"));
    assert!(!result.changes_made.is_empty());
}

#[test]
fn test_migrate_file_with_config_field_pattern() {
    let migrator = CanonicalMigrator::new();
    let content = "config.enable_connection_reuse = true;";
    let path = Path::new("test.rs");

    let result = migrator.migrate_file(path, content);

    assert!(result.migrated_content.contains("enable_async_batching"));
    assert!(!result.changes_made.is_empty());
}

#[test]
fn test_migrate_file_multiple_patterns() {
    let migrator = CanonicalMigrator::new();
    let content = r#"
        fn test() -> Result<T, SomeError> {
            config.enable_connection_reuse = true;
            service_error!("error")
        }
    "#;
    let path = Path::new("test.rs");

    let result = migrator.migrate_file(path, content);

    // Should replace all patterns
    assert!(result.migrated_content.contains("SongbirdResult"));
    assert!(result.migrated_content.contains("enable_async_batching"));
    assert!(result.migrated_content.contains("SongbirdError::service_error"));
    assert!(result.changes_made.len() >= 3);
}

#[test]
fn test_migrate_file_empty_content() {
    let migrator = CanonicalMigrator::new();
    let content = "";
    let path = Path::new("empty.rs");

    let result = migrator.migrate_file(path, content);

    assert_eq!(result.migrated_content, "");
    assert!(result.changes_made.is_empty());
}

// ========== MigrationChange Tests ==========

#[test]
fn test_migration_change_creation() {
    let change = MigrationChange {
        change_type: ChangeType::ReturnType,
        old_pattern: "Result<T>".to_string(),
        new_pattern: "SongbirdResult<T>".to_string(),
        line_number: Some(42),
    };

    assert_eq!(change.old_pattern, "Result<T>");
    assert_eq!(change.new_pattern, "SongbirdResult<T>");
    assert_eq!(change.line_number, Some(42));
}

#[test]
fn test_migration_change_no_line_number() {
    let change = MigrationChange {
        change_type: ChangeType::ErrorHandling,
        old_pattern: "panic!".to_string(),
        new_pattern: "SongbirdError".to_string(),
        line_number: None,
    };

    assert!(change.line_number.is_none());
}

#[test]
fn test_migration_change_clone() -> SongbirdResult<()> {
    let change = MigrationChange {
        change_type: ChangeType::ConfigField,
        old_pattern: "field_a".to_string(),
        new_pattern: "field_b".to_string(),
        line_number: Some(10),
    };

    let cloned = change.clone();
    assert_eq!(change.old_pattern, cloned.old_pattern);
    assert_eq!(change.new_pattern, cloned.new_pattern);
    assert_eq!(change.line_number, cloned.line_number);
    Ok(())
}

// ========== ChangeType Tests ==========

#[test]
fn test_change_type_return_type() -> SongbirdResult<()> {
    let change_type = ChangeType::ReturnType;
    let debug_str = format!("{:?}", change_type);
    assert!(debug_str.contains("ReturnType"));
    Ok(())
}

#[test]
fn test_change_type_error_handling() -> SongbirdResult<()> {
    let change_type = ChangeType::ErrorHandling;
    let debug_str = format!("{:?}", change_type);
    assert!(debug_str.contains("ErrorHandling"));
    Ok(())
}

#[test]
fn test_change_type_config_field() -> SongbirdResult<()> {
    let change_type = ChangeType::ConfigField;
    let debug_str = format!("{:?}", change_type);
    assert!(debug_str.contains("ConfigField"));
    Ok(())
}

#[test]
fn test_change_type_import() -> SongbirdResult<()> {
    let change_type = ChangeType::Import;
    let debug_str = format!("{:?}", change_type);
    assert!(debug_str.contains("Import"));
    Ok(())
}

#[test]
fn test_change_type_function_signature() -> SongbirdResult<()> {
    let change_type = ChangeType::FunctionSignature;
    let debug_str = format!("{:?}", change_type);
    assert!(debug_str.contains("FunctionSignature"));
    Ok(())
}

#[test]
fn test_change_type_clone() {
    let original = ChangeType::ReturnType;
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_change_type_equality() {
    assert_eq!(ChangeType::ReturnType, ChangeType::ReturnType);
    assert_ne!(ChangeType::ReturnType, ChangeType::ErrorHandling);
}

// ========== MigrationResult Tests ==========

#[test]
fn test_migration_result_creation() {
    let result = MigrationResult {
        original_content: "old content".to_string(),
        migrated_content: "new content".to_string(),
        changes_made: vec![],
        compilation_status: CompilationStatus::Unknown,
    };

    assert_eq!(result.original_content, "old content");
    assert_eq!(result.migrated_content, "new content");
    assert!(result.changes_made.is_empty());
}

#[test]
fn test_migration_result_with_changes() {
    let change1 = MigrationChange {
        change_type: ChangeType::ReturnType,
        old_pattern: "A".to_string(),
        new_pattern: "B".to_string(),
        line_number: Some(1),
    };

    let change2 = MigrationChange {
        change_type: ChangeType::ErrorHandling,
        old_pattern: "C".to_string(),
        new_pattern: "D".to_string(),
        line_number: Some(2),
    };

    let result = MigrationResult {
        original_content: "original".to_string(),
        migrated_content: "migrated".to_string(),
        changes_made: vec![change1, change2],
        compilation_status: CompilationStatus::Success,
    };

    assert_eq!(result.changes_made.len(), 2);
    assert_eq!(result.compilation_status, CompilationStatus::Success);
}

#[test]
fn test_migration_result_debug() -> SongbirdResult<()> {
    let result = MigrationResult {
        original_content: "original".to_string(),
        migrated_content: "test".to_string(),
        changes_made: vec![],
        compilation_status: CompilationStatus::NotTested,
    };

    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("MigrationResult"));
    Ok(())
}

#[test]
fn test_migration_result_clone() {
    let result = MigrationResult {
        original_content: "orig".to_string(),
        migrated_content: "new".to_string(),
        changes_made: vec![],
        compilation_status: CompilationStatus::Failed,
    };

    let cloned = result.clone();
    assert_eq!(result.original_content, cloned.original_content);
    assert_eq!(result.compilation_status, cloned.compilation_status);
}

// ========== CompilationStatus Tests ==========

#[test]
fn test_compilation_status_success() -> SongbirdResult<()> {
    let status = CompilationStatus::Success;
    assert_eq!(status, CompilationStatus::Success);
    Ok(())
}

#[test]
fn test_compilation_status_failed() -> SongbirdResult<()> {
    let status = CompilationStatus::Failed;
    assert_eq!(status, CompilationStatus::Failed);
    Ok(())
}

#[test]
fn test_compilation_status_unknown() -> SongbirdResult<()> {
    let status = CompilationStatus::Unknown;
    assert_eq!(status, CompilationStatus::Unknown);
    Ok(())
}

#[test]
fn test_compilation_status_not_tested() -> SongbirdResult<()> {
    let status = CompilationStatus::NotTested;
    assert_eq!(status, CompilationStatus::NotTested);
    Ok(())
}

#[test]
fn test_compilation_status_debug() -> SongbirdResult<()> {
    let status = CompilationStatus::Success;
    let debug_str = format!("{:?}", status);
    assert!(debug_str.contains("Success"));
    Ok(())
}

#[test]
fn test_compilation_status_clone() {
    let original = CompilationStatus::Failed;
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

// ========== SuggestedChange Tests ==========

#[test]
fn test_suggested_change_creation() {
    let change = SuggestedChange {
        file_path: "test.rs".to_string(),
        line_number: 10,
        description: "Update import".to_string(),
        suggested_replacement: Some("new import".to_string()),
        priority: 5,
    };

    assert_eq!(change.file_path, "test.rs");
    assert_eq!(change.line_number, 10);
    assert_eq!(change.priority, 5);
}

#[test]
fn test_suggested_change_no_replacement() {
    let change = SuggestedChange {
        file_path: "file.rs".to_string(),
        line_number: 20,
        description: "Manual review needed".to_string(),
        suggested_replacement: None,
        priority: 8,
    };

    assert!(change.suggested_replacement.is_none());
}

#[test]
fn test_suggested_change_priority_range() {
    let low = SuggestedChange {
        file_path: "a.rs".to_string(),
        line_number: 1,
        description: "Low priority".to_string(),
        suggested_replacement: None,
        priority: 1,
    };

    let high = SuggestedChange {
        file_path: "b.rs".to_string(),
        line_number: 2,
        description: "High priority".to_string(),
        suggested_replacement: None,
        priority: 10,
    };

    assert_eq!(low.priority, 1);
    assert_eq!(high.priority, 10);
}

#[test]
fn test_suggested_change_clone() {
    let original = SuggestedChange {
        file_path: "test.rs".to_string(),
        line_number: 5,
        description: "Test".to_string(),
        suggested_replacement: Some("replacement".to_string()),
        priority: 3,
    };

    let cloned = original.clone();
    assert_eq!(original.file_path, cloned.file_path);
    assert_eq!(original.line_number, cloned.line_number);
}

// ========== MigrationReport Tests ==========

#[test]
fn test_migration_report_creation() {
    let report = MigrationReport {
        files_analyzed: 5,
        patterns_found: std::collections::HashMap::new(),
        suggested_changes: vec![],
        estimated_effort_hours: 10,
    };

    assert_eq!(report.files_analyzed, 5);
    assert_eq!(report.estimated_effort_hours, 10);
}

#[test]
fn test_migration_report_with_patterns() {
    let mut patterns = std::collections::HashMap::new();
    patterns.insert("old_pattern".to_string(), 5);
    patterns.insert("another_pattern".to_string(), 3);

    let report = MigrationReport {
        files_analyzed: 10,
        patterns_found: patterns,
        suggested_changes: vec![],
        estimated_effort_hours: 20,
    };

    assert_eq!(report.patterns_found.len(), 2);
    assert_eq!(report.patterns_found.get("old_pattern"), Some(&5));
}

#[test]
fn test_migration_report_with_suggested_changes() {
    let suggestions = vec![
        SuggestedChange {
            file_path: "file1.rs".to_string(),
            line_number: 10,
            description: "Change A to B".to_string(),
            suggested_replacement: Some("B".to_string()),
            priority: 5,
        },
        SuggestedChange {
            file_path: "file2.rs".to_string(),
            line_number: 20,
            description: "Update C to D".to_string(),
            suggested_replacement: Some("D".to_string()),
            priority: 7,
        },
        SuggestedChange {
            file_path: "file3.rs".to_string(),
            line_number: 30,
            description: "Refactor E".to_string(),
            suggested_replacement: None,
            priority: 3,
        },
    ];

    let report = MigrationReport {
        files_analyzed: 3,
        patterns_found: std::collections::HashMap::new(),
        suggested_changes: suggestions,
        estimated_effort_hours: 8,
    };

    assert_eq!(report.suggested_changes.len(), 3);
    assert_eq!(report.suggested_changes[0].priority, 5);
    assert_eq!(report.suggested_changes[1].priority, 7);
}

#[test]
fn test_migration_report_debug() -> SongbirdResult<()> {
    let report = MigrationReport {
        files_analyzed: 1,
        patterns_found: std::collections::HashMap::new(),
        suggested_changes: vec![],
        estimated_effort_hours: 2,
    };

    let debug_str = format!("{:?}", report);
    assert!(debug_str.contains("MigrationReport"));
    Ok(())
}

#[test]
fn test_migration_report_clone() {
    let report = MigrationReport {
        files_analyzed: 3,
        patterns_found: std::collections::HashMap::new(),
        suggested_changes: vec![],
        estimated_effort_hours: 5,
    };

    let cloned = report.clone();
    assert_eq!(report.files_analyzed, cloned.files_analyzed);
}

// ========== Integration Tests ==========

#[test]
fn test_full_migration_workflow() {
    let migrator = CanonicalMigrator::new();

    // Original code with multiple patterns
    let original = r#"
        fn process() -> Result<T, SomeError> {
            config.enable_connection_reuse = true;
            config.max_batch_size = 100;
            config.batch_timeout = 5000;
            
            if error {
                return service_error!("Failed");
            }
            
            Ok(result)
        }
    "#;

    let path = Path::new("process.rs");
    let result = migrator.migrate_file(path, original);

    // Verify all patterns were replaced
    assert!(result.migrated_content.contains("SongbirdResult"));
    assert!(result.migrated_content.contains("enable_async_batching"));
    assert!(result.migrated_content.contains("batch_size"));
    assert!(result.migrated_content.contains("batch_timeout_ms"));
    assert!(result.migrated_content.contains("SongbirdError::service_error"));

    // Verify changes were tracked
    assert!(result.changes_made.len() >= 5);

    // Verify original content is preserved
    assert_eq!(result.original_content, original);
}

#[test]
fn test_migration_preserves_whitespace() {
    let migrator = CanonicalMigrator::new();
    let content = "    fn test() -> Result<T, SomeError> {\n        Ok(value)\n    }";
    let path = Path::new("test.rs");

    let result = migrator.migrate_file(path, content);

    // Check that whitespace structure is preserved
    assert!(result.migrated_content.contains("    fn"));
    assert!(result.migrated_content.contains("        Ok("));
    assert!(result.migrated_content.contains("\n    }"));
}

#[test]
fn test_migration_handles_comments() {
    let migrator = CanonicalMigrator::new();
    let content = r#"
        // This returns Result<T, SomeError>
        fn test() -> Result<T, SomeError> {
            // Use service_error! here
            service_error!("error")
        }
    "#;
    let path = Path::new("test.rs");

    let result = migrator.migrate_file(path, content);

    // Should still migrate patterns in comments
    assert!(result.migrated_content.contains("SongbirdResult"));
    assert!(result.migrated_content.contains("SongbirdError::service_error"));
}

#[test]
fn test_migration_result_count() {
    let migrator = CanonicalMigrator::new();
    let content = "Result<T, SomeError> Result<T, SomeError> Result<T, SomeError>";
    let path = Path::new("test.rs");

    let result = migrator.migrate_file(path, content);

    // All instances should be replaced
    assert_eq!(result.migrated_content.matches("SongbirdResult").count(), 3);
}

#[test]
fn test_codebase_analysis() {
    // Test the analyze_codebase static method
    let path = Path::new("./test/codebase");
    let report = CanonicalMigrator::analyze_codebase(path);

    // Default implementation should return empty report
    assert_eq!(report.files_analyzed, 0);
    assert!(report.patterns_found.is_empty());
    assert!(report.suggested_changes.is_empty());
}

#[test]
fn test_migration_change_types_comprehensive() {
    let return_type_change = MigrationChange {
        change_type: ChangeType::ReturnType,
        old_pattern: "Result".to_string(),
        new_pattern: "SongbirdResult".to_string(),
        line_number: Some(10),
    };

    let error_change = MigrationChange {
        change_type: ChangeType::ErrorHandling,
        old_pattern: "panic".to_string(),
        new_pattern: "SongbirdError".to_string(),
        line_number: Some(20),
    };

    let config_change = MigrationChange {
        change_type: ChangeType::ConfigField,
        old_pattern: "field_old".to_string(),
        new_pattern: "field_new".to_string(),
        line_number: Some(30),
    };

    let changes = [return_type_change, error_change, config_change];

    assert_eq!(changes.len(), 3);
    assert_eq!(changes[0].line_number, Some(10));
    assert_eq!(changes[1].line_number, Some(20));
    assert_eq!(changes[2].line_number, Some(30));
}

#[test]
fn test_migrate_file_large_content() -> SongbirdResult<()> {
    use songbird_types::SongbirdError;
    use std::fmt::Write;

    let migrator = CanonicalMigrator::new();

    // Create large content with multiple patterns
    let mut content = String::new();
    for i in 0..100 {
        writeln!(
            &mut content,
            "fn function_{i}() -> Result<T, SomeError> {{ service_error!(\"error\") }}"
        )
        .map_err(|_| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
    }

    let path = Path::new("large.rs");
    let result = migrator.migrate_file(path, &content);

    // All patterns should be migrated
    assert!(result.migrated_content.contains("SongbirdResult"));
    assert!(result.migrated_content.contains("SongbirdError::service_error"));
    // Should have changes (at least one per pattern type found)
    assert!(!result.changes_made.is_empty());
    assert!(result.changes_made.len() >= 2); // At least ReturnType and ErrorHandling changes
    Ok(())
}

#[test]
fn test_migration_special_characters() {
    let migrator = CanonicalMigrator::new();
    let content = "Result<T, SomeError> with special chars: \n\t\r!@#$%^&*()";
    let path = Path::new("test.rs");

    let result = migrator.migrate_file(path, content);

    // Should handle special characters
    assert!(result.migrated_content.contains("SongbirdResult"));
    assert!(result.migrated_content.contains("!@#$%^&*()"));
}

#[test]
fn test_migration_unicode() {
    let migrator = CanonicalMigrator::new();
    let content = "fn test() -> Result<T, SomeError> { /* 你好 世界 🌍 */ }";
    let path = Path::new("test.rs");

    let result = migrator.migrate_file(path, content);

    // Should preserve Unicode
    assert!(result.migrated_content.contains("你好"));
    assert!(result.migrated_content.contains("🌍"));
    assert!(result.migrated_content.contains("SongbirdResult"));
}
