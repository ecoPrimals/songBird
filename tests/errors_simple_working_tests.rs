//! Simple Working Error Tests for Songbird Orchestrator
//!
//! This test suite covers basic error types using the actual SongbirdError structure.

use songbird_errors::{
    CircuitBreakerError, DiscoveryError, GamingError, NetworkError, Result, RetryError,
    ServiceError, SongbirdError, ValidationError,
, SongbirdError};

#[tokio::test]
async fn test_error_creation_and_display() -> Result<()> {
    // Test communication error
    let comm_error = SongbirdError::Communication("Connection failed".to_string());
    let error_msg = format!("Communication error: {comm_error}");
    assert!(error_msg.contains("Connection failed"));

    // Test circuit breaker errors
    let open_error = SongbirdError::CircuitBreakerOpen(Box::new(CircuitBreakerError {
        service: "test-service".to_string(),
        message: "Circuit breaker is open for test-service".to_string(),
    }));

    let failure_error = SongbirdError::CircuitBreakerFailure {
        service: "test-service".to_string(),
        message: "Circuit breaker failed for test-service".to_string(),
        suggestion: Some("Check service health and restart if needed".to_string()),
    };

    // Test retry exhausted error
    let retry_error = SongbirdError::RetryExhausted(Box::new(RetryError {
        message: "Connection failed".to_string(),
        attempts: Some(3),
    }));

    // Test discovery error
    let discovery_error = SongbirdError::Discovery(Box::new(DiscoveryError {
        message: "Service not found".to_string(),
        service: Some("test-service".to_string()),
        timeout: Some(30),
        suggestion: Some("Check service configuration and network connectivity".to_string()),
    }));

    // Test network error - using actual fields
    let network_error = SongbirdError::Network(Box::new(NetworkError {
        message: "Connection timeout".to_string(),
        endpoint: Some("192.168.1.100:{}".to_string()),
        port: Some(8080),
        protocol: Some("HTTP".to_string()),
    }));

    // Test network error (alternative)
    let network_error2 = SongbirdError::Network(Box::new(NetworkError {
        message: "Service unavailable".to_string(),
        endpoint: Some("service.example.com".to_string()),
        port: Some(443),
        protocol: Some("HTTPS".to_string()),
    }));

    // Test gaming error - using actual fields
    let gaming_error = SongbirdError::Gaming(Box::new(GamingError {
        message: "Protocol mismatch".to_string(),
        game: Some("StarCraft".to_string()),
    }));

    // Test validation error - using actual fields
    let validation_error = SongbirdError::Validation(Box::new(ValidationError {
        message: "Username too short".to_string(),
        field: Some("username".to_string()),
        expected: Some("minimum 3 characters".to_string()),
    }));

    // Test security error
    let security_error = SongbirdError::Security {
        message: "Access denied".to_string(),
        context: Some("admin".to_string()),
        severity: Some("high".to_string()),
        suggestion: Some("Review security configuration and apply recommended fixes".to_string()),
    };

    // Test authentication error
    let auth_error = SongbirdError::Authentication {
        provider: "oauth".to_string(),
        message: "Invalid token".to_string(),
        suggestion: Some("Check token validity and refresh if needed".to_string()),
    };

    // Test that all errors display properly
    let errors = vec![
        comm_error,
        open_error,
        failure_error,
        retry_error,
        discovery_error,
        network_error,
        network_error2,
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

    let health_error = SongbirdError::Service(Box::new(ServiceError {
        service: "db-service".to_string(),
        message: "Health check failed: Timeout".to_string(),
        status: Some("unhealthy".to_string()),
        suggestion: Some("Check service connectivity and response time".to_string()),
    }));
    assert!(health_error.to_string().contains("db-service"));
    assert!(health_error.to_string().contains("Health check failed"));

            let config_error = SongbirdError::configuration_error("Invalid config");
    assert!(config_error.to_string().contains("Invalid config"));

    Ok(())
}

#[tokio::test]
async fn test_error_fields() -> Result<()> {
    // Test errors with optional fields
    let discovery_with_service = SongbirdError::Discovery(Box::new(DiscoveryError {
        message: "Not found".to_string(),
        service: Some("api".to_string()),
        timeout: None,
        suggestion: None,
    }));
    assert!(discovery_with_service.to_string().contains("api"));

    let discovery_without_service = SongbirdError::Discovery(Box::new(DiscoveryError {
        message: "Not found".to_string(),
        service: None,
        timeout: None,
        suggestion: None,
    }));
    assert!(discovery_without_service.to_string().contains("Not found"));

    let network_with_details = SongbirdError::Network(Box::new(NetworkError {
        message: "Failed - Connection refused".to_string(),
        endpoint: Some("web.example.com:80".to_string()),
        port: Some(80),
        protocol: Some("HTTP".to_string()),
    }));
    assert!(network_with_details
        .to_string()
        .contains("Connection refused"));

    let network_without_details = SongbirdError::Network(Box::new(NetworkError {
        message: "Failed".to_string(),
        endpoint: Some("web.example.com".to_string()),
        port: None,
        protocol: None,
    }));
    assert!(!network_without_details
        .to_string()
        .contains("Connection refused"));

    Ok(())
}
