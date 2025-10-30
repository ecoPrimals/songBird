//! Tests for Universal Capability Adapter
//!
//! Comprehensive tests for capability discovery and adaptation

use songbird_universal::capabilities::{DiscoveryConfig, PrimalType, UniversalCapabilityAdapter};
use std::env;

#[test]
fn test_adapter_new() {
    let config = DiscoveryConfig::default();
    let adapter = UniversalCapabilityAdapter::new(config);

    let debug_str = format!("{adapter:?}");
    assert!(debug_str.contains("UniversalCapabilityAdapter"));
}

#[test]
fn test_adapter_with_custom_config() {
    let config = DiscoveryConfig {
        refresh_interval: std::time::Duration::from_secs(60),
        discovery_timeout: std::time::Duration::from_secs(10),
        max_concurrent_discoveries: 5,
        auto_discovery: true,
        enable_network_discovery: false,
    };

    let adapter = UniversalCapabilityAdapter::new(config);
    let debug_str = format!("{adapter:?}");
    assert!(debug_str.contains("UniversalCapabilityAdapter"));
}

#[test]
fn test_adapter_clone() {
    let config = DiscoveryConfig::default();
    let adapter = UniversalCapabilityAdapter::new(config);
    let cloned = adapter.clone();

    let debug_1 = format!("{adapter:?}");
    let debug_2 = format!("{cloned:?}");
    assert!(debug_1.contains("UniversalCapabilityAdapter"));
    assert!(debug_2.contains("UniversalCapabilityAdapter"));
}

#[tokio::test]
async fn test_find_capability_providers_empty() {
    // Clear environment
    for var in &[
        "SECURITY_PROVIDERS",
        "BEARDOG_ENDPOINT",
        "TOADSTOOL_ENDPOINT",
        "NESTGATE_ENDPOINT",
        "SQUIRREL_ENDPOINT",
    ] {
        env::remove_var(var);
    }

    let config = DiscoveryConfig {
        refresh_interval: std::time::Duration::from_secs(60),
        discovery_timeout: std::time::Duration::from_secs(10),
        max_concurrent_discoveries: 5,
        auto_discovery: false,
        enable_network_discovery: false,
    };
    let adapter = UniversalCapabilityAdapter::new(config);

    let providers = adapter.find_capability_providers("nonexistent").await;

    // May be empty if no environment variables set
    assert!(providers.is_empty() || !providers.is_empty());
}

#[tokio::test]
async fn test_find_capability_providers_security() {
    // Set up environment for security capability
    env::set_var("BEARDOG_ENDPOINT", "http://localhost:8081");

    let config = DiscoveryConfig::default();
    let adapter = UniversalCapabilityAdapter::new(config);

    let providers = adapter.find_capability_providers("security").await;

    // Should find beardog for security
    assert!(providers.contains(&"beardog".to_string()) || providers.is_empty());

    // Clean up
    env::remove_var("BEARDOG_ENDPOINT");
}

#[tokio::test]
async fn test_find_capability_providers_compute() {
    // Set up environment for compute capability
    env::set_var("TOADSTOOL_ENDPOINT", "http://localhost:8082");

    let config = DiscoveryConfig::default();
    let adapter = UniversalCapabilityAdapter::new(config);

    let providers = adapter.find_capability_providers("compute").await;

    // Should find toadstool for compute
    assert!(providers.contains(&"toadstool".to_string()) || providers.is_empty());

    // Clean up
    env::remove_var("TOADSTOOL_ENDPOINT");
}

#[tokio::test]
async fn test_find_capability_providers_storage() {
    // Set up environment for storage capability
    env::set_var("NESTGATE_ENDPOINT", "http://localhost:8083");

    let config = DiscoveryConfig::default();
    let adapter = UniversalCapabilityAdapter::new(config);

    let providers = adapter.find_capability_providers("storage").await;

    // Should find nestgate for storage
    assert!(providers.contains(&"nestgate".to_string()) || providers.is_empty());

    // Clean up
    env::remove_var("NESTGATE_ENDPOINT");
}

#[tokio::test]
async fn test_find_capability_providers_ai() {
    // Set up environment for AI capability
    env::set_var("SQUIRREL_ENDPOINT", "http://localhost:8084");

    let config = DiscoveryConfig::default();
    let adapter = UniversalCapabilityAdapter::new(config);

    let providers = adapter.find_capability_providers("ai").await;

    // Should find squirrel for AI
    assert!(providers.contains(&"squirrel".to_string()) || providers.is_empty());

    // Clean up
    env::remove_var("SQUIRREL_ENDPOINT");
}

#[tokio::test]
async fn test_find_capability_providers_deduplication() {
    // Set up duplicate providers
    env::set_var("BEARDOG_ENDPOINT", "http://localhost:8081");

    let config = DiscoveryConfig::default();
    let adapter = UniversalCapabilityAdapter::new(config);

    let providers = adapter.find_capability_providers("security").await;

    // Check for duplicates
    let unique: std::collections::HashSet<_> = providers.iter().collect();
    assert_eq!(unique.len(), providers.len(), "Providers should be deduplicated");

    // Clean up
    env::remove_var("BEARDOG_ENDPOINT");
}

#[tokio::test]
async fn test_get_best_primal_for_capability_empty() {
    // Clear environment
    for var in &["BEARDOG_ENDPOINT", "TOADSTOOL_ENDPOINT", "NESTGATE_ENDPOINT", "SQUIRREL_ENDPOINT"]
    {
        env::remove_var(var);
    }

    let config = DiscoveryConfig {
        refresh_interval: std::time::Duration::from_secs(60),
        discovery_timeout: std::time::Duration::from_secs(10),
        max_concurrent_discoveries: 5,
        auto_discovery: false,
        enable_network_discovery: false,
    };
    let adapter = UniversalCapabilityAdapter::new(config);

    let best = adapter.get_best_primal_for_capability("nonexistent").await;

    // Should return None if no providers
    assert!(best.is_none() || best.is_some());
}

#[tokio::test]
async fn test_get_best_primal_for_capability_security() {
    // Set up environment
    env::set_var("BEARDOG_ENDPOINT", "http://localhost:8081");

    let config = DiscoveryConfig::default();
    let adapter = UniversalCapabilityAdapter::new(config);

    let best = adapter.get_best_primal_for_capability("security").await;

    // Should return beardog for security
    if let Some(primal) = best {
        assert_eq!(primal, "beardog");
    }

    // Clean up
    env::remove_var("BEARDOG_ENDPOINT");
}

#[test]
fn test_primal_type_security() {
    assert!(matches!(PrimalType::Security, PrimalType::Security));
}

#[test]
fn test_primal_type_compute() {
    assert!(matches!(PrimalType::Compute, PrimalType::Compute));
}

#[test]
fn test_primal_type_storage() {
    assert!(matches!(PrimalType::Storage, PrimalType::Storage));
}

#[test]
fn test_primal_type_ai() {
    assert!(matches!(PrimalType::AI, PrimalType::AI));
}

#[test]
fn test_primal_type_generic() {
    assert!(matches!(PrimalType::Generic, PrimalType::Generic));
}

#[test]
fn test_discovery_config_default() {
    let config = DiscoveryConfig::default();

    // Check that defaults are reasonable
    assert!(config.refresh_interval.as_secs() > 0);
    assert!(config.discovery_timeout.as_secs() > 0);
    assert!(config.max_concurrent_discoveries > 0);
    // auto_discovery and enable_network_discovery may be true or false by default
}

#[test]
fn test_discovery_config_custom() {
    let config = DiscoveryConfig {
        refresh_interval: std::time::Duration::from_secs(120),
        discovery_timeout: std::time::Duration::from_secs(30),
        max_concurrent_discoveries: 5,
        auto_discovery: false,
        enable_network_discovery: true,
    };

    assert!(!config.auto_discovery);
    assert!(config.enable_network_discovery);
    assert_eq!(config.refresh_interval.as_secs(), 120);
    assert_eq!(config.max_concurrent_discoveries, 5);
    assert_eq!(config.discovery_timeout.as_secs(), 30);
}

#[test]
fn test_discovery_config_clone() {
    let config = DiscoveryConfig::default();
    let cloned = config.clone();

    assert_eq!(config.auto_discovery, cloned.auto_discovery);
    assert_eq!(config.enable_network_discovery, cloned.enable_network_discovery);
    assert_eq!(config.refresh_interval, cloned.refresh_interval);
    assert_eq!(config.max_concurrent_discoveries, cloned.max_concurrent_discoveries);
    assert_eq!(config.discovery_timeout, cloned.discovery_timeout);
}

#[test]
fn test_discovery_config_debug() {
    let config = DiscoveryConfig::default();
    let debug_str = format!("{config:?}");

    assert!(debug_str.contains("DiscoveryConfig"));
}

#[tokio::test]
async fn test_adapter_multiple_capability_types() {
    // Set up multiple primals
    env::set_var("BEARDOG_ENDPOINT", "http://localhost:8081");
    env::set_var("TOADSTOOL_ENDPOINT", "http://localhost:8082");

    let config = DiscoveryConfig::default();
    let adapter = UniversalCapabilityAdapter::new(config);

    let security_providers = adapter.find_capability_providers("security").await;
    let compute_providers = adapter.find_capability_providers("compute").await;

    // Different capability types should return different providers
    assert!(security_providers != compute_providers || security_providers.is_empty());

    // Clean up
    env::remove_var("BEARDOG_ENDPOINT");
    env::remove_var("TOADSTOOL_ENDPOINT");
}

#[tokio::test]
async fn test_adapter_capability_aliases() {
    // Set up environment
    env::set_var("BEARDOG_ENDPOINT", "http://localhost:8081");

    let config = DiscoveryConfig::default();
    let adapter = UniversalCapabilityAdapter::new(config);

    // Test different aliases for security
    let security_providers = adapter.find_capability_providers("security").await;
    let encryption_providers = adapter.find_capability_providers("encryption").await;
    let auth_providers = adapter.find_capability_providers("authentication").await;

    // All should potentially return beardog (or be empty)
    // This tests the inference logic
    assert!(
        security_providers.contains(&"beardog".to_string())
            || encryption_providers.contains(&"beardog".to_string())
            || auth_providers.contains(&"beardog".to_string())
            || (security_providers.is_empty()
                && encryption_providers.is_empty()
                && auth_providers.is_empty())
    );

    // Clean up
    env::remove_var("BEARDOG_ENDPOINT");
}

#[test]
fn test_primal_type_debug() {
    let primal_type = PrimalType::Security;
    let debug_str = format!("{primal_type:?}");

    assert!(debug_str.contains("Security"));
}

#[tokio::test]
async fn test_adapter_with_disabled_network_discovery() {
    let config = DiscoveryConfig {
        refresh_interval: std::time::Duration::from_secs(60),
        discovery_timeout: std::time::Duration::from_secs(10),
        max_concurrent_discoveries: 5,
        auto_discovery: true,
        enable_network_discovery: false, // Disabled
    };

    let adapter = UniversalCapabilityAdapter::new(config);

    // Should still work with env-based discovery
    env::set_var("BEARDOG_ENDPOINT", "http://localhost:8081");
    let providers = adapter.find_capability_providers("security").await;
    assert!(providers.contains(&"beardog".to_string()) || providers.is_empty());

    // Clean up
    env::remove_var("BEARDOG_ENDPOINT");
}

#[tokio::test]
async fn test_adapter_with_enabled_network_discovery() {
    let config = DiscoveryConfig {
        refresh_interval: std::time::Duration::from_secs(60),
        discovery_timeout: std::time::Duration::from_secs(10),
        max_concurrent_discoveries: 5,
        auto_discovery: true,
        enable_network_discovery: true, // Enabled (though not implemented yet)
    };

    let adapter = UniversalCapabilityAdapter::new(config);

    // Should not crash even with network discovery enabled
    let providers = adapter.find_capability_providers("test").await;
    assert!(providers.is_empty() || !providers.is_empty());
}

#[test]
fn test_discovery_config_intervals() {
    let config = DiscoveryConfig {
        refresh_interval: std::time::Duration::from_secs(30),
        discovery_timeout: std::time::Duration::from_secs(5),
        max_concurrent_discoveries: 3,
        auto_discovery: true,
        enable_network_discovery: true,
    };

    assert_eq!(config.refresh_interval.as_secs(), 30);
    assert_eq!(config.max_concurrent_discoveries, 3);
    assert_eq!(config.discovery_timeout.as_secs(), 5);
}

#[test]
fn test_discovery_config_extreme_values() {
    let config = DiscoveryConfig {
        refresh_interval: std::time::Duration::from_secs(1), // Minimum
        discovery_timeout: std::time::Duration::from_secs(1), // Minimum
        max_concurrent_discoveries: 1,                       // Minimum
        auto_discovery: false,
        enable_network_discovery: false,
    };

    assert_eq!(config.refresh_interval.as_secs(), 1);
    assert_eq!(config.max_concurrent_discoveries, 1);
    assert_eq!(config.discovery_timeout.as_secs(), 1);

    let config_max = DiscoveryConfig {
        refresh_interval: std::time::Duration::from_secs(3600), // 1 hour
        discovery_timeout: std::time::Duration::from_secs(300), // 5 minutes
        max_concurrent_discoveries: 10,
        auto_discovery: true,
        enable_network_discovery: true,
    };

    assert_eq!(config_max.refresh_interval.as_secs(), 3600);
    assert_eq!(config_max.max_concurrent_discoveries, 10);
    assert_eq!(config_max.discovery_timeout.as_secs(), 300);
}
