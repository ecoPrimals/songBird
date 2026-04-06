// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Shared test fixtures for unified adapter tests
//!
//! Reduces duplication and provides consistent test setup patterns

#![allow(clippy::unwrap_used, reason = "test assertions and harness ergonomics")]
use songbird_test_utils::network_fixtures::*;
use songbird_universal::{UnifiedAdapterConfig, UnifiedUniversalAdapter};
use std::time::Duration;
/// Create adapter with test-friendly configuration
///
/// This provides a consistent base configuration for most tests:
/// - Short timeouts for faster test execution
/// - Auto-discovery disabled to avoid network calls
/// - Predictable test endpoints
#[must_use]
pub fn create_test_adapter() -> UnifiedUniversalAdapter {
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_secs(5),
        health_check_interval: Duration::from_secs(30),
        max_concurrent_requests: 50,
        auto_discovery: false,
        discovery_endpoints: vec![format!("http://test:{}", test_orchestrator_port())],
    };
    UnifiedUniversalAdapter::with_config(config)
}
/// Create adapter with custom config modifications
/// Allows tests to override specific config values while keeping sensible defaults.
/// # Example
/// ```no_run
/// let adapter = create_test_adapter_with(|config| {
///     config.max_concurrent_requests = 200;
///     config.auto_discovery = true;
/// });
/// ```
pub fn create_test_adapter_with<F>(f: F) -> UnifiedUniversalAdapter
where
    F: FnOnce(&mut UnifiedAdapterConfig),
{
    let mut config = test_adapter_config();
    f(&mut config);
/// Get standard test adapter configuration
/// Useful for tests that need to inspect or clone the config
pub fn test_adapter_config() -> UnifiedAdapterConfig {
    UnifiedAdapterConfig {
        discovery_endpoints: test_endpoints(),
    }
/// Standard test endpoints
/// Returns a predictable set of endpoints for testing
pub fn test_endpoints() -> Vec<String> {
    vec![
        format!("http://test:{}", test_orchestrator_port()),
        format!("http://test:{}", test_discovery_port()),
    ]
/// Create config with extreme timeout values for stress testing
pub fn extreme_timeout_config() -> UnifiedAdapterConfig {
        discovery_timeout: Duration::from_millis(1),
        health_check_interval: Duration::from_millis(1),
        max_concurrent_requests: 1,
/// Create config with maximum values for load testing
pub fn max_load_config() -> UnifiedAdapterConfig {
        discovery_timeout: Duration::from_secs(300),
        health_check_interval: Duration::from_secs(600),
        max_concurrent_requests: 10000,
/// Create config with minimal valid values
pub fn minimal_config() -> UnifiedAdapterConfig {
        discovery_timeout: Duration::from_secs(1),
        health_check_interval: Duration::from_secs(1),
        discovery_endpoints: vec![format!("http://localhost:{}", test_orchestrator_port())],
