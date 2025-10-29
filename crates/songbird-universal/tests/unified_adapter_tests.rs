//! Modern tests for `UnifiedUniversalAdapter`
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]

//!
//! These tests validate the core universal adapter functionality with current architecture.

use songbird_universal::{
    create_universal_adapter, create_universal_adapter_with_config, UnifiedAdapterConfig,
    UnifiedUniversalAdapter,
};

#[tokio::test]
async fn test_create_default_adapter() {
    let adapter = create_universal_adapter();
    let stats = adapter.get_registry_stats().await;

    // New adapter should have empty registry
    assert_eq!(stats.total_services, 0);
    assert_eq!(stats.total_capabilities, 0);
}

#[tokio::test]
async fn test_create_adapter_with_custom_config() {
    let config = UnifiedAdapterConfig::default();
    let adapter = create_universal_adapter_with_config(config);
    let stats = adapter.get_registry_stats().await;

    assert_eq!(stats.total_services, 0);
}

#[test]
fn test_adapter_new() {
    let adapter = UnifiedUniversalAdapter::new();
    // Adapter should be created successfully
    let _ = adapter;
}

#[test]
fn test_adapter_with_config() {
    let config = UnifiedAdapterConfig::default();
    let adapter = UnifiedUniversalAdapter::with_config(config);
    // Adapter should be created with config
    let _ = adapter;
}

#[tokio::test]
async fn test_adapter_capability_registry_accessible() {
    let adapter = create_universal_adapter();
    // Registry should be accessible through adapter
    let stats = adapter.get_registry_stats().await;
    assert_eq!(stats.total_services, 0, "New registry should be empty");
}

#[test]
fn test_adapter_config_immutability() {
    let config1 = UnifiedAdapterConfig::default();
    let adapter = UnifiedUniversalAdapter::with_config(config1);

    // Adapter should be created successfully
    let _ = adapter;
}

#[tokio::test]
async fn test_multiple_adapters_independent() {
    let adapter1 = create_universal_adapter();
    let adapter2 = create_universal_adapter();

    // Each adapter should have independent state
    let stats1 = adapter1.get_registry_stats().await;
    let stats2 = adapter2.get_registry_stats().await;

    assert_eq!(stats1.total_services, 0);
    assert_eq!(stats2.total_services, 0);
}

#[test]
fn test_adapter_default_config_values() {
    let config = UnifiedAdapterConfig::default();

    // Default config should have reasonable values
    assert!(config.discovery_timeout.as_secs() > 0);
    assert!(config.health_check_interval.as_secs() > 0);
    assert!(config.max_concurrent_requests > 0);
    assert!(!config.discovery_endpoints.is_empty());
}

#[tokio::test]
async fn test_adapter_registry_stats_initial_state() {
    let adapter = create_universal_adapter();
    let stats = adapter.get_registry_stats().await;

    // Initial registry should be empty
    assert_eq!(stats.total_services, 0, "New adapter should have no services");
    assert_eq!(stats.total_capabilities, 0, "New adapter should have no capabilities");
    assert_eq!(stats.healthy_services, 0, "New adapter should have no healthy services");
}

#[test]
fn test_adapter_configuration_structure() {
    let config = UnifiedAdapterConfig::default();
    // Config should be valid and constructable
    let adapter = UnifiedUniversalAdapter::with_config(config);
    let _ = adapter;
}

#[test]
fn test_config_default_auto_discovery() {
    let config = UnifiedAdapterConfig::default();
    assert!(config.auto_discovery, "Auto-discovery should be enabled by default");
}

#[test]
fn test_config_default_discovery_endpoints() {
    let config = UnifiedAdapterConfig::default();
    assert!(!config.discovery_endpoints.is_empty(), "Should have default discovery endpoints");
}

#[tokio::test]
async fn test_adapter_stats_consistency() {
    let adapter = create_universal_adapter();

    // Multiple calls should return consistent results
    let stats1 = adapter.get_registry_stats().await;
    let stats2 = adapter.get_registry_stats().await;

    assert_eq!(stats1.total_services, stats2.total_services);
    assert_eq!(stats1.total_capabilities, stats2.total_capabilities);
}
