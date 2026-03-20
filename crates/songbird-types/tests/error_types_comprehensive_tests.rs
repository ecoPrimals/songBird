// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Comprehensive tests for `SongbirdError` types
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
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(clippy::unwrap_used, clippy::expect_used)]

use songbird_types::{SecurityError, SongbirdError, SongbirdResult};

// ============================================================================
// SONGBIRD ERROR CREATION TESTS
// ============================================================================

#[test]
fn test_configuration_error() {
    let error = SongbirdError::configuration("Invalid configuration");
    assert!(error.to_string().contains("Invalid configuration"));
}

#[test]
fn test_network_error() {
    let error = SongbirdError::network("Connection failed".to_string());
    assert!(error.to_string().contains("Connection failed"));
}

#[test]
fn test_security_error() {
    let error = SongbirdError::security("Unauthorized access");
    assert!(error.to_string().contains("Unauthorized"));
}

#[test]
fn test_service_error() {
    let error = SongbirdError::service("database", "Connection timeout");
    assert!(error.to_string().contains("database"));
    assert!(error.to_string().contains("Connection timeout"));
}

// ============================================================================
// ERROR DISPLAY TESTS
// ============================================================================

#[test]
fn test_error_display_formatting() {
    let error = SongbirdError::configuration("Test error");
    let display = format!("{error}");
    assert!(!display.is_empty());
}

#[test]
fn test_error_debug_formatting() {
    let error = SongbirdError::network("Test".to_string());
    let debug = format!("{error:?}");
    assert!(!debug.is_empty());
}

// ============================================================================
// ERROR CONVERSION TESTS
// ============================================================================

#[test]
fn test_error_from_string() {
    let error = SongbirdError::from("Test error");
    assert!(error.to_string().contains("Test error"));
}

#[test]
fn test_error_from_io_error() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let songbird_error = SongbirdError::from(io_error);
    assert!(songbird_error.to_string().contains("File not found"));
}

// ============================================================================
// SECURITY ERROR TESTS
// ============================================================================

#[test]
fn test_security_error_full() {
    let error = SecurityError {
        message: "Access denied".to_string(),
        operation: Some("read".to_string()),
        required_permission: Some("admin".to_string()),
        context: Some("Authentication check".to_string()),
        remediation: Some("Contact administrator".to_string()),
    };

    assert_eq!(error.message, "Access denied");
    assert_eq!(error.operation, Some("read".to_string()));
    assert_eq!(error.required_permission, Some("admin".to_string()));
}

#[test]
fn test_security_error_minimal() {
    let error = SecurityError {
        message: "Forbidden".to_string(),
        operation: None,
        required_permission: None,
        context: None,
        remediation: None,
    };

    assert_eq!(error.message, "Forbidden");
    assert!(error.operation.is_none());
}

#[test]
fn test_security_error_with_context() {
    let error = SecurityError {
        message: "Token expired".to_string(),
        operation: Some("authenticate".to_string()),
        required_permission: None,
        context: Some("JWT validation".to_string()),
        remediation: Some("Refresh token".to_string()),
    };

    assert!(error.context.is_some());
    assert!(error.remediation.is_some());
}

// ============================================================================
// RESULT TYPE TESTS
// ============================================================================

#[test]
fn test_result_ok() {
    let result: SongbirdResult<i32> = Ok(42);
    assert!(result.is_ok());
    if let Ok(value) = result {
        assert_eq!(value, 42);
    }
}

#[test]
fn test_result_err() {
    let result: SongbirdResult<String> = Err(SongbirdError::configuration("Test"));
    assert!(result.is_err());
}

#[test]
fn test_result_map() -> SongbirdResult<()> {
    let result: SongbirdResult<i32> = Ok(10);
    let mapped = result.map(|x| x * 2);
    assert_eq!(mapped?, 20);
    Ok(())
}

#[test]
fn test_result_and_then() -> SongbirdResult<()> {
    let result: SongbirdResult<i32> = Ok(5);
    let chained = result.map(|x| x + 5);
    assert_eq!(chained?, 10);
    Ok(())
}

// ============================================================================
// ERROR CHAIN TESTS
// ============================================================================

#[test]
fn test_error_propagation() {
    fn inner_function() -> SongbirdResult<i32> {
        Err(SongbirdError::configuration("Inner error"))
    }

    fn outer_function() -> SongbirdResult<i32> {
        inner_function()?;
        Ok(42)
    }

    let result = outer_function();
    assert!(result.is_err());
}

#[test]
fn test_multiple_error_types() {
    let errors = vec![
        SongbirdError::configuration("Config error"),
        SongbirdError::network("Network error".to_string()),
        SongbirdError::security("Security error"),
        SongbirdError::service("service", "Service error"),
    ];

    assert_eq!(errors.len(), 4);
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

#[test]
fn test_empty_error_message() {
    let error = SongbirdError::configuration("");
    assert!(!error.to_string().is_empty()); // Should still have some text
}

#[test]
fn test_very_long_error_message() {
    let long_message = "Error ".repeat(1000);
    let error = SongbirdError::configuration(long_message);
    assert!(error.to_string().contains("Error"));
}

#[test]
fn test_unicode_error_message() {
    let error = SongbirdError::configuration("错误信息 🚨");
    assert!(error.to_string().contains("错误"));
    assert!(error.to_string().contains("🚨"));
}

#[test]
fn test_error_with_newlines() {
    let error = SongbirdError::configuration("Line 1\nLine 2\nLine 3");
    assert!(error.to_string().contains("Line 1"));
}

// ============================================================================
// SERIALIZATION TESTS
// ============================================================================

#[test]
fn test_security_error_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let error = SecurityError {
        message: "Test".to_string(),
        operation: Some("test_op".to_string()),
        required_permission: Some("test_perm".to_string()),
        context: None,
        remediation: None,
    };

    let json = serde_json::to_string(&error)?;
    assert!(!json.is_empty());

    let deserialized: SecurityError = serde_json::from_str(&json)?;
    assert_eq!(deserialized.message, "Test");
    Ok(())
}

// ============================================================================
// PRACTICAL SCENARIO TESTS
// ============================================================================

#[test]
fn test_config_validation_error() {
    let error = SongbirdError::configuration("Port must be between 1024 and 65535");
    assert!(error.to_string().contains("Port"));
    assert!(error.to_string().contains("1024"));
}

#[test]
fn test_network_timeout_error() {
    let error = SongbirdError::network("Request timeout after 30 seconds".to_string());
    assert!(error.to_string().contains("timeout"));
}

#[test]
fn test_authentication_error() {
    let error = SongbirdError::security("Invalid JWT token");
    assert!(error.to_string().contains("JWT"));
}

#[test]
fn test_service_unavailable_error() {
    let error = SongbirdError::service("payment-service", "Service temporarily unavailable");
    assert!(error.to_string().contains("payment-service"));
    assert!(error.to_string().contains("unavailable"));
}

#[test]
fn test_database_connection_error() {
    let error = SongbirdError::service("database", "Failed to establish connection pool");
    assert!(error.to_string().contains("database"));
    assert!(error.to_string().contains("connection"));
}
