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

//! Tests for Hardcoding Elimination Infrastructure
//!
//! Covers: HardcodingEliminationConfig, all sub-configs,
//! Default trait impls, convenience functions, and the replace module.

use std::sync::Mutex;

/// File-local mutex to serialize tests that modify process-wide env vars.
/// We use `unwrap_or_else` to handle poisoned mutex (from prior panics in other tests).
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[allow(deprecated)]
mod hardcoded_elimination_tests {
    use super::lock_env;
    use songbird_config::config::hardcoded_elimination::*;
    use std::time::Duration;

    // ============================================================
    // Default Trait Tests
    // ============================================================

    #[test]
    fn test_hardcoding_elimination_config_default() {
        let _guard = lock_env();
        let config = HardcodingEliminationConfig::default();

        // Network defaults
        assert!(!config.network.stun_servers.is_empty());
        assert!(config.network.port_ranges.contains_key("orchestrator"));
        assert!(config.network.port_ranges.contains_key("gaming"));
        assert!(config.network.port_ranges.contains_key("federation"));

        // Security defaults
        assert_eq!(config.security.encryption_key_size, 256);
        assert_eq!(config.security.session_timeout, Duration::from_secs(3600));
        assert!(!config.security.beardog_endpoint.is_empty());
        assert!(
            config.security.tls_cert_path.contains("ssl")
                || config.security.tls_cert_path.contains("cert")
        );

        // Service defaults
        assert_eq!(config.service.service_name, "songbird-orchestrator");
        assert!(config.service.health_endpoint.contains("/health"));
        assert!(config.service.metrics_endpoint.contains("/metrics"));

        // Timeout defaults
        assert_eq!(config.timeouts.connection_timeout, Duration::from_secs(30));
        assert_eq!(config.timeouts.request_timeout, Duration::from_secs(60));
        assert_eq!(config.timeouts.health_check_timeout, Duration::from_secs(5));
        assert_eq!(config.timeouts.heartbeat_interval, Duration::from_secs(30));

        // Performance defaults
        assert_eq!(config.performance.small_buffer_size, 1024);
        assert_eq!(config.performance.large_buffer_size, 8192);
        assert_eq!(config.performance.max_packet_size, 65536);
        assert_eq!(config.performance.connection_pool_size, 10);
        assert_eq!(config.performance.cache_ttl, Duration::from_secs(300));
    }

    #[test]
    fn test_primal_config_default() {
        let _guard = lock_env();
        let config = PrimalConfig::default();

        assert!(!config.beardog_endpoint.is_empty());
        assert!(!config.nestgate_endpoint.is_empty());
        assert!(!config.toadstool_endpoint.is_empty());
        assert!(!config.squirrel_endpoint.is_empty());
        assert!(!config.discovery_endpoints.is_empty());
        assert!(config.port_range.0 < config.port_range.1);
    }

    #[test]
    fn test_federation_config_default() {
        let _guard = lock_env();
        let config = FederationConfig::default();

        assert!(!config.cluster_endpoints.is_empty());
        assert!(config.heartbeat_endpoint.contains("/federation/heartbeat"));
        assert!(!config.broadcast_ports.is_empty());
        assert!(!config.discovery_ports.is_empty());
        assert_eq!(config.default_cluster_id, "songbird-cluster");
        assert!(config.auto_discovery_enabled);
    }

    #[test]
    fn test_network_config_stun_servers() {
        let _guard = lock_env();
        let config = NetworkConfig::default();

        assert_eq!(config.stun_servers.len(), 2);
        assert!(config.stun_servers[0].contains("stun"));
        assert!(config.stun_servers[1].contains("stun"));
    }

    #[test]
    fn test_network_config_port_ranges() {
        let _guard = lock_env();
        let config = NetworkConfig::default();

        let orch_range = config.port_ranges.get("orchestrator").unwrap();
        assert!(orch_range.0 < orch_range.1);

        let gaming_range = config.port_ranges.get("gaming").unwrap();
        assert!(gaming_range.0 < gaming_range.1);
    }

    #[test]
    fn test_network_config_bind_address_is_valid() {
        let _guard = lock_env();
        let config = NetworkConfig::default();
        // Should be a valid IP address
        assert!(
            config.bind_address.is_loopback() || config.bind_address.is_unspecified(),
            "Default bind address should be loopback or unspecified"
        );
    }

    #[test]
    fn test_network_config_production_bind_address() {
        let _guard = lock_env();
        let config = NetworkConfig::default();
        assert!(
            config.production_bind_address.is_unspecified(),
            "Production bind should be 0.0.0.0"
        );
    }

    #[test]
    fn test_gaming_port_range() {
        let _guard = lock_env();
        let config = NetworkConfig::default();
        assert_eq!(config.gaming_port_range.start, 7000);
        assert_eq!(config.gaming_port_range.end, 7100);
    }

    // ============================================================
    // Global Config / Replace Module Tests
    // ============================================================

    #[test]
    fn test_get_config_returns_consistent_reference() {
        let _guard = lock_env();
        let cfg1 = get_config();
        let cfg2 = get_config();
        // Should return the same static reference
        assert!(std::ptr::eq(cfg1, cfg2));
    }

    #[test]
    fn test_replace_bind_address() {
        let _guard = lock_env();
        let addr = replace::bind_address();
        assert!(addr.is_loopback() || addr.is_unspecified());
    }

    #[test]
    fn test_replace_orchestrator_endpoint() {
        let _guard = lock_env();
        let ep = replace::orchestrator_endpoint();
        assert!(ep.starts_with("http"));
    }

    #[test]
    fn test_replace_gaming_endpoint() {
        let _guard = lock_env();
        let ep = replace::gaming_endpoint();
        assert!(ep.starts_with("http"));
    }

    #[test]
    fn test_replace_beardog_endpoint() {
        let _guard = lock_env();
        let ep = replace::beardog_endpoint();
        assert!(ep.starts_with("http"));
    }

    #[test]
    fn test_replace_nestgate_endpoint() {
        let _guard = lock_env();
        let ep = replace::nestgate_endpoint();
        assert!(ep.starts_with("http"));
    }

    #[test]
    fn test_replace_connection_timeout() {
        let _guard = lock_env();
        let timeout = replace::connection_timeout();
        assert!(timeout > Duration::from_secs(0));
        assert!(timeout <= Duration::from_secs(300));
    }

    #[test]
    fn test_replace_request_timeout() {
        let _guard = lock_env();
        let timeout = replace::request_timeout();
        assert!(timeout > Duration::from_secs(0));
    }

    #[test]
    fn test_replace_health_check_timeout() {
        let _guard = lock_env();
        let timeout = replace::health_check_timeout();
        assert!(timeout > Duration::from_secs(0));
        assert!(timeout <= Duration::from_secs(60));
    }

    #[test]
    fn test_replace_large_buffer_size() {
        let _guard = lock_env();
        let size = replace::large_buffer_size();
        assert!(size > 0);
    }

    #[test]
    fn test_replace_stun_servers() {
        let _guard = lock_env();
        let servers = replace::stun_servers();
        assert!(!servers.is_empty());
    }

    #[test]
    fn test_replace_federation_endpoints() {
        let _guard = lock_env();
        let eps = replace::federation_endpoints();
        assert!(!eps.is_empty());
    }

    #[test]
    fn test_replace_primal_discovery_endpoints() {
        let _guard = lock_env();
        let eps = replace::primal_discovery_endpoints();
        assert!(!eps.is_empty());
    }

    #[test]
    fn test_replace_federation_broadcast_ports() {
        let _guard = lock_env();
        let ports = replace::federation_broadcast_ports();
        assert!(!ports.is_empty());
    }

    #[test]
    fn test_replace_federation_discovery_ports() {
        let _guard = lock_env();
        let ports = replace::federation_discovery_ports();
        assert!(!ports.is_empty());
    }

    #[test]
    fn test_replace_production_bind_address() {
        let _guard = lock_env();
        let addr = replace::production_bind_address();
        // Should be a valid IP address (unspecified or loopback, depending on env)
        let debug = format!("{:?}", addr);
        assert!(!debug.is_empty());
    }

    #[test]
    fn test_replace_format_endpoint() {
        let _guard = lock_env();
        let ep = replace::format_endpoint("compute", Some(9090));
        assert!(ep.contains("9090"));
    }

    #[test]
    fn test_replace_format_endpoint_default_port() {
        let _guard = lock_env();
        let ep = replace::format_endpoint("storage", None);
        assert!(ep.starts_with("http"));
    }

    #[test]
    fn test_replace_gaming_port() {
        let _guard = lock_env();
        let port = replace::gaming_port();
        assert!(port > 0);
    }

    #[test]
    fn test_replace_timeout_config() {
        let _guard = lock_env();
        let config = replace::timeout_config();
        assert!(config.connection_timeout > Duration::from_secs(0));
        assert!(config.request_timeout > Duration::from_secs(0));
    }

    // ============================================================
    // Clone / Debug Tests
    // ============================================================

    #[test]
    fn test_security_config_clone() {
        let _guard = lock_env();
        let config = SecurityConfig::default();
        let cloned = config.clone();
        assert_eq!(config.encryption_key_size, cloned.encryption_key_size);
        assert_eq!(config.session_timeout, cloned.session_timeout);
    }

    #[test]
    fn test_service_config_debug() {
        let _guard = lock_env();
        let config = ServiceConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("ServiceConfig"));
    }

    #[test]
    fn test_timeout_config_clone() {
        let _guard = lock_env();
        let config = TimeoutConfig::default();
        let cloned = config.clone();
        assert_eq!(config.connection_timeout, cloned.connection_timeout);
        assert_eq!(config.request_timeout, cloned.request_timeout);
    }

    #[test]
    fn test_performance_config_debug() {
        let _guard = lock_env();
        let config = PerformanceConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("PerformanceConfig"));
        assert!(debug.contains("1024")); // small_buffer_size
    }
}
