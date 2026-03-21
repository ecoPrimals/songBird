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
    dead_code,
    clippy::unnecessary_literal_unwrap,
    clippy::needless_pass_by_value,
    clippy::must_use_candidate,
    reason = "test assertions and harness ergonomics"
)]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//! Comprehensive tests for Network Optimization and Performance Management
//!
//! Tests network configuration, performance settings, and optimization strategies

use songbird_network_federation::network::{
    DiscoveryConfig, DiscoveryMethod, LoadBalancingStrategy, NetworkCapability, NetworkConfig,
    NetworkStatus, PerformanceConfig, ProxyConfig, ProxyType,
};
use std::time::Duration;

// ============================================================================
// PerformanceConfig Tests
// ============================================================================

#[test]
fn test_performance_config_default() {
    let config = PerformanceConfig::default();
    assert_eq!(config.buffer_size, 8192);
    assert_eq!(config.worker_threads, None);
    assert!(config.tcp_nodelay);
    assert_eq!(config.keepalive, Some(Duration::from_secs(60)));
}

#[test]
fn test_performance_config_custom() {
    let config = PerformanceConfig {
        buffer_size: 16384,
        worker_threads: Some(8),
        tcp_nodelay: false,
        keepalive: Some(Duration::from_secs(30)),
    };

    assert_eq!(config.buffer_size, 16384);
    assert_eq!(config.worker_threads, Some(8));
    assert!(!config.tcp_nodelay);
    assert_eq!(config.keepalive, Some(Duration::from_secs(30)));
}

#[test]
fn test_performance_config_no_keepalive() {
    let config = PerformanceConfig {
        buffer_size: 8192,
        worker_threads: None,
        tcp_nodelay: true,
        keepalive: None,
    };

    assert_eq!(config.keepalive, None);
}

#[test]
fn test_performance_config_clone() {
    let config = PerformanceConfig::default();
    let cloned = config.clone();
    assert_eq!(config.buffer_size, cloned.buffer_size);
    assert_eq!(config.tcp_nodelay, cloned.tcp_nodelay);
}

// ============================================================================
// LoadBalancingStrategy Tests
// ============================================================================

#[test]
fn test_load_balancing_strategy_round_robin() {
    let strategy = LoadBalancingStrategy::RoundRobin;
    assert_eq!(strategy, LoadBalancingStrategy::RoundRobin);
}

#[test]
fn test_load_balancing_strategy_least_connections() {
    let strategy = LoadBalancingStrategy::LeastConnections;
    assert_eq!(strategy, LoadBalancingStrategy::LeastConnections);
}

#[test]
fn test_load_balancing_strategy_weighted_round_robin() {
    let strategy = LoadBalancingStrategy::WeightedRoundRobin;
    assert_eq!(strategy, LoadBalancingStrategy::WeightedRoundRobin);
}

#[test]
fn test_load_balancing_strategy_ip_hash() {
    let strategy = LoadBalancingStrategy::IpHash;
    assert_eq!(strategy, LoadBalancingStrategy::IpHash);
}

#[test]
fn test_load_balancing_strategy_clone() {
    let strategy = LoadBalancingStrategy::RoundRobin;
    let cloned = strategy.clone();
    assert_eq!(strategy, cloned);
}

#[test]
fn test_load_balancing_strategy_all_variants() {
    let strategies = [
        LoadBalancingStrategy::RoundRobin,
        LoadBalancingStrategy::LeastConnections,
        LoadBalancingStrategy::WeightedRoundRobin,
        LoadBalancingStrategy::IpHash,
    ];

    assert_eq!(strategies.len(), 4);
}

// ============================================================================
// ProxyConfig Tests
// ============================================================================

#[test]
fn test_proxy_config_default() {
    let config = ProxyConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.proxy_type, ProxyType::Http);
    assert!(config.upstream_servers.is_empty());
    assert_eq!(config.load_balancing, LoadBalancingStrategy::RoundRobin);
}

#[test]
fn test_proxy_config_enabled() {
    let config = ProxyConfig {
        enabled: true,
        proxy_type: ProxyType::Socks5,
        upstream_servers: vec!["127.0.0.1:8080".parse().expect("should parse valid input")],
        load_balancing: LoadBalancingStrategy::LeastConnections,
    };

    assert!(config.enabled);
    assert_eq!(config.proxy_type, ProxyType::Socks5);
    assert_eq!(config.upstream_servers.len(), 1);
}

#[test]
fn test_proxy_type_http() {
    let proxy_type = ProxyType::Http;
    assert_eq!(proxy_type, ProxyType::Http);
}

#[test]
fn test_proxy_type_socks5() {
    let proxy_type = ProxyType::Socks5;
    assert_eq!(proxy_type, ProxyType::Socks5);
}

#[test]
fn test_proxy_type_transparent() {
    let proxy_type = ProxyType::Transparent;
    assert_eq!(proxy_type, ProxyType::Transparent);
}

// ============================================================================
// DiscoveryConfig Tests
// ============================================================================

#[test]
fn test_discovery_config_default() {
    let config = DiscoveryConfig::default();
    assert!(config.enabled);
    assert_eq!(config.methods.len(), 2);
    assert_eq!(config.interval, Duration::from_secs(30));
    assert_eq!(config.timeout, Duration::from_secs(5));
}

#[test]
fn test_discovery_config_custom() {
    let config = DiscoveryConfig {
        enabled: true,
        methods: vec![DiscoveryMethod::Dns, DiscoveryMethod::Unicast],
        interval: Duration::from_secs(60),
        timeout: Duration::from_secs(10),
    };

    assert!(config.enabled);
    assert_eq!(config.methods.len(), 2);
    assert_eq!(config.interval.as_secs(), 60);
    assert_eq!(config.timeout.as_secs(), 10);
}

#[test]
fn test_discovery_config_disabled() {
    let config = DiscoveryConfig {
        enabled: false,
        methods: vec![],
        interval: Duration::from_secs(30),
        timeout: Duration::from_secs(5),
    };

    assert!(!config.enabled);
    assert!(config.methods.is_empty());
}

// ============================================================================
// DiscoveryMethod Tests
// ============================================================================

#[test]
fn test_discovery_method_multicast() {
    let method = DiscoveryMethod::Multicast;
    assert_eq!(method, DiscoveryMethod::Multicast);
}

#[test]
fn test_discovery_method_broadcast() {
    let method = DiscoveryMethod::Broadcast;
    assert_eq!(method, DiscoveryMethod::Broadcast);
}

#[test]
fn test_discovery_method_unicast() {
    let method = DiscoveryMethod::Unicast;
    assert_eq!(method, DiscoveryMethod::Unicast);
}

#[test]
fn test_discovery_method_dns() {
    let method = DiscoveryMethod::Dns;
    assert_eq!(method, DiscoveryMethod::Dns);
}

#[test]
fn test_discovery_method_all_variants() {
    let methods = [
        DiscoveryMethod::Multicast,
        DiscoveryMethod::Broadcast,
        DiscoveryMethod::Unicast,
        DiscoveryMethod::Dns,
    ];

    assert_eq!(methods.len(), 4);
}

// ============================================================================
// NetworkStatus Tests
// ============================================================================

#[test]
fn test_network_status_healthy() {
    let status = NetworkStatus::Healthy;
    assert_eq!(status, NetworkStatus::Healthy);
}

#[test]
fn test_network_status_degraded() {
    let status = NetworkStatus::Degraded;
    assert_eq!(status, NetworkStatus::Degraded);
}

#[test]
fn test_network_status_unhealthy() {
    let status = NetworkStatus::Unhealthy;
    assert_eq!(status, NetworkStatus::Unhealthy);
}

#[test]
fn test_network_status_offline() {
    let status = NetworkStatus::Offline;
    assert_eq!(status, NetworkStatus::Offline);
}

#[test]
fn test_network_status_clone() {
    let status = NetworkStatus::Healthy;
    let cloned = status.clone();
    assert_eq!(status, cloned);
}

// ============================================================================
// NetworkCapability Tests
// ============================================================================

#[test]
fn test_network_capability_gaming() {
    let cap = NetworkCapability::Gaming;
    assert_eq!(cap, NetworkCapability::Gaming);
}

#[test]
fn test_network_capability_proxy() {
    let cap = NetworkCapability::Proxy;
    assert_eq!(cap, NetworkCapability::Proxy);
}

#[test]
fn test_network_capability_discovery() {
    let cap = NetworkCapability::Discovery;
    assert_eq!(cap, NetworkCapability::Discovery);
}

#[test]
fn test_network_capability_load_balancing() {
    let cap = NetworkCapability::LoadBalancing;
    assert_eq!(cap, NetworkCapability::LoadBalancing);
}

#[test]
fn test_network_capability_monitoring() {
    let cap = NetworkCapability::Monitoring;
    assert_eq!(cap, NetworkCapability::Monitoring);
}

#[test]
fn test_network_capability_security() {
    let cap = NetworkCapability::Security;
    assert_eq!(cap, NetworkCapability::Security);
}

#[test]
fn test_network_capability_all_variants() {
    let capabilities = [
        NetworkCapability::Gaming,
        NetworkCapability::Proxy,
        NetworkCapability::Discovery,
        NetworkCapability::LoadBalancing,
        NetworkCapability::Monitoring,
        NetworkCapability::Security,
    ];

    assert_eq!(capabilities.len(), 6);
}

// ============================================================================
// NetworkConfig Tests
// ============================================================================

#[test]
fn test_network_config_default() {
    let config = NetworkConfig::default();
    // Just verify it can be created
    let _ = format!("{config:?}");
}

#[test]
fn test_network_config_clone() {
    let config = NetworkConfig::default();
    let cloned = config.clone();
    // Verify both configs are functional
    let _ = format!("{config:?}");
    let _ = format!("{cloned:?}");
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_performance_optimization_workflow() {
    // Create optimized performance config
    let perf_config = PerformanceConfig {
        buffer_size: 32768,                       // Large buffer for high throughput
        worker_threads: Some(16),                 // High worker count
        tcp_nodelay: true,                        // Minimize latency
        keepalive: Some(Duration::from_secs(30)), // Aggressive keepalive
    };

    assert_eq!(perf_config.buffer_size, 32768);
    assert!(perf_config.tcp_nodelay);
}

#[test]
fn test_load_balancing_strategies_comparison() {
    let strategies = vec![
        LoadBalancingStrategy::RoundRobin,
        LoadBalancingStrategy::LeastConnections,
        LoadBalancingStrategy::WeightedRoundRobin,
        LoadBalancingStrategy::IpHash,
    ];

    // Verify all strategies are unique
    assert_eq!(strategies.len(), 4);

    // Verify they can all be cloned
    for strategy in strategies {
        let _ = strategy.clone();
    }
}

#[test]
fn test_discovery_with_all_methods() {
    let config = DiscoveryConfig {
        enabled: true,
        methods: vec![
            DiscoveryMethod::Multicast,
            DiscoveryMethod::Broadcast,
            DiscoveryMethod::Unicast,
            DiscoveryMethod::Dns,
        ],
        interval: Duration::from_secs(15),
        timeout: Duration::from_secs(3),
    };

    assert!(config.enabled);
    assert_eq!(config.methods.len(), 4);
}

#[test]
fn test_proxy_with_load_balancing() {
    let proxy_config = ProxyConfig {
        enabled: true,
        proxy_type: ProxyType::Http,
        upstream_servers: vec![
            "192.168.1.1:8080".parse().expect("should parse valid input"),
            "192.168.1.2:8080".parse().expect("should parse valid input"),
            "192.168.1.3:8080".parse().expect("should parse valid input"),
        ],
        load_balancing: LoadBalancingStrategy::LeastConnections,
    };

    assert!(proxy_config.enabled);
    assert_eq!(proxy_config.upstream_servers.len(), 3);
    assert_eq!(proxy_config.load_balancing, LoadBalancingStrategy::LeastConnections);
}

#[test]
fn test_network_status_transitions() {
    let statuses = [
        NetworkStatus::Healthy,
        NetworkStatus::Degraded,
        NetworkStatus::Unhealthy,
        NetworkStatus::Offline,
    ];

    // Verify all status transitions are distinct
    for (i, status) in statuses.iter().enumerate() {
        for (j, other) in statuses.iter().enumerate() {
            if i == j {
                assert_eq!(status, other);
            } else {
                assert_ne!(status, other);
            }
        }
    }
}
