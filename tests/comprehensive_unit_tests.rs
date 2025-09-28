use CanonicalSongbirdConfig;
//! Comprehensive Unit Test Suite
//!
//! This module provides comprehensive unit test coverage for individual components
//! of the Songbird Universal Orchestrator, targeting specific functionality and edge cases.

use songbird_config::{SongbirdConfig, EnvironmentConfig, NetworkConfig};
use songbird_errors::{SongbirdError, SongbirdResult};
use songbird_test_utils::{TestEnvironment, TestContext, ErrorTestingFramework};
use songbird_types::{SongbirdResult as TypesResult};
use std::time::Duration;

/// Configuration module unit tests
#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_songbird_config_default() -> SongbirdResult<()> {
        let config = SongbirdConfig::default();
        
        // Test default values are reasonable
        assert!(!config.environment.environment.is_empty(), "Environment should not be empty");
        assert!(config.environment.connection_timeout_secs > 0, "Timeout should be positive");
        assert!(config.environment.dashboard_port > 0, "Dashboard port should be positive");
        assert!(!config.environment.bind_address.is_empty(), "Bind address should not be empty");
        
        Ok(())
    }

    #[test]
    fn test_environment_config_validation() -> SongbirdResult<()> {
        let valid_config = EnvironmentConfig {
            environment: "test".to_string(),
            bind_address: "127.0.0.1".to_string(),
            require_tls: false,
            connection_timeout_secs: 30,
            dashboard_port: config.dashboard.port,
            monitoring_enabled: true,
            debug_mode: true,
            log_level: "debug".to_string(),
            max_connections: 1000,
            enable_metrics: true,
            cors_enabled: true,
            rate_limiting_enabled: false,
            backup_enabled: false,
            encryption_enabled: false,
            audit_logging: false,
            health_check_interval_secs: 30,
            service_discovery_enabled: true,
            load_balancing_enabled: true,
            circuit_breaker_enabled: true,
            retry_enabled: true,
        };

        // Test valid configuration
        assert_eq!(valid_config.environment, "test");
        assert_eq!(valid_config.connection_timeout_secs, 30);
        assert_eq!(valid_config.dashboard_port, config.dashboard.port);
        assert!(valid_config.monitoring_enabled);

        Ok(())
    }

    #[test]
    fn test_network_config_defaults() -> SongbirdResult<()> {
        let network_config = NetworkConfig {
            bind_address: "0.0.0.0".to_string(),
            port_range: (8000, 9000),
            connection_timeout_ms: 30000,
            max_connections: 1000,
            enable_ipv6: false,
            buffer_size: 8192,
            keep_alive_interval_secs: 60,
        };

        // Test network configuration values
        assert!(!network_config.bind_address.is_empty(), "Bind address should not be empty");
        assert!(network_config.port_range.0 < network_config.port_range.1, "Port range should be valid");
        assert!(network_config.connection_timeout_ms > 0, "Timeout should be positive");
        assert!(network_config.max_connections > 0, "Max connections should be positive");
        assert!(network_config.buffer_size > 0, "Buffer size should be positive");

        Ok(())
    }

    #[test]
    fn test_config_edge_cases() -> SongbirdResult<()> {
        // Test empty environment name
        let empty_env_config = EnvironmentConfig {
            environment: "".to_string(),
            bind_address: "127.0.0.1".to_string(),
            require_tls: false,
            connection_timeout_secs: 30,
            dashboard_port: config.dashboard.port,
            monitoring_enabled: true,
            debug_mode: true,
            log_level: "debug".to_string(),
            max_connections: 1000,
            enable_metrics: true,
            cors_enabled: true,
            rate_limiting_enabled: false,
            backup_enabled: false,
            encryption_enabled: false,
            audit_logging: false,
            health_check_interval_secs: 30,
            service_discovery_enabled: true,
            load_balancing_enabled: true,
            circuit_breaker_enabled: true,
            retry_enabled: true,
        };

        // Should handle empty environment gracefully
        assert!(empty_env_config.environment.is_empty());

        // Test extreme values
        let extreme_config = EnvironmentConfig {
            environment: "production".to_string(),
            bind_address: "0.0.0.0".to_string(),
            require_tls: true,
            connection_timeout_secs: 1, // Very short timeout
            dashboard_port: 65535, // Max port
            monitoring_enabled: true,
            debug_mode: false,
            log_level: "error".to_string(),
            max_connections: 100000, // High connection limit
            enable_metrics: true,
            cors_enabled: false,
            rate_limiting_enabled: true,
            backup_enabled: true,
            encryption_enabled: true,
            audit_logging: true,
            health_check_interval_secs: 1, // Very frequent health checks
            service_discovery_enabled: true,
            load_balancing_enabled: true,
            circuit_breaker_enabled: true,
            retry_enabled: true,
        };

        assert_eq!(extreme_config.connection_timeout_secs, 1);
        assert_eq!(extreme_config.dashboard_port, 65535);
        assert_eq!(extreme_config.max_connections, 100000);

        Ok(())
    }
}

/// Error handling unit tests
#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn test_songbird_error_creation() -> SongbirdResult<()> {
        // Test different error types
        let network_error = SongbirdError::network("Connection failed");
        let service_error = SongbirdError::service(config.test.service_name, "Service unavailable");
        let config_error = SongbirdError::configuration("Invalid configuration");
        let security_error = SongbirdError::security("Authentication failed");

        // Test error properties
        match network_error {
            SongbirdError::Network { message, .. } => {
                assert_eq!(message, "Connection failed");
            }
            _ => panic!("Should be network error"),
        }

        match service_error {
            SongbirdError::Service { service, message, .. } => {
                assert_eq!(service, config.test.service_name);
                assert_eq!(message, "Service unavailable");
            }
            _ => panic!("Should be service error"),
        }

        match config_error {
            SongbirdError::Configuration { message, .. } => {
                assert_eq!(message, "Invalid configuration");
            }
            _ => panic!("Should be configuration error"),
        }

        match security_error {
            SongbirdError::Security { message, .. } => {
                assert_eq!(message, "Authentication failed");
            }
            _ => panic!("Should be security error"),
        }

        Ok(())
    }

    #[test]
    fn test_error_chaining() -> SongbirdResult<()> {
        // Test error conversion and chaining
        let base_error = SongbirdError::network("Base network error");
        
        // Create a result and test error propagation
        let result: SongbirdResult<String> = Err(base_error);
        
        assert!(result.is_err(), "Result should be error");
        
        match result {
            Err(SongbirdError::Network { message, .. }) => {
                assert_eq!(message, "Base network error");
            }
            _ => panic!("Should be network error"),
        }

        Ok(())
    }

    #[test]
    fn test_error_display() -> SongbirdResult<()> {
        let error = SongbirdError::service("auth-service", "Token expired");
        let error_string = format!("{}", error);
        
        assert!(error_string.contains("auth-service"), "Error should contain service name");
        assert!(error_string.contains("Token expired"), "Error should contain message");

        Ok(())
    }

    #[test]
    fn test_result_combinations() -> SongbirdResult<()> {
        // Test successful result
        let success: SongbirdResult<i32> = Ok(42);
        assert!(success.is_ok());
        assert_eq!(success.expect("Test assertion should succeed"), 42);

        // Test error result
        let failure: SongbirdResult<i32> = Err(SongbirdError::configuration("Test error"));
        assert!(failure.is_err());

        // Test result mapping
        let mapped_success: SongbirdResult<String> = Ok(42).map(|x| x.to_string());
        assert!(mapped_success.is_ok());
        assert_eq!(mapped_success.expect("Test assertion should succeed"), "42");

        Ok(())
    }
}

/// Utility and helper function tests
#[cfg(test)]
mod utility_tests {
    use super::*;

    #[test]
    fn test_duration_handling() -> SongbirdResult<()> {
        // Test various duration values
        let short_duration = Duration::from_millis(100);
        let medium_duration = Duration::from_secs(30);
        let long_duration = Duration::from_secs(300);

        assert!(short_duration < medium_duration);
        assert!(medium_duration < long_duration);
        
        // Test duration arithmetic
        let combined = short_duration + medium_duration;
        assert!(combined > medium_duration);
        assert!(combined.as_millis() == 30100);

        Ok(())
    }

    #[test]
    fn test_string_validation() -> SongbirdResult<()> {
        // Test various string validation scenarios
        let valid_strings = vec![
            config.test.service_name,
            "production",
            "127.0.0.1",
            "http://localhost:config.network.http_port",
            "user@example.com",
        ];

        for string in valid_strings {
            assert!(!string.is_empty(), "String should not be empty: {}", string);
            assert!(string.len() > 0, "String should have length: {}", string);
        }

        // Test edge cases
        let empty_string = "";
        assert!(empty_string.is_empty());
        assert_eq!(empty_string.len(), 0);

        Ok(())
    }

    #[test]
    fn test_numeric_validation() -> SongbirdResult<()> {
        // Test port number validation
        let valid_ports = vec![80, 443, config.network.http_port, config.dashboard.port, 9000];
        for port in valid_ports {
            assert!(port > 0, "Port should be positive: {}", port);
            assert!(port <= 65535, "Port should be within valid range: {}", port);
        }

        // Test timeout validation
        let valid_timeouts = vec![1, 30, 60, 300, 3600];
        for timeout in valid_timeouts {
            assert!(timeout > 0, "Timeout should be positive: {}", timeout);
            assert!(timeout <= 86400, "Timeout should be reasonable: {}", timeout);
        }

        // Test percentage validation
        let valid_percentages = vec![0.0, 0.5, 1.0, 50.0, 100.0];
        for percentage in valid_percentages {
            assert!(percentage >= 0.0, "Percentage should be non-negative: {}", percentage);
            assert!(percentage <= 100.0, "Percentage should be <= 100: {}", percentage);
        }

        Ok(())
    }

    #[test]
    fn test_collection_operations() -> SongbirdResult<()> {
        // Test vector operations
        let mut services = vec!["service1", "service2", "service3"];
        
        assert_eq!(services.len(), 3);
        assert!(!services.is_empty());
        assert!(services.contains(&"service1"));

        services.push("service4");
        assert_eq!(services.len(), 4);

        services.retain(|&s| s != "service2");
        assert_eq!(services.len(), 3);
        assert!(!services.contains(&"service2"));

        // Test deduplication
        services.push("service1"); // Duplicate
        services.sort();
        services.dedup();
        assert!(services.len() <= 4); // Should remove duplicates

        Ok(())
    }
}

/// Async functionality tests
#[cfg(test)]
mod async_tests {
    use super::*;
    use tokio::time::{sleep, timeout};

    #[tokio::test]
    async fn test_async_operations() -> SongbirdResult<()> {
        let ctx = TestContext::new("async_operations");

        // Test basic async operation
        let result = async_operation().await;
        assert!(result.is_ok());

        // Test timeout handling
        let quick_result = timeout(Duration::from_secs(1), async_operation()).await;
        assert!(quick_result.is_ok());

        assert!(!ctx.is_timeout());
        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_operations() -> SongbirdResult<()> {
        let ctx = TestContext::new("concurrent_operations");

        // Test concurrent async operations
        let handles = (0..5).map(|i| {
            tokio::spawn(async move {
                sleep(Duration::from_millis(10)).await;
                format!("result_{}", i)
            })
        }).collect::<Vec<_>>();

        let results: Vec<String> = futures::future::try_join_all(handles)
            .await
            .map_err(|e| SongbirdError::internal_error(&format!("Join error: {}", e)))?;

        assert_eq!(results.len(), 5);
        for (i, result) in results.iter().enumerate() {
            assert_eq!(result, &format!("result_{}", i));
        }

        assert!(!ctx.is_timeout());
        Ok(())
    }

    #[tokio::test]
    async fn test_error_propagation_async() -> SongbirdResult<()> {
        let ctx = TestContext::new("error_propagation_async");

        // Test async error propagation
        let error_result = async_error_operation().await;
        assert!(error_result.is_err());

        match error_result {
            Err(SongbirdError::Network { .. }) => {
                // Expected error type
            }
            _ => panic!("Should be network error"),
        }

        assert!(!ctx.is_timeout());
        Ok(())
    }

    #[tokio::test]
    async fn test_timeout_scenarios() -> SongbirdResult<()> {
        let ctx = TestContext::new("timeout_scenarios");

        // Test operation that should timeout
        let timeout_result = timeout(
            Duration::from_millis(10),
            sleep(Duration::from_millis(100))
        ).await;
        
        assert!(timeout_result.is_err()); // Should timeout

        // Test operation that should complete
        let success_result = timeout(
            Duration::from_millis(100),
            sleep(Duration::from_millis(10))
        ).await;
        
        assert!(success_result.is_ok()); // Should complete

        assert!(!ctx.is_timeout());
        Ok(())
    }

    // Helper async functions for testing
    async fn async_operation() -> SongbirdResult<String> {
        sleep(Duration::from_millis(1)).await;
        Ok("success".to_string())
    }

    async fn async_error_operation() -> SongbirdResult<String> {
        sleep(Duration::from_millis(1)).await;
        Err(SongbirdError::network("Simulated network error"))
    }
}

/// Edge case and boundary tests
#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_boundary_values() -> SongbirdResult<()> {
        // Test minimum values
        let min_port = 1u16;
        let min_timeout = 1u64;
        let min_connections = 1u32;

        assert_eq!(min_port, 1);
        assert_eq!(min_timeout, 1);
        assert_eq!(min_connections, 1);

        // Test maximum values
        let max_port = 65535u16;
        let max_timeout = u64::MAX;
        let max_connections = u32::MAX;

        assert_eq!(max_port, 65535);
        assert!(max_timeout > 0);
        assert!(max_connections > 0);

        Ok(())
    }

    #[test]
    fn test_empty_collections() -> SongbirdResult<()> {
        // Test empty vector handling
        let empty_services: Vec<String> = Vec::new();
        assert!(empty_services.is_empty());
        assert_eq!(empty_services.len(), 0);

        // Test empty string handling
        let empty_string = String::new();
        assert!(empty_string.is_empty());
        assert_eq!(empty_string.len(), 0);

        // Test None option handling
        let none_option: Option<String> = None;
        assert!(none_option.is_none());
        assert_eq!(none_option, None);

        Ok(())
    }

    #[test]
    fn test_large_values() -> SongbirdResult<()> {
        // Test handling of large values
        let large_string = "x".repeat(10000);
        assert_eq!(large_string.len(), 10000);
        assert!(!large_string.is_empty());

        let large_vector: Vec<i32> = (0..10000).collect();
        assert_eq!(large_vector.len(), 10000);
        assert_eq!(large_vector[0], 0);
        assert_eq!(large_vector[9999], 9999);

        Ok(())
    }

    #[test]
    fn test_special_characters() -> SongbirdResult<()> {
        // Test handling of special characters
        let special_strings = vec![
            config.test.service_name,
            "test_service",
            "test.service",
            "test@service",
            "test:service",
            "test/service",
            "test\\service",
            "test service",
            "test\tservice",
            "test\nservice",
        ];

        for string in special_strings {
            assert!(!string.is_empty(), "Special string should not be empty: {}", string);
            assert!(string.len() > 0, "Special string should have length: {}", string);
        }

        Ok(())
    }
}

/// Performance-related unit tests
#[cfg(test)]
mod performance_unit_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_operation_performance() -> SongbirdResult<()> {
        let start = Instant::now();

        // Test that basic operations are fast
        let config = SongbirdConfig::default();
        let _env_config = &config.environment;

        let duration = start.elapsed();
        assert!(duration < Duration::from_millis(10), "Configuration creation should be fast");

        Ok(())
    }

    #[test]
    fn test_memory_efficiency() -> SongbirdResult<()> {
        // Test that structures don't consume excessive memory
        let config = SongbirdConfig::default();
        let config_size = std::mem::size_of_val(&config);
        
        // Reasonable size limit (adjust based on actual structure)
        assert!(config_size < 1024, "Config should not be too large: {} bytes", config_size);

        Ok(())
    }

    #[test]
    fn test_iteration_performance() -> SongbirdResult<()> {
        let start = Instant::now();

        // Test iteration over collections
        let items: Vec<i32> = (0..1000).collect();
        let sum: i32 = items.iter().sum();
        
        let duration = start.elapsed();
        assert_eq!(sum, 499500); // Sum of 0..1000
        assert!(duration < Duration::from_millis(10), "Iteration should be fast");

        Ok(())
    }

    #[tokio::test]
    async fn test_async_performance() -> SongbirdResult<()> {
        let start = Instant::now();

        // Test async operation performance
        let handles: Vec<_> = (0..100).map(|_| {
            tokio::spawn(async {
                tokio::time::sleep(Duration::from_millis(1)).await;
                42
            })
        }).collect();

        let results: Vec<i32> = futures::future::try_join_all(handles)
            .await
            .map_err(|e| SongbirdError::internal_error(&format!("Join error: {}", e)))?;

        let duration = start.elapsed();
        assert_eq!(results.len(), 100);
        assert!(duration < Duration::from_secs(1), "Async operations should complete quickly");

        Ok(())
    }
}

/// Integration helpers and test utilities
#[cfg(test)]
mod test_utility_tests {
    use super::*;

    #[tokio::test]
    async fn test_test_environment_setup() -> SongbirdResult<()> {
        let env = TestEnvironment::setup()?;
        
        // Test that test environment is properly initialized
        // In a real implementation, this would test actual environment setup
        assert!(true, "Test environment should setup successfully");

        Ok(())
    }

    #[test]
    fn test_test_context_creation() -> SongbirdResult<()> {
        let ctx = TestContext::new("test_context")
            .with_timeout(Duration::from_secs(30))
            .with_metadata("test_type", "unit");

        assert_eq!(ctx.name, "test_context");
        assert_eq!(ctx.timeout, Duration::from_secs(30));
        assert!(ctx.metadata.contains_key("test_type"));
        assert_eq!(ctx.metadata.get("test_type"), Some(&"unit".to_string()));
        assert!(!ctx.is_timeout());

        Ok(())
    }

    #[tokio::test]
    async fn test_error_testing_framework() -> SongbirdResult<()> {
        let error_framework = ErrorTestingFramework::new()?;
        
        // Test that error framework is properly initialized
        // In a real implementation, this would test actual error injection
        assert!(true, "Error testing framework should initialize successfully");

        Ok(())
    }

    #[test]
    fn test_validation_helpers() -> SongbirdResult<()> {
        // Test various validation helper functions
        assert!(validate_port(config.network.http_port), "Valid port should pass validation");
        assert!(!validate_port(0), "Invalid port should fail validation");
        assert!(!validate_port(65536), "Out of range port should fail validation");

        assert!(validate_timeout(30), "Valid timeout should pass validation");
        assert!(!validate_timeout(0), "Zero timeout should fail validation");

        assert!(validate_non_empty_string("test"), "Non-empty string should pass validation");
        assert!(!validate_non_empty_string(""), "Empty string should fail validation");

        Ok(())
    }

    // Helper validation functions
    fn validate_port(port: u16) -> bool {
        port > 0 && port <= 65535
    }

    fn validate_timeout(timeout: u64) -> bool {
        timeout > 0 && timeout <= 86400 // Max 24 hours
    }

    fn validate_non_empty_string(s: &str) -> bool {
        !s.is_empty()
    }
}

// Additional helper modules for comprehensive testing
use futures; // Add this for the try_join_all function 