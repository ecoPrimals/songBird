//! Comprehensive tests for canonical validation types
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]

use songbird_canonical::validation::*;
use songbird_types::{SongbirdError, SongbirdResult};

// ============================================================================
// ValidationResult Tests
// ============================================================================

#[test]
fn test_validation_result_success() {
    let result = ValidationResult::success();

    assert!(result.is_valid);
    assert!(result.errors.is_empty());
    assert!(result.warnings.is_empty());
}

#[test]
fn test_validation_result_failure() {
    let errors = vec!["error1".to_string(), "error2".to_string()];
    let result = ValidationResult::failure(errors.clone());

    assert!(!result.is_valid);
    assert_eq!(result.errors.len(), 2);
    assert_eq!(result.errors, errors);
}

#[test]
fn test_validation_result_failure_single_error() {
    let result = ValidationResult::failure(vec!["single error".to_string()]);

    assert!(!result.is_valid);
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0], "single error");
}

#[test]
fn test_validation_result_failure_empty_errors() {
    let result = ValidationResult::failure(vec![]);

    assert!(!result.is_valid);
    assert!(result.errors.is_empty());
}

#[test]
fn test_validation_result_success_serialization() -> SongbirdResult<()> {
    let result = ValidationResult::success();
    let serialized = serde_json::to_string(&result);

    assert!(serialized.is_ok());
    Ok(())
}

#[test]
fn test_validation_result_failure_serialization() -> SongbirdResult<()> {
    let result = ValidationResult::failure(vec!["error".to_string()]);
    let serialized = serde_json::to_string(&result);

    assert!(serialized.is_ok());
    Ok(())
}

#[test]
fn test_validation_result_deserialization() -> Result<(), Box<dyn std::error::Error>> {
    let result = ValidationResult::success();
    let serialized = serde_json::to_string(&result).map_err(|e| SongbirdError::Serialization {
        format: Some("JSON".to_string()),
        message: format!("Serialization failed: {e}"),
        debug_info: None,
    })?;
    let deserialized: Result<ValidationResult, _> = serde_json::from_str(&serialized);

    assert!(deserialized.is_ok());
    assert!(
        deserialized.ok_or_else(|| SongbirdError::configuration(format!("Error: {}", e)))?.is_valid
    );
    Ok(())
}

#[test]
fn test_validation_result_round_trip_success() -> Result<(), Box<dyn std::error::Error>> {
    let original = ValidationResult::success();
    let serialized =
        serde_json::to_string(&original).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Serialization failed: {e}"),
            debug_info: None,
        })?;
    let deserialized: ValidationResult = serde_json::from_str(&serialized)
        .map_err(|e| SongbirdError::configuration("Error occurred".to_string()))?;

    assert_eq!(original.is_valid, deserialized.is_valid);
    assert_eq!(original.errors.len(), deserialized.errors.len());
    Ok(())
}

#[test]
fn test_validation_result_round_trip_failure() -> Result<(), Box<dyn std::error::Error>> {
    let errors = vec!["error1".to_string(), "error2".to_string()];
    let original = ValidationResult::failure(errors);
    let serialized =
        serde_json::to_string(&original).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Serialization failed: {e}"),
            debug_info: None,
        })?;
    let deserialized: ValidationResult = serde_json::from_str(&serialized)
        .map_err(|e| SongbirdError::configuration("Error occurred".to_string()))?;

    assert_eq!(original.is_valid, deserialized.is_valid);
    assert_eq!(original.errors, deserialized.errors);
    Ok(())
}

#[test]
fn test_validation_result_with_warnings() {
    let mut result = ValidationResult::success();
    result.warnings.push("warning1".to_string());
    result.warnings.push("warning2".to_string());

    assert!(result.is_valid);
    assert_eq!(result.warnings.len(), 2);
}

#[test]
fn test_validation_result_failure_with_warnings() -> SongbirdResult<()> {
    let mut result = ValidationResult::failure(vec!["error".to_string()]);
    result.warnings.push("warning".to_string());

    assert!(!result.is_valid);
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.warnings.len(), 1);
    Ok(())
}

#[test]
fn test_validation_result_clone() -> SongbirdResult<()> {
    let original = ValidationResult::success();
    let cloned = original.clone();

    assert_eq!(original.is_valid, cloned.is_valid);
    Ok(())
}

#[test]
fn test_validation_result_debug() -> SongbirdResult<()> {
    let result = ValidationResult::success();
    let debug_output = format!("{result:?}");

    assert!(debug_output.contains("ValidationResult"));
    Ok(())
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_validation_workflow_success() {
    // Simulate a validation workflow
    let data_valid = true;

    let result = if data_valid {
        ValidationResult::success()
    } else {
        ValidationResult::failure(vec!["data invalid".to_string()])
    };

    assert!(result.is_valid);
}

#[test]
fn test_validation_workflow_failure() {
    // Simulate a validation workflow with errors
    let mut errors = Vec::new();

    if true {
        // Simulating validation check
        errors.push("field1 is required".to_string());
    }

    if true {
        // Simulating another validation check
        errors.push("field2 must be positive".to_string());
    }

    let result = if errors.is_empty() {
        ValidationResult::success()
    } else {
        ValidationResult::failure(errors)
    };

    assert!(!result.is_valid);
    assert_eq!(result.errors.len(), 2);
}

#[test]
fn test_validation_workflow_with_warnings() {
    let mut result = ValidationResult::success();

    // Add warnings for non-critical issues
    result.warnings.push("deprecated field used".to_string());
    result.warnings.push("suboptimal configuration".to_string());

    assert!(result.is_valid);
    assert_eq!(result.warnings.len(), 2);
}

#[test]
fn test_validation_accumulate_errors() -> SongbirdResult<()> {
    // Accumulate errors from multiple checks
    let errors = vec![
        "required field missing".to_string(),
        "invalid format".to_string(),
        "constraint violation".to_string(),
    ];

    let result = ValidationResult::failure(errors);

    assert!(!result.is_valid);
    assert_eq!(result.errors.len(), 3);
    Ok(())
}

#[test]
fn test_validation_result_serialization_with_all_fields() -> Result<(), Box<dyn std::error::Error>>
{
    let mut result = ValidationResult::failure(vec!["error1".to_string()]);
    result.warnings.push("warning1".to_string());

    let serialized = serde_json::to_string(&result).map_err(|e| SongbirdError::Serialization {
        format: Some("JSON".to_string()),
        message: format!("Serialization failed: {e}"),
        debug_info: None,
    })?;
    let deserialized: ValidationResult = serde_json::from_str(&serialized)
        .map_err(|e| SongbirdError::configuration("Error occurred".to_string()))?;

    assert_eq!(result.is_valid, deserialized.is_valid);
    assert_eq!(result.errors, deserialized.errors);
    assert_eq!(result.warnings, deserialized.warnings);
    Ok(())
}

#[test]
fn test_validation_result_empty_success() {
    let result = ValidationResult::success();

    assert!(result.is_valid);
    assert!(result.errors.is_empty());
    assert!(result.warnings.is_empty());
}

#[test]
fn test_validation_result_multiple_errors() {
    let errors = vec![
        "error1".to_string(),
        "error2".to_string(),
        "error3".to_string(),
        "error4".to_string(),
        "error5".to_string(),
    ];
    let result = ValidationResult::failure(errors.clone());

    assert!(!result.is_valid);
    assert_eq!(result.errors.len(), 5);
    assert_eq!(result.errors, errors);
}

#[test]
fn test_validation_result_success_immutability() {
    let result = ValidationResult::success();

    // Verify initial state
    assert!(result.is_valid);
    assert!(result.errors.is_empty());

    // Clone and modify
    let mut modified = result.clone();
    modified.warnings.push("new warning".to_string());

    // Original should be unchanged
    assert!(result.warnings.is_empty());
}

#[test]
fn test_validation_result_error_messages_preserved() {
    let error_message = "This is a very specific error message with details";
    let result = ValidationResult::failure(vec![error_message.to_string()]);

    assert_eq!(result.errors[0], error_message);
}

#[test]
fn test_validation_result_warning_messages_preserved() {
    let mut result = ValidationResult::success();
    let warning_message = "This is a warning message";
    result.warnings.push(warning_message.to_string());

    assert_eq!(result.warnings[0], warning_message);
}

#[test]
fn test_validation_result_failure_with_detailed_errors() {
    let errors = vec![
        "Field 'username' is required".to_string(),
        "Field 'email' must be a valid email address".to_string(),
        "Field 'age' must be between 0 and 120".to_string(),
    ];
    let result = ValidationResult::failure(errors.clone());

    assert!(!result.is_valid);
    assert_eq!(result.errors, errors);
}
