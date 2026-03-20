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

//! Coverage tests for `songbird_config::zero_touch::infant_config`
//!
//! Tests the zero-touch configuration system including type construction,
//! serialization, discovery configuration, and environment-based defaults.

use songbird_config::zero_touch::infant_config::*;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

struct ScopedEnv {
    vars: Vec<(String, Option<String>)>,
}

impl ScopedEnv {
    fn new() -> Self {
        Self {
            vars: Vec::new(),
        }
    }

    fn set(&mut self, key: &str, value: &str) -> &mut Self {
        let old = std::env::var(key).ok();
        self.vars.push((key.to_string(), old));
        songbird_process_env::set_var(key, value);
        self
    }

    fn remove(&mut self, key: &str) -> &mut Self {
        let old = std::env::var(key).ok();
        self.vars.push((key.to_string(), old));
        songbird_process_env::remove_var(key);
        self
    }
}

impl Drop for ScopedEnv {
    fn drop(&mut self) {
        for (key, old) in self.vars.drain(..).rev() {
            match old {
                Some(val) => songbird_process_env::set_var(&key, &val),
                None => songbird_process_env::remove_var(&key),
            }
        }
    }
}

// ==================== TYPE CONSTRUCTION TESTS ====================

#[test]
fn test_security_level_variants() {
    let levels = vec![
        SecurityLevel::None,
        SecurityLevel::Basic,
        SecurityLevel::Encrypted,
        SecurityLevel::StrongAuth,
        SecurityLevel::Maximum,
    ];
    for level in &levels {
        let debug = format!("{level:?}");
        assert!(!debug.is_empty());
    }
    assert_eq!(SecurityLevel::None, SecurityLevel::None);
    assert_ne!(SecurityLevel::None, SecurityLevel::Maximum);
}

#[test]
fn test_security_level_serialization() {
    let level = SecurityLevel::StrongAuth;
    let json = serde_json::to_string(&level).unwrap();
    let deserialized: SecurityLevel = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, SecurityLevel::StrongAuth);
}

#[test]
fn test_fallback_behavior_fail() {
    let fb = FallbackBehavior::Fail;
    let json = serde_json::to_string(&fb).unwrap();
    assert!(json.contains("Fail"));
}

#[test]
fn test_fallback_behavior_retry() {
    let fb = FallbackBehavior::Retry {
        max_attempts: 3,
        backoff_ms: 100,
    };
    let json = serde_json::to_string(&fb).unwrap();
    let deserialized: FallbackBehavior = serde_json::from_str(&json).unwrap();
    if let FallbackBehavior::Retry {
        max_attempts,
        backoff_ms,
    } = deserialized
    {
        assert_eq!(max_attempts, 3);
        assert_eq!(backoff_ms, 100);
    } else {
        panic!("Expected Retry variant");
    }
}

#[test]
fn test_fallback_behavior_degraded() {
    let fb = FallbackBehavior::DegradedMode {
        degraded_operations: vec!["read_only".to_string()],
    };
    let debug = format!("{fb:?}");
    assert!(debug.contains("DegradedMode"));
}

#[test]
fn test_fallback_behavior_local() {
    let fb = FallbackBehavior::LocalFallback;
    let json = serde_json::to_string(&fb).unwrap();
    let deserialized: FallbackBehavior = serde_json::from_str(&json).unwrap();
    assert!(matches!(deserialized, FallbackBehavior::LocalFallback));
}

#[test]
fn test_quality_requirements() {
    let qr = QualityRequirements {
        max_response_time_ms: Some(500),
        min_availability: Some(0.999),
        min_throughput_rps: Some(1000.0),
        security_level: SecurityLevel::Encrypted,
    };
    let json = serde_json::to_string(&qr).unwrap();
    let de: QualityRequirements = serde_json::from_str(&json).unwrap();
    assert_eq!(de.max_response_time_ms, Some(500));
    assert_eq!(de.security_level, SecurityLevel::Encrypted);
}

#[test]
fn test_quality_requirements_optional_fields() {
    let qr = QualityRequirements {
        max_response_time_ms: None,
        min_availability: None,
        min_throughput_rps: None,
        security_level: SecurityLevel::None,
    };
    let json = serde_json::to_string(&qr).unwrap();
    assert!(json.contains("null"));
}

#[test]
fn test_capability_requirement() {
    let cr = CapabilityRequirement {
        capability_type: "security".to_string(),
        required_operations: vec!["encrypt".to_string(), "decrypt".to_string()],
        quality_requirements: QualityRequirements {
            max_response_time_ms: Some(100),
            min_availability: Some(0.99),
            min_throughput_rps: None,
            security_level: SecurityLevel::StrongAuth,
        },
        fallback_behavior: FallbackBehavior::Fail,
    };
    assert_eq!(cr.capability_type, "security");
    assert_eq!(cr.required_operations.len(), 2);
}

#[test]
fn test_service_identity() {
    let id = ServiceIdentity {
        service_id: "songbird-node-1".to_string(),
        provides_capabilities: vec!["discovery".to_string(), "orchestration".to_string()],
        metadata: HashMap::from([
            ("version".to_string(), "0.2.1".to_string()),
            ("region".to_string(), "us-east-1".to_string()),
        ]),
    };
    assert_eq!(id.service_id, "songbird-node-1");
    assert_eq!(id.provides_capabilities.len(), 2);
    assert_eq!(id.metadata.len(), 2);
}

// ==================== DISCOVERY CONFIG TESTS ====================

#[test]
fn test_discovery_method_environment() {
    let dm = DiscoveryMethod::Environment {
        patterns: vec!["CAPABILITY_*_ENDPOINT".to_string()],
    };
    let debug = format!("{dm:?}");
    assert!(debug.contains("Environment"));
}

#[test]
fn test_discovery_method_http_registry() {
    let dm = DiscoveryMethod::HttpRegistry {
        endpoint_env_var: "REGISTRY_ENDPOINT".to_string(),
        api_path: "/v1/services".to_string(),
    };
    let json = serde_json::to_string(&dm).unwrap();
    assert!(json.contains("HttpRegistry"));
}

#[test]
fn test_discovery_method_dns_srv() {
    let dm = DiscoveryMethod::DnsSrv {
        domain_env_var: "SERVICE_DOMAIN".to_string(),
    };
    let json = serde_json::to_string(&dm).unwrap();
    let de: DiscoveryMethod = serde_json::from_str(&json).unwrap();
    if let DiscoveryMethod::DnsSrv {
        domain_env_var,
    } = de
    {
        assert_eq!(domain_env_var, "SERVICE_DOMAIN");
    } else {
        panic!("Expected DnsSrv");
    }
}

#[test]
fn test_discovery_config() {
    let dc = DiscoveryConfig {
        methods: vec![DiscoveryMethod::Environment {
            patterns: vec!["*_ENDPOINT".to_string()],
        }],
        timeout: Duration::from_secs(30),
        refresh_interval: Duration::from_secs(300),
        enable_cache: true,
        cache_ttl: Duration::from_secs(600),
    };
    assert!(dc.enable_cache);
    assert_eq!(dc.timeout, Duration::from_secs(30));
}

// ==================== NETWORK CONFIG TESTS ====================

#[test]
fn test_network_config() {
    let nc = NetworkConfig {
        bind_address: std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
        service_port: 8080,
        health_port: 8081,
        metrics_port: 9090,
        connection_limits: ConnectionLimits {
            max_connections: 1000,
            max_connections_per_ip: 100,
            connection_backlog: 128,
        },
        timeouts: NetworkTimeouts {
            connection_timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(60),
            idle_timeout: Duration::from_secs(300),
        },
    };
    assert_eq!(nc.service_port, 8080);
    assert_eq!(nc.connection_limits.max_connections, 1000);
    assert_eq!(nc.timeouts.connection_timeout, Duration::from_secs(30));
}

#[test]
fn test_connection_limits_serialization() {
    let cl = ConnectionLimits {
        max_connections: 5000,
        max_connections_per_ip: 50,
        connection_backlog: 256,
    };
    let json = serde_json::to_string(&cl).unwrap();
    let de: ConnectionLimits = serde_json::from_str(&json).unwrap();
    assert_eq!(de.max_connections, 5000);
}

#[test]
fn test_network_timeouts_serialization() {
    let nt = NetworkTimeouts {
        connection_timeout: Duration::from_secs(10),
        request_timeout: Duration::from_secs(30),
        idle_timeout: Duration::from_secs(120),
    };
    let json = serde_json::to_string(&nt).unwrap();
    let de: NetworkTimeouts = serde_json::from_str(&json).unwrap();
    assert_eq!(de.request_timeout, Duration::from_secs(30));
}

// ==================== BOOTSTRAP CONFIG TESTS ====================

#[test]
fn test_bootstrap_config() {
    let bc = BootstrapConfig {
        enable_infant_discovery: true,
        discovery_phases: vec![
            DiscoveryPhase::EnvironmentScan,
            DiscoveryPhase::RegistryQuery,
            DiscoveryPhase::NetworkProbe,
            DiscoveryPhase::CapabilityTest,
        ],
        max_bootstrap_time: Duration::from_secs(60),
        fail_on_missing_required: true,
    };
    assert!(bc.enable_infant_discovery);
    assert!(bc.fail_on_missing_required);
    assert_eq!(bc.discovery_phases.len(), 4);
    assert_eq!(bc.max_bootstrap_time, Duration::from_secs(60));
}

#[test]
fn test_discovery_phase_variants() {
    let phases = vec![
        DiscoveryPhase::EnvironmentScan,
        DiscoveryPhase::RegistryQuery,
        DiscoveryPhase::NetworkProbe,
        DiscoveryPhase::CapabilityTest,
    ];
    for phase in &phases {
        let debug = format!("{phase:?}");
        assert!(!debug.is_empty());
    }
    assert_eq!(DiscoveryPhase::EnvironmentScan, DiscoveryPhase::EnvironmentScan);
    assert_ne!(DiscoveryPhase::EnvironmentScan, DiscoveryPhase::NetworkProbe);
}

// ==================== ZERO TOUCH CONFIG TESTS ====================

#[test]
fn test_zero_touch_config_from_environment() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SERVICE_ID", "test-node-42");
    env.set("SERVICE_PORT", "8080");
    env.remove("SONGBIRD_ENV");

    let config = ZeroTouchConfig::from_environment();
    assert!(config.is_ok(), "from_environment should succeed: {:?}", config.err());
    let config = config.unwrap();
    assert_eq!(config.self_identity.service_id, "test-node-42");
}

#[test]
fn test_zero_touch_config_default_service_id() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SERVICE_ID");
    env.remove("HOSTNAME");
    env.set("SERVICE_PORT", "8080");

    let config = ZeroTouchConfig::from_environment();
    assert!(config.is_ok());
    let config = config.unwrap();
    // Should generate a default service ID
    assert!(!config.self_identity.service_id.is_empty());
}

#[test]
fn test_zero_touch_config_serialization() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SERVICE_ID", "serde-test");
    env.set("SERVICE_PORT", "8080");

    let config = ZeroTouchConfig::from_environment().unwrap();
    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("serde-test"));

    let deserialized: ZeroTouchConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.self_identity.service_id, "serde-test");
}

#[test]
fn test_zero_touch_config_has_discovery_methods() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SERVICE_ID", "discovery-test");
    env.set("SERVICE_PORT", "8080");

    let config = ZeroTouchConfig::from_environment().unwrap();
    assert!(!config.discovery.methods.is_empty(), "Should have at least one discovery method");
}

#[test]
fn test_zero_touch_config_network() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SERVICE_ID", "network-test");
    env.set("SERVICE_PORT", "9999");

    let config = ZeroTouchConfig::from_environment().unwrap();
    assert!(config.network.connection_limits.max_connections > 0);
}

#[test]
fn test_zero_touch_config_debug() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SERVICE_ID", "debug-test");
    env.set("SERVICE_PORT", "8080");

    let config = ZeroTouchConfig::from_environment().unwrap();
    let debug = format!("{config:?}");
    assert!(debug.contains("ZeroTouchConfig"));
}

#[test]
fn test_zero_touch_config_clone() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SERVICE_ID", "clone-test");
    env.set("SERVICE_PORT", "8080");

    let config = ZeroTouchConfig::from_environment().unwrap();
    let cloned = config.clone();
    assert_eq!(config.self_identity.service_id, cloned.self_identity.service_id);
}
