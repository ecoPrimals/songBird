// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::*;

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
fn discover_dns_sd_returns_empty() {
    // DNS-SD not yet implemented — should return empty, not error
    let results = ServiceLocator::discover_from_dns_sd("any-capability");
    assert!(results.is_empty());
}
