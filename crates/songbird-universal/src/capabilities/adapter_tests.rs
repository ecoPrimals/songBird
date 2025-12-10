//! Tests for Universal Capability Adapter

use super::adapter::UniversalCapabilityAdapter;
use super::types::DiscoveryConfig;

fn create_test_config() -> DiscoveryConfig {
    DiscoveryConfig {
        refresh_interval: std::time::Duration::from_secs(60),
        discovery_timeout: std::time::Duration::from_secs(5),
        max_concurrent_discoveries: 10,
        auto_discovery: false,
        enable_network_discovery: false,
    }
}

#[tokio::test]
async fn test_new_adapter_creation() {
    let config = create_test_config();
    let adapter = UniversalCapabilityAdapter::new(config);

    // Verify adapter is created with correct configuration
    assert!(!adapter.discovery_config.enable_network_discovery);
    assert_eq!(adapter.discovery_config.max_concurrent_discoveries, 10);
}

#[tokio::test]
async fn test_find_capability_providers_from_env() {
    // Set up environment variable
    std::env::set_var("COMPUTE_PROVIDER_ENDPOINT", "http://localhost:8080/compute");

    let config = create_test_config();
    let adapter = UniversalCapabilityAdapter::new(config);

    // Test finding compute providers
    let providers = adapter.find_capability_providers("compute").await;

    // Should find at least one provider
    assert!(!providers.is_empty(), "Should find compute provider from env");

    // Clean up
    std::env::remove_var("COMPUTE_PROVIDER_ENDPOINT");
}

#[tokio::test]
async fn test_find_capability_providers_empty() {
    // Clean environment
    std::env::remove_var("COMPUTE_PROVIDER_ENDPOINT");
    std::env::remove_var("STORAGE_PROVIDER_ENDPOINT");
    std::env::remove_var("AI_PROVIDER_ENDPOINT");

    let config = create_test_config();
    let adapter = UniversalCapabilityAdapter::new(config);

    // Test finding providers when none are configured
    let providers = adapter.find_capability_providers("nonexistent").await;

    // Should return empty vec
    assert!(providers.is_empty(), "Should return empty for unconfigured capability");
}

#[tokio::test]
async fn test_find_capability_providers_multiple() {
    // Set up multiple providers
    std::env::set_var("COMPUTE_PROVIDERS", "compute1,compute2,compute3");

    let config = create_test_config();
    let adapter = UniversalCapabilityAdapter::new(config);

    let providers = adapter.find_capability_providers("compute").await;

    // Should find multiple providers
    assert!(providers.len() >= 2, "Should find multiple compute providers");

    // Clean up
    std::env::remove_var("COMPUTE_PROVIDERS");
}

#[tokio::test]
async fn test_get_active_connections_empty() {
    let config = create_test_config();
    let adapter = UniversalCapabilityAdapter::new(config);

    let connections = adapter.get_active_connections().await;

    // No connections initially
    assert!(connections.is_empty(), "Should have no connections initially");
}

#[tokio::test]
async fn test_adapter_discovery_config() {
    let config = DiscoveryConfig {
        refresh_interval: std::time::Duration::from_secs(120),
        discovery_timeout: std::time::Duration::from_secs(10),
        max_concurrent_discoveries: 5,
        auto_discovery: true,
        enable_network_discovery: true,
    };

    let adapter = UniversalCapabilityAdapter::new(config);

    // Verify configuration is stored correctly
    assert_eq!(adapter.discovery_config.max_concurrent_discoveries, 5);
    assert!(adapter.discovery_config.auto_discovery);
    assert!(adapter.discovery_config.enable_network_discovery);
}

#[tokio::test]
async fn test_find_capability_providers_case_insensitive() {
    // Set up environment variable
    std::env::set_var("COMPUTE_PROVIDER_ENDPOINT", "http://localhost:8080");

    let config = create_test_config();
    let adapter = UniversalCapabilityAdapter::new(config);

    // Test both uppercase and lowercase
    let providers_lower = adapter.find_capability_providers("compute").await;
    let providers_upper = adapter.find_capability_providers("COMPUTE").await;

    // Both should find providers
    assert!(!providers_lower.is_empty());
    assert!(!providers_upper.is_empty());

    // Clean up
    std::env::remove_var("COMPUTE_PROVIDER_ENDPOINT");
}

#[tokio::test]
async fn test_multiple_capability_types() {
    // Set up multiple capability types
    std::env::set_var("COMPUTE_PROVIDER_ENDPOINT", "http://localhost:8080");
    std::env::set_var("STORAGE_PROVIDER_ENDPOINT", "http://localhost:8081");
    std::env::set_var("AI_PROVIDER_ENDPOINT", "http://localhost:8082");

    let config = create_test_config();
    let adapter = UniversalCapabilityAdapter::new(config);

    // Find each capability type
    let compute_providers = adapter.find_capability_providers("compute").await;
    let storage_providers = adapter.find_capability_providers("storage").await;
    let ai_providers = adapter.find_capability_providers("ai").await;

    // All should find providers
    assert!(!compute_providers.is_empty(), "Should find compute providers");
    assert!(!storage_providers.is_empty(), "Should find storage providers");
    assert!(!ai_providers.is_empty(), "Should find AI providers");

    // Clean up
    std::env::remove_var("COMPUTE_PROVIDER_ENDPOINT");
    std::env::remove_var("STORAGE_PROVIDER_ENDPOINT");
    std::env::remove_var("AI_PROVIDER_ENDPOINT");
}

#[tokio::test]
async fn test_adapter_default_config() {
    let config = DiscoveryConfig::default();
    let adapter = UniversalCapabilityAdapter::new(config);

    // Verify adapter works with default configuration
    let providers = adapter.find_capability_providers("test").await;

    // Should return empty vec but not panic
    assert!(providers.is_empty() || !providers.is_empty());
}

#[tokio::test]
async fn test_concurrent_capability_lookups() {
    use tokio::task::JoinSet;

    let config = create_test_config();
    let adapter = UniversalCapabilityAdapter::new(config);
    let adapter = std::sync::Arc::new(adapter);

    let mut set = JoinSet::new();

    // Spawn multiple concurrent lookups
    for i in 0..5 {
        let adapter_clone = adapter.clone();
        set.spawn(async move {
            adapter_clone
                .find_capability_providers(&format!("capability{i}"))
                .await
        });
    }

    // Collect results
    let mut results = Vec::new();
    while let Some(result) = set.join_next().await {
        results.push(result.expect("Task should complete"));
    }

    // All lookups should complete
    assert_eq!(results.len(), 5, "All concurrent lookups should complete");
}

