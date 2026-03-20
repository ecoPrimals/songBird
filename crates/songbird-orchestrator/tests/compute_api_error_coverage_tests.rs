// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Comprehensive error path tests for Compute API
//!
//! Tests error handling, edge cases, and failure scenarios in the compute API.
//! Targets code paths identified as needing coverage in the audit.

#![allow(clippy::unwrap_used)] // Test code
use songbird_orchestrator::server::compute_api::ApiError;
use std::error::Error;
#[test]
fn test_api_error_routing() {
    let error = ApiError::Routing("Service unavailable".to_string());
    let display = format!("{}", error);
    assert!(display.contains("Routing error"));
    assert!(display.contains("Service unavailable"));
}
#[test]
fn test_api_error_execution() {
    let error = ApiError::Execution("Task failed".to_string());
    let display = format!("{}", error);
    assert!(display.contains("Execution error"));
    assert!(display.contains("Task failed"));
}

#[test]
fn test_api_error_invalid_request() {
    let error = ApiError::InvalidRequest("Missing parameter".to_string());
    let display = format!("{}", error);
    assert!(display.contains("Invalid request"));
    assert!(display.contains("Missing parameter"));
}

#[test]
fn test_api_error_not_found() {
    let error = ApiError::NotFound("Job abc123".to_string());
    let display = format!("{}", error);
    assert!(display.contains("Not found"));
    assert!(display.contains("Job abc123"));
}

#[test]
fn test_api_error_trait_implementation() {
    let error = ApiError::Routing("test".to_string());
    // Test Error trait
    assert!(error.source().is_none());
    // Test Display
    let display_str = format!("{}", error);
    assert!(!display_str.is_empty());
    // Test Debug
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("Routing"));
}

#[test]
fn test_api_error_types_distinct() {
    let routing = ApiError::Routing("r".to_string());
    let execution = ApiError::Execution("e".to_string());
    let invalid = ApiError::InvalidRequest("i".to_string());
    let not_found = ApiError::NotFound("n".to_string());
    // Ensure different error types produce different messages
    assert_ne!(format!("{}", routing), format!("{}", execution));
    assert_ne!(format!("{}", invalid), format!("{}", not_found));
    assert_ne!(format!("{}", routing), format!("{}", invalid));
}

#[test]
fn test_api_error_empty_message() {
    let error = ApiError::Routing(String::new());
    let display = format!("{}", error);
    assert!(!display.is_empty());
}

#[test]
fn test_api_error_long_message() {
    let long_msg = "a".repeat(1000);
    let error = ApiError::Execution(long_msg.clone());
    let display = format!("{}", error);
    assert!(display.contains(&long_msg));
}

#[test]
fn test_api_error_special_characters() {
    let special = "Error: Failed with\n\ttab\rand \"quotes\"";
    let error = ApiError::InvalidRequest(special.to_string());
    let display = format!("{}", error);
    assert!(display.contains(special));
}

#[test]
fn test_api_error_unicode() {
    let unicode = "错误: サービスが見つかりません 🚫";
    let error = ApiError::NotFound(unicode.to_string());
    let display = format!("{}", error);
    assert!(display.contains(unicode));
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_error_chain() {
        // Test that errors can be used in Result chains
        let result: Result<(), ApiError> = Err(ApiError::Routing("test".to_string()));
        assert!(result.is_err());
        match result {
            Err(ApiError::Routing(msg)) => assert_eq!(msg, "test"),
            _ => panic!("Expected Routing error"),
        }
    }

    #[test]
    fn test_error_propagation() {
        fn inner_function() -> Result<(), ApiError> {
            Err(ApiError::Execution("inner error".to_string()))
        }

        fn outer_function() -> Result<(), ApiError> {
            inner_function()?;
            Ok(())
        }

        let result = outer_function();
        assert!(matches!(result.expect_err("testing error case"), ApiError::Execution(_)));
    }

    #[test]
    fn test_error_matching() {
        let errors = vec![
            ApiError::Routing("r".to_string()),
            ApiError::Execution("e".to_string()),
            ApiError::InvalidRequest("i".to_string()),
            ApiError::NotFound("n".to_string()),
        ];
        for error in errors {
            match error {
                ApiError::Routing(_)
                | ApiError::Execution(_)
                | ApiError::InvalidRequest(_)
                | ApiError::NotFound(_) => {
                    // All variants covered
                }
            }
        }
    }
}

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_concurrent_error_creation() {
        use std::thread;
        let handles: Vec<_> = (0..10)
            .map(|i| thread::spawn(move || ApiError::Routing(format!("Error {}", i))))
            .collect();
        for handle in handles {
            let error = handle.join().expect("test precondition");
            assert!(format!("{}", error).contains("Error"));
        }
    }

    #[test]
    fn test_error_in_async_context() {
        // Ensure errors work in async contexts
        async fn async_error() -> Result<(), ApiError> {
            Err(ApiError::Execution("async error".to_string()))
        }

        let runtime = tokio::runtime::Runtime::new().expect("test precondition");
        let result = runtime.block_on(async_error());
        assert!(result.is_err());
    }
}
