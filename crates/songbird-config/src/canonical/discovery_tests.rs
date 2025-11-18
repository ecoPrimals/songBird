//! Tests for canonical discovery configuration

use super::discovery::*;

#[test]
fn test_discovery_config_has_all_fields() {
    let config = DiscoveryConfig::default();

    assert!(config.service_discovery.enabled);
    assert!(config.capability_discovery.enabled);
    assert!(config.auto_discovery);
    assert!(!config.common_ports.is_empty());
    assert_eq!(config.scan_timeout_secs, 5);
}

#[test]
fn test_service_discovery_config_fields() {
    let config = ServiceDiscoveryConfig::default();

    assert!(config.enabled);
    assert_eq!(config.discovery_interval_secs, 30);
    assert_eq!(config.max_concurrent_discoveries, 10);
}

#[test]
fn test_capability_discovery_config_fields() {
    let config = CapabilityDiscoveryConfig::default();

    assert!(config.enabled);
    assert_eq!(config.cache_ttl_secs, 300);
    assert_eq!(config.discovery_batch_size, 10);
}

#[test]
fn test_network_discovery_config_fields() {
    let config = NetworkDiscoveryConfig::default();

    // Network discovery disabled by default for security
    assert!(!config.enabled);
    assert!(!config.scan_local_network);
}

#[test]
fn test_discovery_config_clone() {
    let config = DiscoveryConfig::default();
    let cloned = config.clone();

    assert_eq!(config.scan_timeout_secs, cloned.scan_timeout_secs);
}

#[test]
fn test_service_discovery_custom() {
    let config = ServiceDiscoveryConfig {
        enabled: false,
        discovery_interval_secs: 60,
        max_concurrent_discoveries: 20,
        discovery_timeout_secs: 45,
    };

    assert!(!config.enabled);
    assert_eq!(config.discovery_interval_secs, 60);
}

#[test]
fn test_capability_discovery_custom() {
    let config = CapabilityDiscoveryConfig {
        enabled: false,
        cache_ttl_secs: 600,
        discovery_batch_size: 50,
        max_retry_attempts: 5,
    };

    assert!(!config.enabled);
    assert_eq!(config.cache_ttl_secs, 600);
}

#[test]
fn test_network_discovery_custom() {
    let config = NetworkDiscoveryConfig {
        enabled: true,
        scan_local_network: true,
        scan_ports: vec![80, 443],
        discovery_protocols: vec!["http".to_string()],
    };

    assert!(config.enabled);
    assert_eq!(config.scan_ports.len(), 2);
}

#[test]
fn test_discovery_config_common_ports() {
    let config = DiscoveryConfig::default();

    assert!(config.common_ports.contains(&80));
    assert!(config.common_ports.contains(&443));
    assert!(config.common_ports.contains(&8080));
}

#[test]
fn test_service_discovery_equality() {
    let config1 = ServiceDiscoveryConfig::default();
    let config2 = ServiceDiscoveryConfig::default();

    assert_eq!(config1, config2);
}

// Total: 10 tests
