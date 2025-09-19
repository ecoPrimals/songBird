//! Individual error type tests for all SongbirdError variants

use songbird_errors::{
    AuthError, CircuitBreakerError, DeploymentError, DiscoveryError, GamingError, IoError,
    NetworkError, NotFoundError, ProtocolError, ServiceError, SongbirdError, ValidationError,
, SongbirdError};
use std::time::Duration;

#[test]
fn test_songbird_error_config() {
    let config_error = SongbirdError::Config {
        field: Some("database_url".to_string()),
        message: "Invalid database URL".to_string(),
        context: Some("Database configuration".to_string()),
        suggestion: Some("Check the database URL format".to_string()),
    };

    assert!(matches!(config_error, SongbirdError::Config { .. }));
    
    let error_string = format!("{config_error}");
    assert!(error_string.contains("Invalid database URL"));
    assert!(error_string.contains("database_url"));
}

#[test]
fn test_songbird_error_configuration() {
    let error = SongbirdError::config_error("port", "Invalid port number");
    let error_string = format!("{error}");
    assert!(error_string.contains("Invalid port number"));
    assert!(error_string.contains("port"));
}

#[test]
fn test_songbird_error_network() {
    let network_error = SongbirdError::Network(Box::new(NetworkError {
        service: Some("api".to_string()),
        message: "Connection refused".to_string(),
        details: Some("Port 8080 not responding".to_string()),
        endpoint: Some("localhost:{}".to_string()),
        suggestion: Some("Check if service is running".to_string()),
    }));

    assert!(matches!(network_error, SongbirdError::Network(_)));
    
    let error_string = format!("{network_error}");
    assert!(error_string.contains("Connection refused"));
    assert!(error_string.contains("api"));
}

#[test]
fn test_songbird_error_communication() {
    let comm_error = SongbirdError::communication_error("Failed to send message");
    let error_string = format!("{comm_error}");
    assert!(error_string.contains("Failed to send message"));
}

#[test]
fn test_songbird_error_discovery() {
    let discovery_error = SongbirdError::Discovery(Box::new(DiscoveryError {
        message: "Service not found".to_string(),
        service: Some("user-service".to_string()),
        timeout: Some(5000),
        suggestion: Some("Check service registry".to_string()),
    }));

    assert!(matches!(discovery_error, SongbirdError::Discovery(_)));
    
    let error_string = format!("{discovery_error}");
    assert!(error_string.contains("Service not found"));
    assert!(error_string.contains("user-service"));
}

#[test]
fn test_songbird_error_auth() {
    let auth_error = SongbirdError::security("Login failed");

    assert!(matches!(auth_error, SongbirdError::Auth(_)));
    
    let error_string = format!("{auth_error}");
    assert!(error_string.contains("Login failed"));
    assert!(error_string.contains("john_doe"));
}

#[test]
fn test_songbird_error_authentication() {
    let auth_error = SongbirdError::Authentication {
        provider: "ldap".to_string(),
        message: "Authentication failed".to_string(),
        suggestion: Some("Check LDAP configuration".to_string()),
    };

    assert!(matches!(auth_error, SongbirdError::Authentication { .. }));
    
    let error_string = format!("{auth_error}");
    assert!(error_string.contains("Authentication failed"));
    assert!(error_string.contains("ldap"));
}

#[test]
fn test_songbird_error_gaming() {
    let gaming_error = SongbirdError::Gaming(Box::new(GamingError {
        message: "Game connection failed".to_string(),
        protocol: Some("TCP".to_string()),
        game: Some("chess".to_string()),
        suggestion: Some("Check game server status".to_string()),
    }));

    assert!(matches!(gaming_error, SongbirdError::Gaming(_)));
    
    let error_string = format!("{gaming_error}");
    assert!(error_string.contains("Game connection failed"));
    assert!(error_string.contains("chess"));
}

#[test]
fn test_songbird_error_security() {
    let security_error = SongbirdError::security_error("Encryption failed");
    
    let error_string = format!("{security_error}");
    assert!(error_string.contains("Encryption failed"));
}

#[test]
fn test_songbird_error_protocol() {
    let protocol_error = SongbirdError::Protocol(Box::new(ProtocolError {
        protocol: "http".to_string(),
        message: "Invalid request format".to_string(),
        version: Some("1.1".to_string()),
        suggestion: Some("Check HTTP request format and headers".to_string()),
    }));

    assert!(matches!(protocol_error, SongbirdError::Protocol { .. }));

    let error_string = format!("{protocol_error}");
    assert!(error_string.contains("Invalid request format"));
    assert!(error_string.contains("http"));
}

#[test]
fn test_songbird_error_service() {
    let service_error = SongbirdError::Service(Box::new(ServiceError {
        service: "database".to_string(),
        message: "Connection pool exhausted".to_string(),
        status: Some("overloaded".to_string()),
        suggestion: Some("Check database connection pool configuration".to_string()),
    }));

    assert!(matches!(service_error, SongbirdError::Service { .. }));

    let error_string = format!("{service_error}");
    assert!(error_string.contains("Connection pool exhausted"));
    assert!(error_string.contains("database"));
}

#[test]
fn test_songbird_error_validation() {
    let validation_error = SongbirdError::Validation(Box::new(ValidationError {
        field: "email".to_string(),
        message: "Invalid email format".to_string(),
        value: Some("invalid-email".to_string()),
        expected: Some("valid email format".to_string()),
        suggestion: Some("Use a valid email address format".to_string()),
    }));

    assert!(matches!(validation_error, SongbirdError::Validation { .. }));

    let error_string = format!("{validation_error}");
    assert!(error_string.contains("Invalid email format"));
    assert!(error_string.contains("email"));
}

#[test]
fn test_songbird_error_not_found() {
    let not_found_error = SongbirdError::NotFound(Box::new(NotFoundError {
        resource: "user".to_string(),
        message: "User not found".to_string(),
        searched_paths: Some(vec!["/users/123".to_string()]),
        suggestion: Some("Check user ID and database connection".to_string()),
    }));

    assert!(matches!(not_found_error, SongbirdError::NotFound(_)));

    let error_string = format!("{not_found_error}");
    assert!(error_string.contains("User not found"));
    assert!(error_string.contains("user"));
}

#[test]
fn test_songbird_error_io() {
    let io_error = SongbirdError::Io(Box::new(IoError {
        message: "File not found".to_string(),
        path: Some("/path/to/file".to_string()),
        operation: Some("read".to_string()),
        suggestion: Some("Check file path and permissions".to_string()),
    }));

    assert!(matches!(io_error, SongbirdError::Io { .. }));

    let error_string = format!("{io_error}");
    assert!(error_string.contains("File not found"));
}

#[test]
fn test_songbird_error_load_balancer() {
    let lb_error = SongbirdError::LoadBalancer {
        message: "No healthy backends available".to_string(),
        backend: Some("backend-1".to_string()),
        suggestion: Some("Check backend health and configuration".to_string()),
    };

    assert!(matches!(lb_error, SongbirdError::LoadBalancer { .. }));

    let error_string = format!("{lb_error}");
    assert!(error_string.contains("No healthy backends available"));
}

#[test]
fn test_songbird_error_tunnel_creation() {
    let tunnel_error = SongbirdError::TunnelCreation {
        message: "Failed to establish tunnel".to_string(),
        tunnel_type: Some("SSH".to_string()),
        endpoint: Some("remote.example.com:22".to_string()),
        suggestion: Some("Check SSH configuration and network connectivity".to_string()),
    };

    assert!(matches!(tunnel_error, SongbirdError::TunnelCreation { .. }));

    let error_string = format!("{tunnel_error}");
    assert!(error_string.contains("Failed to establish tunnel"));
}

#[test]
fn test_songbird_error_encryption_failed() {
    let encryption_error = SongbirdError::EncryptionFailed {
        message: "AES encryption failed".to_string(),
        algorithm: Some("AES-256".to_string()),
        key_info: Some("Key rotation needed".to_string()),
        suggestion: Some("Check encryption key and try again".to_string()),
    };

    assert!(matches!(encryption_error, SongbirdError::EncryptionFailed { .. }));

    let error_string = format!("{encryption_error}");
    assert!(error_string.contains("AES encryption failed"));
}

#[test]
fn test_songbird_error_decryption_failed() {
    let decryption_error = SongbirdError::DecryptionFailed {
        message: "Failed to decrypt data".to_string(),
        algorithm: Some("AES-256".to_string()),
        key_info: Some("Invalid key".to_string()),
        suggestion: Some("Verify decryption key and try again".to_string()),
    };

    assert!(matches!(decryption_error, SongbirdError::DecryptionFailed { .. }));

    let error_string = format!("{decryption_error}");
    assert!(error_string.contains("Failed to decrypt data"));
}

#[test]
fn test_songbird_error_network_detection() {
    let detection_error = SongbirdError::NetworkDetection {
        message: "Network interface not found".to_string(),
        interface: Some("eth0".to_string()),
        detection_type: Some("interface_scan".to_string()),
        suggestion: Some("Check network interface configuration".to_string()),
    };

    assert!(matches!(detection_error, SongbirdError::NetworkDetection { .. }));

    let error_string = format!("{detection_error}");
    assert!(error_string.contains("Network interface not found"));
}

#[test]
fn test_songbird_error_unsupported_channel_type() {
    let channel_error = SongbirdError::UnsupportedChannelType {
        message: "Channel type not supported".to_string(),
        channel_type: Some("websocket".to_string()),
        supported_types: Some(vec!["http".to_string(), "grpc".to_string()]),
        suggestion: Some("Use a supported channel type".to_string()),
    };

    assert!(matches!(channel_error, SongbirdError::UnsupportedChannelType { .. }));

    let error_string = format!("{channel_error}");
    assert!(error_string.contains("Channel type not supported"));
}

#[test]
fn test_songbird_error_deployment() {
    let deployment_error = SongbirdError::Deployment(Box::new(DeploymentError {
        message: "Deployment failed".to_string(),
        stage: Some("validation".to_string()),
        resource: Some("service.yaml".to_string()),
        suggestion: Some("Check deployment configuration".to_string()),
    }));

    assert!(matches!(deployment_error, SongbirdError::Deployment(_)));

    let error_string = format!("{deployment_error}");
    assert!(error_string.contains("Deployment failed"));
}

#[test]
fn test_songbird_error_circuit_breaker_open() {
    let cb_error = SongbirdError::CircuitBreakerOpen(Box::new(CircuitBreakerError {
        service: "api".to_string(),
        message: "Circuit breaker is open".to_string(),
        failure_count: Some(5),
        failure_threshold: Some(3),
        timeout_duration: Some(Duration::from_secs(30)),
        suggestion: Some("Wait for circuit breaker to close".to_string()),
    }));

    assert!(matches!(cb_error, SongbirdError::CircuitBreakerOpen(_)));

    let error_string = format!("{cb_error}");
    assert!(error_string.contains("Circuit breaker is open"));
} 