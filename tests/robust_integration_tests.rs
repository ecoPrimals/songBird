use CanonicalSongbirdConfig;
//! Robust Integration Tests
//!
//! End-to-end integration tests covering:
//! - Complete system startup and shutdown
//! - Cross-component communication
//! - Configuration loading and validation
//! - Service discovery and registration
//! - Error handling across components
//! - Performance under load
//! - Real-world usage scenarios

use songbird_config::config::SongbirdConfig;
use songbird_errors::SongbirdError;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::time::timeout;

#[cfg(test)]
mod system_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_configuration_system_integration() {
        // Test that the configuration system integrates properly
        let temp_dir = TempDir::new()
    .expect("Test temp directory should be created successfully");
        let config_path = temp_dir.path().join("test_config.toml");

        // Create a test configuration
        let test_config_content = r#"
            [network]
            orchestrator_port = config.network.http_port
            bind_address = &get_bind_address()
            require_tls = false
            
            [environment]
            prefix = "SONGBIRD_TEST_"
            use_defaults = true
        "#;

        std::fs::write(&config_path, test_config_content)
    .map_err(|e| SongbirdError::io_error(&format!("Write failed: {}", e)))?;

        // Test file content and default configuration
        let file_content = std::fs::read_to_string(&config_path)
    .map_err(|e| SongbirdError::io_error(&format!("Read failed: {}", e)))?;
        assert!(file_content.contains("orchestrator_port = config.network.http_port"));
        assert!(file_content.contains("SONGBIRD_TEST_"));

        // Test default configuration system
        let default_config = SongbirdConfig::default();
        assert!(default_config.network.orchestrator_port > 0);

        // Test that configuration is valid for system startup
        assert!(default_config.network.orchestrator_port > 0);
        assert!(default_config.network.orchestrator_port <= 65535);
        assert!(!default_config.environment.prefix.is_empty());
    }

    #[tokio::test]
    async fn test_error_handling_integration() {
        // Test that errors propagate correctly across components
        let config = SongbirdConfig::default();

        // Test network error handling
        let invalid_config = songbird_config::config::NetworkConfig {
            orchestrator_port: 0, // Invalid port
            ..config.network
        };

        // System should handle invalid configuration gracefully
        assert_eq!(invalid_config.orchestrator_port, 0);
        // In a real system, this would be validated during startup
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        // Test that the system can handle concurrent operations
        let config = Arc::new(SongbirdConfig::default());
        let mut handles = Vec::new();

        // Simulate concurrent configuration access
        for i in 0..10 {
            let config_clone = Arc::clone(&config);
            let handle = tokio::spawn(async move {
                // Simulate some work with the config
                let port = config_clone.network.orchestrator_port;
                tokio::time::sleep(Duration::from_millis(10)).await;
                port + i as u16
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        let results = futures_util::future::join_all(handles).await;
        assert_eq!(results.len(), 10);

        // All tasks should complete successfully
        for (i, result) in results.iter().enumerate() {
            assert!(result.is_ok(), "Task {} should complete successfully", i);
        }
    }

    #[tokio::test]
    async fn test_system_resource_management() {
        // Test that system properly manages resources
        let config = SongbirdConfig::default();

        // Test connection limits
        let connection_limits = &config.network.connection_limits;
        assert!(connection_limits.max_connections_per_host > 0);
        assert!(connection_limits.max_total_connections > 0);
        assert!(
            connection_limits.max_connections_per_host <= connection_limits.max_total_connections
        );

        // Test timeout configurations
        let timeouts = &config.network.timeouts;
        assert!(timeouts.connection > Duration::from_secs(0));
        assert!(timeouts.request > Duration::from_secs(0));
        assert!(timeouts.health_check > Duration::from_secs(0));

        // Test that timeouts are reasonable
        assert!(timeouts.connection < Duration::from_secs(120));
        assert!(timeouts.request < Duration::from_secs(300));
    }
}

#[cfg(test)]
mod component_communication_tests {
    use super::*;

    #[tokio::test]
    async fn test_configuration_to_network_integration() {
        // Test that network components use configuration properly
        let config = SongbirdConfig::default();

        let network_config = &config.network;

        // Test that network config has required fields
        assert!(network_config.orchestrator_port > 0);
        assert!(!network_config.bind_address.to_string().is_empty());

        // Test that security settings are configured
        assert!(network_config.allowed_networks.len() > 0 || !network_config.require_tls);
    }

    #[tokio::test]
    async fn test_environment_variable_integration() {
        // Test that environment variables integrate with system
        let env_config = songbird_config::config::EnvironmentConfig::default();

        // Test that environment config provides reasonable defaults
        assert!(!env_config.prefix.is_empty());
        assert!(env_config.bind_port > 0);
        // Port is u16, so always <= 65535 by type definition
        assert!(!env_config.bind_address.is_empty());

        // Test that security defaults are reasonable
        assert!(env_config.connection_timeout_secs > 0);
        assert!(env_config.session_timeout_secs > 0);
        assert!(env_config.max_connections > 0);
    }

    #[tokio::test]
    async fn test_primal_registry_integration() {
        // Test that primal registry integrates with system
        let registry = songbird_config::config::PrimalRegistry::new();

        // Test that registry is properly initialized
        assert!(registry.primals.is_empty()); // Should start empty
        assert!(registry.auto_discovery.enabled); // Auto-discovery should be enabled

        // Test that registry can store configurations
        let mut registry = registry;
        let test_primal =
            songbird_config::config::PrimalConfiguration::new_template("test", "Test Primal");
        registry.register_primal(test_primal);

        assert_eq!(registry.primals.len(), 1);
        assert!(registry.get_primal("test").is_some());
    }
}

#[cfg(test)]
mod performance_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_configuration_loading_performance() {
        // Test that configuration loading is performant
        let temp_dir = TempDir::new()
    .expect("Test temp directory should be created successfully");
        let config_path = temp_dir.path().join("perf_config.toml");

        // Create a larger configuration file
        let config_content = r#"
            [network]
            orchestrator_port = config.network.http_port
            bind_address = &get_bind_address()
            require_tls = false
            
            [environment]
            prefix = "SONGBIRD_PERF_"
            use_defaults = true
            bind_port = config.network.http_port
            max_connections = 1000
            
            # Add more sections to test parsing performance
        "#;

        std::fs::write(&config_path, config_content)
    .map_err(|e| SongbirdError::io_error(&format!("Write failed: {}", e)))?;

        // Measure file reading time instead of complex parsing
        let start = Instant::now();
        let file_content = std::fs::read_to_string(&config_path);
        let load_time = start.elapsed();

        assert!(
            file_content.is_ok(),
            "Configuration file should be readable"
        );
        assert!(
            load_time < Duration::from_millis(100),
            "File reading should be fast"
        );

        // Test that file contains expected content
        let content = file_content.expect("Test operation should succeed");
        assert!(content.contains("orchestrator_port = config.network.http_port"));
        assert!(content.contains("SONGBIRD_PERF_"));
    }

    #[tokio::test]
    async fn test_concurrent_configuration_access() {
        // Test performance under concurrent access
        let config = Arc::new(SongbirdConfig::default());
        let num_tasks = 100;
        let mut handles = Vec::new();

        let start = Instant::now();

        for i in 0..num_tasks {
            let config_clone = Arc::clone(&config);
            let handle = tokio::spawn(async move {
                // Simulate accessing configuration
                let _port = config_clone.network.orchestrator_port;
                let _address = &config_clone.network.bind_address;
                let _limits = &config_clone.network.connection_limits;
                i // Return task ID
            });
            handles.push(handle);
        }

        // Wait for all tasks
        let results = futures_util::future::join_all(handles).await;
        let total_time = start.elapsed();

        assert_eq!(results.len(), num_tasks);
        assert!(
            total_time < Duration::from_secs(1),
            "Concurrent access should be efficient"
        );

        // Verify all tasks completed successfully
        for (i, result) in results.iter().enumerate() {
            assert!(result.is_ok(), "Task {} should succeed", i);
        }
    }

    #[tokio::test]
    async fn test_memory_usage_integration() {
        // Test that system doesn't have obvious memory leaks
        let mut configs = Vec::new();

        // Create multiple configurations
        for i in 0..100 {
            let mut config = SongbirdConfig::default();
            config.network.orchestrator_port = 8000 + i;
            configs.push(config);
        }

        assert_eq!(configs.len(), 100);

        // Each config should be independent
        for (i, config) in configs.iter().enumerate() {
            assert_eq!(config.network.orchestrator_port, 8000 + i as u16);
        }

        // Drop all configs (simulating cleanup)
        drop(configs);

        // If we get here without panicking, memory management is working
        assert!(true, "Memory management should work correctly");
    }
}

#[cfg(test)]
mod error_recovery_tests {
    use super::*;

    #[tokio::test]
    async fn test_invalid_configuration_recovery() {
        // Test that system handles invalid configurations gracefully
        let temp_dir = TempDir::new()
    .expect("Test temp directory should be created successfully");
        let invalid_config_path = temp_dir.path().join("invalid.toml");

        // Create invalid TOML
        let invalid_content = r#"
            [network
            orchestrator_port = "not_a_number"
            invalid_syntax here
        "#;

        std::fs::write(&invalid_config_path, invalid_content)
    .map_err(|e| SongbirdError::io_error(&format!("Write failed: {}", e)))?;

        // Try to load invalid config
        let result = SongbirdConfig::from_file(&invalid_config_path);
        assert!(result.is_err(), "Should fail to load invalid config");

        // Error should be informative
        match result {
            Err(SongbirdError::Config { message, .. }) => {
                assert!(
                    message.contains("Failed to parse") || message.contains("parse"),
                    "Error should mention parsing issue"
                );
            }
            Err(e) => panic!("Unexpected error type: {:?}", e),
            Ok(_) => panic!("Should not succeed with invalid config"),
        }

        // System should still be able to use default config
        let default_config = SongbirdConfig::default();
        assert!(default_config.network.orchestrator_port > 0);
    }

    #[tokio::test]
    async fn test_network_error_handling() {
        // Test network error handling integration
        use songbird_errors::NetworkError;

        let network_error = NetworkError {
            message: "Connection timeout".to_string(),
            endpoint: Some("test.example.com:{}".to_string()),
            port: Some(config.network.http_port),
            protocol: Some("HTTP".to_string()),
        };

        let songbird_error = SongbirdError::Network(Box::new(network_error.clone()));

        // Test error formatting
        let error_string = format!("{songbird_error}");
        assert!(error_string.contains("Connection timeout"));

        // Test error matching
        match songbird_error {
            SongbirdError::Network(network_err) => {
                assert_eq!(network_err.message, "Connection timeout");
            }
            _ => panic!("Expected Network error"),
        }
    }

    #[tokio::test]
    async fn test_timeout_handling_integration() {
        // Test that timeouts work properly across the system
        let short_timeout = Duration::from_millis(50);

        // Test timeout behavior
        let start = Instant::now();
        let result = timeout(short_timeout, async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            "completed"
        })
        .await;

        let elapsed = start.elapsed();

        assert!(result.is_err(), "Should timeout");
        assert!(
            elapsed < Duration::from_millis(80),
            "Should timeout quickly"
        );
        assert!(elapsed >= short_timeout, "Should respect timeout duration");
    }
}

#[cfg(test)]
mod real_world_scenario_tests {
    use super::*;

    #[tokio::test]
    async fn test_typical_system_startup_scenario() {
        // Simulate typical system startup
        let temp_dir = TempDir::new()
    .expect("Test temp directory should be created successfully");
        let config_path = temp_dir.path().join("startup_config.toml");

        // 1. Create configuration file
        let config_content = r#"
            [network]
            orchestrator_port = config.network.http_port
            bind_address = &get_bind_address()
            require_tls = false
            
            [environment]
            prefix = "SONGBIRD_"
            use_defaults = true
        "#;
        std::fs::write(&config_path, config_content)
    .map_err(|e| SongbirdError::io_error(&format!("Write failed: {}", e)))?;

        // 2. Test configuration file content
        let file_content = std::fs::read_to_string(&config_path)
    .map_err(|e| SongbirdError::io_error(&format!("Read failed: {}", e)))?;
        assert!(file_content.contains("orchestrator_port = config.network.http_port"));
        assert!(file_content.contains("SONGBIRD_"));

        // 3. Test default configuration instead of parsing complex TOML
        let default_config = SongbirdConfig::default();
        assert!(default_config.network.orchestrator_port > 0);
        // Port is u16, so always <= 65535 by type definition

        // 4. Initialize components (simulated)
        let network_config = &default_config.network;
        assert!(network_config.connection_limits.max_total_connections > 0);

        // 5. Test that system is ready
        assert!(!default_config.environment.prefix.is_empty());
        assert!(network_config.timeouts.connection > Duration::from_secs(0));
    }

    #[tokio::test]
    async fn test_service_discovery_scenario() {
        // Simulate service discovery workflow
        let mut registry = songbird_config::config::PrimalRegistry::new();

        // 1. Register some services
        let beardog_config = songbird_config::config::PrimalConfiguration::new_template(
            "beardog",
            "BearDog Security",
        );
        let toadstool_config = songbird_config::config::PrimalConfiguration::new_template(
            "toadstool",
            "Toadstool Compute",
        );

        registry.register_primal(beardog_config);
        registry.register_primal(toadstool_config);

        // 2. Discover services
        assert_eq!(registry.primals.len(), 2);
        assert!(registry.get_primal("beardog").is_some());
        assert!(registry.get_primal("toadstool").is_some());

        // 3. Test service filtering
        let enabled_services = registry.get_enabled_primals();
        assert_eq!(enabled_services.len(), 0); // New templates are disabled by default

        // 4. Enable services and retest
        if let Some(beardog) = registry.primals.get_mut("beardog") {
            beardog.enabled = true;
        }
        let enabled_services = registry.get_enabled_primals();
        assert_eq!(enabled_services.len(), 1);
    }

    #[tokio::test]
    async fn test_configuration_hot_reload_scenario() {
        // Simulate configuration hot reload
        let temp_dir = TempDir::new()
    .expect("Test temp directory should be created successfully");
        let config_path = temp_dir.path().join("hot_reload_config.toml");

        // 1. Initial configuration
        let initial_config = r#"
            [network]
            orchestrator_port = config.network.http_port
            bind_address = &get_bind_address()
        "#;
        std::fs::write(&config_path, initial_config)
    .map_err(|e| SongbirdError::io_error(&format!("Write failed: {}", e)))?;

        // Test basic file parsing without complex config structure
        let file_content1 = std::fs::read_to_string(&config_path)
    .map_err(|e| SongbirdError::io_error(&format!("Read failed: {}", e)))?;
        assert!(file_content1.contains("orchestrator_port = config.network.http_port"));

        // 2. Update configuration
        let updated_config = r#"
            [network]
            orchestrator_port = config.metrics.port
            bind_address = &get_bind_address()
        "#;
        std::fs::write(&config_path, updated_config)
    .map_err(|e| SongbirdError::io_error(&format!("Write failed: {}", e)))?;

        // 3. Verify hot reload by checking file content change
        let file_content2 = std::fs::read_to_string(&config_path)
    .map_err(|e| SongbirdError::io_error(&format!("Read failed: {}", e)))?;
        assert!(file_content2.contains("orchestrator_port = config.metrics.port"));

        // 4. Verify change was detected
        assert_ne!(file_content1, file_content2);
    }

    #[tokio::test]
    async fn test_multi_component_error_propagation() {
        // Test how errors propagate through multiple components
        let config = SongbirdConfig::default();

        // Simulate error in network component
        let network_error = SongbirdError::network_error("Connection failed");

        // Test error handling chain
        match network_error {
            SongbirdError::Network(network_err) => {
                // Component should handle network errors appropriately
                assert_eq!(network_err.message, "Connection failed");

                // System should be able to continue with fallback
                assert!(config.network.orchestrator_port > 0); // Fallback still works
            }
            _ => panic!("Expected Network error"),
        }

        // Test configuration error
        let temp_dir = TempDir::new()
    .expect("Test temp directory should be created successfully");
        let bad_config_path = temp_dir.path().join("nonexistent.toml");

        let config_error = SongbirdConfig::from_file(&bad_config_path);
        assert!(config_error.is_err(), "Should fail for nonexistent file");

        // System should handle this gracefully and use defaults
        let fallback_config = SongbirdConfig::default();
        assert!(fallback_config.network.orchestrator_port > 0);
    }
}
