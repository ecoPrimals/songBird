// Allow unwrap/expect in tests - idiomatic for test code
#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Comprehensive tests for Error Helper Traits
//!
//! Coverage goal: 64% → 85%+
//!
//! Tests the UnwrapElimination trait and all helper methods for converting
//! standard errors into SongbirdError with proper context.

use songbird_types::error_helpers::UnwrapElimination;
use songbird_types::{SongbirdError, SongbirdResult};
use std::io;

// ============================================================================
// UNWRAP ELIMINATION TRAIT TESTS - or_config_error
// ============================================================================

#[test]
fn test_or_config_error_ok() {
    let result: Result<i32, &str> = Ok(42);
    let songbird_result = result.or_config_error("test_field");

    assert!(songbird_result.is_ok());
    assert_eq!(songbird_result.expect("test precondition"), 42);
}

#[test]
fn test_or_config_error_err() {
    let result: Result<i32, &str> = Err("invalid value");
    let songbird_result = result.or_config_error("port");

    assert!(songbird_result.is_err());
    let err = songbird_result.expect_err("testing error case");

    match err {
        SongbirdError::Configuration {
            message,
            field,
            suggestion,
        } => {
            assert!(message.contains("port"));
            assert!(message.contains("invalid value"));
            assert_eq!(field, Some("port".to_string()));
            assert!(suggestion.is_some());
        }
        _ => panic!("Expected Configuration error"),
    }
}

#[test]
fn test_or_config_error_with_io_error() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let result: Result<(), io::Error> = Err(io_err);
    let songbird_result = result.or_config_error("config_file");

    assert!(songbird_result.is_err());
    match songbird_result.expect_err("testing error case") {
        SongbirdError::Configuration {
            message,
            ..
        } => {
            assert!(message.contains("config_file"));
        }
        _ => panic!("Expected Configuration error"),
    }
}

// ============================================================================
// UNWRAP ELIMINATION TRAIT TESTS - or_network_error
// ============================================================================

#[test]
fn test_or_network_error_ok() {
    let result: Result<String, &str> = Ok("connected".to_string());
    let songbird_result = result.or_network_error("connection test");

    assert!(songbird_result.is_ok());
    assert_eq!(songbird_result.expect("test precondition"), "connected");
}

#[test]
fn test_or_network_error_err() {
    let result: Result<(), &str> = Err("connection refused");
    let songbird_result = result.or_network_error("TCP connection");

    assert!(songbird_result.is_err());
    match songbird_result.expect_err("testing error case") {
        SongbirdError::Network {
            message,
            interface,
            suggestion,
        } => {
            assert!(message.contains("TCP connection"));
            assert!(message.contains("connection refused"));
            assert!(interface.is_none());
            assert!(suggestion.is_some());
        }
        _ => panic!("Expected Network error"),
    }
}

#[test]
fn test_or_network_error_with_custom_message() {
    let result: Result<(), String> = Err("timeout after 30s".to_string());
    let songbird_result = result.or_network_error("discovery service");

    assert!(songbird_result.is_err());
    match songbird_result.expect_err("testing error case") {
        SongbirdError::Network {
            message,
            ..
        } => {
            assert!(message.contains("discovery service"));
            assert!(message.contains("timeout"));
        }
        _ => panic!("Expected Network error"),
    }
}

// ============================================================================
// UNWRAP ELIMINATION TRAIT TESTS - or_service_error
// ============================================================================

#[test]
fn test_or_service_error_ok() {
    let result: Result<bool, &str> = Ok(true);
    let songbird_result = result.or_service_error("test-service");

    assert!(songbird_result.is_ok());
}

#[test]
fn test_or_service_error_err() {
    let result: Result<(), &str> = Err("service unavailable");
    let songbird_result = result.or_service_error("storage");

    assert!(songbird_result.is_err());
    match songbird_result.expect_err("testing error case") {
        SongbirdError::Service {
            service,
            message,
            suggested_alternatives,
            recovery_actions,
        } => {
            assert_eq!(service, "storage");
            assert!(message.contains("service unavailable"));
            assert!(!recovery_actions.is_empty());
            let _ = suggested_alternatives; // May be empty
        }
        _ => panic!("Expected Service error"),
    }
}

// ============================================================================
// UNWRAP ELIMINATION TRAIT TESTS - or_discovery_error
// ============================================================================

#[test]
fn test_or_discovery_error_ok() {
    let result: Result<Vec<String>, &str> = Ok(vec!["service1".to_string()]);
    let songbird_result = result.or_discovery_error("DNS-SD");

    assert!(songbird_result.is_ok());
}

#[test]
fn test_or_discovery_error_err() {
    let result: Result<(), &str> = Err("no services found");
    let songbird_result = result.or_discovery_error("mDNS");

    assert!(songbird_result.is_err());
    match songbird_result.expect_err("testing error case") {
        SongbirdError::Discovery {
            message,
            backend,
            ..
        } => {
            assert!(message.contains("no services found"));
            assert!(backend.is_some());
        }
        _ => panic!("Expected Discovery error"),
    }
}

// ============================================================================
// UNWRAP ELIMINATION TRAIT TESTS - or_registry_error
// ============================================================================

#[test]
fn test_or_registry_error_ok() {
    let result: Result<u64, &str> = Ok(12345);
    let songbird_result = result.or_registry_error("register");

    assert!(songbird_result.is_ok());
}

#[test]
fn test_or_registry_error_err() {
    let result: Result<(), &str> = Err("service already registered");
    let songbird_result = result.or_registry_error("duplicate_registration");

    assert!(songbird_result.is_err());
    match songbird_result.expect_err("testing error case") {
        SongbirdError::Registry {
            message,
            operation,
            ..
        } => {
            assert!(message.contains("service already registered"));
            assert_eq!(operation, "duplicate_registration");
        }
        _ => panic!("Expected Registry error"),
    }
}

// ============================================================================
// CHAINING AND COMPOSITION TESTS
// ============================================================================

#[test]
fn test_chained_conversions() {
    // Simulate a chain of operations that might fail
    fn operation_chain() -> SongbirdResult<String> {
        let value = std::env::var("NONEXISTENT_VAR").or_config_error("NONEXISTENT_VAR")?;
        Ok(value)
    }

    let result = operation_chain();
    assert!(result.is_err());
    match result.expect_err("testing error case") {
        SongbirdError::Configuration {
            ..
        } => {}
        _ => panic!("Expected Configuration error"),
    }
}

#[test]
fn test_multiple_error_types() {
    fn test_with_io_error() -> SongbirdResult<()> {
        std::fs::read_to_string("/nonexistent/file").or_config_error("data_file")?;
        Ok(())
    }

    fn test_with_parse_error() -> SongbirdResult<()> {
        "not_a_number".parse::<u32>().or_config_error("port_number")?;
        Ok(())
    }

    assert!(test_with_io_error().is_err());
    assert!(test_with_parse_error().is_err());
}

// ============================================================================
// ERROR CONTEXT PRESERVATION TESTS
// ============================================================================

#[test]
fn test_error_context_preserved_in_config() {
    let result: Result<(), String> = Err("detailed error information".to_string());
    let songbird_result = result.or_config_error("critical_setting");

    match songbird_result.expect_err("testing error case") {
        SongbirdError::Configuration {
            message,
            field,
            suggestion,
        } => {
            assert!(message.contains("detailed error information"));
            assert!(message.contains("critical_setting"));
            assert_eq!(field, Some("critical_setting".to_string()));
            assert!(suggestion.expect("test precondition").contains("configuration"));
        }
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_error_context_preserved_in_network() {
    let result: Result<(), &str> = Err("connection timeout");
    let songbird_result = result.or_network_error("remote host");

    match songbird_result.expect_err("testing error case") {
        SongbirdError::Network {
            message,
            suggestion,
            ..
        } => {
            assert!(message.contains("connection timeout"));
            assert!(message.contains("remote host"));
            assert!(suggestion.expect("test precondition").contains("network"));
        }
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_error_suggestions_present() {
    let result: Result<(), &str> = Err("test error");

    // Config errors should suggest checking configuration
    let config_err = result.or_config_error("field");
    if let Err(SongbirdError::Configuration {
        suggestion,
        ..
    }) = config_err
    {
        assert!(suggestion.is_some());
        assert!(suggestion.expect("test precondition").contains("configuration"));
    }

    // Network errors should suggest checking network
    let network_err = result.or_network_error("context");
    if let Err(SongbirdError::Network {
        suggestion,
        ..
    }) = network_err
    {
        assert!(suggestion.is_some());
        assert!(suggestion.expect("test precondition").contains("network"));
    }
}

// ============================================================================
// RECOVERY ACTIONS TESTS
// ============================================================================

#[test]
fn test_service_error_has_recovery_actions() {
    let result: Result<(), &str> = Err("service down");
    let songbird_result = result.or_service_error("compute");

    match songbird_result.expect_err("testing error case") {
        SongbirdError::Service {
            recovery_actions,
            ..
        } => {
            assert!(!recovery_actions.is_empty());
            assert!(recovery_actions.iter().any(|a| a.contains("retry")));
        }
        _ => panic!("Expected Service error"),
    }
}

// ============================================================================
// REAL-WORLD USAGE PATTERNS
// ============================================================================

#[test]
fn test_parse_port_with_error_conversion() {
    fn parse_port(input: &str) -> SongbirdResult<u16> {
        input.parse::<u16>().or_config_error("port")
    }

    assert!(parse_port("8080").is_ok());
    assert_eq!(parse_port("8080").expect("should parse valid input"), 8080);

    assert!(parse_port("invalid").is_err());
    assert!(parse_port("99999").is_err());
}

#[test]
fn test_env_var_with_error_conversion() {
    fn get_required_env(key: &str) -> SongbirdResult<String> {
        std::env::var(key).or_config_error(key)
    }

    // This will fail unless the var is set
    let result = get_required_env("NONEXISTENT_TEST_VAR_12345");
    assert!(result.is_err());

    match result.expect_err("testing error case") {
        SongbirdError::Configuration {
            field,
            ..
        } => {
            assert_eq!(field, Some("NONEXISTENT_TEST_VAR_12345".to_string()));
        }
        _ => panic!("Expected Configuration error"),
    }
}

#[test]
fn test_network_operation_with_error_conversion() {
    fn connect_to_service() -> SongbirdResult<()> {
        // Simulate a network operation that fails
        let result: Result<(), &str> = Err("connection refused");
        result.or_network_error("service discovery")
    }

    let result = connect_to_service();
    assert!(result.is_err());

    match result.expect_err("testing error case") {
        SongbirdError::Network {
            message,
            ..
        } => {
            assert!(message.contains("connection refused"));
            assert!(message.contains("service discovery"));
        }
        _ => panic!("Expected Network error"),
    }
}

#[test]
fn test_service_call_with_error_conversion() {
    fn call_storage_service() -> SongbirdResult<Vec<u8>> {
        // Simulate service call that fails
        let result: Result<Vec<u8>, &str> = Err("service timeout");
        result.or_service_error("storage-api")
    }

    let result = call_storage_service();
    assert!(result.is_err());

    match result.expect_err("testing error case") {
        SongbirdError::Service {
            service,
            message,
            ..
        } => {
            assert_eq!(service, "storage-api");
            assert!(message.contains("timeout"));
        }
        _ => panic!("Expected Service error"),
    }
}

// ============================================================================
// ERROR TYPE EDGE CASES
// ============================================================================

#[test]
fn test_empty_error_message() {
    let result: Result<(), &str> = Err("");
    let songbird_result = result.or_config_error("field");

    assert!(songbird_result.is_err());
    // Should handle empty error message gracefully
}

#[test]
fn test_very_long_error_message() {
    let long_msg = "A".repeat(10000);
    let result: Result<(), String> = Err(long_msg.clone());
    let songbird_result = result.or_network_error("context");

    assert!(songbird_result.is_err());
    match songbird_result.expect_err("testing error case") {
        SongbirdError::Network {
            message,
            ..
        } => {
            assert!(message.contains(&long_msg));
        }
        _ => panic!("Expected Network error"),
    }
}

#[test]
fn test_unicode_in_errors() {
    let result: Result<(), &str> = Err("错误 🔥 エラー");
    let songbird_result = result.or_service_error("multilang-service");

    assert!(songbird_result.is_err());
    match songbird_result.expect_err("testing error case") {
        SongbirdError::Service {
            message,
            ..
        } => {
            assert!(message.contains("错误"));
            assert!(message.contains("🔥"));
            assert!(message.contains("エラー"));
        }
        _ => panic!("Expected Service error"),
    }
}

// ============================================================================
// MULTIPLE CONVERSIONS IN SAME FUNCTION
// ============================================================================

#[test]
fn test_multiple_or_errors_in_function() {
    fn complex_operation() -> SongbirdResult<()> {
        // Might fail with config error
        std::env::var("REQUIRED_VAR").or_config_error("REQUIRED_VAR")?;

        // Might fail with network error
        let result: Result<(), &str> = Ok(());
        result.or_network_error("connectivity check")?;

        // Might fail with service error
        let result: Result<(), &str> = Ok(());
        result.or_service_error("backend")?;

        Ok(())
    }

    // Should compile and run without issues
    let result = complex_operation();
    assert!(result.is_err()); // Will fail on REQUIRED_VAR
}

// ============================================================================
// TYPE INFERENCE TESTS
// ============================================================================

#[test]
fn test_type_inference_with_different_ok_types() {
    // Should work with various Ok types
    let int_result: Result<i32, &str> = Ok(42);
    assert!(int_result.or_config_error("int_field").is_ok());

    let string_result: Result<String, &str> = Ok("test".to_string());
    assert!(string_result.or_config_error("string_field").is_ok());

    let vec_result: Result<Vec<u8>, &str> = Ok(vec![1, 2, 3]);
    assert!(vec_result.or_config_error("vec_field").is_ok());
}

#[test]
fn test_type_inference_with_different_err_types() {
    // Should work with various error types
    let str_err: Result<(), &str> = Err("error");
    assert!(str_err.or_config_error("field").is_err());

    let string_err: Result<(), String> = Err("error".to_string());
    assert!(string_err.or_config_error("field").is_err());

    let io_err: Result<(), std::io::Error> = Err(std::io::Error::other("error"));
    assert!(io_err.or_config_error("field").is_err());
}

// ============================================================================
// ERGONOMICS TESTS
// ============================================================================

#[test]
fn test_ergonomic_question_mark_operator() {
    fn ergonomic_function() -> SongbirdResult<String> {
        let var = std::env::var("PATH").or_config_error("PATH")?;
        Ok(var)
    }

    // Should compile and work with ? operator
    let result = ergonomic_function();
    // PATH usually exists, so this should pass
    let _ = result; // May succeed or fail depending on environment
}

#[test]
fn test_method_chaining() {
    fn chained_operations() -> SongbirdResult<u16> {
        "8080".parse::<u16>().or_config_error("port")
    }

    assert_eq!(chained_operations().expect("test precondition"), 8080);
}
