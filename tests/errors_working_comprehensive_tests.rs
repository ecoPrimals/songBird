//! Comprehensive Error Tests for Songbird Orchestrator
//!
//! This test suite covers error types, error handling, validation,
//! and error recovery mechanisms.

use std::time::Duration;

use songbird_lib::errors::{Result, SongbirdError};
use songbird_lib::errors::validation::ConfigValidator;

#[tokio::test]
async fn test_comprehensive_error_scenarios() -> Result<()> {
    // Test RetryExhausted Error with specific attempts
    let retry_error = SongbirdError::RetryExhausted {
        attempts: 3,
        last_error: "Connection failed".to_string(),
    };

    let retry_msg = format!("Retry error: {}", retry_error);
    assert!(retry_msg.contains("3"));
    assert!(retry_msg.contains("Connection failed"));

    // Test Rate Limit Exceeded Error
    let rate_error = SongbirdError::RateLimitExceeded("Too many requests".to_string());
    let rate_msg = format!("Rate limit error: {}", rate_error);
    assert!(rate_msg.contains("Too many requests"));

    // Test Execution Failed Error
    let exec_error = SongbirdError::ExecutionFailed("Command failed".to_string());
    let exec_msg = format!("Execution error: {}", exec_error);
    assert!(exec_msg.contains("Command failed"));

    // Test Resource Exhausted Error
    let resource_error = SongbirdError::ResourceExhausted {
        resource: "memory".to_string(),
        message: "Out of memory".to_string(),
    };
    let resource_msg = format!("Resource error: {}", resource_error);
    assert!(resource_msg.contains("memory"));
    assert!(resource_msg.contains("Out of memory"));

    // Test Network Error with Details
    let network_detailed = SongbirdError::Network {
        service: Some("api-service".to_string()),
        message: "Connection failed".to_string(),
        details: Some("Timeout after 30 seconds".to_string()),
    };
    let network_msg = format!("Network error: {}", network_detailed);
    assert!(network_msg.contains("api-service"));
    assert!(network_msg.contains("Connection failed"));
    assert!(network_msg.contains("Timeout after 30 seconds"));

    // Test Discovery Error
    let discovery_error = SongbirdError::Discovery {
        message: "Service not found".to_string(),
        service: Some("user-service".to_string()),
    };
    let discovery_msg = format!("Discovery error: {}", discovery_error);
    assert!(discovery_msg.contains("Service not found"));
    assert!(discovery_msg.contains("user-service"));

    // Test Authentication Error
    let auth_error = SongbirdError::Authentication {
        provider: "oauth2".to_string(),
        message: "Invalid token".to_string(),
    };
    let auth_msg = format!("Auth error: {}", auth_error);
    assert!(auth_msg.contains("oauth2"));
    assert!(auth_msg.contains("Invalid token"));

    // Test Gaming Error
    let gaming_error = SongbirdError::Gaming {
        message: "Protocol mismatch".to_string(),
        protocol: Some("UDP".to_string()),
    };
    let gaming_msg = format!("Gaming error: {}", gaming_error);
    assert!(gaming_msg.contains("Protocol mismatch"));
    assert!(gaming_msg.contains("UDP"));

    // Test Security Error
    let security_error = SongbirdError::Security {
        message: "Access denied".to_string(),
        context: Some("admin_panel".to_string()),
    };
    let security_msg = format!("Security error: {}", security_error);
    assert!(security_msg.contains("Access denied"));
    assert!(security_msg.contains("admin_panel"));

    // Test Configuration Error
    let config_error = SongbirdError::Config {
        message: "Invalid port number".to_string(),
        field: Some("port".to_string()),
    };
    let config_msg = format!("Config error: {}", config_error);
    assert!(config_msg.contains("Invalid port number"));
    assert!(config_msg.contains("port"));

    // Test Validation Error
    let validation_error = SongbirdError::Validation {
        field: "email".to_string(),
        message: "Invalid format".to_string(),
    };
    let validation_msg = format!("Validation error: {}", validation_error);
    assert!(validation_msg.contains("email"));
    assert!(validation_msg.contains("Invalid format"));

    // Test Service Error
    let service_error = SongbirdError::Service {
        service: "payment-service".to_string(),
        message: "Payment failed".to_string(),
    };
    let service_msg = format!("Service error: {}", service_error);
    assert!(service_msg.contains("payment-service"));
    assert!(service_msg.contains("Payment failed"));

    // Test IO Error
    let io_error = SongbirdError::Io {
        message: "File not found".to_string(),
    };
    let io_msg = format!("IO error: {}", io_error);
    assert!(io_msg.contains("File not found"));

    // Test Protocol Error
    let protocol_error = SongbirdError::Protocol {
        protocol: "HTTP".to_string(),
        message: "Invalid response".to_string(),
    };
    let protocol_msg = format!("Protocol error: {}", protocol_error);
    assert!(protocol_msg.contains("HTTP"));
    assert!(protocol_msg.contains("Invalid response"));

    // Test NotFound Error
    let not_found_error = SongbirdError::NotFound {
        resource: "user".to_string(),
        message: "User not found".to_string(),
    };
    let not_found_msg = format!("NotFound error: {}", not_found_error);
    assert!(not_found_msg.contains("user"));
    assert!(not_found_msg.contains("User not found"));

    // Test Deployment Error
    let deployment_error = SongbirdError::Deployment {
        service: "web-service".to_string(),
        message: "Deployment failed".to_string(),
    };
    let deployment_msg = format!("Deployment error: {}", deployment_error);
    assert!(deployment_msg.contains("web-service"));
    assert!(deployment_msg.contains("Deployment failed"));

    // Test TunnelCreation Error
    let tunnel_error = SongbirdError::TunnelCreation("VPN setup failed".to_string());
    let tunnel_msg = format!("Tunnel error: {}", tunnel_error);
    assert!(tunnel_msg.contains("VPN setup failed"));

    // Test Encryption/Decryption Errors
    let encryption_error = SongbirdError::EncryptionFailed("Key not found".to_string());
    let encryption_msg = format!("Encryption error: {}", encryption_error);
    assert!(encryption_msg.contains("Key not found"));

    let decryption_error = SongbirdError::DecryptionFailed("Invalid key".to_string());
    let decryption_msg = format!("Decryption error: {}", decryption_error);
    assert!(decryption_msg.contains("Invalid key"));

    // Test NetworkDetection Error
    let network_detection_error = SongbirdError::NetworkDetection("NAT type unknown".to_string());
    let network_detection_msg = format!("Network detection error: {}", network_detection_error);
    assert!(network_detection_msg.contains("NAT type unknown"));

    // Test UnsupportedChannelType Error
    let unsupported_error = SongbirdError::UnsupportedChannelType;
    let unsupported_msg = format!("Unsupported error: {}", unsupported_error);
    assert!(unsupported_msg.contains("Unsupported channel type"));

    // Test CompositionFailed Error
    let composition_error = SongbirdError::PluginComposition("Plugin incompatible".to_string());
    let composition_msg = format!("Composition error: {}", composition_error);
    assert!(composition_msg.contains("Plugin incompatible"));

    // Test PluginNotFound Error
    let plugin_error = SongbirdError::PluginNotFound("missing-plugin".to_string());
    let plugin_msg = format!("Plugin error: {}", plugin_error);
    assert!(plugin_msg.contains("missing-plugin"));

    Ok(())
}

#[tokio::test]
async fn test_error_conversion_scenarios() -> Result<()> {
    // Test From implementations
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let songbird_error: SongbirdError = io_error.into();
    assert!(songbird_error.to_string().contains("File not found"));

    let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let songbird_error: SongbirdError = json_error.into();
    assert!(songbird_error.to_string().contains("json_parser"));

    Ok(())
}

#[tokio::test]
async fn test_error_helper_functions() -> Result<()> {
    // Test helper functions
    let service_error = SongbirdError::service_error("test-service", "Test failed".to_string());
    assert!(service_error.to_string().contains("test-service"));
    assert!(service_error.to_string().contains("Test failed"));

    let health_error = SongbirdError::health_check_failed("db-service", "Timeout".to_string());
    assert!(health_error.to_string().contains("db-service"));
    assert!(health_error.to_string().contains("Health check failed"));

    let config_error = SongbirdError::configuration_error("Invalid config".to_string());
    assert!(config_error.to_string().contains("Invalid config"));

    Ok(())
}

#[tokio::test]
async fn test_error_debug_and_clone() -> Result<()> {
    let error = SongbirdError::Communication("Test error".to_string());

    // Test Debug formatting
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("Communication"));
    assert!(debug_str.contains("Test error"));

    // Test Clone
    let cloned = error.clone();
    assert_eq!(error.to_string(), cloned.to_string());

    Ok(())
}

#[tokio::test]
async fn test_complex_error_scenarios() -> Result<()> {
    // Test error chaining scenarios
    let primary_error = SongbirdError::Network {
        service: Some("auth-service".to_string()),
        message: "Connection timeout".to_string(),
        details: Some("After 3 retry attempts".to_string()),
    };

    let secondary_error = SongbirdError::CircuitBreakerOpen {
        message: "auth-service: Too many network failures".to_string(),
    };

    // Test that both errors contain expected information
    assert!(primary_error.to_string().contains("auth-service"));
    assert!(primary_error.to_string().contains("Connection timeout"));
    assert!(primary_error.to_string().contains("After 3 retry attempts"));

    assert!(secondary_error.to_string().contains("auth-service"));
    assert!(secondary_error
        .to_string()
        .contains("Too many network failures"));

    Ok(())
}

#[tokio::test]
async fn test_error_trait_implementations() -> Result<()> {
    let error = SongbirdError::Communication("Test error".to_string());

    // Test Error trait
    let error_trait: &dyn std::error::Error = &error;
    assert!(!error_trait.to_string().is_empty());

    // Test Display trait
    let display_str = format!("{error}");
    assert!(display_str.contains("Test error"));

    // Test Debug trait
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("Communication"));

    Ok(())
}

#[test]
fn test_songbird_error_config() {
    let config_error = SongbirdError::Config {
        field: Some("port".to_string()),
        message: "Invalid port number".to_string(),
    };

    assert!(matches!(config_error, SongbirdError::Config { .. }));

    let error_string = format!("{config_error}");
    assert!(error_string.contains("Invalid port number"));
    assert!(error_string.contains("port"));
}

#[test]
fn test_songbird_error_configuration() {
    let config_error = SongbirdError::Configuration {
        field: "database_url".to_string(),
        message: "Invalid database URL".to_string(),
    };

    assert!(matches!(config_error, SongbirdError::Configuration { .. }));

    let error_string = format!("{config_error}");
    assert!(error_string.contains("Invalid database URL"));
    assert!(error_string.contains("database_url"));
}

#[test]
fn test_songbird_error_network() {
    let network_error = SongbirdError::Network {
        service: Some("api".to_string()),
        message: "Connection timeout".to_string(),
        details: Some("Host unreachable".to_string()),
    };

    assert!(matches!(network_error, SongbirdError::Network { .. }));

    let error_string = format!("{network_error}");
    assert!(error_string.contains("Connection timeout"));
    assert!(error_string.contains("api"));
    assert!(error_string.contains("Host unreachable"));
}

#[test]
fn test_songbird_error_communication() {
    let comm_error = SongbirdError::Communication("Message delivery failed".to_string());

    assert!(matches!(comm_error, SongbirdError::Communication(_)));

    let error_string = format!("{comm_error}");
    assert!(error_string.contains("Message delivery failed"));
}

#[test]
fn test_songbird_error_discovery() {
    let discovery_error = SongbirdError::Discovery {
        message: "Service not found".to_string(),
        service: Some("user-service".to_string()),
    };

    assert!(matches!(discovery_error, SongbirdError::Discovery { .. }));

    let error_string = format!("{discovery_error}");
    assert!(error_string.contains("Service not found"));
    assert!(error_string.contains("user-service"));
}

#[test]
fn test_songbird_error_auth() {
    let auth_error = SongbirdError::Auth {
        message: "Invalid credentials".to_string(),
        user: Some("john_doe".to_string()),
    };

    assert!(matches!(auth_error, SongbirdError::Auth { .. }));

    let error_string = format!("{auth_error}");
    assert!(error_string.contains("Invalid credentials"));
    assert!(error_string.contains("john_doe"));
}

#[test]
fn test_songbird_error_authentication() {
    let auth_error = SongbirdError::Authentication {
        provider: "oauth2".to_string(),
        message: "Token expired".to_string(),
    };

    assert!(matches!(auth_error, SongbirdError::Authentication { .. }));

    let error_string = format!("{auth_error}");
    assert!(error_string.contains("Token expired"));
    assert!(error_string.contains("oauth2"));
}

#[test]
fn test_songbird_error_gaming() {
    let gaming_error = SongbirdError::Gaming {
        message: "Protocol version mismatch".to_string(),
        protocol: Some("starcraft".to_string()),
    };

    assert!(matches!(gaming_error, SongbirdError::Gaming { .. }));

    let error_string = format!("{gaming_error}");
    assert!(error_string.contains("Protocol version mismatch"));
    assert!(error_string.contains("starcraft"));
}

#[test]
fn test_songbird_error_security() {
    let security_error = SongbirdError::Security {
        message: "Encryption failed".to_string(),
        context: Some("data_transmission".to_string()),
    };

    assert!(matches!(security_error, SongbirdError::Security { .. }));

    let error_string = format!("{security_error}");
    assert!(error_string.contains("Encryption failed"));
    assert!(error_string.contains("data_transmission"));
}

#[test]
fn test_songbird_error_protocol() {
    let protocol_error = SongbirdError::Protocol {
        protocol: "http".to_string(),
        message: "Invalid request format".to_string(),
    };

    assert!(matches!(protocol_error, SongbirdError::Protocol { .. }));

    let error_string = format!("{protocol_error}");
    assert!(error_string.contains("Invalid request format"));
    assert!(error_string.contains("http"));
}

#[test]
fn test_songbird_error_service() {
    let service_error = SongbirdError::Service {
        service: "database".to_string(),
        message: "Connection pool exhausted".to_string(),
    };

    assert!(matches!(service_error, SongbirdError::Service { .. }));

    let error_string = format!("{service_error}");
    assert!(error_string.contains("Connection pool exhausted"));
    assert!(error_string.contains("database"));
}

#[test]
fn test_songbird_error_validation() {
    let validation_error = SongbirdError::Validation {
        field: "email".to_string(),
        message: "Invalid email format".to_string(),
    };

    assert!(matches!(validation_error, SongbirdError::Validation { .. }));

    let error_string = format!("{validation_error}");
    assert!(error_string.contains("Invalid email format"));
    assert!(error_string.contains("email"));
}

#[test]
fn test_songbird_error_not_found() {
    let not_found_error = SongbirdError::NotFound {
        resource: "user".to_string(),
        message: "User not found".to_string(),
    };

    assert!(matches!(not_found_error, SongbirdError::NotFound { .. }));

    let error_string = format!("{not_found_error}");
    assert!(error_string.contains("User not found"));
    assert!(error_string.contains("user"));
}

#[test]
fn test_songbird_error_io() {
    let io_error = SongbirdError::Io {
        message: "File not found".to_string(),
    };

    assert!(matches!(io_error, SongbirdError::Io { .. }));

    let error_string = format!("{io_error}");
    assert!(error_string.contains("File not found"));
}

#[test]
fn test_songbird_error_load_balancer() {
    let lb_error = SongbirdError::LoadBalancer {
        message: "No healthy backends available".to_string(),
    };

    assert!(matches!(lb_error, SongbirdError::LoadBalancer { .. }));

    let error_string = format!("{lb_error}");
    assert!(error_string.contains("No healthy backends available"));
}

#[test]
fn test_songbird_error_tunnel_creation() {
    let tunnel_error = SongbirdError::TunnelCreation("Failed to establish tunnel".to_string());

    assert!(matches!(tunnel_error, SongbirdError::TunnelCreation(_)));

    let error_string = format!("{tunnel_error}");
    assert!(error_string.contains("Failed to establish tunnel"));
}

#[test]
fn test_songbird_error_encryption_failed() {
    let encryption_error = SongbirdError::EncryptionFailed("Invalid key".to_string());

    assert!(matches!(
        encryption_error,
        SongbirdError::EncryptionFailed(_)
    ));

    let error_string = format!("{encryption_error}");
    assert!(error_string.contains("Invalid key"));
}

#[test]
fn test_songbird_error_decryption_failed() {
    let decryption_error = SongbirdError::DecryptionFailed("Corrupted data".to_string());

    assert!(matches!(
        decryption_error,
        SongbirdError::DecryptionFailed(_)
    ));

    let error_string = format!("{decryption_error}");
    assert!(error_string.contains("Corrupted data"));
}

#[test]
fn test_songbird_error_network_detection() {
    let network_detection_error =
        SongbirdError::NetworkDetection("Network interface not found".to_string());

    assert!(matches!(
        network_detection_error,
        SongbirdError::NetworkDetection(_)
    ));

    let error_string = format!("{network_detection_error}");
    assert!(error_string.contains("Network interface not found"));
}

#[test]
fn test_songbird_error_unsupported_channel_type() {
    let unsupported_error = SongbirdError::UnsupportedChannelType;

    assert!(matches!(
        unsupported_error,
        SongbirdError::UnsupportedChannelType
    ));

    let error_string = format!("{unsupported_error}");
    assert!(error_string.contains("Unsupported channel type"));
}

#[test]
fn test_songbird_error_deployment() {
    let deployment_error = SongbirdError::Deployment {
        service: "web-app".to_string(),
        message: "Deployment failed".to_string(),
    };

    assert!(matches!(deployment_error, SongbirdError::Deployment { .. }));

    let error_string = format!("{deployment_error}");
    assert!(error_string.contains("Deployment failed"));
    assert!(error_string.contains("web-app"));
}

#[test]
fn test_songbird_error_circuit_breaker_open() {
    let cb_error = SongbirdError::CircuitBreakerOpen {
        service: "test-service".to_string(),
        message: "Circuit breaker is open".to_string(),
    };

    assert!(matches!(cb_error, SongbirdError::CircuitBreakerOpen { .. }));

    let error_string = format!("{cb_error}");
    assert!(error_string.contains("Circuit breaker is open"));
}

#[test]
fn test_songbird_error_circuit_breaker_failure() {
    let cb_failure_error = SongbirdError::CircuitBreakerFailure {
        message: "test-service: Circuit breaker failure".to_string(),
    };

    assert!(matches!(
        cb_failure_error,
        SongbirdError::CircuitBreakerFailure { .. }
    ));

    let error_string = format!("{cb_failure_error}");
    assert!(error_string.contains("Circuit breaker failure"));
}

#[test]
fn test_songbird_error_retry_exhausted() {
    let retry_error = SongbirdError::RetryExhausted {
        attempts: 3,
        message: "Connection failed".to_string(),
    };

    assert!(matches!(retry_error, SongbirdError::RetryExhausted { .. }));

    let error_string = format!("{retry_error}");
    assert!(error_string.contains("3"));
}

#[test]
fn test_songbird_error_rate_limit_exceeded() {
    let rate_limit_error = SongbirdError::RateLimitExceeded("Too many requests".to_string());

    assert!(matches!(
        rate_limit_error,
        SongbirdError::RateLimitExceeded(_)
    ));

    let error_string = format!("{rate_limit_error}");
    assert!(error_string.contains("Too many requests"));
}

#[test]
fn test_songbird_error_execution_failed() {
    let execution_error = SongbirdError::ExecutionFailed("Command execution failed".to_string());

    assert!(matches!(execution_error, SongbirdError::ExecutionFailed(_)));

    let error_string = format!("{execution_error}");
    assert!(error_string.contains("Command execution failed"));
}

#[test]
fn test_songbird_error_helper_methods() {
    let service_error = SongbirdError::service_error("auth", "Authentication failed".to_string());
    assert!(matches!(service_error, SongbirdError::Service { .. }));

    let health_error = SongbirdError::health_check_failed("database", "Connection timeout".to_string());
    assert!(matches!(health_error, SongbirdError::Service { .. }));

    let config_error = SongbirdError::configuration_error("Invalid configuration".to_string());
    assert!(matches!(config_error, SongbirdError::Config { .. }));
}

#[test]
fn test_songbird_error_from_str() {
    let error: SongbirdError = "Test error message".into();
    assert!(matches!(error, SongbirdError::Communication(_)));

    let error_string = format!("{error}");
    assert!(error_string.contains("Test error message"));
}

#[test]
fn test_songbird_error_from_string() {
    let error: SongbirdError = "Test error message".to_string().into();
    assert!(matches!(error, SongbirdError::Communication(_)));

    let error_string = format!("{error}");
    assert!(error_string.contains("Test error message"));
}

#[test]
fn test_songbird_error_from_io_error() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let songbird_error: SongbirdError = io_error.into();

    assert!(matches!(songbird_error, SongbirdError::Io { .. }));

    let error_string = format!("{songbird_error}");
    assert!(error_string.contains("File not found"));
}

#[test]
fn test_songbird_error_from_addr_parse_error() {
    let parse_result = "invalid_address".parse::<std::net::SocketAddr>();
    assert!(parse_result.is_err());

    let addr_error = parse_result.unwrap_err();
    let songbird_error: SongbirdError = addr_error.into();

    assert!(matches!(songbird_error, SongbirdError::Network { .. }));
}

#[test]
fn test_songbird_error_from_json_error() {
    let json_error = serde_json::from_str::<serde_json::Value>("invalid json");
    assert!(json_error.is_err());

    let json_err = json_error.unwrap_err();
    let songbird_error: SongbirdError = json_err.into();

    assert!(matches!(songbird_error, SongbirdError::Network { .. }));
}

#[test]
fn test_result_type_ok() {
    let success_value = "Success".to_string();
    let success_result: Result<String> = Ok(success_value.clone());

    assert!(success_result.is_ok());
    assert_eq!(success_result.unwrap(), success_value);
}

#[test]
fn test_result_type_err() {
    let expected_error = SongbirdError::Config {
        field: Some("test".to_string()),
        message: "Test error".to_string(),
    };
    let error_result: Result<String> = Err(expected_error);

    assert!(error_result.is_err());

    let error = error_result.unwrap_err();
    assert!(matches!(error, SongbirdError::Config { .. }));
}

#[test]
fn test_config_validator_port_validation() {
    // Valid port
    let result = ConfigValidator::validate_port(8080, "api_port");
    assert!(result.is_ok());

    // Invalid port (0)
    let result = ConfigValidator::validate_port(0, "invalid_port");
    assert!(result.is_err());

    // Privileged port (should warn but succeed)
    let result = ConfigValidator::validate_port(80, "http_port");
    assert!(result.is_ok());
}

#[test]
fn test_config_validator_port_range_validation() {
    // Valid port range
    let result = ConfigValidator::validate_port_range(8000, 9000);
    assert!(result.is_ok());

    // Invalid port range (start > end)
    let result = ConfigValidator::validate_port_range(9000, 8000);
    assert!(result.is_err());

    // Small port range (should warn but succeed)
    let result = ConfigValidator::validate_port_range(8000, 8005);
    assert!(result.is_ok());
}

#[test]
fn test_config_validator_url_validation() {
    // Valid HTTP URL
    let result = ConfigValidator::validate_url("https://example.com", "api_url");
    assert!(result.is_ok());

    // Valid WebSocket URL
    let result = ConfigValidator::validate_url("wss://example.com/ws", "ws_url");
    assert!(result.is_ok());

    // Invalid URL
    let result = ConfigValidator::validate_url("invalid-url", "bad_url");
    assert!(result.is_err());

    // Unsupported scheme
    let result = ConfigValidator::validate_url("ftp://example.com", "ftp_url");
    assert!(result.is_err());
}

#[test]
fn test_config_validator_http_url_validation() {
    // Valid HTTP URL
    let result = ConfigValidator::validate_http_url("https://api.example.com", "api_url");
    assert!(result.is_ok());

    // Invalid - WebSocket URL
    let result = ConfigValidator::validate_http_url("wss://example.com", "ws_url");
    assert!(result.is_err());
}

#[test]
fn test_config_validator_websocket_url_validation() {
    // Valid WebSocket URL
    let result = ConfigValidator::validate_websocket_url("wss://example.com/ws", "ws_url");
    assert!(result.is_ok());

    // Invalid - HTTP URL
    let result = ConfigValidator::validate_websocket_url("https://example.com", "http_url");
    assert!(result.is_err());
}

#[test]
fn test_config_validator_ip_address_validation() {
    // Valid IPv4
    let result = ConfigValidator::validate_ip_address("192.168.1.1", "ipv4");
    assert!(result.is_ok());

    // Valid IPv6
    let result = ConfigValidator::validate_ip_address("::1", "ipv6");
    assert!(result.is_ok());

    // Invalid IP
    let result = ConfigValidator::validate_ip_address("invalid.ip", "bad_ip");
    assert!(result.is_err());
}

#[test]
fn test_config_validator_socket_address_validation() {
    // Valid socket address
    let result = ConfigValidator::validate_socket_address("127.0.0.1:8080", "socket_addr");
    assert!(result.is_ok());

    // Invalid socket address
    let result = ConfigValidator::validate_socket_address("invalid:address", "bad_addr");
    assert!(result.is_err());
}

#[test]
fn test_config_validator_timeout_validation() {
    // Valid timeout
    let result = ConfigValidator::validate_timeout(5000, "request_timeout", 1000, 30000);
    assert!(result.is_ok());

    // Timeout too short
    let result = ConfigValidator::validate_timeout(500, "short_timeout", 1000, 30000);
    assert!(result.is_err());

    // Timeout too long
    let result = ConfigValidator::validate_timeout(60000, "long_timeout", 1000, 30000);
    assert!(result.is_err());
}

#[test]
fn test_config_validator_connection_timeout() {
    // Valid connection timeout
    let result = ConfigValidator::validate_connection_timeout(5000);
    assert!(result.is_ok());

    // Too short
    let result = ConfigValidator::validate_connection_timeout(50);
    assert!(result.is_err());

    // Too long
    let result = ConfigValidator::validate_connection_timeout(120000);
    assert!(result.is_err());
}

#[test]
fn test_config_validator_request_timeout() {
    // Valid request timeout
    let result = ConfigValidator::validate_request_timeout(30000);
    assert!(result.is_ok());

    // Too short
    let result = ConfigValidator::validate_request_timeout(500);
    assert!(result.is_err());

    // Too long
    let result = ConfigValidator::validate_request_timeout(600000);
    assert!(result.is_err());
}

#[test]
fn test_config_validator_retry_config() {
    // Valid retry config
    let result = ConfigValidator::validate_retry_config(3, 1000);
    assert!(result.is_ok());

    // Too many retries
    let result = ConfigValidator::validate_retry_config(15, 1000);
    assert!(result.is_err());

    // Invalid retry delay
    let result = ConfigValidator::validate_retry_config(3, 50000);
    assert!(result.is_err());
}

#[test]
fn test_config_validator_thread_pool_size() {
    // Valid thread pool size
    let result = ConfigValidator::validate_thread_pool_size(4, "worker_threads");
    assert!(result.is_ok());

    // Invalid thread pool size (0)
    let result = ConfigValidator::validate_thread_pool_size(0, "zero_threads");
    assert!(result.is_err());

    // Very large thread pool (should warn but succeed)
    let result = ConfigValidator::validate_thread_pool_size(100, "many_threads");
    assert!(result.is_ok());
}

#[test]
fn test_config_validator_buffer_size() {
    // Valid buffer size
    let result = ConfigValidator::validate_buffer_size(8192, "buffer", 1024, 65536);
    assert!(result.is_ok());

    // Buffer too small
    let result = ConfigValidator::validate_buffer_size(512, "small_buffer", 1024, 65536);
    assert!(result.is_err());

    // Buffer too large
    let result = ConfigValidator::validate_buffer_size(131072, "large_buffer", 1024, 65536);
    assert!(result.is_err());
}

#[test]
fn test_config_validator_memory_limit() {
    // Valid memory limit
    let result = ConfigValidator::validate_memory_limit(512);
    assert!(result.is_ok());

    // Memory limit too small
    let result = ConfigValidator::validate_memory_limit(32);
    assert!(result.is_err());
}

#[test]
fn test_config_validator_percentage() {
    // Valid percentage
    let result = ConfigValidator::validate_percentage(75.5, "cpu_usage");
    assert!(result.is_ok());

    // Invalid percentage (negative)
    let result = ConfigValidator::validate_percentage(-10.0, "negative_percent");
    assert!(result.is_err());

    // Invalid percentage (over 100)
    let result = ConfigValidator::validate_percentage(150.0, "over_hundred");
    assert!(result.is_err());
}

#[test]
fn test_config_validator_rate_limit() {
    // Valid rate limit
    let result = ConfigValidator::validate_rate_limit(100.0, "requests_per_second");
    assert!(result.is_ok());

    // Invalid rate limit (negative)
    let result = ConfigValidator::validate_rate_limit(-5.0, "negative_rate");
    assert!(result.is_err());

    // Invalid rate limit (zero)
    let result = ConfigValidator::validate_rate_limit(0.0, "zero_rate");
    assert!(result.is_err());
}

#[test]
fn test_config_validator_basic_config() {
    // Test basic configuration validation
    let result = ConfigValidator::validate_basic_config();
    assert!(result.is_ok());
}

#[test]
fn test_error_std_error_trait() {
    let error = SongbirdError::Config {
        field: Some("test".to_string()),
        message: "Test error".to_string(),
    };

    // Test that it implements std::error::Error
    let _: &dyn std::error::Error = &error;
}

#[test]
fn test_error_debug_trait() {
    let error = SongbirdError::Service {
        service: "test".to_string(),
        message: "Test error".to_string(),
    };

    let debug_string = format!("{error:?}");
    assert!(debug_string.contains("Service"));
    assert!(debug_string.contains("test"));
}

#[test]
fn test_error_clone_trait() {
    let error = SongbirdError::Network {
        service: Some("api".to_string()),
        message: "Connection failed".to_string(),
        details: None,
    };

    let cloned_error = error.clone();
    assert_eq!(format!("{error}"), format!("{cloned_error}"));
}

#[test]
fn test_error_performance() {
    let start = std::time::Instant::now();

    // Test creating many errors quickly
    for i in 0..1000 {
        let error = SongbirdError::Service {
            service: format!("service_{i}"),
            message: "Test error".to_string(),
        };

        let _ = format!("{error}");
    }

    let duration = start.elapsed();
    assert!(duration < Duration::from_millis(100)); // Should be fast
}

#[test]
fn test_error_memory_usage() {
    let error = SongbirdError::Config {
        field: Some("test".to_string()),
        message: "test message".to_string(),
    };

    let size = std::mem::size_of_val(&error);
    assert!(size < 1000); // Should be reasonable size
}
