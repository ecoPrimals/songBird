// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]
#![allow(clippy::expect_used, reason = "test assertions")]

use super::*;
use std::collections::HashMap;
use std::env::VarError;
use std::sync::{Mutex, OnceLock};

/// `BIOMEOS_SOCKET_DIR` is process-global; serialize tests that mutate it.
static BIOMEOS_SOCKET_DIR_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[test]
fn new_creates_locator_without_panicking() {
    let locator = ServiceLocator::new();
    let config = locator.self_config();
    // Self-config should have valid bind address
    let addr = config.bind_address();
    assert!(addr.ip().is_ipv4() || addr.ip().is_ipv6());
}

#[test]
fn default_matches_new() {
    let from_new = ServiceLocator::new();
    let from_default = ServiceLocator::default();
    assert_eq!(from_new.self_config().bind_address(), from_default.self_config().bind_address());
}

#[test]
fn discover_by_capability_returns_empty_when_nothing_configured() {
    let locator = ServiceLocator::new();
    let results = locator.discover_by_capability("nonexistent-capability");
    assert!(results.is_empty());
}

#[test]
fn discover_from_environment_parses_comma_separated_endpoints() {
    songbird_process_env::set_var(
        "SONGBIRD_CAPABILITY_STORAGE_ENDPOINTS",
        "127.0.0.1:3000,127.0.0.1:3001",
    );

    let locator = ServiceLocator::new();
    let results = locator.discover_by_capability("storage");

    songbird_process_env::remove_var("SONGBIRD_CAPABILITY_STORAGE_ENDPOINTS");

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].port(), 3000);
    assert_eq!(results[1].port(), 3001);
}

#[test]
fn discover_from_environment_handles_dashes_in_capability() {
    songbird_process_env::set_var("SONGBIRD_CAPABILITY_KEY_VALUE_ENDPOINTS", "10.0.0.1:6379");

    let locator = ServiceLocator::new();
    let results = locator.discover_by_capability("key-value");

    songbird_process_env::remove_var("SONGBIRD_CAPABILITY_KEY_VALUE_ENDPOINTS");

    assert_eq!(results.len(), 1);
}

#[test]
fn register_self_does_not_panic() {
    let locator = ServiceLocator::new();
    // Registration will fail gracefully (no registry configured), but should not panic
    let result = locator.register_self(&["compute", "storage"]);
    assert!(result.is_ok());
}

#[test]
fn discover_dns_sd_returns_empty_when_no_matching_sockets() {
    let results = ServiceLocator::discover_from_dns_sd("any-capability");
    assert!(results.is_empty());
}

#[test]
fn discover_dns_sd_resolves_tcp_from_sidecar_next_to_domain_sock() {
    let _guard = BIOMEOS_SOCKET_DIR_TEST_LOCK.get_or_init(|| Mutex::new(())).lock();

    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::File::create(dir.path().join("storage.sock")).expect("touch sock");
    std::fs::write(dir.path().join("storage-ipc-port"), "tcp:127.0.0.1:19190\n").expect("sidecar");

    songbird_process_env::set_var("BIOMEOS_SOCKET_DIR", dir.path().to_str().expect("utf8 path"));

    let results = ServiceLocator::discover_from_dns_sd("storage");

    songbird_process_env::remove_var("BIOMEOS_SOCKET_DIR");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].to_string(), "127.0.0.1:19190");
}

#[test]
fn discover_dns_sd_matches_prefixed_instance_socks() {
    let _guard = BIOMEOS_SOCKET_DIR_TEST_LOCK.get_or_init(|| Mutex::new(())).lock();

    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::File::create(dir.path().join("cache-replica-2.sock")).expect("touch sock");
    std::fs::write(dir.path().join("cache-replica-2-ipc-port"), "127.0.0.1:6400\n")
        .expect("sidecar");

    songbird_process_env::set_var("BIOMEOS_SOCKET_DIR", dir.path().to_str().expect("utf8 path"));

    let results = ServiceLocator::discover_from_dns_sd("cache");

    songbird_process_env::remove_var("BIOMEOS_SOCKET_DIR");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].port(), 6400);
}

#[test]
fn discover_by_capability_with_env_unset_skips_registry_query_when_url_missing() {
    let locator = ServiceLocator::new();
    let vars: HashMap<String, String> = HashMap::new();
    let env = move |key: &str| -> Result<String, VarError> {
        vars.get(key).cloned().ok_or(VarError::NotPresent)
    };
    // No SONGBIRD_CAPABILITY_* , DNS-SD stub empty, registry URL absent → Err on registry → final empty
    assert!(locator.discover_by_capability_with("compute", env).is_empty());
}

#[test]
fn discover_by_capability_with_registry_url_set_returns_empty_until_http_impl() {
    let locator = ServiceLocator::new();
    let vars: HashMap<String, String> = HashMap::from([(
        "SONGBIRD_REGISTRY_URL".to_string(),
        "http://registry.test:8500".to_string(),
    )]);
    let env = move |key: &str| -> Result<String, VarError> {
        vars.get(key).cloned().ok_or(VarError::NotPresent)
    };
    assert!(locator.discover_by_capability_with("storage", env).is_empty());
}

#[test]
fn discover_by_capability_with_empty_env_endpoints_falls_through_chain() {
    let locator = ServiceLocator::new();
    let vars: HashMap<String, String> =
        HashMap::from([("SONGBIRD_CAPABILITY_CACHE_ENDPOINTS".to_string(), String::new())]);
    let env = move |key: &str| -> Result<String, VarError> {
        vars.get(key).cloned().ok_or(VarError::NotPresent)
    };
    assert!(locator.discover_by_capability_with("cache", env).is_empty());
}

#[test]
fn discover_by_capability_with_invalid_tokens_skips_but_keeps_valid_addrs() {
    let locator = ServiceLocator::new();
    let vars: HashMap<String, String> = HashMap::from([(
        "SONGBIRD_CAPABILITY_METRICS_ENDPOINTS".to_string(),
        "not-a-socket,127.0.0.1:9090".to_string(),
    )]);
    let env = move |key: &str| -> Result<String, VarError> {
        vars.get(key).cloned().ok_or(VarError::NotPresent)
    };
    let results = locator.discover_by_capability_with("metrics", env);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], "127.0.0.1:9090".parse().unwrap());
}

#[test]
fn discover_by_capability_with_prefers_env_before_dns_stub() {
    let locator = ServiceLocator::new();
    let vars: HashMap<String, String> = HashMap::from([(
        "SONGBIRD_CAPABILITY_QUEUE_ENDPOINTS".to_string(),
        "[::1]:6000".to_string(),
    )]);
    let env = move |key: &str| -> Result<String, VarError> {
        vars.get(key).cloned().ok_or(VarError::NotPresent)
    };
    let results = locator.discover_by_capability_with("queue", env);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].port(), 6000);
}
