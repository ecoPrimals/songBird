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
    clippy::unnecessary_literal_unwrap,
    reason = "test assertions and harness ergonomics"
)]

//! Coverage tests for `songbird_config::capability_discovery`
//!
//! Tests the capability-based service discovery system.

use songbird_config::capability_discovery::{
    CapabilityDiscovery, DiscoveryMethod, ServiceEndpoint,
};
use std::sync::Mutex;

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

// ==================== CONSTRUCTION TESTS ====================

#[test]
fn test_discovery_default() {
    let _g = lock_env();
    let d = CapabilityDiscovery::default();
    // Default should not panic
    drop(d);
}

#[test]
fn test_discovery_new() {
    let _g = lock_env();
    let d = CapabilityDiscovery::new();
    drop(d);
}

#[test]
fn test_discovery_with_methods() {
    let _g = lock_env();
    let d = CapabilityDiscovery::with_methods(vec![DiscoveryMethod::Environment]);
    drop(d);
}

#[test]
fn test_discovery_with_multiple_methods() {
    let _g = lock_env();
    let d = CapabilityDiscovery::with_methods(vec![
        DiscoveryMethod::Environment,
        DiscoveryMethod::DnsSD,
        DiscoveryMethod::MDNS,
    ]);
    drop(d);
}

// ==================== DISCOVERY METHOD TESTS ====================

#[test]
fn test_discovery_method_equality() {
    assert_eq!(DiscoveryMethod::Environment, DiscoveryMethod::Environment);
    assert_eq!(DiscoveryMethod::DnsSD, DiscoveryMethod::DnsSD);
    assert_eq!(DiscoveryMethod::MDNS, DiscoveryMethod::MDNS);
    assert_ne!(DiscoveryMethod::Environment, DiscoveryMethod::DnsSD);
}

#[test]
fn test_discovery_method_registry() {
    let r = DiscoveryMethod::Registry {
        endpoint: "http://localhost:8500".to_string(),
    };
    assert_eq!(
        r,
        DiscoveryMethod::Registry {
            endpoint: "http://localhost:8500".to_string()
        }
    );
}

#[test]
fn test_discovery_method_config_file() {
    let c = DiscoveryMethod::ConfigFile {
        path: "/etc/songbird/services.yaml".to_string(),
    };
    if let DiscoveryMethod::ConfigFile {
        path,
    } = &c
    {
        assert_eq!(path, "/etc/songbird/services.yaml");
    } else {
        panic!("Expected ConfigFile variant");
    }
}

#[test]
fn test_discovery_method_debug() {
    let m = DiscoveryMethod::Environment;
    let debug = format!("{m:?}");
    assert!(debug.contains("Environment"));
}

// ==================== ENVIRONMENT DISCOVERY TESTS ====================

#[tokio::test]
async fn test_env_compute_discovery() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("COMPUTE_ENDPOINT", "http://compute.local:9001");

    let d = CapabilityDiscovery::with_methods(vec![DiscoveryMethod::Environment]);
    let providers = d.discover_compute().await.unwrap();
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0].url, "http://compute.local:9001");
    assert!(providers[0].capabilities.contains(&"compute".to_string()));
}

#[tokio::test]
async fn test_env_storage_discovery() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("STORAGE_ENDPOINT", "http://storage.local:9002");

    let d = CapabilityDiscovery::with_methods(vec![DiscoveryMethod::Environment]);
    let providers = d.discover_storage().await.unwrap();
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0].url, "http://storage.local:9002");
}

#[tokio::test]
async fn test_env_security_discovery() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SECURITY_ENDPOINT", "http://security.local:9003");

    let d = CapabilityDiscovery::with_methods(vec![DiscoveryMethod::Environment]);
    let providers = d.discover_security().await.unwrap();
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0].url, "http://security.local:9003");
}

#[tokio::test]
async fn test_env_ai_discovery() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("AI_ENDPOINT", "http://ai.local:9004");

    let d = CapabilityDiscovery::with_methods(vec![DiscoveryMethod::Environment]);
    let providers = d.discover_ai().await.unwrap();
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0].url, "http://ai.local:9004");
}

#[tokio::test]
async fn test_no_providers_returns_error() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("NONEXISTENT_ENDPOINT");
    env.remove("NONEXISTENT_URL");

    let d = CapabilityDiscovery::with_methods(vec![DiscoveryMethod::Environment]);
    let result = d.find_providers_by_capability("nonexistent").await;
    assert!(result.is_err(), "Should error when no providers found");
}

// ==================== CACHE TESTS ====================

#[tokio::test]
async fn test_cache_returns_same_results() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("CACHETEST_ENDPOINT", "http://cache.local:5555");

    let d = CapabilityDiscovery::with_methods(vec![DiscoveryMethod::Environment]);

    let first = d.find_providers_by_capability("cachetest").await.unwrap();
    let second = d.find_providers_by_capability("cachetest").await.unwrap();
    assert_eq!(first.len(), second.len());
    assert_eq!(first[0].url, second[0].url);
}

#[tokio::test]
async fn test_clear_cache() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("CLEARCACHE_ENDPOINT", "http://clear.local:5556");

    let d = CapabilityDiscovery::with_methods(vec![DiscoveryMethod::Environment]);
    let _ = d.find_providers_by_capability("clearcache").await.unwrap();

    d.clear_cache("clearcache").await;
    // After clearing, should re-discover
    let result = d.find_providers_by_capability("clearcache").await.unwrap();
    assert_eq!(result.len(), 1);
}

#[tokio::test]
async fn test_clear_all_caches() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("CLEARALL_ENDPOINT", "http://clearall.local:5557");

    let d = CapabilityDiscovery::with_methods(vec![DiscoveryMethod::Environment]);
    let _ = d.find_providers_by_capability("clearall").await.unwrap();

    d.clear_all_caches().await;
    // After clearing, should re-discover
    let result = d.find_providers_by_capability("clearall").await.unwrap();
    assert_eq!(result.len(), 1);
}

// ==================== SERVICE ENDPOINT TESTS ====================

#[test]
fn test_service_endpoint_fields() {
    let ep = ServiceEndpoint {
        id: "test-svc".to_string(),
        url: "http://test:8080".to_string(),
        capabilities: vec!["compute".to_string(), "ai".to_string()],
        health_score: 0.95,
        last_seen: std::time::SystemTime::now(),
    };

    assert_eq!(ep.id, "test-svc");
    assert_eq!(ep.url, "http://test:8080");
    assert_eq!(ep.capabilities.len(), 2);
    assert!((ep.health_score - 0.95).abs() < f64::EPSILON);
}

#[test]
fn test_service_endpoint_clone() {
    let ep = ServiceEndpoint {
        id: "clone-test".to_string(),
        url: "http://test:8080".to_string(),
        capabilities: vec!["compute".to_string()],
        health_score: 1.0,
        last_seen: std::time::SystemTime::now(),
    };

    let cloned = ep.clone();
    assert_eq!(cloned.id, ep.id);
    assert_eq!(cloned.url, ep.url);
}

#[test]
fn test_service_endpoint_debug() {
    let ep = ServiceEndpoint {
        id: "debug-test".to_string(),
        url: "http://test:8080".to_string(),
        capabilities: vec![],
        health_score: 0.5,
        last_seen: std::time::SystemTime::now(),
    };

    let debug = format!("{ep:?}");
    assert!(debug.contains("debug-test"));
}

#[test]
fn test_service_endpoint_serialization() {
    let ep = ServiceEndpoint {
        id: "serde-test".to_string(),
        url: "http://test:8080".to_string(),
        capabilities: vec!["compute".to_string()],
        health_score: 0.8,
        last_seen: std::time::SystemTime::now(),
    };

    let json = serde_json::to_string(&ep).unwrap();
    assert!(json.contains("serde-test"));

    let deserialized: ServiceEndpoint = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, "serde-test");
    assert_eq!(deserialized.url, "http://test:8080");
}
