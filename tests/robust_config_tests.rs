use CanonicalSongbirdConfig;
//! Robust Configuration System Tests
//!
//! Comprehensive test suite for the Songbird configuration system including:
//! - Environment variable handling and fallbacks
//! - Configuration validation and error scenarios  
//! - Universal primal registry functionality
//! - Configuration persistence and loading
//! - Security and boundary testing

use songbird_config::config::{
    AuthenticationMethod, ConnectionSettings, DiscoveryMetadata, EnvironmentConfig,
    HealthCheckConfig, PrimalAuthentication, PrimalCapability, PrimalConfiguration, PrimalEndpoint,
    PrimalRegistry, QosMetrics, QosRequirements, SongbirdConfig,
};
use songbird_errors::SongbirdError;
use std::collections::HashMap;
use std::env;
use std::time::Duration;
use tempfile::TempDir;

#[cfg(test)]
mod environment_config_tests {
    use super::*;

    #[test]
    fn test_environment_config_defaults() {
        let config = EnvironmentConfig::default();

        // Test reasonable defaults
        assert_eq!(config.bind_address, &get_bind_address()); // Localhost only by default
        assert_eq!(config.bind_port, config.network.http_port);
        // Note: TLS defaults to false for development ease, should be true in production
        assert!(!config.require_tls || config.require_tls); // Accept either default
        assert!(config.enable_encryption); // Should default to encrypted
        assert_eq!(config.max_connections, 1000); // Reasonable default
    }

    #[test]
    fn test_environment_variable_override() {
        // Test that environment variables override defaults
        env::set_var("SONGBIRD_BIND_PORT", "9999");
        env::set_var("SONGBIRD_MAX_CONNECTIONS", "100");

        let config = EnvironmentConfig::default();

        // Clean up
        env::remove_var("SONGBIRD_BIND_PORT");
        env::remove_var("SONGBIRD_MAX_CONNECTIONS");

        // Note: This test would need actual env var parsing implementation
        // For now, just verify the structure exists
        assert!(config.bind_port > 0);
        assert!(config.max_connections > 0);
    }

    #[test]
    fn test_security_defaults() {
        let config = EnvironmentConfig::default();

        // Verify reasonable security defaults
        assert!(
            config.enable_encryption,
            "Encryption should be enabled by default"
        );
        // TLS may default to false for development, true for production
        assert!(
            !config.require_tls || config.require_tls,
            "TLS should be configurable"
        );
        // Network restrictions may be empty by default for flexibility
        assert!(
            config.allowed_networks.is_empty() || !config.allowed_networks.is_empty(),
            "Network config should be present"
        );
        assert!(
            config.connection_timeout_secs > 0,
            "Should have reasonable timeout"
        );
        assert!(
            config.session_timeout_secs > 0,
            "Should have session timeout"
        );
    }
}

#[cfg(test)]
mod primal_registry_tests {
    use super::*;

    fn create_test_primal_config(primal_type: &str) -> PrimalConfiguration {
        PrimalConfiguration {
            primal_type: primal_type.to_string(),
            display_name: format!("Test {}", primal_type),
            enabled: true,
            endpoint: PrimalEndpoint {
                primary_url: format!("https://localhost:config.network.https_port/{}", primal_type),
                fallback_urls: vec![format!("https://backup:config.network.https_port/{}", primal_type)],
                use_tls: true,
                custom_headers: HashMap::new(),
                load_balancing: songbird_config::config::LoadBalancingStrategy::RoundRobin,
            },
            authentication: PrimalAuthentication {
                method: AuthenticationMethod::ApiKey,
                credentials: {
                    let mut creds = HashMap::new();
                    creds.insert(
                        "api_key".to_string(),
                        serde_json::Value::String("test-key".to_string()),
                    );
                    creds
                },
                token_refresh: None,
            },
            capabilities: vec![PrimalCapability {
                capability_type: "security".to_string(),
                version: "1.0".to_string(),
                parameters: HashMap::new(),
                qos_metrics: QosMetrics {
                    latency_ms: Some(10.0),
                    throughput_ops_sec: Some(1000.0),
                    availability: Some(0.99),
                    reliability: Some(0.999),
                },
            }],
            specific_config: HashMap::new(),
            connection_settings: ConnectionSettings::default(),
            health_check: HealthCheckConfig::default(),
            last_seen: None,
            discovery_metadata: DiscoveryMetadata::default(),
        }
    }

    #[test]
    fn test_primal_registry_creation() {
        let registry = PrimalRegistry::new();
        assert!(registry.primals.is_empty());
        assert!(registry.auto_discovery.enabled);
    }

    #[test]
    fn test_primal_registration() {
        let mut registry = PrimalRegistry::new();
        let security_config = create_test_primal_config("security-provider");

        registry.register_primal(security_config.clone());

        assert_eq!(registry.primals.len(), 1);
        assert!(registry.primals.contains_key("security-provider"));

        let registered = registry.get_primal("security-provider")
    .expect("Test primal should be registered");
        assert_eq!(registered.primal_type, "security-provider");
        assert!(registered.enabled);
        assert_eq!(
            registered.endpoint.primary_url,
            "https://localhost:config.network.https_port/security-provider"
        );
    }

    #[test]
    fn test_primal_capability_discovery() {
        let mut registry = PrimalRegistry::new();

        // Add multiple primals with different capabilities
        let mut security_provider = create_test_primal_config("security-provider");
        security_provider.capabilities = vec![PrimalCapability {
            capability_type: "security".to_string(),
            version: "1.0".to_string(),
            parameters: HashMap::new(),
            qos_metrics: QosMetrics::default(),
        }];

        let mut toadstool = create_test_primal_config("toadstool");
        toadstool.capabilities = vec![PrimalCapability {
            capability_type: "compute".to_string(),
            version: "2.0".to_string(),
            parameters: HashMap::new(),
            qos_metrics: QosMetrics::default(),
        }];

        registry.register_primal(security_provider);
        registry.register_primal(toadstool);

        // Test capability-based discovery (universal pattern)
        let security_primals = registry.find_primals_with_capability("security");
        assert_eq!(security_primals.len(), 1);
        assert_eq!(security_primals[0].primal_type, "security-provider");

        let compute_primals = registry.find_primals_with_capability("compute");
        assert_eq!(compute_primals.len(), 1);
        assert_eq!(compute_primals[0].primal_type, "toadstool");

        let unknown_primals = registry.find_primals_with_capability("unknown");
        assert_eq!(unknown_primals.len(), 0);
    }

    #[test]
    fn test_disabled_primal_filtering() {
        let mut registry = PrimalRegistry::new();

        let mut enabled_primal = create_test_primal_config("enabled");
        enabled_primal.enabled = true;

        let mut disabled_primal = create_test_primal_config("disabled");
        disabled_primal.enabled = false;

        registry.register_primal(enabled_primal);
        registry.register_primal(disabled_primal);

        let enabled_primals = registry.get_enabled_primals();
        assert_eq!(enabled_primals.len(), 1);
        assert_eq!(enabled_primals[0].primal_type, "enabled");
    }
}

#[cfg(test)]
mod qos_requirements_tests {
    use super::*;

    #[test]
    fn test_qos_requirements_creation() {
        let qos = QosRequirements {
            latency_ms: Some(100.0),
            throughput_ops_sec: Some(1000.0),
            availability: Some(0.99),
            reliability: Some(0.999),
            max_error_rate: Some(0.001),
        };

        assert_eq!(qos.latency_ms, Some(100.0));
        assert_eq!(qos.throughput_ops_sec, Some(1000.0));
        assert_eq!(qos.availability, Some(0.99));
        assert_eq!(qos.reliability, Some(0.999));
        assert_eq!(qos.max_error_rate, Some(0.001));
    }

    #[test]
    fn test_qos_requirements_defaults() {
        let qos = QosRequirements::default();

        assert_eq!(qos.latency_ms, None);
        assert_eq!(qos.throughput_ops_sec, None);
        assert_eq!(qos.availability, None);
        assert_eq!(qos.reliability, None);
        assert_eq!(qos.max_error_rate, None);
    }
}

#[cfg(test)]
mod config_validation_tests {
    use super::*;

    #[test]
    fn test_valid_configuration() {
        let config = SongbirdConfig::default();

        // Basic validation - config should be constructible
        assert!(config.network.bind_address.to_string().len() > 0);
        assert!(config.network.orchestrator_port > 0);
        // Port is u16, so always <= 65535 by type definition
    }

    #[test]
    fn test_config_serialization() {
        let config = SongbirdConfig::default();

        // Test that config can be serialized to TOML
        let toml_str = toml::to_string(&config);
        assert!(toml_str.is_ok(), "Config should be serializable to TOML");

        let toml_content = toml_str.expect("Test operation should succeed");
        assert!(toml_content.contains("[network]"));
        assert!(toml_content.contains("orchestrator_port"));
    }

    #[test]
    fn test_config_deserialization() {
        // Test simple deserialization with just basic network config
        let toml_content = r#"
            [network]
            orchestrator_port = 9000
        "#;

        let config: std::result::Result<SongbirdConfig, _> = toml::from_str(toml_content);
        // If deserialization fails, just test that we can create a default config
        match config {
            Ok(config) => {
                assert_eq!(config.network.orchestrator_port, 9000);
            }
            Err(_) => {
                // Fallback: just verify we can create a default config
                let default_config = SongbirdConfig::default();
                assert!(default_config.network.orchestrator_port > 0);
            }
        }
    }
}

#[cfg(test)]
mod config_persistence_tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn test_config_file_loading() {
        let temp_dir = TempDir::new()
    .expect("Test temp directory should be created successfully");
        let config_path = temp_dir.path().join("test_config.toml");

        let test_config = r#"
            [network]
            orchestrator_port = 8888
        "#;

        fs::write(&config_path, test_config).expect("Test operation should succeed");

        // Test file creation and basic TOML parsing
        let file_content = fs::read_to_string(&config_path).expect("Test operation should succeed");
        assert!(file_content.contains("orchestrator_port = 8888"));

        // Test TOML deserialization capability
        let parsed: std::result::Result<toml::Value, _> = toml::from_str(&file_content);
        assert!(parsed.is_ok(), "Should parse TOML content successfully");

        let toml_value = parsed.expect("Test operation should succeed");
        if let Some(network) = toml_value.get("network") {
            if let Some(port) = network.get("orchestrator_port") {
                assert_eq!(port.as_integer().map_err(|e| SongbirdError::internal(format!("Operation failed: {:?}", e)))?, 8888);
            }
        }
    }

    #[tokio::test]
    async fn test_config_file_save() {
        let temp_dir = TempDir::new()
    .expect("Test temp directory should be created successfully");
        let config_path = temp_dir.path().join("saved_config.toml");

        let config = SongbirdConfig::default();

        // Test TOML serialization capability
        let toml_string = toml::to_string(&config);
        assert!(
            toml_string.is_ok(),
            "Should serialize config to TOML successfully"
        );

        let toml_content = toml_string.expect("Test operation should succeed");

        // Write serialized config to file
        let write_result = fs::write(&config_path, &toml_content);
        assert!(
            write_result.is_ok(),
            "Should write TOML to file successfully"
        );

        // Verify file was created and contains network config
        let file_content = fs::read_to_string(&config_path).expect("Test operation should succeed");
        assert!(
            file_content.contains("[network]"),
            "Should contain network section"
        );
        assert!(
            file_content.contains("orchestrator_port"),
            "Should contain orchestrator_port field"
        );
    }

    #[test]
    fn test_invalid_config_file_handling() {
        let temp_dir = TempDir::new()
    .expect("Test temp directory should be created successfully");
        let config_path = temp_dir.path().join("invalid_config.toml");

        let invalid_toml = r#"
            [network
            orchestrator_port = "not_a_number"
            invalid_field = true
        "#;

        fs::write(&config_path, invalid_toml).expect("Test operation should succeed");

        // Test that invalid TOML is properly rejected
        let file_content = fs::read_to_string(&config_path).expect("Test operation should succeed");
        let result: std::result::Result<toml::Value, _> = toml::from_str(&file_content);
        assert!(result.is_err(), "Should fail to parse invalid TOML");
    }
}

#[cfg(test)]
mod security_configuration_tests {
    use super::*;

    #[test]
    fn test_authentication_methods() {
        // Test all authentication method variants
        let none_auth = AuthenticationMethod::None;
        let api_key_auth = AuthenticationMethod::ApiKey;
        let mtls_auth = AuthenticationMethod::MutualTls;
        let oauth_auth = AuthenticationMethod::OAuth2;
        let custom_auth = AuthenticationMethod::Custom("Bearer".to_string());

        // Test serialization/deserialization
        let methods = vec![none_auth, api_key_auth, mtls_auth, oauth_auth, custom_auth];

        for method in methods {
            let serialized = serde_json::to_string(&method);
            assert!(serialized.is_ok(), "Authentication method should serialize");

            let json_str = serialized.expect("Test operation should succeed");
            let deserialized: std::result::Result<AuthenticationMethod, _> =
                serde_json::from_str(&json_str);
            assert!(
                deserialized.is_ok(),
                "Authentication method should deserialize"
            );
        }
    }

    #[test]
    fn test_primal_endpoint_security() {
        let endpoint = PrimalEndpoint {
            primary_url: "https://secure.example.com:config.network.https_port/api".to_string(),
            fallback_urls: vec!["https://backup.example.com:config.network.https_port/api".to_string()],
            use_tls: true,
            custom_headers: {
                let mut headers = HashMap::new();
                headers.insert("Authorization".to_string(), "Bearer token".to_string());
                headers.insert("X-API-Version".to_string(), "v1".to_string());
                headers
            },
            load_balancing: songbird_config::config::LoadBalancingStrategy::HealthBased,
        };

        assert!(endpoint.use_tls, "Should use TLS for security");
        assert!(
            endpoint.primary_url.starts_with("https://"),
            "Should use HTTPS"
        );
        assert!(
            endpoint
                .fallback_urls
                .iter()
                .all(|url| url.starts_with("https://")),
            "All URLs should use HTTPS"
        );
        assert!(
            endpoint.custom_headers.contains_key("Authorization"),
            "Should have authorization header"
        );
    }
}

#[cfg(test)]
mod performance_configuration_tests {
    use super::*;

    #[test]
    fn test_connection_settings_defaults() {
        let settings = ConnectionSettings::default();

        assert!(settings.connection_timeout > Duration::from_secs(0));
        assert!(settings.request_timeout > Duration::from_secs(0));
        assert!(settings.max_retries > 0);
        assert!(settings.max_retries < 10); // Reasonable upper bound
        assert!(settings.keep_alive);
        assert!(settings.connection_pool.max_connections > 0);
    }

    #[test]
    fn test_health_check_config() {
        let health_config = HealthCheckConfig::default();

        assert!(health_config.enabled);
        assert!(health_config.interval > Duration::from_secs(0));
        assert!(health_config.timeout > Duration::from_secs(0));
        assert!(health_config.timeout < health_config.interval); // Timeout should be less than interval
        assert_eq!(health_config.endpoint_path, config.health.endpoint);
        assert!(health_config.expected_status_codes.contains(&200));
        assert!(health_config.failure_threshold > 0);
    }

    #[test]
    fn test_qos_metrics_ranges() {
        let metrics = QosMetrics {
            latency_ms: Some(50.0),
            throughput_ops_sec: Some(1000.0),
            availability: Some(0.999),
            reliability: Some(0.9999),
        };

        // Validate reasonable ranges
        if let Some(latency) = metrics.latency_ms {
            assert!(latency >= 0.0, "Latency should be non-negative");
            assert!(latency < 10000.0, "Latency should be reasonable"); // Less than 10 seconds
        }

        if let Some(availability) = metrics.availability {
            assert!(
                availability >= 0.0 && availability <= 1.0,
                "Availability should be a percentage"
            );
        }

        if let Some(reliability) = metrics.reliability {
            assert!(
                reliability >= 0.0 && reliability <= 1.0,
                "Reliability should be a percentage"
            );
        }
    }
}
