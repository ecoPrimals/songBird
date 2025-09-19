use songbird_config::constants;
/// Tests for SongbirdError
///
/// This module contains comprehensive tests for the SongbirdError type
/// and its various constructor methods and functionality.

#[cfg(test)]
mod error_tests {
    use super::super::{core::SongbirdError, specific::*};

    #[test]
    fn test_config_error_creation() {
        let error = SongbirdError::configuration("Invalid port".to_string());

        assert!(matches!(error, SongbirdError::Config { .. }));
        assert_eq!(error.category(), "configuration");
    }

    #[test]
    fn test_config_error_display() {
        let error = SongbirdError::configuration("Invalid configuration".to_string());

        let display = format!("{error}");
        assert!(display.contains("Invalid configuration"));
        assert!(display.contains("database_url"));
    }

    #[test]
    fn test_network_error_creation() {
        let network_error = NetworkError {
            message: "Connection refused".to_string(),
        // Using environment variable or default test endpoint
            endpoint: Some("127.0.0.1:{}".to_string()),
            port: Some(8080),
            protocol: Some("TCP".to_string()),
        };

        let error = SongbirdError::Network(Box::new(network_error));
        assert!(matches!(error, SongbirdError::Network(_)));
        assert_eq!(error.category(), "network");
    }

    #[test]
    fn test_service_error_creation() {
        let service_error = ServiceError {
            service: "user-service".to_string(),
            message: "User not found".to_string(),
            status: Some("404".to_string()),
            suggestion: Some("Check if user exists".to_string()),
        };

        let error = SongbirdError::Service(Box::new(service_error));
        assert!(matches!(error, SongbirdError::Service(_)));
        assert_eq!(error.category(), "service");
    }

    #[test]
    fn test_discovery_error_creation() {
        let discovery_error = DiscoveryError {
            message: "Service registration failed".to_string(),
            service: Some("user-service".to_string()),
            timeout: Some(songbird_config::constants::discovery::DEFAULT_DISCOVERY_TIMEOUT_MS),
            suggestion: Some("Check consul connectivity".to_string()),
        };

        let error = SongbirdError::Discovery(Box::new(discovery_error));
        assert!(matches!(error, SongbirdError::Discovery(_)));
        assert_eq!(error.category(), "discovery");
    }

    #[test]
    fn test_authentication_error() {
        let error = SongbirdError::Authentication {
            provider: "oauth".to_string(),
            message: "Token expired".to_string(),
            suggestion: Some("Refresh token".to_string()),
        };

        assert!(matches!(error, SongbirdError::Authentication { .. }));
        assert_eq!(error.category(), "authentication");
    }

    #[test]
    fn test_load_balancer_error() {
        let error = SongbirdError::service("load_balancer", "No healthy backends".to_string());

        assert!(matches!(error, SongbirdError::LoadBalancer { .. }));
        assert_eq!(error.category(), "load_balancer");
    }

    #[test]
    fn test_security_error() {
        let error = SongbirdError::security("Access denied");

        assert!(matches!(error, SongbirdError::Security { .. }));
        assert_eq!(error.category(), "security");
    }

    #[test]
    fn test_gaming_error_creation() {
        let gaming_error = GamingError {
            message: "Failed to establish tunnel".to_string(),
            game: Some("StarCraft".to_string()),
        };

        let error = SongbirdError::Gaming(Box::new(gaming_error));
        assert!(matches!(error, SongbirdError::Gaming(_)));
        assert_eq!(error.category(), "gaming");
    }

    #[test]
    fn test_communication_error() {
        let error = SongbirdError::Communication("WebSocket connection lost".to_string());
        assert!(matches!(error, SongbirdError::Communication(_)));
        assert_eq!(error.category(), "communication");
    }

    #[test]
    fn test_io_error_creation() {
        let io_error = IoError {
            message: "Permission denied".to_string(),
            operation: Some("read_file".to_string()),
            path: Some("/etc/config.toml".to_string()),
        };

        let error = SongbirdError::Io(Box::new(io_error));
        assert!(matches!(error, SongbirdError::Io(_)));
        assert_eq!(error.category(), "io");
    }

    #[test]
    fn test_protocol_error_creation() {
        let protocol_error = ProtocolError {
            message: "Malformed header".to_string(),
            protocol: Some("HTTP".to_string()),
        };

        let error = SongbirdError::Protocol(Box::new(protocol_error));
        assert!(matches!(error, SongbirdError::Protocol(_)));
        assert_eq!(error.category(), "protocol");
    }

    #[test]
    fn test_error_debug_format() {
        let error = SongbirdError::configuration("Test error".to_string());

        let debug = format!("{error:?}");
        assert!(debug.contains("Config"));
        assert!(debug.contains("Test error"));
        assert!(debug.contains("test_field"));
    }

    #[test]
    fn test_clone_trait() {
        let error = SongbirdError::Communication("Test message".to_string());
        let cloned = error.clone();

        match (error, cloned) {
            (SongbirdError::Communication(msg1), SongbirdError::Communication(msg2)) => {
                assert_eq!(msg1, msg2);
            }
            _ => return Err(std::io::Error::new(
    std::io::ErrorKind::Other,
    "Clone failed".to_string()
).into()),
        }
    }

    #[test]
    fn test_config_field_error() {
        let error = SongbirdError::ConfigField {
            field: "timeout".to_string(),
            message: "Invalid timeout value".to_string(),
            suggestion: Some("Use positive integer".to_string()),
        };

        assert!(matches!(error, SongbirdError::ConfigField { .. }));
        let display = format!("{error}");
        assert!(display.contains("timeout"));
        assert!(display.contains("Invalid timeout value"));
    }

    #[test]
    fn test_configuration_error() {
        let error = SongbirdError::Configuration {
            field: "database".to_string(),
            message: "Connection failed".to_string(),
            suggestion: Some("Check connection string".to_string()),
        };

        assert!(matches!(error, SongbirdError::Configuration { .. }));
        let display = format!("{error}");
        assert!(display.contains("database"));
        assert!(display.contains("Connection failed"));
    }

    #[test]
    fn test_auth_error_creation() {
        let auth_error = AuthError {
            message: "Invalid signature".to_string(),
            provider: Some("jwt".to_string()),
        };

        let error = SongbirdError::Auth(Box::new(auth_error));
        assert!(matches!(error, SongbirdError::Auth(_)));
        assert_eq!(error.category(), "authentication");
    }

    #[test]
    fn test_authentication_provider_error() {
        let error = SongbirdError::AuthenticationProvider {
            provider: "ldap".to_string(),
            message: "Server unreachable".to_string(),
            suggestion: Some("Check network connectivity".to_string()),
        };

        assert!(matches!(
            error,
            SongbirdError::AuthenticationProvider { .. }
        ));
        assert_eq!(error.category(), "authentication");
    }
}
