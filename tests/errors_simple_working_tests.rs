//! Simple Working Error Tests for Songbird Orchestrator
//!
//! This test suite covers basic error types using the actual SongbirdError structure.

use songbird_errors::{Result, SongbirdError};

#[tokio::test]
async fn test_error_creation_and_display() -> Result<()> {
    // Test communication error
    let comm_error = SongbirdError::Communication("Connection failed".to_string());
    let error_msg = format!("Communication error: {comm_error}");
    assert!(error_msg.contains("Connection failed"));

    // Test circuit breaker errors
    let open_error = SongbirdError::CircuitBreakerOpen {
        service: "test-service".to_string(),
        message: "Circuit breaker is open".to_string(),
    };

    let failure_error = SongbirdError::CircuitBreakerFailure {
        service: "test-service".to_string(),
        message: "Circuit breaker failed".to_string(),
    };

    // Test retry exhausted error
    let retry_error = SongbirdError::RetryExhausted {
        attempts: 3,
        message: "Connection failed".to_string(),
    };

    // Test discovery error
    let discovery_error = SongbirdError::Discovery {
        message: "Service not found".to_string(),
        service: Some("test-service".to_string()),
    };

    // Test network error
    let network_error = SongbirdError::Network {
        service: Some("test-service".to_string()),
        message: "Connection timeout".to_string(),
        details: Some("After 30 seconds".to_string()),
    };

    // Test configuration error
    let config_error = SongbirdError::Config {
        message: "Invalid port".to_string(),
        field: Some("port".to_string()),
    };

    // Test gaming error
    let gaming_error = SongbirdError::Gaming {
        message: "Protocol mismatch".to_string(),
        protocol: Some("UDP".to_string()),
    };

    // Test validation error
    let validation_error = SongbirdError::Validation {
        field: "username".to_string(),
        message: "Too short".to_string(),
    };

    // Test security error
    let security_error = SongbirdError::Security {
        message: "Access denied".to_string(),
        context: Some("admin".to_string()),
    };

    // Test authentication error
    let auth_error = SongbirdError::Authentication {
        provider: "oauth".to_string(),
        message: "Invalid token".to_string(),
    };

    // Test that all errors display properly
    let errors = vec![
        comm_error,
        open_error,
        failure_error,
        retry_error,
        discovery_error,
        network_error,
        config_error,
        gaming_error,
        validation_error,
        security_error,
        auth_error,
    ];

    for error in errors {
        let display_str = error.to_string();
        assert!(!display_str.is_empty());
    }

    Ok(())
}

#[tokio::test]
async fn test_error_conversion() -> Result<()> {
    // Test From implementations
    let str_error: SongbirdError = "test error".into();
    assert!(str_error.to_string().contains("test error"));

    let string_error: SongbirdError = "test error".to_string().into();
    assert!(string_error.to_string().contains("test error"));

    Ok(())
}

#[tokio::test]
async fn test_error_constructors() -> Result<()> {
    // Test convenience constructors
    let service_error = SongbirdError::service_error("test-service", "Test failed".to_string());
    assert!(service_error.to_string().contains("test-service"));
    assert!(service_error.to_string().contains("Test failed"));

    let health_error = SongbirdError::health_check_failed("db-service", "Timeout");
    assert!(health_error.to_string().contains("db-service"));
    assert!(health_error.to_string().contains("Health check failed"));

    let config_error = SongbirdError::configuration_error("Invalid config".to_string());
    assert!(config_error.to_string().contains("Invalid config"));

    Ok(())
}

#[tokio::test]
async fn test_error_fields() -> Result<()> {
    // Test errors with optional fields
    let discovery_with_service = SongbirdError::Discovery {
        message: "Not found".to_string(),
        service: Some("api".to_string()),
    };
    assert!(discovery_with_service.to_string().contains("api"));

    let discovery_without_service = SongbirdError::Discovery {
        message: "Not found".to_string(),
        service: None,
    };
    assert!(discovery_without_service.to_string().contains("Not found"));

    let network_with_details = SongbirdError::Network {
        service: Some("web".to_string()),
        message: "Failed".to_string(),
        details: Some("Connection refused".to_string()),
    };
    assert!(network_with_details
        .to_string()
        .contains("Connection refused"));

    let network_without_details = SongbirdError::Network {
        service: Some("web".to_string()),
        message: "Failed".to_string(),
        details: None,
    };
    assert!(!network_without_details
        .to_string()
        .contains("Connection refused"));

    Ok(())
}
