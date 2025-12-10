//! Tests for capability error types

use super::CapabilityError;

#[test]
fn test_network_error_display() {
    let err = CapabilityError::NetworkError("connection refused".to_string());
    assert_eq!(err.to_string(), "Network error: connection refused");
}

#[test]
fn test_parse_error_display() {
    let err = CapabilityError::ParseError("invalid JSON".to_string());
    assert_eq!(err.to_string(), "Parse error: invalid JSON");
}

#[test]
fn test_primal_not_found_display() {
    let err = CapabilityError::PrimalNotFound("beardog".to_string());
    assert_eq!(err.to_string(), "Primal not found: beardog");
}

#[test]
fn test_capability_unavailable_display() {
    let err = CapabilityError::CapabilityUnavailable("storage".to_string());
    assert_eq!(err.to_string(), "Capability unavailable: storage");
}

#[test]
fn test_error_trait_implementation() {
    let err = CapabilityError::NetworkError("test".to_string());
    // Verify it implements std::error::Error
    let _: &dyn std::error::Error = &err;
}

#[test]
fn test_network_error_debug() {
    let err = CapabilityError::NetworkError("timeout".to_string());
    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("NetworkError"));
    assert!(debug_str.contains("timeout"));
}

#[test]
fn test_parse_error_debug() {
    let err = CapabilityError::ParseError("malformed".to_string());
    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("ParseError"));
    assert!(debug_str.contains("malformed"));
}

#[test]
fn test_primal_not_found_debug() {
    let err = CapabilityError::PrimalNotFound("squirrel".to_string());
    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("PrimalNotFound"));
    assert!(debug_str.contains("squirrel"));
}

#[test]
fn test_capability_unavailable_debug() {
    let err = CapabilityError::CapabilityUnavailable("compute".to_string());
    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("CapabilityUnavailable"));
    assert!(debug_str.contains("compute"));
}

#[test]
fn test_error_with_empty_message() {
    let err = CapabilityError::NetworkError(String::new());
    assert_eq!(err.to_string(), "Network error: ");
}

#[test]
fn test_error_with_special_characters() {
    let err = CapabilityError::ParseError("invalid: {\"key\": \"value\"}".to_string());
    let display = err.to_string();
    assert!(display.contains("Parse error"));
    assert!(display.contains("{\"key\": \"value\"}"));
}

#[test]
fn test_error_with_unicode() {
    let err = CapabilityError::PrimalNotFound("🦊 fox-primal".to_string());
    let display = err.to_string();
    assert!(display.contains("Primal not found"));
    assert!(display.contains("🦊 fox-primal"));
}

#[test]
fn test_all_error_variants_are_unique() {
    let errors = vec![
        CapabilityError::NetworkError("net".to_string()),
        CapabilityError::ParseError("parse".to_string()),
        CapabilityError::PrimalNotFound("primal".to_string()),
        CapabilityError::CapabilityUnavailable("cap".to_string()),
    ];

    // Verify each error displays differently
    let displays: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
    for i in 0..displays.len() {
        for j in (i + 1)..displays.len() {
            assert_ne!(displays[i], displays[j], "Error variants should display differently");
        }
    }
}

#[test]
fn test_error_size_is_reasonable() {
    // Errors should be small for efficient Result<T, E> handling
    let size = std::mem::size_of::<CapabilityError>();
    // String is 24 bytes on 64-bit, enum discriminant adds 8, so ~32 bytes
    assert!(size <= 40, "CapabilityError should be reasonably sized, got {}", size);
}
