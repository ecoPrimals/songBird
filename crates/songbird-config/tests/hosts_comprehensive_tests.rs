//! Comprehensive tests for host configuration
//!
//! These tests validate the robust, thread-safe host configuration system
//! without relying on environment variable mutation.

use songbird_config::defaults::hosts::*;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_types::{SongbirdError, SongbirdResult};

// ============================================================================
// GLOBAL CONFIGURATION TESTS
// ============================================================================

#[test]
fn test_default_host_default_value() {
    // Global config should return a valid host
    let host = default_host();
    assert!(!host.is_empty());
}

#[test]
fn test_global_config_consistency() {
    // Multiple calls should return identical values
    let host1 = default_host();
    let host2 = default_host();
    let disc1 = discovery_host();
    let disc2 = discovery_host();

    assert_eq!(host1, host2, "Default host should be consistent");
    assert_eq!(disc1, disc2, "Discovery host should be consistent");
}

#[test]
fn test_global_config_thread_safety() -> SongbirdResult<()> {
    use songbird_types::{SongbirdError, SongbirdResult};
    use std::thread;

    // Spawn multiple threads reading configuration simultaneously
    let hosts: Vec<_> = (0..10)
        .map(|_| {
            thread::spawn(|| {
                let host = default_host();
                assert!(!host.is_empty());
                host
            })
        })
        .map(|h| h.join().or_else(|_| SongbirdError::configuration(format!("Error: {}", e)))?)
        .collect();

    // All threads should see the same value
    let first_host = &hosts[0];
    for host in &hosts {
        assert_eq!(host, first_host, "All threads should see the same host");
    }
    Ok(())
}

// ============================================================================
// DEPENDENCY INJECTION TESTS (HostConfig Struct)
// ============================================================================

#[test]
fn test_host_config_with_defaults() {
    let config = HostConfig::with_defaults();
    assert_eq!(config.default_host, "127.0.0.1");
    assert_eq!(config.bind_address, "0.0.0.0");
    assert_eq!(config.discovery_host, "127.0.0.1");
    assert_eq!(config.orchestrator_host, "127.0.0.1");
}

#[test]
fn test_host_config_custom_values() {
    let mut config = HostConfig::with_defaults();
    config.default_host = "192.168.1.100".to_string();
    config.discovery_host = "10.0.0.1".to_string();

    assert_eq!(config.default_host, "192.168.1.100");
    assert_eq!(config.discovery_host, "10.0.0.1");
}

#[test]
fn test_host_config_ipv6() {
    let mut config = HostConfig::with_defaults();
    config.default_host = "::1".to_string();
    assert_eq!(config.default_host, "::1");

    config.default_host = "fe80::1".to_string();
    assert_eq!(config.default_host, "fe80::1");
}

#[test]
fn test_host_config_ipv4_addresses() {
    let addresses = ["192.168.1.1", "10.0.0.1", "172.16.0.1", "127.0.0.1"];

    for addr in addresses {
        let mut config = HostConfig::with_defaults();
        config.default_host = addr.to_string();
        assert_eq!(config.default_host, addr);
    }
}

#[test]
fn test_host_config_special_addresses() {
    let mut config = HostConfig::with_defaults();

    // Localhost
    config.bind_address = "127.0.0.1".to_string();
    assert_eq!(config.bind_address, "127.0.0.1");

    // All interfaces
    config.bind_address = "0.0.0.0".to_string();
    assert_eq!(config.bind_address, "0.0.0.0");

    // IPv6 localhost
    config.default_host = "::1".to_string();
    assert_eq!(config.default_host, "::1");
}

#[test]
fn test_host_config_hostname_formats() {
    let hostnames =
        ["localhost", "songbird.local", "api.songbird.dev", "service-01.cluster.internal"];

    for hostname in hostnames {
        let mut config = HostConfig::with_defaults();
        config.default_host = hostname.to_string();
        assert_eq!(config.default_host, hostname);
    }
}

#[test]
fn test_different_hosts_for_different_services() {
    let mut config = HostConfig::with_defaults();
    config.default_host = "192.168.1.1".to_string();
    config.discovery_host = "192.168.1.2".to_string();
    config.orchestrator_host = "192.168.1.3".to_string();

    assert_eq!(config.default_host, "192.168.1.1");
    assert_eq!(config.discovery_host, "192.168.1.2");
    assert_eq!(config.orchestrator_host, "192.168.1.3");
}

#[test]
fn test_service_host_fallback() {
    let config = HostConfig::with_defaults();

    // Service host should fall back to default
    let custom_host = config.service_host("CUSTOM_SERVICE");
    assert_eq!(custom_host, config.default_host);
}

#[test]
fn test_orchestrator_host_default() -> SongbirdResult<()> {
    let config = HostConfig::with_defaults();
    // Orchestrator defaults to default_host
    assert_eq!(config.orchestrator_host, config.default_host);
    Ok(())
}

#[test]
fn test_discovery_host_default() -> SongbirdResult<()> {
    let config = HostConfig::with_defaults();
    // Discovery defaults to default_host
    assert_eq!(config.discovery_host, config.default_host);
    Ok(())
}

#[test]
fn test_host_config_is_clone() -> SongbirdResult<()> {
    let config1 = HostConfig::with_defaults();
    let config2 = config1.clone();

    assert_eq!(config1.default_host, config2.default_host);
    assert_eq!(config1.bind_address, config2.bind_address);
    Ok(())
}

#[test]
fn test_host_config_is_debug() -> SongbirdResult<()> {
    let config = HostConfig::with_defaults();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("HostConfig"));
    Ok(())
}

// ============================================================================
// PRODUCTION BEHAVIOR TESTS
// ============================================================================

#[test]
fn test_production_environment_detection() {
    let mut config = HostConfig::with_defaults();

    config.environment = "production".to_string();
    config.is_production = true;
    assert!(config.is_production);

    config.environment = "staging".to_string();
    config.is_production = true;
    assert!(config.is_production);

    config.environment = "development".to_string();
    config.is_production = false;
    assert!(!config.is_production);
}

#[test]
fn test_bind_address_production_vs_development() {
    let mut prod_config = HostConfig::with_defaults();
    prod_config.bind_address = "0.0.0.0".to_string(); // All interfaces
    assert_eq!(prod_config.bind_address, "0.0.0.0");

    let mut dev_config = HostConfig::with_defaults();
    dev_config.bind_address = "127.0.0.1".to_string(); // Localhost only
    assert_eq!(dev_config.bind_address, "127.0.0.1");
}

#[test]
fn test_all_global_functions_return_valid_values() {
    // All functions should return non-empty strings
    assert!(!default_host().is_empty());
    assert!(!bind_address().is_empty());
    assert!(!discovery_host().is_empty());
    assert!(!orchestrator_host().is_empty());
    assert!(!environment().is_empty());
}

#[test]
fn test_host_consistency_across_calls() {
    // Multiple calls should be fast and return the same value
    let start = std::time::Instant::now();

    for _ in 0..1000 {
        let _ = default_host();
    }

    let duration = start.elapsed();

    // Should be essentially instant (< 10ms) since it's just cloning a cached value
    assert!(duration.as_millis() < 10, "Host access should be fast (cached)");
}

#[test]
fn test_service_host_dynamic() {
    // Test the dynamic service_host function
    let host = service_host("CUSTOM_SERVICE");
    assert!(!host.is_empty());

    let host2 = service_host("ANOTHER_SERVICE");
    assert!(!host2.is_empty());
}

// ============================================================================
// ARCHITECTURAL VALIDATION
// ============================================================================

#[test]
fn test_no_environment_mutation_needed() {
    // Validate we can create multiple configs without env var mutation
    let config1 = HostConfig::with_defaults();
    let mut config2 = HostConfig::with_defaults();
    config2.default_host = "custom.host".to_string();

    // Configs are independent
    assert_ne!(config1.default_host, config2.default_host);

    // Global config is unaffected
    let global_host = default_host();
    assert!(!global_host.is_empty());
}

#[test]
fn test_config_independence() {
    // Each config instance is independent
    let mut config1 = HostConfig::with_defaults();
    let mut config2 = HostConfig::with_defaults();

    config1.default_host = "host1.example.com".to_string();
    config2.default_host = "host2.example.com".to_string();

    assert_eq!(config1.default_host, "host1.example.com");
    assert_eq!(config2.default_host, "host2.example.com");
}

#[test]
fn test_empty_host_handling() {
    // Test that we can set empty host (though not recommended)
    let mut config = HostConfig::with_defaults();
    config.default_host = String::new();
    assert!(config.default_host.is_empty());
}

#[test]
fn test_host_with_port() {
    // Test host with port notation
    let mut config = HostConfig::with_defaults();
    config.default_host = "192.168.1.100:8080".to_string();
    assert_eq!(config.default_host, "192.168.1.100:8080");
}

#[test]
fn test_long_hostname() {
    let mut config = HostConfig::with_defaults();
    let long_hostname = "very-long-subdomain.with-multiple-parts.and-more-subdomains.example.com";
    config.default_host = long_hostname.to_string();
    assert_eq!(config.default_host, long_hostname);
}

#[test]
fn test_rapid_config_creation() {
    // Creating many configs should be fast (no env access each time when using defaults)
    let start = std::time::Instant::now();

    for _ in 0..1000 {
        let _ = HostConfig::with_defaults();
    }

    let duration = start.elapsed();
    assert!(duration.as_millis() < 100, "Config creation should be fast");
}
