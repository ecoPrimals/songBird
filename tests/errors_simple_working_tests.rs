//! Simple Working Error Tests for Songbird Orchestrator
//!
//! This test suite covers basic error types using the actual SongbirdError structure.

use songbird_lib::errors::{Result, SongbirdError};
use songbird_lib::errors::validation::ConfigValidator;

#[test]
fn test_config_error() {
    let error = SongbirdError::Config {
        field: Some("port".to_string()),
        message: "Invalid port number".to_string(),
    };
    
    assert!(matches!(error, SongbirdError::Config { .. }));
    let error_string = format!("{}", error);
    assert!(error_string.contains("Invalid port number"));
}

#[test]
fn test_network_error() {
    let error = SongbirdError::Network {
        service: "api".to_string(),
        message: "Connection failed".to_string(),
        details: Some("Timeout".to_string()),
    };
    
    assert!(matches!(error, SongbirdError::Network { .. }));
    let error_string = format!("{}", error);
    assert!(error_string.contains("Connection failed"));
}

#[test]
fn test_service_error() {
    let error = SongbirdError::Service {
        service: "database".to_string(),
        message: "Query failed".to_string(),
    };
    
    assert!(matches!(error, SongbirdError::Service { .. }));
    let error_string = format!("{}", error);
    assert!(error_string.contains("Query failed"));
}

#[test]
fn test_auth_error() {
    let error = SongbirdError::Auth {
        message: "Invalid credentials".to_string(),
        user: Some("testuser".to_string()),
    };
    
    assert!(matches!(error, SongbirdError::Auth { .. }));
    let error_string = format!("{}", error);
    assert!(error_string.contains("Invalid credentials"));
}

#[test]
fn test_io_error() {
    let error = SongbirdError::Io {
        message: "File not found".to_string(),
    };
    
    assert!(matches!(error, SongbirdError::Io { .. }));
    let error_string = format!("{}", error);
    assert!(error_string.contains("File not found"));
}

#[test]
fn test_validation_error() {
    let error = SongbirdError::Validation {
        field: "email".to_string(),
        message: "Invalid format".to_string(),
    };
    
    assert!(matches!(error, SongbirdError::Validation { .. }));
    let error_string = format!("{}", error);
    assert!(error_string.contains("Invalid format"));
}

#[test]
fn test_gaming_error() {
    let error = SongbirdError::Gaming {
        message: "Protocol error".to_string(),
        protocol: Some("TCP".to_string()),
    };
    
    assert!(matches!(error, SongbirdError::Gaming { .. }));
    let error_string = format!("{}", error);
    assert!(error_string.contains("Protocol error"));
}

#[test]
fn test_security_error() {
    let error = SongbirdError::Security {
        message: "Access denied".to_string(),
        context: Some("admin".to_string()),
    };
    
    assert!(matches!(error, SongbirdError::Security { .. }));
    let error_string = format!("{}", error);
    assert!(error_string.contains("Access denied"));
}

#[test]
fn test_discovery_error() {
    let error = SongbirdError::Discovery {
        message: "Service not found".to_string(),
        service: Some("api".to_string()),
    };
    
    assert!(matches!(error, SongbirdError::Discovery { .. }));
    let error_string = format!("{}", error);
    assert!(error_string.contains("Service not found"));
}

#[test]
fn test_circuit_breaker_errors() {
    let open_error = SongbirdError::CircuitBreakerOpen {
        message: "Circuit is open".to_string(),
    };
    
    let failure_error = SongbirdError::CircuitBreakerFailure {
        message: "Circuit failure".to_string(),
    };
    
    assert!(matches!(open_error, SongbirdError::CircuitBreakerOpen { .. }));
    assert!(matches!(failure_error, SongbirdError::CircuitBreakerFailure { .. }));
}

#[test]
fn test_retry_exhausted_error() {
    let error = SongbirdError::RetryExhausted {
        attempts: 3,
        last_error: "Connection failed".to_string(),
    };
    
    assert!(matches!(error, SongbirdError::RetryExhausted { .. }));
    let error_string = format!("{}", error);
    assert!(error_string.contains("3 attempts"));
}

#[test]
fn test_rate_limit_error() {
    let error = SongbirdError::RateLimitExceeded {
        message: "Too many requests".to_string(),
    };
    
    assert!(matches!(error, SongbirdError::RateLimitExceeded { .. }));
    let error_string = format!("{}", error);
    assert!(error_string.contains("Too many requests"));
}

#[test]
fn test_result_type() {
    let success: Result<String> = Ok("Success".to_string());
    assert!(success.is_ok());
    
    let failure: Result<String> = Err(SongbirdError::Config {
        field: None,
        message: "Test error".to_string(),
    });
    assert!(failure.is_err());
}

#[test]
fn test_error_from_io() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let songbird_error: SongbirdError = io_error.into();
    assert!(matches!(songbird_error, SongbirdError::Io { .. }));
}

#[test]
fn test_error_from_string() {
    let error: SongbirdError = "Test error".into();
    assert!(matches!(error, SongbirdError::Protocol { .. }));
}

#[test]
fn test_config_validator_port() {
    assert!(ConfigValidator::validate_port(8080, "test_port").is_ok());
    assert!(ConfigValidator::validate_port(0, "test_port").is_err());
}

#[test]
fn test_config_validator_port_range() {
    assert!(ConfigValidator::validate_port_range(8000, 9000).is_ok());
    assert!(ConfigValidator::validate_port_range(9000, 8000).is_err());
}

#[test]
fn test_config_validator_url() {
    assert!(ConfigValidator::validate_url("http://example.com", "test_url").is_ok());
    assert!(ConfigValidator::validate_url("not-a-url", "test_url").is_err());
}

#[test]
fn test_config_validator_timeout() {
    assert!(ConfigValidator::validate_connection_timeout(5000).is_ok());
    assert!(ConfigValidator::validate_connection_timeout(50).is_err());
}

#[test]
fn test_error_helper_methods() {
    let service_error = SongbirdError::service_error("test-service", "Test error".to_string());
    assert!(matches!(service_error, SongbirdError::Service { .. }));
    
    let config_error = SongbirdError::configuration_error("Invalid config".to_string());
    assert!(matches!(config_error, SongbirdError::Config { .. }));
}

#[test]
fn test_error_cloning() {
    let original = SongbirdError::Config {
        field: Some("test".to_string()),
        message: "Test error".to_string(),
    };
    
    let cloned = original.clone();
    assert_eq!(format!("{}", original), format!("{}", cloned));
}

#[test]
fn test_error_debug() {
    let error = SongbirdError::Config {
        field: Some("port".to_string()),
        message: "Invalid port".to_string(),
    };
    
    let debug_string = format!("{:?}", error);
    assert!(debug_string.contains("Config"));
    assert!(debug_string.contains("port"));
} 