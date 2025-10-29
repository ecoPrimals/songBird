//! Comprehensive tests for the response module
//!
//! These tests ensure complete coverage of response types and their methods.

use songbird_types::errors::SongbirdError;
use songbird_types::response::{
    AIFirstResponse, BoolResponse, JsonResponse, PaginatedResponse, ResponseError,
    SongbirdResponse, StringResponse,
};

// ==================== SongbirdResponse Tests ====================

#[test]
fn test_songbird_response_success() {
    let response = SongbirdResponse::success(42);
    assert!(response.is_success());
    assert!(!response.is_error());
    assert_eq!(response.data, Some(42));
    assert!(response.error.is_none());
    assert!(response.metadata.is_none());
}

#[test]
fn test_songbird_response_error() {
    let response: SongbirdResponse<String> = SongbirdResponse::error("req-123", "Failed");
    assert!(!response.is_success());
    assert!(response.is_error());
    assert!(response.data.is_none());
    assert!(response.error.is_some());

    let error = response.error.unwrap();
    assert_eq!(error.code, "ERROR");
    assert_eq!(error.message, "An error occurred");
}

#[test]
fn test_songbird_response_from_error() {
    let songbird_error = SongbirdError::Configuration {
        message: "Invalid config".to_string(),
        field: None,
        suggestion: None,
    };

    let response: SongbirdResponse<String> = SongbirdResponse::from_error(&songbird_error);
    assert!(!response.is_success());
    assert!(response.is_error());

    let error = response.error.unwrap();
    assert_eq!(error.code, "SONGBIRD_ERROR");
    assert!(error.message.contains("Invalid config"));
}

#[test]
fn test_songbird_response_with_metadata() {
    let mut response = SongbirdResponse::success("data");
    response.with_metadata("request_id", "req-456");
    response.with_metadata("timestamp", "2025-10-25T12:00:00Z");

    assert!(response.metadata.is_some());
    let metadata = response.metadata.as_ref().unwrap();
    assert_eq!(metadata.len(), 2);
    assert_eq!(metadata.get("request_id"), Some(&"req-456".to_string()));
    assert_eq!(metadata.get("timestamp"), Some(&"2025-10-25T12:00:00Z".to_string()));
}

#[test]
fn test_songbird_response_with_metadata_chain() {
    let mut response = SongbirdResponse::success(100);
    response.with_metadata("source", "api").with_metadata("version", "v1");

    let metadata = response.metadata.as_ref().unwrap();
    assert_eq!(metadata.len(), 2);
}

#[test]
fn test_songbird_response_get_data_success() {
    let response = SongbirdResponse::success("hello");
    let result = response.get_data();
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), "hello");
}

#[test]
fn test_songbird_response_get_data_error() {
    let response: SongbirdResponse<String> = SongbirdResponse::error("req", "error");
    let result = response.get_data();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "An error occurred");
}

#[test]
fn test_songbird_response_get_data_success_but_no_data() {
    let response: SongbirdResponse<String> = SongbirdResponse {
        success: true,
        data: None,
        error: None,
        metadata: None,
    };

    let result = response.get_data();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Response marked as successful but contains no data");
}

#[test]
fn test_songbird_response_into_result_success() {
    let response = SongbirdResponse::success(42);
    let result = response.into_result();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_songbird_response_into_result_error() {
    let response: SongbirdResponse<i32> = SongbirdResponse::error("req", "failed");
    let result = response.into_result();
    assert!(result.is_err());
}

#[test]
fn test_songbird_response_into_result_success_no_data() {
    let response: SongbirdResponse<String> = SongbirdResponse {
        success: true,
        data: None,
        error: None,
        metadata: None,
    };

    let result = response.into_result();
    assert!(result.is_err());
}

#[test]
fn test_songbird_response_clone() {
    let response = SongbirdResponse::success("test");
    let cloned = response.clone();
    assert_eq!(cloned.success, response.success);
    assert_eq!(cloned.data, response.data);
}

// ==================== ResponseError Tests ====================

#[test]
fn test_response_error_creation() {
    let error = ResponseError {
        code: "NOT_FOUND".to_string(),
        message: "Resource not found".to_string(),
        details: None,
    };

    assert_eq!(error.code, "NOT_FOUND");
    assert_eq!(error.message, "Resource not found");
    assert!(error.details.is_none());
}

#[test]
fn test_response_error_with_details() {
    let mut details = std::collections::HashMap::new();
    details.insert("field".to_string(), "username".to_string());
    details.insert("constraint".to_string(), "unique".to_string());

    let error = ResponseError {
        code: "VALIDATION_ERROR".to_string(),
        message: "Validation failed".to_string(),
        details: Some(details),
    };

    assert!(error.details.is_some());
    let details = error.details.unwrap();
    assert_eq!(details.len(), 2);
    assert_eq!(details.get("field"), Some(&"username".to_string()));
}

#[test]
fn test_response_error_clone() {
    let error = ResponseError {
        code: "ERROR".to_string(),
        message: "message".to_string(),
        details: None,
    };

    let cloned = error.clone();
    assert_eq!(cloned.code, error.code);
    assert_eq!(cloned.message, error.message);
}

// ==================== AIFirstResponse Tests ====================

#[test]
fn test_ai_first_response_new() {
    let response = AIFirstResponse::new("Hello, World!");
    assert_eq!(response.data, "Hello, World!");
    assert!(response.context.is_none());
    assert!(response.confidence.is_none());
    assert!(response.suggested_actions.is_empty());
}

#[test]
fn test_ai_first_response_with_context() {
    let mut response = AIFirstResponse::new(42);
    response.with_context("This is the answer to life");

    assert_eq!(response.context, Some("This is the answer to life".to_string()));
}

#[test]
fn test_ai_first_response_with_confidence() {
    let mut response = AIFirstResponse::new("data");
    response.with_confidence(0.85);

    assert_eq!(response.confidence, Some(0.85));
}

#[test]
fn test_ai_first_response_with_confidence_clamping() {
    let mut response1 = AIFirstResponse::new("data");
    response1.with_confidence(1.5);
    assert_eq!(response1.confidence, Some(1.0));

    let mut response2 = AIFirstResponse::new("data");
    response2.with_confidence(-0.5);
    assert_eq!(response2.confidence, Some(0.0));
}

#[test]
fn test_ai_first_response_with_action() {
    let mut response = AIFirstResponse::new("data");
    response.with_action("Click here to continue");

    assert_eq!(response.suggested_actions.len(), 1);
    assert_eq!(response.suggested_actions[0], "Click here to continue");
}

#[test]
fn test_ai_first_response_with_multiple_actions() {
    let mut response = AIFirstResponse::new("data");
    response.with_action("Action 1");
    response.with_action("Action 2");
    response.with_action("Action 3");

    assert_eq!(response.suggested_actions.len(), 3);
    assert_eq!(response.suggested_actions[2], "Action 3");
}

#[test]
fn test_ai_first_response_builder_chain() {
    let mut response = AIFirstResponse::new("result");
    response
        .with_context("Analysis complete")
        .with_confidence(0.92)
        .with_action("Review results")
        .with_action("Export data");

    assert!(response.context.is_some());
    assert_eq!(response.confidence, Some(0.92));
    assert_eq!(response.suggested_actions.len(), 2);
}

#[test]
fn test_ai_first_response_clone() {
    let response = AIFirstResponse::new("test");
    let cloned = response.clone();
    assert_eq!(cloned.data, response.data);
}

// ==================== PaginatedResponse Tests ====================

#[test]
fn test_paginated_response_single_page() {
    let items = vec![1, 2, 3, 4, 5];
    let response = PaginatedResponse::new(items, 0, 10, 5);

    assert_eq!(response.items.len(), 5);
    assert_eq!(response.page, 0);
    assert_eq!(response.per_page, 10);
    assert_eq!(response.total, 5);
    assert_eq!(response.total_pages, 1);
    assert!(!response.has_more);
}

#[test]
fn test_paginated_response_multiple_pages() {
    let items = vec![1, 2, 3, 4, 5];
    let response = PaginatedResponse::new(items, 0, 5, 25);

    assert_eq!(response.items.len(), 5);
    assert_eq!(response.page, 0);
    assert_eq!(response.per_page, 5);
    assert_eq!(response.total, 25);
    assert_eq!(response.total_pages, 5);
    assert!(response.has_more);
}

#[test]
fn test_paginated_response_last_page() {
    let items = vec![1, 2];
    let response = PaginatedResponse::new(items, 4, 5, 22);

    assert_eq!(response.page, 4);
    assert_eq!(response.total_pages, 5);
    assert!(!response.has_more);
}

#[test]
fn test_paginated_response_middle_page() {
    let items = vec![11, 12, 13, 14, 15];
    let response = PaginatedResponse::new(items, 2, 5, 50);

    assert_eq!(response.page, 2);
    assert_eq!(response.total_pages, 10);
    assert!(response.has_more);
}

#[test]
fn test_paginated_response_empty_items() {
    let items: Vec<i32> = vec![];
    let response = PaginatedResponse::new(items, 0, 10, 0);

    assert!(response.items.is_empty());
    assert_eq!(response.total, 0);
    assert_eq!(response.total_pages, 0);
    assert!(!response.has_more);
}

#[test]
fn test_paginated_response_exact_page_fit() {
    let items = vec![1, 2, 3, 4, 5];
    let response = PaginatedResponse::new(items, 0, 5, 10);

    assert_eq!(response.total_pages, 2);
    assert!(response.has_more);
}

#[test]
fn test_paginated_response_clone() {
    let items = vec![1, 2, 3];
    let response = PaginatedResponse::new(items, 0, 10, 3);
    let cloned = response.clone();

    assert_eq!(cloned.items, response.items);
    assert_eq!(cloned.page, response.page);
}

// ==================== Type Alias Tests ====================

#[test]
fn test_string_response_ok() {
    let response = StringResponse::ok("Success!");
    assert!(response.is_success());
    assert_eq!(response.data, Some("Success!".to_string()));
}

#[test]
fn test_string_response_ok_conversion() {
    let response = StringResponse::ok("Message");
    let result = response.into_result();
    assert_eq!(result, Ok("Message".to_string()));
}

#[test]
fn test_bool_response_boolean() {
    let response_true = BoolResponse::boolean(true);
    assert!(response_true.is_success());
    assert_eq!(response_true.data, Some(true));

    let response_false = BoolResponse::boolean(false);
    assert!(response_false.is_success());
    assert_eq!(response_false.data, Some(false));
}

#[test]
fn test_json_response() {
    let json_value = serde_json::json!({"key": "value", "number": 42});
    let response = JsonResponse::success(json_value.clone());

    assert!(response.is_success());
    assert_eq!(response.data, Some(json_value));
}

// ==================== From Trait Tests ====================

#[test]
fn test_from_result_ok() {
    let result: Result<i32, SongbirdError> = Ok(100);
    let response: SongbirdResponse<i32> = result.into();

    assert!(response.is_success());
    assert_eq!(response.data, Some(100));
}

#[test]
fn test_from_result_err() {
    let result: Result<String, SongbirdError> = Err(SongbirdError::Configuration {
        message: "Config failed".to_string(),
        field: None,
        suggestion: None,
    });
    let response: SongbirdResponse<String> = result.into();

    assert!(response.is_error());
    assert!(response.error.is_some());
}

// ==================== Serialization Tests ====================

#[test]
fn test_songbird_response_serialization() {
    let response = SongbirdResponse::success("test");
    let json = serde_json::to_string(&response).expect("Failed to serialize");
    assert!(json.contains("test"));
    assert!(json.contains("\"success\":true"));
}

#[test]
fn test_songbird_response_deserialization() {
    let json = r#"{"success":true,"data":"hello","error":null,"metadata":null}"#;
    let response: SongbirdResponse<String> =
        serde_json::from_str(json).expect("Failed to deserialize");
    assert!(response.is_success());
    assert_eq!(response.data, Some("hello".to_string()));
}

#[test]
fn test_response_error_serialization() {
    let error = ResponseError {
        code: "ERR".to_string(),
        message: "Error occurred".to_string(),
        details: None,
    };
    let json = serde_json::to_string(&error).expect("Failed to serialize");
    assert!(json.contains("ERR"));
}

#[test]
fn test_ai_first_response_serialization() {
    let response = AIFirstResponse::new("data");
    let json = serde_json::to_string(&response).expect("Failed to serialize");
    assert!(json.contains("data"));
}

#[test]
fn test_paginated_response_serialization() {
    let response = PaginatedResponse::new(vec![1, 2, 3], 0, 10, 3);
    let json = serde_json::to_string(&response).expect("Failed to serialize");
    assert!(json.contains("\"page\":0"));
    assert!(json.contains("\"total\":3"));
}

// ==================== Edge Case Tests ====================

#[test]
fn test_response_with_complex_data() {
    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    struct ComplexData {
        id: u64,
        name: String,
        tags: Vec<String>,
    }

    let data = ComplexData {
        id: 123,
        name: "Test".to_string(),
        tags: vec!["tag1".to_string(), "tag2".to_string()],
    };

    let response = SongbirdResponse::success(data.clone());
    assert_eq!(response.data, Some(data));
}

#[test]
fn test_error_without_details() {
    let response: SongbirdResponse<String> = SongbirdResponse {
        success: false,
        data: None,
        error: Some(ResponseError {
            code: "ERR".to_string(),
            message: "Unknown error".to_string(),
            details: None,
        }),
        metadata: None,
    };

    let result = response.get_data();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Unknown error");
}

#[test]
fn test_error_with_no_error_object() {
    let response: SongbirdResponse<String> = SongbirdResponse {
        success: false,
        data: None,
        error: None,
        metadata: None,
    };

    let result = response.get_data();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Unknown error");
}

#[test]
fn test_confidence_edge_cases() {
    let mut response = AIFirstResponse::new("test");

    response.with_confidence(0.0);
    assert_eq!(response.confidence, Some(0.0));

    response.with_confidence(1.0);
    assert_eq!(response.confidence, Some(1.0));

    response.with_confidence(0.5);
    assert_eq!(response.confidence, Some(0.5));
}

#[test]
fn test_pagination_calculation_edge_cases() {
    // Test exact division
    let response1 = PaginatedResponse::new(vec![1, 2, 3], 0, 3, 9);
    assert_eq!(response1.total_pages, 3);

    // Test with remainder
    let response2 = PaginatedResponse::new(vec![1, 2, 3], 0, 3, 10);
    assert_eq!(response2.total_pages, 4);
}
