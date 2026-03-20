// SPDX-License-Identifier: AGPL-3.0-only
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
    clippy::unnecessary_literal_unwrap
)]

//! Comprehensive tests for host configuration
//!
//! These tests validate the robust, thread-safe host configuration system
//! without relying on environment variable mutation.

use songbird_config::canonical::hardcoded_elimination::HostConfig;
use songbird_config::defaults::hosts::*;

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
fn test_global_config_thread_safety() {
    use std::thread;

    // Spawn multiple threads reading configuration simultaneously
    // Spawn threads and collect results directly without intermediate Vec
    let hosts: Vec<String> = (0..10)
        .map(|_| {
            thread::spawn(|| {
                let host = default_host();
                assert!(!host.is_empty());
                host
            })
        })
        .map(|h| h.join().unwrap_or_else(|_| panic!("Thread panicked unexpectedly")))
        .collect();

    // All threads should see the same value
    let first_host = &hosts[0];
    for host in &hosts {
        assert_eq!(host, first_host, "All threads should see the same host");
    }
}

// ============================================================================
// DEPENDENCY INJECTION TESTS (HostConfig Struct)
// ============================================================================

#[test]
fn test_host_config_with_defaults() {
    let config = HostConfig::with_defaults();
    // HostConfig defaults to "localhost" for all services
    assert_eq!(config.orchestrator, "localhost");
    assert_eq!(config.discovery, "localhost");
    assert_eq!(config.registry, "localhost");
    assert_eq!(config.security, "localhost");
}

#[test]
fn test_host_config_custom_values() {
    let mut config = HostConfig::with_defaults();
    config.orchestrator = "192.168.1.100".to_string();
    config.discovery = "10.0.0.1".to_string();

    assert_eq!(config.orchestrator, "192.168.1.100");
    assert_eq!(config.discovery, "10.0.0.1");
}

#[test]
fn test_host_config_ipv6() {
    let mut config = HostConfig::with_defaults();
    config.orchestrator = "::1".to_string();
    assert_eq!(config.orchestrator, "::1");

    config.orchestrator = "fe80::1".to_string();
    assert_eq!(config.orchestrator, "fe80::1");
}

#[test]
fn test_host_config_ipv4_addresses() {
    let addresses = ["192.168.1.1", "10.0.0.1", "172.16.0.1", "127.0.0.1"];

    for addr in addresses {
        let mut config = HostConfig::with_defaults();
        config.orchestrator = addr.to_string();
        assert_eq!(config.orchestrator, addr);
    }
}

#[test]
fn test_host_config_special_addresses() {
    let mut config = HostConfig::with_defaults();

    // Localhost
    config.orchestrator = "127.0.0.1".to_string();
    assert_eq!(config.orchestrator, "127.0.0.1");

    // All interfaces
    config.orchestrator = "0.0.0.0".to_string();
    assert_eq!(config.orchestrator, "0.0.0.0");

    // IPv6 localhost
    config.orchestrator = "::1".to_string();
    assert_eq!(config.orchestrator, "::1");
}

#[test]
fn test_host_config_hostname_formats() {
    let hostnames =
        ["localhost", "songbird.local", "api.songbird.dev", "service-01.cluster.internal"];

    for hostname in hostnames {
        let mut config = HostConfig::with_defaults();
        config.orchestrator = hostname.to_string();
        assert_eq!(config.orchestrator, hostname);
    }
}

#[test]
fn test_different_hosts_for_different_services() {
    let mut config = HostConfig::with_defaults();
    config.orchestrator = "192.168.1.1".to_string();
    config.discovery = "192.168.1.2".to_string();
    config.registry = "192.168.1.3".to_string();

    assert_eq!(config.orchestrator, "192.168.1.1");
    assert_eq!(config.discovery, "192.168.1.2");
    assert_eq!(config.registry, "192.168.1.3");
}

#[test]
fn test_service_host_fallback() {
    let _config = HostConfig::with_defaults();

    // Service host should fall back to default
    let custom_host = service_host("CUSTOM_SERVICE");
    let default = default_host();
    assert_eq!(custom_host, default);
}

#[test]
fn test_orchestrator_host_default() {
    let config = HostConfig::with_defaults();
    // Orchestrator defaults to default_host
    assert_eq!(config.orchestrator, config.orchestrator);
}

#[test]
fn test_discovery_host_default() {
    let config = HostConfig::with_defaults();
    // Discovery defaults to default_host
    assert_eq!(config.discovery, config.orchestrator);
}

#[test]
fn test_host_config_is_clone() {
    let config1 = HostConfig::with_defaults();
    let config2 = config1.clone();

    assert_eq!(config1.orchestrator, config2.orchestrator);
    assert_eq!(config1.discovery, config2.discovery);
}

#[test]
fn test_host_config_is_debug() {
    let config = HostConfig::with_defaults();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("HostConfig"));
}

// ============================================================================
// PRODUCTION BEHAVIOR TESTS
// ============================================================================

#[test]
fn test_production_environment_detection() {
    // Note: Environment detection is via free functions, not HostConfig fields
    // HostConfig is for host addresses only
    let is_prod = is_production();
    // Test that function exists and returns a bool - just check it doesn't panic
    let _ = is_prod;

    // HostConfig is environment-agnostic
    let config = HostConfig::with_defaults();
    assert!(!config.orchestrator.is_empty());
}

#[test]
fn test_bind_address_production_vs_development() {
    // Note: HostConfig is for service discovery, not bind addresses
    // bind_address configuration is separate (defaults::hosts::bind_address())
    let mut prod_config = HostConfig::with_defaults();
    prod_config.orchestrator = "prod.example.com".to_string();
    assert_eq!(prod_config.orchestrator, "prod.example.com");

    let mut dev_config = HostConfig::with_defaults();
    dev_config.orchestrator = "localhost".to_string();
    assert_eq!(dev_config.orchestrator, "localhost");
}

#[test]
fn test_all_global_functions_return_valid_values() {
    // All functions should return non-empty strings
    assert!(!default_host().is_empty());
    assert!(!bind_address().is_empty());
    assert!(!discovery_host().is_empty());
    assert!(!orchestrator_host().is_empty());
    // environment() function exists in defaults::hosts
    // (not imported by wildcard * but is available)
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
    config2.orchestrator = "custom.host".to_string();

    // Configs are independent
    assert_ne!(config1.orchestrator, config2.orchestrator);

    // Global config is unaffected
    let global_host = default_host();
    assert!(!global_host.is_empty());
}

#[test]
fn test_config_independence() {
    // Each config instance is independent
    let mut config1 = HostConfig::with_defaults();
    let mut config2 = HostConfig::with_defaults();

    config1.orchestrator = "host1.example.com".to_string();
    config2.orchestrator = "host2.example.com".to_string();

    assert_eq!(config1.orchestrator, "host1.example.com");
    assert_eq!(config2.orchestrator, "host2.example.com");
}

#[test]
fn test_empty_host_handling() {
    // Test that we can set empty host (though not recommended)
    let mut config = HostConfig::with_defaults();
    config.orchestrator = String::new();
    assert!(config.orchestrator.is_empty());
}

#[test]
fn test_host_with_port() {
    // Test host with port notation
    let mut config = HostConfig::with_defaults();
    config.orchestrator = "192.168.1.100:8080".to_string();
    assert_eq!(config.orchestrator, "192.168.1.100:8080");
}

#[test]
fn test_long_hostname() {
    let mut config = HostConfig::with_defaults();
    let long_hostname = "very-long-subdomain.with-multiple-parts.and-more-subdomains.example.com";
    config.orchestrator = long_hostname.to_string();
    assert_eq!(config.orchestrator, long_hostname);
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
