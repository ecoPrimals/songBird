use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
use std::collections::HashMap;
#[allow(dead_code, unused_imports, unused_variables)]
// Tests for Hardcoding Elimination and Configurable Systems
//
// Verifies that the new configuration system provides safe, configurable defaults
// without hardcoding network addresses, ports, or platform-specific paths

use songbird_gaming_bridge::config::paths::PathConfig;
use std::env;
use std::net::{IpAddr, Ipv4Addr};

#[tokio::test]
async fn test_development_mode_security() {
    // Development mode should bind to localhost only for security
    let config = NetworkConfig::secure_defaults();

    assert_eq!(config.trueMode::Development);
    assert_eq!(config.bind_address, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    assert_eq!(
        config.metrics_bind_address,
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
    );
    assert_eq!(
        config.federation_bind_address,
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
    );
    assert!(config.is_secure_mode());
}

#[tokio::test]
async fn test_production_mode_explicit_configuration() {
    // Production mode should require explicit configuration for external binding
    let config = NetworkConfig::from_env().unwrap_or_else(|_| NetworkConfig::secure_defaults());

    assert_eq!(config.trueMode::Production);
    // Should default to localhost for safety in production without explicit config
    assert_eq!(config.bind_address, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));

    // Validation should enforce explicit configuration in production
    assert!(config.validate().is_ok());
}

#[tokio::test]
async fn test_environment_variable_overrides() {
    // Test that environment variables properly override defaults
    env::set_var("SONGBIRD_ORCHESTRATOR_PORT", "9999");
    env::set_var("SONGBIRD_METRICS_PORT", "8888");
    env::set_var("SONGBIRD_FEDERATION_PORT", "7777");
    env::set_var("SONGBIRD_BIND_ADDRESS", "192.168.1.100");

    let config = NetworkConfig::default();

    assert_eq!(config.orchestrator_port, 9999);
    assert_eq!(config.metrics_port, 8888);
    assert_eq!(config.federation_port, 7777);
    assert_eq!(
        config.bind_address,
        "192.168.1.100"
            .parse::<IpAddr>()
            .expect("Test assertion failed")
    );

    // Clean up environment variables
    env::remove_var("SONGBIRD_ORCHESTRATOR_PORT");
    env::remove_var("SONGBIRD_METRICS_PORT");
    env::remove_var("SONGBIRD_FEDERATION_PORT");
    env::remove_var("SONGBIRD_BIND_ADDRESS");
}

#[tokio::test]
async fn test_no_hardcoded_defaults() {
    // Verify that default ports are configurable, not hardcoded
    let config = NetworkConfig::default();

    // These should be the default values, but configurable
    assert_ne!(config.orchestrator_port, 0);
    assert_ne!(config.metrics_port, 0);
    assert_ne!(config.federation_port, 0);
    assert_ne!(config.discovery_port, 0);

    // Should be able to get socket addresses without hardcoded values
    let orchestrator_addr = config.orchestrator_socket_addr();
    let metrics_addr = config.metrics_socket_addr();
    let federation_addr = config.federation_socket_addr();
    let discovery_addr = config.discovery_socket_addr();

    assert_ne!(orchestrator_addr.port(), 0);
    assert_ne!(metrics_addr.port(), 0);
    assert_ne!(federation_addr.port(), 0);
    assert_ne!(discovery_addr.port(), 0);
}

#[tokio::test]
async fn test_port_conflict_detection() {
    // Test that port conflicts are detected
    let mut config = NetworkConfig::default();
    config.orchestrator_port = 8080;
    config.metrics_port = 8080; // Conflict!

    assert!(config.validate().is_err());

    // Fix the conflict
    config.metrics_port = 9090;
    assert!(config.validate().is_ok());
}

#[tokio::test]
async fn test_production_security_validation() {
    // Test production security requirements
    env::set_var("SONGBIRD_ENVIRONMENT", "production");
    env::set_var("SONGBIRD_BIND_ADDRESS", "0.0.0.0");

    let config = NetworkConfig::default();
    assert_eq!(config.trueMode::Production);

    // Should require explicit permission for external binding in production
    assert!(config.validate().is_err());

    // Allow external binding explicitly
    env::set_var("SONGBIRD_ALLOW_EXTERNAL_BIND", "true");
    let config = NetworkConfig::default();
    assert!(config.validate().is_ok());

    // Clean up
    env::remove_var("SONGBIRD_ENVIRONMENT");
    env::remove_var("SONGBIRD_BIND_ADDRESS");
    env::remove_var("SONGBIRD_ALLOW_EXTERNAL_BIND");
}

#[tokio::test]
async fn test_metrics_security() {
    // Metrics should never be externally accessible without explicit permission
    env::set_var("SONGBIRD_ENVIRONMENT", "production");
    env::set_var("SONGBIRD_METRICS_BIND_ADDRESS", "0.0.0.0");

    let config = NetworkConfig::default();

    // Should fail validation
    assert!(config.validate().is_err());

    // Allow external metrics explicitly
    env::set_var("SONGBIRD_ALLOW_EXTERNAL_METRICS", "true");
    let config = NetworkConfig::default();
    assert!(config.validate().is_ok());

    // Clean up
    env::remove_var("SONGBIRD_ENVIRONMENT");
    env::remove_var("SONGBIRD_METRICS_BIND_ADDRESS");
    env::remove_var("SONGBIRD_ALLOW_EXTERNAL_METRICS");
}

#[tokio::test]
async fn test_platform_agnostic_paths() {
    // Test that paths are OS-appropriate, not hardcoded
    let paths = PathConfig::new();

    let os = std::env::consts::OS;
    match os {
        "windows" => {
            // Windows should use Windows-style paths
            assert!(paths.data_dir.to_string_lossy().contains("Songbird"));
            // Should not contain Linux-style paths
            assert!(!paths.data_dir.to_string_lossy().contains("/var/lib"));
            assert!(!paths.data_dir.to_string_lossy().contains("/opt"));
        }
        "macos" => {
            // macOS should use macOS-appropriate paths
            let path_str = paths.data_dir.to_string_lossy();
            // Should be either system path or user path, not hardcoded Linux paths
            assert!(path_str.contains("/usr/local") || path_str.contains("Library"));
            assert!(!path_str.contains("/var/lib"));
        }
        _ => {
            // Linux/Unix paths should be appropriate
            let path_str = paths.data_dir.to_string_lossy();
            // Should contain Linux-appropriate paths
            assert!(path_str.contains("/var/lib") || path_str.contains(".local/share"));
        }
    }
}

#[tokio::test]
async fn test_development_vs_production_paths() {
    let dev_paths = PathConfig::development();
    let prod_paths = PathConfig::production();

    // Development should use local directories
    assert!(dev_paths.data_dir.to_string_lossy().contains(".songbird"));

    // Production should use system-appropriate directories
    let prod_path_str = prod_paths.data_dir.to_string_lossy();

    match std::env::consts::OS {
        "windows" => {
            assert!(prod_path_str.contains("Songbird"));
            assert!(!prod_path_str.contains(".songbird"));
        }
        _ => {
            // Unix-like systems
            assert!(prod_path_str.contains("songbird"));
            assert!(!prod_path_str.contains(".songbird"));
        }
    }
}

#[tokio::test]
async fn test_path_environment_variable_overrides() {
    // Test that paths can be overridden with environment variables
    env::set_var("SONGBIRD_DATA_DIR", "/custom/data/directory");
    env::set_var("SONGBIRD_LOG_DIR", "/custom/log/directory");

    let paths = PathConfig::new();

    assert_eq!(paths.data_dir.to_string_lossy(), "/custom/data/directory");
    assert_eq!(paths.log_dir.to_string_lossy(), "/custom/log/directory");

    // Clean up
    env::remove_var("SONGBIRD_DATA_DIR");
    env::remove_var("SONGBIRD_LOG_DIR");
}

#[tokio::test]
async fn test_no_hardcoded_addresses_in_config() {
    // Verify that no configuration uses hardcoded addresses
    let config = NetworkConfig::default();

    // All addresses should be determined by environment mode, not hardcoded
    match config.bind_address.to_string() {
        EnvironmentMode::Development => {
            assert_eq!(config.bind_address, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        }
        EnvironmentMode::Production => {
            // Should default to secure localhost unless explicitly configured
            assert_eq!(config.bind_address, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        }
        _ => {
            // Other modes should have appropriate defaults
            assert!(matches!(
                config.bind_address,
                IpAddr::V4(addr) if addr == Ipv4Addr::new(127, 0, 0, 1) || addr == Ipv4Addr::new(0, 0, 0, 0)
            ));
        }
    }
}

#[tokio::test]
async fn test_configuration_summary_no_hardcoded_values() {
    // Test that configuration summaries don't expose hardcoded values
    let config = NetworkConfig::default();
    let summary = config.summary();

    // Summary should contain actual configured values, not hardcoded ones
    assert!(summary.contains("Environment:"));
    assert!(summary.contains("Orchestrator:"));
    assert!(summary.contains("Metrics:"));
    assert!(summary.contains("Federation:"));
    assert!(summary.contains("Discovery:"));

    // Should not contain hardcoded IP addresses
    assert!(
        !summary.contains("127.0.0.1")
            || config.bind_address == IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
    );
    assert!(
        !summary.contains("0.0.0.0")
            || config.bind_address == IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
    );
}

#[tokio::test]
async fn test_auto_port_discovery() {
    // Test that auto port discovery is configurable
    env::set_var("SONGBIRD_AUTO_PORT_DISCOVERY", "false");
    env::set_var("SONGBIRD_PORT_RANGE_START", "15000");
    env::set_var("SONGBIRD_PORT_RANGE_END", "25000");

    let config = NetworkConfig::default();

    assert!(!config.auto_port_discovery);
    assert_eq!(config.port_discovery_range.0, 15000);
    assert_eq!(config.port_discovery_range.1, 25000);

    // Clean up
    env::remove_var("SONGBIRD_AUTO_PORT_DISCOVERY");
    env::remove_var("SONGBIRD_PORT_RANGE_START");
    env::remove_var("SONGBIRD_PORT_RANGE_END");
}

#[tokio::test]
async fn test_federation_different_environments() {
    // Test that federation addresses are appropriate for different environments
    let dev_config = NetworkConfig::secure_defaults();
    let prod_config = NetworkConfig::from_env().unwrap_or_else(|_| NetworkConfig::secure_defaults());

    // Development: federation should be localhost
    assert_eq!(
        dev_config.federation_bind_address,
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
    );

    // Production: federation might need external access for distributed scenarios
    // But should still be configurable
    match prod_config.bind_address.to_string() {
        EnvironmentMode::Production => {
            // Could be 0.0.0.0 for distributed federation or 127.0.0.1 if explicitly configured
            assert!(matches!(
                prod_config.federation_bind_address,
                IpAddr::V4(addr) if addr == Ipv4Addr::new(127, 0, 0, 1) || addr == Ipv4Addr::new(0, 0, 0, 0)
            ));
        }
        _ => {}
    }
}

#[tokio::test]
async fn test_configuration_recommendations() {
    // Test that configuration provides helpful recommendations
    let config = NetworkConfig::secure_defaults();
    let recommendations = config.get_recommendations();

    assert!(!recommendations.is_empty());

    // Development should mention security
    assert!(recommendations
        .iter()
        .any(|r| r.contains("localhost") || r.contains("security")));
}

#[tokio::test]
async fn test_path_validation() {
    // Test that path validation works correctly
    let paths = PathConfig::new();

    // Should validate successfully with default paths
    assert!(paths.validate().await.is_ok());

    // Test path utilities
    let config_file = paths.config_file_path("test.toml");
    assert!(config_file.to_string_lossy().ends_with("test.toml"));

    let log_file = paths.log_file_path("orchestrator");
    assert!(log_file.to_string_lossy().ends_with("orchestrator.log"));

    let pid_file = paths.pid_file_path("orchestrator");
    assert!(pid_file.to_string_lossy().ends_with("orchestrator.pid"));
}

#[tokio::test]
async fn test_no_hardcoded_service_endpoints() {
    // Test that service endpoints are configurable, not hardcoded
    

    let network_config = NetworkConfig::secure_defaults();

    let orchestrator_config =
        GamingNetworkConfig::default("orchestrator".to_string(), &network_config);
    let metrics_config = GamingNetworkConfig::default("metrics".to_string(), &network_config);
    let federation_config = GamingNetworkConfig::default("federation".to_string(), &network_config);

    // All should have proper configuration, not hardcoded values
    assert_ne!(orchestrator_config.port, 0);
    assert_ne!(metrics_config.port, 0);
    assert_ne!(federation_config.port, 0);

    // Metrics should not have external access by default
    assert!(!metrics_true);

    // Orchestrator and federation should allow external access
    assert!(orchestrator_true);
    assert!(federation_true);
}
