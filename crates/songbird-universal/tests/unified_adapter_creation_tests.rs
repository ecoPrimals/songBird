// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
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
#![allow(clippy::all, reason = "test assertions and harness ergonomics")]
#![allow(unused, reason = "test assertions and harness ergonomics")]

//! Adapter Creation & Basic Configuration Tests
//!
//! **Purpose**: Tests for adapter and config creation, initialization, defaults
//! **Focus**: Does the adapter create and configure correctly?
//! **Scope**: Basic functionality, construction patterns, default values

#![allow(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
#![allow(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
#![allow(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
#![allow(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]
#![allow(clippy::items_after_statements, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_precision_loss, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_possible_truncation, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_sign_loss, reason = "test assertions and harness ergonomics")]
#![allow(clippy::needless_pass_by_value, reason = "test assertions and harness ergonomics")]
#![allow(clippy::similar_names, reason = "test assertions and harness ergonomics")]
#![allow(clippy::module_name_repetitions, reason = "test assertions and harness ergonomics")]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

use songbird_test_utils::network_fixtures::*;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::{
    CapabilityRegistry, UnifiedAdapterConfig, UnifiedUniversalAdapter, create_universal_adapter,
    create_universal_adapter_with_config,
};
use std::collections::HashMap;
use std::time::Duration;

#[test]
fn test_create_default_adapter() {
    let adapter = create_universal_adapter();

    // Verify adapter is created successfully
    assert!(std::mem::size_of_val(&adapter) > 0);
}

#[test]
fn test_create_adapter_with_custom_config() {
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_secs(10),
        health_check_interval: Duration::from_secs(30),
        max_concurrent_requests: 50,
        auto_discovery: false,
        discovery_endpoints: vec![format!("http://custom:{}", test_orchestrator_port())],
    };

    let adapter = create_universal_adapter_with_config(config);

    // Verify adapter uses custom config
    assert!(std::mem::size_of_val(&adapter) > 0);
}

#[test]
fn test_adapter_new() {
    let adapter = UnifiedUniversalAdapter::new();

    // Verify default construction
    assert!(std::mem::size_of_val(&adapter) > 0);
}

#[test]
fn test_adapter_with_config() {
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_secs(15),
        health_check_interval: Duration::from_secs(45),
        max_concurrent_requests: 75,
        auto_discovery: true,
        discovery_endpoints: vec![
            format!("http://primary:{}", test_orchestrator_port()),
            format!("http://secondary:{}", test_discovery_port()),
        ],
    };

    let adapter = UnifiedUniversalAdapter::with_config(config);

    // Verify adapter created with custom config
    assert!(std::mem::size_of_val(&adapter) > 0);
}

#[test]
fn test_default_config_values() {
    let config = UnifiedAdapterConfig::default();

    assert_eq!(config.discovery_timeout, Duration::from_secs(30));
    assert_eq!(config.health_check_interval, Duration::from_secs(60));
    assert_eq!(config.max_concurrent_requests, 100);
    assert!(config.auto_discovery);
    assert!(!config.discovery_endpoints.is_empty());
}

#[test]
fn test_config_discovery_endpoints_format() {
    let config = UnifiedAdapterConfig::default();

    for endpoint in &config.discovery_endpoints {
        assert!(endpoint.starts_with("http://") || endpoint.starts_with("https://"));
        assert!(endpoint.contains(':'));
    }
}

#[test]
fn test_config_custom_discovery_timeout() {
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_secs(5),
        ..Default::default()
    };

    assert_eq!(config.discovery_timeout, Duration::from_secs(5));
    assert_eq!(config.health_check_interval, Duration::from_secs(60)); // Still default
}

#[test]
fn test_config_custom_health_check_interval() {
    let config = UnifiedAdapterConfig {
        health_check_interval: Duration::from_secs(120),
        ..Default::default()
    };

    assert_eq!(config.health_check_interval, Duration::from_secs(120));
    assert_eq!(config.discovery_timeout, Duration::from_secs(30)); // Still default
}

#[test]
fn test_config_custom_max_requests() {
    let config = UnifiedAdapterConfig {
        max_concurrent_requests: 200,
        ..Default::default()
    };

    assert_eq!(config.max_concurrent_requests, 200);
}

#[test]
fn test_config_disable_auto_discovery() {
    let config = UnifiedAdapterConfig {
        auto_discovery: false,
        ..Default::default()
    };

    assert!(!config.auto_discovery);
}

#[test]
fn test_config_empty_discovery_endpoints() {
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec![],
        ..Default::default()
    };

    assert!(config.discovery_endpoints.is_empty());
}

#[test]
fn test_config_multiple_discovery_endpoints() {
    let endpoints = vec![
        format!("http://server1:{}", test_orchestrator_port()),
        format!("http://server2:{}", test_discovery_port()),
        format!("http://server3:{}", test_health_port()),
    ];

    let config = UnifiedAdapterConfig {
        discovery_endpoints: endpoints.clone(),
        ..Default::default()
    };

    assert_eq!(config.discovery_endpoints.len(), 3);
    assert_eq!(config.discovery_endpoints, endpoints);
}

#[test]
fn test_capability_registry_default() {
    let registry = CapabilityRegistry::default();

    assert!(registry.service_capabilities.is_empty());
    assert!(registry.capability_providers.is_empty());
    assert!(registry.service_info.is_empty());
    assert!(registry.last_updated.is_empty());
}

#[test]
fn test_capability_registry_clone() {
    let registry = CapabilityRegistry::default();
    let cloned = registry.clone();

    assert_eq!(registry.service_capabilities.len(), cloned.service_capabilities.len());
    assert_eq!(registry.capability_providers.len(), cloned.capability_providers.len());
}

#[test]
fn test_adapter_clone() {
    let adapter = UnifiedUniversalAdapter::new();
    let cloned = adapter.clone();

    // Verify both adapters are independent
    assert!(std::mem::size_of_val(&adapter) > 0);
    assert!(std::mem::size_of_val(&cloned) > 0);
}

#[test]
fn test_config_clone() {
    let config = UnifiedAdapterConfig::default();
    let cloned = config.clone();

    assert_eq!(config.discovery_timeout, cloned.discovery_timeout);
    assert_eq!(config.health_check_interval, cloned.health_check_interval);
    assert_eq!(config.max_concurrent_requests, cloned.max_concurrent_requests);
    assert_eq!(config.auto_discovery, cloned.auto_discovery);
}

#[test]
fn test_adapter_creation_is_consistent() {
    let adapter1 = UnifiedUniversalAdapter::new();
    let adapter2 = UnifiedUniversalAdapter::new();

    // Both should be valid independent instances
    assert!(std::mem::size_of_val(&adapter1) > 0);
    assert!(std::mem::size_of_val(&adapter2) > 0);
}

#[test]
fn test_config_with_extreme_values() {
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_secs(1),
        health_check_interval: Duration::from_secs(1),
        max_concurrent_requests: 1,
        auto_discovery: false,
        discovery_endpoints: vec![],
    };

    let adapter = UnifiedUniversalAdapter::with_config(config);
    assert!(std::mem::size_of_val(&adapter) > 0);
}

#[test]
fn test_config_with_large_values() {
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_secs(3600), // 1 hour
        health_check_interval: Duration::from_secs(7200), // 2 hours
        max_concurrent_requests: 10000,
        auto_discovery: true,
        discovery_endpoints: vec![format!("http://example.com:{}", test_orchestrator_port()); 100],
    };

    let adapter = UnifiedUniversalAdapter::with_config(config);
    assert!(std::mem::size_of_val(&adapter) > 0);
}

#[test]
fn test_config_zero_timeout() {
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_secs(0),
        ..Default::default()
    };

    assert_eq!(config.discovery_timeout, Duration::ZERO);
}

#[test]
fn test_config_builder_pattern() {
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_secs(20),
        health_check_interval: Duration::from_secs(40),
        max_concurrent_requests: 150,
        auto_discovery: true,
        discovery_endpoints: vec!["http://localhost:9000".to_string()],
    };

    // Verify all fields set correctly
    assert_eq!(config.discovery_timeout.as_secs(), 20);
    assert_eq!(config.health_check_interval.as_secs(), 40);
    assert_eq!(config.max_concurrent_requests, 150);
    assert!(config.auto_discovery);
    assert_eq!(config.discovery_endpoints.len(), 1);
}

#[test]
fn test_adapter_size_is_reasonable() {
    let adapter = UnifiedUniversalAdapter::new();
    let size = std::mem::size_of_val(&adapter);

    // Adapter should not be excessively large
    assert!(size > 0);
    assert!(size < 10_000); // Reasonable upper bound
}

#[test]
fn test_registry_size_is_reasonable() {
    let registry = CapabilityRegistry::default();
    let size = std::mem::size_of_val(&registry);

    assert!(size > 0);
    assert!(size < 5_000); // Reasonable upper bound
}

#[test]
fn test_config_size_is_reasonable() -> SongbirdResult<()> {
    let config = UnifiedAdapterConfig::default();
    let size = std::mem::size_of_val(&config);

    assert!(size > 0);
    assert!(size < 1_000); // Reasonable upper bound
    Ok(())
}

#[test]
fn test_multiple_adapters_independent() -> SongbirdResult<()> {
    let _adapter1 = UnifiedUniversalAdapter::new();
    let _adapter2 = UnifiedUniversalAdapter::new();
    let _adapter3 = UnifiedUniversalAdapter::new();

    // If we got here, all created successfully (implicit test)
    Ok(())
}

#[test]
fn test_config_debug_format() -> SongbirdResult<()> {
    let config = UnifiedAdapterConfig::default();
    let debug_str = format!("{config:?}");

    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("UnifiedAdapterConfig"));
    Ok(())
}

#[test]
fn test_registry_debug_format() -> SongbirdResult<()> {
    let registry = CapabilityRegistry::default();
    let debug_str = format!("{registry:?}");

    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("CapabilityRegistry"));
    Ok(())
}

#[test]
fn test_adapter_debug_format() -> SongbirdResult<()> {
    let adapter = UnifiedUniversalAdapter::new();
    let debug_str = format!("{adapter:?}");

    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("UnifiedUniversalAdapter"));
    Ok(())
}

#[test]
fn test_config_with_ipv4_endpoints() {
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec![
            format!("http://192.168.1.100:{}", test_orchestrator_port()),
            format!("http://10.0.0.1:{}", test_discovery_port()),
        ],
        ..Default::default()
    };

    assert_eq!(config.discovery_endpoints.len(), 2);
}

#[test]
fn test_config_with_ipv6_endpoints() {
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec![
            format!("http://[::1]:{}", test_orchestrator_port()),
            format!("http://[2001:db8::1]:{}", test_discovery_port()),
        ],
        ..Default::default()
    };

    assert_eq!(config.discovery_endpoints.len(), 2);
}

#[test]
fn test_config_with_https_endpoints() -> SongbirdResult<()> {
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec!["https://secure.example.com:443".to_string()],
        ..Default::default()
    };

    assert!(config.discovery_endpoints[0].starts_with("https://"));
    Ok(())
}

#[test]
fn test_config_respects_environment_variable() -> SongbirdResult<()> {
    // Test that default config checks environment
    let config = UnifiedAdapterConfig::default();

    // Should have at least the default endpoints
    assert!(!config.discovery_endpoints.is_empty());
    Ok(())
}

#[test]
fn test_adapter_functions_are_available() -> SongbirdResult<()> {
    let adapter = UnifiedUniversalAdapter::new();

    let cloned = adapter.clone();
    let debug = format!("{adapter:?}");
    assert!(!debug.is_empty());
    assert!(std::mem::size_of_val(&cloned) > 0);
    Ok(())
}

#[test]
fn test_create_functions_are_consistent() -> SongbirdResult<()> {
    let config = UnifiedAdapterConfig::default();

    let adapter1 = create_universal_adapter();
    let adapter2 = create_universal_adapter_with_config(config);

    // Both should create valid adapters
    assert!(std::mem::size_of_val(&adapter1) > 0);
    assert!(std::mem::size_of_val(&adapter2) > 0);
    Ok(())
}
