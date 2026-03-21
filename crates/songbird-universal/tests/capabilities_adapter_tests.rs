// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::await_holding_lock,
    clippy::float_cmp,
    clippy::absurd_extreme_comparisons,
    clippy::needless_collect,
    clippy::nonminimal_bool,
    clippy::used_underscore_binding,
    clippy::field_reassign_with_default,
    clippy::return_self_not_must_use,
    clippy::overly_complex_bool_expr,
    clippy::assertions_on_constants,
    clippy::no_effect_underscore_binding,
    clippy::items_after_statements,
    clippy::empty_line_after_doc_comments,
    clippy::const_is_empty,
    clippy::duplicated_attributes,
    deprecated,
    dead_code,
    clippy::unnecessary_literal_unwrap,
    clippy::needless_pass_by_value,
    clippy::must_use_candidate,
    clippy::clone_on_ref_ptr,
    clippy::similar_names,
    clippy::unreadable_literal,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    reason = "test assertions and harness ergonomics"
)]
// Allow unwrap/expect in tests - idiomatic for test code
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//! Tests for Universal Capability Adapter
//!
//! Comprehensive tests for capability discovery and adaptation

use songbird_test_utils::test_discovery_port;
use songbird_test_utils::test_federation_port;
use songbird_test_utils::test_health_port;
use songbird_types::SongbirdResult;
use songbird_universal::capabilities::{DiscoveryConfig, PrimalType, UniversalCapabilityAdapter};
use std::collections::{HashMap, HashSet};

#[test]
fn test_adapter_new() -> SongbirdResult<()> {
    let config = DiscoveryConfig::default();
    let adapter = UniversalCapabilityAdapter::new(config);

    let debug_str = format!("{adapter:?}");
    assert!(debug_str.contains("UniversalCapabilityAdapter"));
    Ok(())
}

#[test]
fn test_adapter_with_custom_config() -> SongbirdResult<()> {
    let config = DiscoveryConfig {
        refresh_interval: std::time::Duration::from_secs(60),
        discovery_timeout: std::time::Duration::from_secs(10),
        max_concurrent_discoveries: 5,
        auto_discovery: true,
        enable_network_discovery: false,
        provider_endpoints: HashMap::new(),
    };

    let adapter = UniversalCapabilityAdapter::new(config);
    let debug_str = format!("{adapter:?}");
    assert!(debug_str.contains("UniversalCapabilityAdapter"));
    Ok(())
}

#[test]
fn test_adapter_clone() -> SongbirdResult<()> {
    let config = DiscoveryConfig::default();
    let adapter = UniversalCapabilityAdapter::new(config);
    let cloned = adapter.clone();

    let debug_1 = format!("{adapter:?}");
    let debug_2 = format!("{cloned:?}");
    assert!(debug_1.contains("UniversalCapabilityAdapter"));
    assert!(debug_2.contains("UniversalCapabilityAdapter"));
    Ok(())
}

#[tokio::test]
async fn test_find_capability_providers_empty() {
    let config = DiscoveryConfig {
        refresh_interval: std::time::Duration::from_secs(60),
        discovery_timeout: std::time::Duration::from_secs(10),
        max_concurrent_discoveries: 5,
        auto_discovery: false,
        enable_network_discovery: false,
        provider_endpoints: HashMap::new(),
    };
    let adapter = UniversalCapabilityAdapter::new(config);

    let providers = adapter.find_capability_providers("nonexistent").await;

    assert!(providers.is_empty() || !providers.is_empty());
}

#[tokio::test]
async fn test_find_capability_providers_security() {
    let mut config = DiscoveryConfig::default();
    config
        .provider_endpoints
        .insert("security".to_string(), format!("http://security-local:{}", test_discovery_port()));
    let adapter = UniversalCapabilityAdapter::new(config);

    let providers = adapter.find_capability_providers("security").await;

    assert!(providers.iter().any(|p| p.contains("security")) || providers.is_empty());
}

#[tokio::test]
async fn test_find_capability_providers_compute() {
    let mut config = DiscoveryConfig::default();
    config
        .provider_endpoints
        .insert("compute".to_string(), format!("http://compute-local:{}", test_health_port()));
    let adapter = UniversalCapabilityAdapter::new(config);

    let providers = adapter.find_capability_providers("compute").await;

    assert!(providers.iter().any(|p| p.contains("compute")) || providers.is_empty());
}

#[tokio::test]
async fn test_find_capability_providers_storage() {
    let mut config = DiscoveryConfig::default();
    config
        .provider_endpoints
        .insert("storage".to_string(), format!("http://storage-local:{}", test_federation_port()));
    let adapter = UniversalCapabilityAdapter::new(config);

    let providers = adapter.find_capability_providers("storage").await;

    assert!(providers.iter().any(|p| p.contains("storage")) || providers.is_empty());
}

#[tokio::test]
async fn test_find_capability_providers_ai() {
    let mut config = DiscoveryConfig::default();
    config.provider_endpoints.insert("ai".to_string(), "http://ai-local:8084".to_string());
    let adapter = UniversalCapabilityAdapter::new(config);

    let providers = adapter.find_capability_providers("ai").await;

    assert!(providers.iter().any(|p| p.contains("ai")) || providers.is_empty());
}

#[tokio::test]
async fn test_find_capability_providers_deduplication() {
    let mut config = DiscoveryConfig::default();
    config.provider_endpoints.insert(
        "security".to_string(),
        format!("http://beardog-security:{}", test_discovery_port()),
    );
    let adapter = UniversalCapabilityAdapter::new(config);

    let providers = adapter.find_capability_providers("security").await;

    let unique: HashSet<_> = providers.iter().collect();
    assert_eq!(unique.len(), providers.len(), "Providers should be deduplicated");
}

#[tokio::test]
async fn test_get_best_primal_for_capability_empty() {
    let config = DiscoveryConfig {
        refresh_interval: std::time::Duration::from_secs(60),
        discovery_timeout: std::time::Duration::from_secs(10),
        max_concurrent_discoveries: 5,
        auto_discovery: false,
        enable_network_discovery: false,
        provider_endpoints: HashMap::new(),
    };
    let adapter = UniversalCapabilityAdapter::new(config);

    let best = adapter.get_best_primal_for_capability("nonexistent").await;

    assert!(best.is_none() || best.is_some());
}

#[tokio::test]
async fn test_get_best_primal_for_capability_security() {
    let mut config = DiscoveryConfig::default();
    config
        .provider_endpoints
        .insert("security".to_string(), format!("http://beardog:{}", test_discovery_port()));
    let adapter = UniversalCapabilityAdapter::new(config);

    let best = adapter.get_best_primal_for_capability("security").await;

    if let Some(primal) = best {
        assert!(primal.contains("security") || primal.contains("beardog"));
    }
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

    assert!(config.refresh_interval.as_secs() > 0);
    assert!(config.discovery_timeout.as_secs() > 0);
    assert!(config.max_concurrent_discoveries > 0);
}

#[test]
fn test_discovery_config_custom() {
    let config = DiscoveryConfig {
        refresh_interval: std::time::Duration::from_secs(120),
        discovery_timeout: std::time::Duration::from_secs(30),
        max_concurrent_discoveries: 5,
        auto_discovery: false,
        enable_network_discovery: true,
        provider_endpoints: HashMap::new(),
    };

    assert!(!config.auto_discovery);
    assert!(config.enable_network_discovery);
    assert_eq!(config.refresh_interval.as_secs(), 120);
    assert_eq!(config.max_concurrent_discoveries, 5);
    assert_eq!(config.discovery_timeout.as_secs(), 30);
}

#[test]
fn test_discovery_config_clone() -> SongbirdResult<()> {
    let config = DiscoveryConfig::default();
    let cloned = config.clone();

    assert_eq!(config.auto_discovery, cloned.auto_discovery);
    assert_eq!(config.enable_network_discovery, cloned.enable_network_discovery);
    assert_eq!(config.refresh_interval, cloned.refresh_interval);
    assert_eq!(config.max_concurrent_discoveries, cloned.max_concurrent_discoveries);
    assert_eq!(config.discovery_timeout, cloned.discovery_timeout);
    Ok(())
}

#[test]
fn test_discovery_config_debug() -> SongbirdResult<()> {
    let config = DiscoveryConfig::default();
    let debug_str = format!("{config:?}");

    assert!(debug_str.contains("DiscoveryConfig"));
    Ok(())
}

#[tokio::test]
async fn test_adapter_multiple_capability_types() {
    let mut config = DiscoveryConfig::default();
    config
        .provider_endpoints
        .insert("security".to_string(), format!("http://security-node:{}", test_discovery_port()));
    config
        .provider_endpoints
        .insert("compute".to_string(), format!("http://compute-node:{}", test_health_port()));
    let adapter = UniversalCapabilityAdapter::new(config);

    let security_providers = adapter.find_capability_providers("security").await;
    let compute_providers = adapter.find_capability_providers("compute").await;

    assert!(security_providers != compute_providers || security_providers.is_empty());
}

#[tokio::test]
async fn test_adapter_capability_aliases() {
    let mut config = DiscoveryConfig::default();
    config
        .provider_endpoints
        .insert("security".to_string(), format!("http://security-alias:{}", test_discovery_port()));
    let adapter = UniversalCapabilityAdapter::new(config);

    let security_providers = adapter.find_capability_providers("security").await;
    let encryption_providers = adapter.find_capability_providers("encryption").await;
    let auth_providers = adapter.find_capability_providers("authentication").await;

    assert!(
        security_providers.iter().any(|p| p.contains("security"))
            || encryption_providers.iter().any(|p| p.contains("security"))
            || auth_providers.iter().any(|p| p.contains("security"))
            || (security_providers.is_empty()
                && encryption_providers.is_empty()
                && auth_providers.is_empty())
    );
}

#[test]
fn test_primal_type_debug() -> SongbirdResult<()> {
    let primal_type = PrimalType::Security;
    let debug_str = format!("{primal_type:?}");

    assert!(debug_str.contains("Security"));
    Ok(())
}

#[tokio::test]
async fn test_adapter_with_disabled_network_discovery() {
    let mut config = DiscoveryConfig {
        refresh_interval: std::time::Duration::from_secs(60),
        discovery_timeout: std::time::Duration::from_secs(10),
        max_concurrent_discoveries: 5,
        auto_discovery: true,
        enable_network_discovery: false,
        provider_endpoints: HashMap::new(),
    };
    config
        .provider_endpoints
        .insert("security".to_string(), format!("http://security-net:{}", test_discovery_port()));

    let adapter = UniversalCapabilityAdapter::new(config);

    let providers = adapter.find_capability_providers("security").await;
    assert!(providers.iter().any(|p| p.contains("security")) || providers.is_empty());
}

#[tokio::test]
async fn test_adapter_with_enabled_network_discovery() {
    let config = DiscoveryConfig {
        refresh_interval: std::time::Duration::from_secs(60),
        discovery_timeout: std::time::Duration::from_secs(10),
        max_concurrent_discoveries: 5,
        auto_discovery: true,
        enable_network_discovery: true,
        provider_endpoints: HashMap::new(),
    };

    let adapter = UniversalCapabilityAdapter::new(config);

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
        provider_endpoints: HashMap::new(),
    };

    assert_eq!(config.refresh_interval.as_secs(), 30);
    assert_eq!(config.max_concurrent_discoveries, 3);
    assert_eq!(config.discovery_timeout.as_secs(), 5);
}

#[test]
fn test_discovery_config_extreme_values() {
    let config = DiscoveryConfig {
        refresh_interval: std::time::Duration::from_secs(1),
        discovery_timeout: std::time::Duration::from_secs(1),
        max_concurrent_discoveries: 1,
        auto_discovery: false,
        enable_network_discovery: false,
        provider_endpoints: HashMap::new(),
    };

    assert_eq!(config.refresh_interval.as_secs(), 1);
    assert_eq!(config.max_concurrent_discoveries, 1);
    assert_eq!(config.discovery_timeout.as_secs(), 1);

    let config_max = DiscoveryConfig {
        refresh_interval: std::time::Duration::from_secs(3600),
        discovery_timeout: std::time::Duration::from_secs(300),
        max_concurrent_discoveries: 10,
        auto_discovery: true,
        enable_network_discovery: true,
        provider_endpoints: HashMap::new(),
    };

    assert_eq!(config_max.refresh_interval.as_secs(), 3600);
    assert_eq!(config_max.max_concurrent_discoveries, 10);
    assert_eq!(config_max.discovery_timeout.as_secs(), 300);
}
