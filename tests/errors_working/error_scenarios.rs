use CanonicalSongbirdConfig;
//! Complex error scenarios and comprehensive async tests

use std::time::Duration;
use songbird_errors::{
    AuthError, CircuitBreakerError, DeploymentError, DiscoveryError, ExecutionError, GamingError,
    IoError, NetworkError, NotFoundError, PluginNotFoundError, ProtocolError, RateLimitError,
    ResourceExhaustedError, Result, RetryError, ServiceError, SongbirdError, ValidationError,
, SongbirdError};

#[tokio::test]
async fn test_comprehensive_error_scenarios() -> Result<()> {
    // Test RetryExhausted Error with specific attempts
    let retry_error = SongbirdError::RetryExhausted(Box::new(RetryError {
        attempts: 3,
        message: "Connection failed".to_string(),
        duration: Some("5s".to_string()),
        suggestion: Some("Check network connectivity and try again".to_string()),
    }));

    let retry_msg = format!("Retry error: {retry_error}");
    assert!(retry_msg.contains("3"));
    assert!(retry_msg.contains("Connection failed"));

    // Test Rate Limit Exceeded Error
    let rate_error = SongbirdError::RateLimitExceeded(Box::new(RateLimitError {
        message: "Too many requests".to_string(),
        service: Some("api-service".to_string()),
        limit: Some(100),
        suggestion: Some("Wait before retrying".to_string()),
    }));
    let rate_msg = format!("Rate limit error: {rate_error}");
    assert!(rate_msg.contains("Too many requests"));

    // Test Execution Failed Error
    let exec_error = SongbirdError::ExecutionFailed(Box::new(ExecutionError {
        message: "Command failed".to_string(),
        command: Some("test-command".to_string()),
        exit_code: Some(1),
        suggestion: Some("Check command syntax".to_string()),
    }));
    let exec_msg = format!("Execution error: {exec_error}");
    assert!(exec_msg.contains("Command failed"));

    // Test Resource Exhausted Error
    let resource_error = SongbirdError::ResourceExhausted(Box::new(ResourceExhaustedError {
        resource: "memory".to_string(),
        message: "Out of memory".to_string(),
        current_usage: Some("8GB".to_string()),
        limit: Some("8GB".to_string()),
        suggestion: Some("Increase memory allocation".to_string()),
    }));
    let resource_msg = format!("Resource error: {resource_error}");
    assert!(resource_msg.contains("memory"));
    assert!(resource_msg.contains("Out of memory"));

    // Test Network Error with Details
    let network_detailed = SongbirdError::Network(Box::new(NetworkError {
        service: Some("api-service".to_string()),
        message: "Connection failed".to_string(),
        details: Some("Timeout after 30 seconds".to_string()),
        endpoint: Some("https://api.example.com".to_string()),
        suggestion: Some("Check network connectivity".to_string()),
    }));
    let network_msg = format!("Network error: {network_detailed}");
    assert!(network_msg.contains("api-service"));
    assert!(network_msg.contains("Connection failed"));
    assert!(network_msg.contains("Timeout after 30 seconds"));

    // Test Discovery Error
    let discovery_error = SongbirdError::Discovery(Box::new(DiscoveryError {
        message: "Service not found".to_string(),
        service: Some("user-service".to_string()),
        timeout: Some(5000),
        suggestion: Some("Check service registry".to_string()),
    }));
    let discovery_msg = format!("Discovery error: {discovery_error}");
    assert!(discovery_msg.contains("Service not found"));
    assert!(discovery_msg.contains("user-service"));

    // Test Authentication Error
    let auth_error = SongbirdError::Authentication {
        provider: "oauth2".to_string(),
        message: "Invalid token".to_string(),
        suggestion: Some("Refresh your authentication token".to_string()),
    };
    let auth_msg = format!("Auth error: {auth_error}");
    assert!(auth_msg.contains("oauth2"));
    assert!(auth_msg.contains("Invalid token"));

    // Test Gaming Error
    let gaming_error = SongbirdError::Gaming(Box::new(GamingError {
        message: "Protocol mismatch".to_string(),
        protocol: Some("UDP".to_string()),
        game: Some("test-game".to_string()),
        suggestion: Some("Check protocol compatibility".to_string()),
    }));
    let gaming_msg = format!("Gaming error: {gaming_error}");
    assert!(gaming_msg.contains("Protocol mismatch"));

    // Test Plugin Not Found Error
    let plugin_error = SongbirdError::PluginNotFound(Box::new(PluginNotFoundError {
        message: "Plugin not found".to_string(),
        name: Some("test-plugin".to_string()),
        paths: Some(vec!["/plugins/test-plugin".to_string()]),
        suggestion: Some("Install the required plugin".to_string()),
    }));
    let plugin_msg = format!("Plugin error: {plugin_error}");
    assert!(plugin_msg.contains("Plugin not found"));

    // Test Not Found Error
    let not_found_error = SongbirdError::NotFound(Box::new(NotFoundError {
        resource: "user".to_string(),
        message: "User not found".to_string(),
        searched_paths: Some(vec!["/users/123".to_string()]),
        suggestion: Some("Check user ID".to_string()),
    }));
    let not_found_msg = format!("Not found error: {not_found_error}");
    assert!(not_found_msg.contains("user"));

    Ok(())
}

#[tokio::test]
async fn test_error_conversion_scenarios() -> Result<()> {
    // Test converting from standard io::Error
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let songbird_err: SongbirdError = io_err.into();
    assert!(matches!(songbird_err, SongbirdError::Io(_)));

    // Test error message formatting
    let error = SongbirdError::config_error("test", "Invalid configuration");
    let msg = format!("{error}");
    assert!(msg.contains("Invalid configuration"));

    Ok(())
}

#[tokio::test]
async fn test_error_helper_functions() -> Result<()> {
    // Test various error helper functions
    let config_err = SongbirdError::config_error("database", "Invalid connection string");
    assert!(matches!(config_err, SongbirdError::Config { .. }));

    let network_err = SongbirdError::network_error("Connection timeout");
    assert!(matches!(network_err, SongbirdError::Network(_)));

    let auth_err = SongbirdError::authentication_error("oauth2", "Token expired");
    assert!(matches!(auth_err, SongbirdError::Authentication { .. }));

    Ok(())
}

#[tokio::test]
async fn test_error_debug_and_clone() -> Result<()> {
    let error = SongbirdError::config_error("test", "Test error");
    
    // Test Debug trait
    let debug_str = format!("{:?}", error);
    assert!(!debug_str.is_empty());

    // Test Clone trait
    let cloned_error = error.clone();
    assert_eq!(format!("{error}"), format!("{cloned_error}"));

    Ok(())
}

#[tokio::test]
async fn test_complex_error_scenarios() -> Result<()> {
    // Test error chaining and complex scenarios
    let network_error = SongbirdError::Network(Box::new(NetworkError {
        service: Some("database".to_string()),
        message: "Connection failed".to_string(),
        details: Some("Timeout after 30 seconds".to_string()),
        endpoint: Some("db.example.com:config.database.postgres_port".to_string()),
        suggestion: Some("Check database connectivity".to_string()),
    }));

    // Test error conversion and display
    let error_msg = format!("{network_error}");
    assert!(error_msg.contains("Connection failed"));
    assert!(error_msg.contains("database"));

    // Test with circuit breaker error
    let cb_error = SongbirdError::CircuitBreakerOpen(Box::new(CircuitBreakerError {
        service: "api".to_string(),
        message: "Circuit breaker is open".to_string(),
        failure_count: Some(5),
        failure_threshold: Some(3),
        timeout_duration: Some(Duration::from_secs(30)),
        suggestion: Some("Wait for circuit breaker to close".to_string()),
    }));

    let cb_msg = format!("{cb_error}");
    assert!(cb_msg.contains("Circuit breaker is open"));

    Ok(())
}

#[tokio::test]
async fn test_error_trait_implementations() -> Result<()> {
    let error = SongbirdError::config_error("test", "Test error");
    
    // Test Send trait
    let _: Box<dyn Send> = Box::new(error.clone());
    
    // Test Sync trait  
    let _: Box<dyn Sync> = Box::new(error.clone());

    // Test Error trait
    use std::error::Error;
    let _: Box<dyn Error> = Box::new(error);

    Ok(())
} 