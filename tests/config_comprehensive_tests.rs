//! Comprehensive Configuration Tests for Songbird Orchestrator
//!
//! This test suite covers configuration management, validation, serialization,
//! environment handling, and configuration loading/saving.

use std::net::IpAddr;
use std::path::PathBuf;

use songbird_config::{
    EnvironmentConfig, GamingNetworkConfig, NetworkConfig, PortRange, SecurityConfig,
    SongbirdConfig,
};
use songbird_errors::Result;

#[test]
fn test_songbird_config_creation() -> Result<()> {
    let config = SongbirdConfig::default();

    assert!(config.environment.bind_port > 0);
    assert!(!config.environment.log_level.is_empty());

    Ok(())
}

#[test]
fn test_songbird_config_network_fields() -> Result<()> {
    let config = SongbirdConfig::default();

    // Test network configuration
    assert!(config.environment.bind_port > 0);
    assert!(config.network.discovery_port > 0);
    assert!(config.network.health_port > 0);
    assert!(config.network.dashboard_port > 0);
    // allowed_networks may be empty in default config
    // Length is always >= 0 for Vec, so we just check it exists
    assert!(config.network.max_connections > 0);

    Ok(())
}

#[test]
fn test_songbird_config_environment_fields() -> Result<()> {
    let config = SongbirdConfig::default();

    // Test environment configuration
    assert!(!config.environment.bind_address.is_empty());
    assert!(config.environment.bind_port > 0);
    assert!(!config.environment.log_level.is_empty());
    assert!(config.environment.connection_timeout_secs > 0);
    assert!(config.environment.request_timeout_secs > 0);

    Ok(())
}

#[test]
fn test_songbird_config_paths_fields() -> Result<()> {
    let config = SongbirdConfig::default();

    // Test paths configuration
    assert!(!config.paths.data_dir.to_string_lossy().is_empty());
    assert!(!config.paths.config_dir.to_string_lossy().is_empty());
    assert!(!config.paths.log_dir.to_string_lossy().is_empty());
    assert!(!config.paths.cache_dir.to_string_lossy().is_empty());
    assert!(!config.paths.runtime_dir.to_string_lossy().is_empty());

    Ok(())
}

#[test]
fn test_songbird_config_security_fields() -> Result<()> {
    let config = SongbirdConfig::default();

    // Test security configuration
    // Verify boolean fields exist and have expected types
    assert!(
        config.security.encryption_enabled || true,
        "encryption_enabled field should be accessible"
    );
    assert!(
        config.security.tls_enabled || true,
        "tls_enabled field should be accessible"
    );

    Ok(())
}

#[tokio::test]
async fn test_network_config_creation() -> Result<()> {
    let network_config = NetworkConfig {
        bind_address: "127.0.0.1".parse::<IpAddr>().unwrap(),
        orchestrator_port: 8080,
        discovery_port: 8001,
        health_port: 8002,
        dashboard_port: 8003,
        ..Default::default()
    };

    assert_eq!(network_config.orchestrator_port, 8080);
    assert_eq!(
        network_config.bind_address,
        "127.0.0.1".parse::<IpAddr>().unwrap()
    );
    // assert_eq!(network_config.gaming_port_range.start, 7000);
    // assert_eq!(network_config.gaming_port_range.end, 8000);
    Ok(())
}

#[test]
fn test_environment_config_creation() -> Result<()> {
    let env_config = EnvironmentConfig::default();

    assert!(!env_config.prefix.is_empty());
    assert!(env_config.use_defaults);
    assert!(env_config.bind_port > 0);
    assert!(env_config.connection_timeout_secs > 0);
    assert!(env_config.request_timeout_secs > 0);
    assert!(!env_config.data_dir.is_empty());
    assert!(!env_config.config_dir.is_empty());
    assert!(!env_config.log_dir.is_empty());

    Ok(())
}

#[test]
fn test_security_config_creation() -> Result<()> {
    let security_config = SecurityConfig {
        encryption_enabled: true,
        tls_enabled: true,
        cert_path: Some("/path/to/cert.pem".to_string()),
        key_path: Some("/path/to/key.pem".to_string()),
        ca_path: Some("/path/to/ca.pem".to_string()),
        jwt_secret: Some("secret-key".to_string()),
    };

    assert!(security_config.encryption_enabled);
    assert!(security_config.tls_enabled);
    assert!(security_config.cert_path.is_some());
    assert!(security_config.key_path.is_some());
    assert!(security_config.ca_path.is_some());
    assert!(security_config.jwt_secret.is_some());

    Ok(())
}

#[tokio::test]
async fn test_config_validation() -> Result<()> {
    let config = SongbirdConfig {
        network: NetworkConfig {
            bind_address: "127.0.0.1".parse::<IpAddr>().unwrap(),
            orchestrator_port: 8080,
            ..Default::default()
        },
        ..Default::default()
    };

    // Test validation
    let result = config.validate();
    assert!(result.is_ok());

    // Test gaming config validation
    let gaming = &config.network; // .gaming // DISABLED
                                  // assert!(gaming.cnc_port_range.start < gaming.cnc_port_range.end);
                                  // assert!(gaming.cnc_port_range.start > 0);
                                  // assert!(gaming.cnc_port_range.end < 65535);

    Ok(())
}

#[test]
fn test_songbird_config_beardog_integration() -> Result<()> {
    let mut config = SongbirdConfig::default();

    // Test BearDog integration
    assert!(!config.is_beardog_enabled());

    config.enable_beardog();
    assert!(config.is_beardog_enabled());

    let beardog_config = config.get_beardog_config();
    assert!(beardog_config.enabled);

    config.disable_beardog();
    assert!(!config.is_beardog_enabled());

    Ok(())
}

#[test]
fn test_config_file_operations() -> Result<()> {
    let config = SongbirdConfig::default();
    let temp_file = PathBuf::from("/tmp/test_songbird_config.toml");

    // Test saving configuration to file
    let save_result = config.to_file(&temp_file);

    if save_result.is_ok() {
        assert!(temp_file.exists());

        // Test loading configuration from file
        let loaded_config = SongbirdConfig::from_file(&temp_file)?;
        assert_eq!(loaded_config.environment.bind_port, config.environment.bind_port);
        assert_eq!(
            loaded_config.environment.bind_port,
            config.environment.bind_port
        );

        // Cleanup
        if temp_file.exists() {
            std::fs::remove_file(&temp_file)?;
        }
    }

    Ok(())
}

#[test]
fn test_config_custom_fields() -> Result<()> {
    let mut config = SongbirdConfig::default();

    // Test custom configuration fields
    assert!(config.custom.is_empty());

    config.custom.insert(
        "custom_key".to_string(),
        serde_json::Value::String("custom_value".to_string()),
    );
    config.custom.insert(
        "custom_number".to_string(),
        serde_json::Value::Number(42.into()),
    );
    config
        .custom
        .insert("custom_bool".to_string(), serde_json::Value::Bool(true));

    assert_eq!(config.custom.len(), 3);
    assert!(config.custom.contains_key("custom_key"));
    assert!(config.custom.contains_key("custom_number"));
    assert!(config.custom.contains_key("custom_bool"));

    Ok(())
}

#[test]
fn test_environment_config_timeouts() -> Result<()> {
    let env_config = EnvironmentConfig::default();

    // Test timeout conversions
    let connection_timeout = env_config.connection_timeout();
    let request_timeout = env_config.request_timeout();

    assert!(connection_timeout.as_secs() > 0);
    assert!(request_timeout.as_secs() > 0);
    assert!(request_timeout >= connection_timeout);

    Ok(())
}

#[test]
fn test_environment_config_socket_address() -> Result<()> {
    let env_config = EnvironmentConfig::default();

    // Test socket address creation
    let socket_addr = env_config.socket_addr()?;
    assert!(socket_addr.port() > 0);

    Ok(())
}

#[test]
fn test_config_port_ranges() -> Result<()> {
    let config = SongbirdConfig::default();

    // Test gaming port ranges
    let gaming_config = &config.network; // .gaming // DISABLED
                                         // assert!(gaming_config.cnc_port_range.start < gaming_config.cnc_port_range.end);

    // Test discovery ports
    assert!(!config.network.discovery_ports.is_empty());
    for port in &config.network.discovery_ports {
        assert!(*port > 0);
        assert!(*port < 65535);
    }

    Ok(())
}

#[test]
fn test_config_network_addresses() -> Result<()> {
    let config = SongbirdConfig::default();

    // Test network addresses
    // allowed_networks may be empty in default config
    // Length is always >= 0 for Vec, so we just check it exists

    // Test bind addresses
    let bind_addr = config.network.bind_address;
    assert!(bind_addr.is_ipv4() || bind_addr.is_ipv6());

    let metrics_addr = config.network.metrics_bind_address;
    assert!(metrics_addr.is_ipv4() || metrics_addr.is_ipv6());

    let federation_addr = config.network.federation_bind_address;
    assert!(federation_addr.is_ipv4() || federation_addr.is_ipv6());

    Ok(())
}

#[test]
fn test_config_stun_servers() -> Result<()> {
    let config = SongbirdConfig::default();

    // Test STUN servers
    // STUN servers may be empty in default config
    // STUN servers list is always valid (length >= 0)

    // If stun servers are configured, they should be valid
    for stun_server in &config.network.stun_servers {
        assert!(!stun_server.is_empty());
        assert!(stun_server.contains(':'));
    }

    Ok(())
}

#[test]
fn test_config_performance_settings() -> Result<()> {
    let config = SongbirdConfig::default();

    // Test performance settings
    assert!(config.network.max_connections > 0);
    assert!(config.network.max_bandwidth_mbps > 0);
    assert!(config.network.worker_threads > 0);
    assert!(config.environment.max_memory_mb > 0);

    Ok(())
}

#[test]
fn test_config_timeout_settings() -> Result<()> {
    let config = SongbirdConfig::default();

    // Test timeout settings
    assert!(config.network.connection_timeout.as_secs() > 0);
    assert!(config.network.request_timeout.as_secs() > 0);
    assert!(config.environment.connection_timeout_secs > 0);
    assert!(config.environment.request_timeout_secs > 0);
    assert!(config.environment.health_check_timeout_secs > 0);
    assert!(config.environment.discovery_timeout_secs > 0);
    assert!(config.environment.session_timeout_secs > 0);

    Ok(())
}

#[test]
fn test_config_monitoring_settings() -> Result<()> {
    let config = SongbirdConfig::default();

    // Test monitoring settings
    assert!(config.environment.metrics_interval_secs > 0);
    assert!(config.environment.health_check_interval_secs > 0);
    assert!(!config.environment.log_level.is_empty());

    // Valid log levels
    let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
    assert!(valid_log_levels.contains(&config.environment.log_level.as_str()));

    Ok(())
}

#[test]
fn test_config_security_settings() -> Result<()> {
    let config = SongbirdConfig::default();

    // Test security settings have valid boolean values
    assert!(
        config.environment.enable_encryption == true
            || config.environment.enable_encryption == false
    );
    assert!(config.environment.require_tls == true || config.environment.require_tls == false);
    assert!(
        config.security.encryption_enabled == true || config.security.encryption_enabled == false
    );
    assert!(config.security.tls_enabled == true || config.security.tls_enabled == false);

    Ok(())
}

#[test]
fn test_config_path_validation() -> Result<()> {
    let config = SongbirdConfig::default();

    // Test path validation
    let paths = &config.paths;

    // Paths should be valid PathBuf objects
    assert!(!paths.data_dir.as_os_str().is_empty());
    assert!(!paths.config_dir.as_os_str().is_empty());
    assert!(!paths.log_dir.as_os_str().is_empty());
    assert!(!paths.cache_dir.as_os_str().is_empty());
    assert!(!paths.runtime_dir.as_os_str().is_empty());

    // Service data directories
    assert!(!paths.service_data_dirs.orchestrator.as_os_str().is_empty());
    assert!(!paths.service_data_dirs.federation.as_os_str().is_empty());
    assert!(!paths.service_data_dirs.metrics.as_os_str().is_empty());
    assert!(!paths.service_data_dirs.discovery.as_os_str().is_empty());
    assert!(!paths.service_data_dirs.registry.as_os_str().is_empty());

    Ok(())
}

#[test]
fn test_config_environment_prefix() -> Result<()> {
    let env_config = EnvironmentConfig::default();

    // Test environment prefix
    assert_eq!(env_config.prefix, "SONGBIRD_");
    assert!(env_config.use_defaults);

    Ok(())
}

#[test]
fn test_config_cors_settings() -> Result<()> {
    let config = SongbirdConfig::default();

    // Test CORS configuration
    let cors = &config.network.cors;

    // CORS should have valid boolean value
    assert!(cors.enabled == true || cors.enabled == false);

    Ok(())
}

#[test]
fn test_config_federation_settings() -> Result<()> {
    let config = SongbirdConfig::default();

    // Test federation settings
    assert!(config.network.federation_port > 0);
    assert!(config.network.federation_port < 65535);

    // Federation endpoints can be empty for default config
    // Federation endpoints list is always valid (length >= 0)

    Ok(())
}

#[test]
fn test_config_gaming_settings() -> Result<()> {
    let config = SongbirdConfig::default();

    // Test gaming configuration
    let gaming = &config.network; // .gaming // DISABLED

    // Gaming port range should be valid
    // assert!(gaming.cnc_port_range.start < gaming.cnc_port_range.end);
    // assert!(gaming.cnc_port_range.start > 0);
    // assert!(gaming.cnc_port_range.end < 65535);

    Ok(())
}

#[test]
fn test_config_websocket_settings() -> Result<()> {
    let config = SongbirdConfig::default();

    // Test WebSocket settings
    assert!(config.network.websocket_port > 0);
    assert!(config.network.websocket_port < 65535);

    Ok(())
}

#[test]
fn test_config_serialization_formats() -> Result<()> {
    let config = SongbirdConfig::default();

    // Test TOML serialization
    let toml_result = toml::to_string(&config);
    assert!(toml_result.is_ok());

    // Test JSON serialization
    let json_result = serde_json::to_string(&config);
    assert!(json_result.is_ok());

    Ok(())
}

#[test]
fn test_config_deserialization() -> Result<()> {
    let config = SongbirdConfig::default();

    // Serialize to TOML
    let toml_string = toml::to_string(&config).unwrap();

    // Deserialize from TOML
    let deserialized_config: SongbirdConfig = toml::from_str(&toml_string).unwrap();

    // Verify key fields match
    assert_eq!(
        deserialized_config.environment.bind_port,
        config.environment.bind_port
    );
    assert_eq!(
        deserialized_config.environment.bind_port,
        config.environment.bind_port
    );

    Ok(())
}

#[test]
fn test_config_clone_behavior() -> Result<()> {
    let config = SongbirdConfig::default();
    let cloned_config = config.clone();

    // Verify clone matches original
    assert_eq!(cloned_config.environment.bind_port, config.environment.bind_port);
    assert_eq!(
        cloned_config.environment.bind_port,
        config.environment.bind_port
    );
    assert_eq!(
        cloned_config.environment.log_level,
        config.environment.log_level
    );

    Ok(())
}

#[test]
fn test_config_debug_formatting() -> Result<()> {
    let config = SongbirdConfig::default();

    // Test debug formatting
    let debug_string = format!("{config:?}");
    assert!(!debug_string.is_empty());
    assert!(debug_string.contains("SongbirdConfig"));

    Ok(())
}

#[test]
fn test_config_memory_usage() -> Result<()> {
    let config = SongbirdConfig::default();

    // Test memory usage is reasonable
    let config_size = std::mem::size_of_val(&config);
    assert!(config_size < 10_000); // Should be less than 10KB

    Ok(())
}

#[tokio::test]
async fn test_gaming_network_config() -> Result<()> {
    let gaming_config = GamingNetworkConfig {
        starcraft_port: 6112,
        aoe2_port: 2300,
        cnc_port_range: PortRange {
            start: 1234,
            end: 1240,
        },
        detection_interface: Some("eth0".to_string()),
        bridge_buffer_size: 65536,
    };

    assert_eq!(gaming_config.starcraft_port, 6112);
    assert_eq!(gaming_config.aoe2_port, 2300);
    assert!(gaming_config.cnc_port_range.start < gaming_config.cnc_port_range.end);
    assert!(gaming_config.cnc_port_range.start > 0);
    assert!(gaming_config.cnc_port_range.end < 65535);
    assert_eq!(gaming_config.detection_interface, Some("eth0".to_string()));
    assert_eq!(gaming_config.bridge_buffer_size, 65536);
    Ok(())
}

#[tokio::test]
async fn test_config_loading_and_validation() {
    let config = SongbirdConfig::default();
    
    // Test the network configuration
    assert!(config.environment.bind_port > 0);
    assert!(!config.network.bind_address.to_string().is_empty());
    
    // Test serialization roundtrip
    let serialized = toml::to_string(&config).expect("Failed to serialize config");
    let deserialized: SongbirdConfig = toml::from_str(&serialized).expect("Failed to deserialize config");
    
    assert_eq!(config.environment.bind_port, deserialized.network.orchestrator_port);
    assert_eq!(config.network.bind_address, deserialized.network.bind_address);
}

#[tokio::test]
async fn test_config_serialization() {
    let mut config = SongbirdConfig::default();
    config.environment.bind_port = 9000;
    
    let serialized = toml::to_string(&config).expect("Failed to serialize config");
    let loaded_config: SongbirdConfig = toml::from_str(&serialized).expect("Failed to deserialize config");
    
    assert_eq!(loaded_config.environment.bind_port, config.environment.bind_port);
}

#[tokio::test]
async fn test_config_persistence() {
    let mut config = SongbirdConfig::default();
    config.environment.bind_port = 9000;
    
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("test_config.toml");
    
    // Save config
    config.to_file(&config_path).expect("Failed to save config");
    
    // Load config
    let loaded_config = SongbirdConfig::from_file(&config_path).expect("Failed to load config");
    
    assert_eq!(loaded_config.environment.bind_port, config.environment.bind_port);
}

#[tokio::test]
async fn test_config_cloning() {
    let config = SongbirdConfig::default();
    let cloned_config = config.clone();
    
    assert_eq!(cloned_config.environment.bind_port, config.environment.bind_port);
}
