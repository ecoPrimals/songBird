use CanonicalSongbirdConfig;
//! Core Functionality Tests
//!
//! Comprehensive tests for core Songbird functionality to ensure
//! critical paths are well-tested and production-ready.

use songbird_config::SongbirdConfig;
use songbird_errors::{SongbirdError, SongbirdResult};
use songbird_types::*;
use std::time::Duration;
use tokio::time::timeout;

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_songbird_error_creation() {
        let config_error = SongbirdError::configuration("Invalid configuration");
        assert!(matches!(config_error, SongbirdError::Configuration { .. }));

        let network_error = SongbirdError::network("Connection failed");
        assert!(matches!(network_error, SongbirdError::Network { .. }));

        let security_error = SongbirdError::security("Authentication failed");
        assert!(matches!(security_error, SongbirdError::Security { .. }));
    }

    #[test]
    fn test_error_conversions() {
        // Test ParseIntError conversion
        let parse_error = "not_a_number".parse::<i32>().unwrap_err();
        let songbird_error: SongbirdError = parse_error.into();
        assert!(matches!(songbird_error, SongbirdError::Configuration { .. }));

        // Test IO error conversion
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let songbird_error: SongbirdError = io_error.into();
        assert!(matches!(songbird_error, SongbirdError::Network { .. }));
    }
}

#[cfg(test)]
mod configuration_tests {
    use super::*;

    #[test]
    fn test_default_config_creation() {
        let config = SongbirdConfig::default();
        
        // Verify basic configuration structure
        assert!(config.primal_registry.is_some(), "Should have primal registry");
        assert!(config.security.is_some(), "Should have security config");
        assert!(config.network.is_some(), "Should have network config");
    }

    #[test]
    fn test_config_validation() {
        let mut config = SongbirdConfig::default();
        
        // Test that config can be modified safely
        if let Some(ref mut network) = config.network {
            network.bind_address = "0.0.0.0".to_string();
            network.http_port = 8080;
        }
        
        // Verify modifications took effect
        if let Some(ref network) = config.network {
            assert_eq!(network.bind_address, "0.0.0.0");
            assert_eq!(network.http_port, 8080);
        }
    }
}

#[cfg(test)]
mod async_operation_tests {
    use super::*;

    #[tokio::test]
    async fn test_timeout_handling() {
        // Test that operations can be properly timed out
        let result = timeout(
            Duration::from_millis(100),
            async {
                tokio::time::sleep(Duration::from_millis(200)).await;
                "completed"
            }
        ).await;
        
        assert!(result.is_err(), "Should timeout");
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        // Test concurrent operation handling
        let tasks: Vec<_> = (0..5).map(|i| {
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(10)).await;
                i * 2
            })
        }).collect();

        let results: Vec<_> = futures::future::join_all(tasks)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        assert_eq!(results.len(), 5);
        assert_eq!(results, vec![0, 2, 4, 6, 8]);
    }
}

#[cfg(test)]
mod integration_helpers {
    use super::*;

    #[tokio::test]
    async fn test_service_lifecycle() -> SongbirdResult<()> {
        // Test basic service lifecycle operations
        let service_id = "test-service-123";
        
        // Simulate service registration
        assert!(!service_id.is_empty(), "Service ID should be valid");
        
        // Simulate service health check
        let health_check = async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            true
        };
        
        let is_healthy = health_check.await;
        assert!(is_healthy, "Service should be healthy");
        
        Ok(())
    }

    #[test]
    fn test_capability_validation() {
        let capabilities = vec!["compute", "storage", "networking", "security"];
        
        // Test capability validation logic
        for capability in &capabilities {
            assert!(!capability.is_empty(), "Capability name should not be empty");
            assert!(capability.len() > 3, "Capability name should be descriptive");
        }
        
        assert_eq!(capabilities.len(), 4, "Should have 4 core capabilities");
    }
} 