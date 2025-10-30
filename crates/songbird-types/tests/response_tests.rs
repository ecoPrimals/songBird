//! Comprehensive tests for `SongbirdResponse` and `ResponseError`
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
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]

use songbird_types::{ResponseError, SongbirdError, SongbirdResponse};
use std::collections::HashMap;

// ============================================================================
// SONGBIRD RESPONSE SUCCESS TESTS
// ============================================================================

#[test]
fn test_response_success_string() {
    let response = SongbirdResponse::success("test data".to_string());

    assert!(response.success);
    assert!(response.is_success());
    assert!(!response.is_error());
    assert!(response.data.is_some());
    assert_eq!(response.data.unwrap(), "test data");
    assert!(response.error.is_none());
}

#[test]
fn test_response_success_integer() {
    let response = SongbirdResponse::success(42);

    assert!(response.success);
    assert!(response.data.is_some());
    assert_eq!(response.data.unwrap(), 42);
}

#[test]
fn test_response_success_json() {
    let data = serde_json::json!({
        "name": "test",
        "value": 123
    });

    let response = SongbirdResponse::success(data.clone());

    assert!(response.success);
    assert!(response.data.is_some());
    assert_eq!(response.data.unwrap(), data);
}

#[test]
fn test_response_success_vec() {
    let items = vec![1, 2, 3, 4, 5];
    let response = SongbirdResponse::success(items.clone());

    assert!(response.success);
    assert_eq!(response.data.unwrap(), items);
}

// ============================================================================
// SONGBIRD RESPONSE ERROR TESTS
// ============================================================================

#[test]
fn test_response_error_creation() {
    let response: SongbirdResponse<String> =
        SongbirdResponse::error("req-123", "Something went wrong");

    assert!(!response.success);
    assert!(response.is_error());
    assert!(!response.is_success());
    assert!(response.data.is_none());
    assert!(response.error.is_some());
}

#[test]
fn test_response_from_songbird_error() {
    let error = SongbirdError::configuration("Invalid config");
    let response: SongbirdResponse<String> = SongbirdResponse::from_error(&error);

    assert!(!response.success);
    assert!(response.data.is_none());
    assert!(response.error.is_some());

    let response_error = response.error.unwrap();
    assert_eq!(response_error.code, "SONGBIRD_ERROR");
    assert!(response_error.message.contains("Invalid config"));
}

#[test]
fn test_response_error_types() {
    let network_error = SongbirdError::network("Connection failed".to_string());
    let response: SongbirdResponse<i32> = SongbirdResponse::from_error(&network_error);

    assert!(!response.success);
    assert!(response.error.is_some());
}

// ============================================================================
// RESPONSE METADATA TESTS
// ============================================================================

#[test]
fn test_response_with_metadata() {
    let mut response = SongbirdResponse::success(42);
    response.with_metadata("request_id", "123");
    response.with_metadata("timestamp", "2025-10-22");

    assert!(response.metadata.is_some());
    let metadata = response.metadata.unwrap();
    assert_eq!(metadata.len(), 2);
    assert_eq!(metadata.get("request_id"), Some(&"123".to_string()));
}

#[test]
fn test_response_multiple_metadata() {
    let mut response = SongbirdResponse::success("test");
    response
        .with_metadata("key1", "value1")
        .with_metadata("key2", "value2")
        .with_metadata("key3", "value3");

    let metadata = response.metadata.unwrap();
    assert_eq!(metadata.len(), 3);
}

// ============================================================================
// RESPONSE ERROR STRUCT TESTS
// ============================================================================

#[test]
fn test_response_error_struct() {
    let error = ResponseError {
        code: "ERR_NOT_FOUND".to_string(),
        message: "Resource not found".to_string(),
        details: None,
    };

    assert_eq!(error.code, "ERR_NOT_FOUND");
    assert_eq!(error.message, "Resource not found");
    assert!(error.details.is_none());
}

#[test]
fn test_response_error_with_details() {
    let mut details = HashMap::new();
    details.insert("resource_type".to_string(), "user".to_string());
    details.insert("resource_id".to_string(), "123".to_string());

    let error = ResponseError {
        code: "ERR_VALIDATION".to_string(),
        message: "Validation failed".to_string(),
        details: Some(details.clone()),
    };

    assert!(error.details.is_some());
    let error_details = error.details.unwrap();
    assert_eq!(error_details.len(), 2);
    assert_eq!(error_details.get("resource_type"), Some(&"user".to_string()));
}

// ============================================================================
// SERIALIZATION TESTS
// ============================================================================

#[test]
fn test_response_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let response = SongbirdResponse::success("test data".to_string());
    let json = serde_json::to_string(&response)?;

    assert!(!json.is_empty());
    assert!(json.contains("test data"));
    assert!(json.contains("success"));
    Ok(())
}

#[test]
fn test_response_deserialization() -> Result<(), Box<dyn std::error::Error>> {
    let response = SongbirdResponse::success(42);
    let json = serde_json::to_string(&response)?;
    let deserialized: SongbirdResponse<i32> = serde_json::from_str(&json)?;

    assert_eq!(response.success, deserialized.success);
    assert_eq!(response.data, deserialized.data);
    Ok(())
}

#[test]
fn test_response_error_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let error = ResponseError {
        code: "ERR_TEST".to_string(),
        message: "Test error".to_string(),
        details: None,
    };

    let json = serde_json::to_string(&error)?;
    let deserialized: ResponseError = serde_json::from_str(&json)?;

    assert_eq!(error.code, deserialized.code);
    assert_eq!(error.message, deserialized.message);
    Ok(())
}

// ============================================================================
// COMPLEX SCENARIO TESTS
// ============================================================================

#[test]
fn test_success_response_workflow() {
    // Simulate successful operation
    let result = perform_operation(true);

    assert!(result.success);
    assert!(result.data.is_some());
    assert_eq!(result.data.unwrap(), "Operation successful");
}

#[test]
fn test_error_response_workflow() {
    // Simulate failed operation
    let result = perform_operation(false);

    assert!(!result.success);
    assert!(result.data.is_none());
    assert!(result.error.is_some());
}

#[test]
fn test_response_with_full_metadata() {
    let mut response = SongbirdResponse::success(vec![1, 2, 3]);
    response
        .with_metadata("request_id", "req-789")
        .with_metadata("user_id", "user-456")
        .with_metadata("timestamp", "2025-10-22T12:00:00Z")
        .with_metadata("source", "api");

    assert!(response.metadata.is_some());
    let metadata = response.metadata.unwrap();
    assert_eq!(metadata.len(), 4);
    assert!(metadata.contains_key("request_id"));
    assert!(metadata.contains_key("timestamp"));
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn perform_operation(should_succeed: bool) -> SongbirdResponse<String> {
    if should_succeed {
        SongbirdResponse::success("Operation successful".to_string())
    } else {
        SongbirdResponse::error("req-001", "Operation failed")
    }
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

#[test]
fn test_response_empty_string() {
    let response = SongbirdResponse::success(String::new());

    assert!(response.success);
    assert!(response.data.is_some());
    assert_eq!(response.data.unwrap(), "");
}

#[test]
fn test_response_empty_vec() {
    let response: SongbirdResponse<Vec<i32>> = SongbirdResponse::success(vec![]);

    assert!(response.success);
    assert!(response.data.is_some());
    assert_eq!(response.data.unwrap().len(), 0);
}

#[test]
fn test_response_large_data() {
    let large_vec: Vec<i32> = (0..10000).collect();
    let response = SongbirdResponse::success(large_vec);

    assert!(response.success);
    assert_eq!(response.data.unwrap().len(), 10000);
}

#[test]
fn test_multiple_error_conversions() {
    let errors = vec![
        SongbirdError::configuration("Config error"),
        SongbirdError::network("Network error".to_string()),
        SongbirdError::security("Unauthorized"),
        SongbirdError::service("test-service", "Service error"),
    ];

    for error in errors {
        let response: SongbirdResponse<String> = SongbirdResponse::from_error(&error);
        assert!(!response.success);
        assert!(response.error.is_some());
    }
}
