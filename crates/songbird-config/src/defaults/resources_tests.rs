// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Comprehensive tests for default resource configuration
//!
//! Phase 3 Test Coverage Expansion - Week 1
//! Target: 0% → 100% coverage for resources.rs (57 lines)

use super::*;
use songbird_test_utils::ScopedEnv;

// =============================================================================
// MAX SERVICES TESTS
// =============================================================================

#[tokio::test]
async fn test_max_services_default() {
    let _env = ScopedEnv::remove("SONGBIRD_MAX_SERVICES").await;
    assert_eq!(max_services(), 1000);
}

#[tokio::test]
async fn test_max_services_from_env() {
    let _env = ScopedEnv::set("SONGBIRD_MAX_SERVICES", "5000").await;
    assert_eq!(max_services(), 5000);
}

#[tokio::test]
async fn test_max_services_invalid_env() {
    let _env = ScopedEnv::set("SONGBIRD_MAX_SERVICES", "invalid").await;
    assert_eq!(max_services(), 1000); // Falls back to default
}

// =============================================================================
// MAX CACHE SIZE TESTS
// =============================================================================

#[tokio::test]
async fn test_max_cache_size_default() {
    let _env = ScopedEnv::remove("SONGBIRD_MAX_CACHE_SIZE").await;
    assert_eq!(max_cache_size(), 10_000);
}

#[tokio::test]
async fn test_max_cache_size_from_env() {
    let _env = ScopedEnv::set("SONGBIRD_MAX_CACHE_SIZE", "50000").await;
    assert_eq!(max_cache_size(), 50_000);
}

#[tokio::test]
async fn test_max_cache_size_invalid_env() {
    let _env = ScopedEnv::set("SONGBIRD_MAX_CACHE_SIZE", "not_a_number").await;
    assert_eq!(max_cache_size(), 10_000); // Falls back to default
}

// =============================================================================
// BUFFER POOL SIZE TESTS
// =============================================================================

#[tokio::test]
async fn test_buffer_pool_size_default() {
    let _env = ScopedEnv::remove("SONGBIRD_BUFFER_POOL_SIZE").await;
    assert_eq!(get_buffer_pool_size(), 100);
}

#[tokio::test]
async fn test_buffer_pool_size_from_env() {
    let _env = ScopedEnv::set("SONGBIRD_BUFFER_POOL_SIZE", "200").await;
    assert_eq!(get_buffer_pool_size(), 200);
}

#[tokio::test]
async fn test_buffer_pool_size_invalid_env() {
    let _env = ScopedEnv::set("SONGBIRD_BUFFER_POOL_SIZE", "xyz").await;
    assert_eq!(get_buffer_pool_size(), 100); // Falls back to default
}

#[tokio::test]
async fn test_buffer_pool_size_zero() {
    let _env = ScopedEnv::set("SONGBIRD_BUFFER_POOL_SIZE", "0").await;
    assert_eq!(get_buffer_pool_size(), 0);
}

// =============================================================================
// MAX CONNECTIONS TESTS
// =============================================================================

#[tokio::test]
async fn test_max_connections_default() {
    let _env = ScopedEnv::remove("SONGBIRD_MAX_CONNECTIONS").await;
    assert_eq!(get_max_connections(), 1000);
}

#[tokio::test]
async fn test_max_connections_from_env() {
    let _env = ScopedEnv::set("SONGBIRD_MAX_CONNECTIONS", "2000").await;
    assert_eq!(get_max_connections(), 2000);
}

#[tokio::test]
async fn test_max_connections_invalid_env() {
    let _env = ScopedEnv::set("SONGBIRD_MAX_CONNECTIONS", "abc").await;
    assert_eq!(get_max_connections(), 1000); // Falls back to default
}

#[tokio::test]
async fn test_max_connections_large_value() {
    let _env = ScopedEnv::set("SONGBIRD_MAX_CONNECTIONS", "100000").await;
    assert_eq!(get_max_connections(), 100_000);
}

// =============================================================================
// MAX SESSIONS TESTS
// =============================================================================

#[tokio::test]
async fn test_max_sessions_default() {
    let _env = ScopedEnv::remove("SONGBIRD_MAX_SESSIONS").await;
    assert_eq!(get_max_sessions(), 1000);
}

#[tokio::test]
async fn test_max_sessions_from_env() {
    let _env = ScopedEnv::set("SONGBIRD_MAX_SESSIONS", "3000").await;
    assert_eq!(get_max_sessions(), 3000);
}

#[tokio::test]
async fn test_max_sessions_invalid_env() {
    let _env = ScopedEnv::set("SONGBIRD_MAX_SESSIONS", "invalid_value").await;
    assert_eq!(get_max_sessions(), 1000); // Falls back to default
}

#[tokio::test]
async fn test_max_sessions_edge_cases() {
    let _env = ScopedEnv::set("SONGBIRD_MAX_SESSIONS", "1").await;
    assert_eq!(get_max_sessions(), 1);
}

// =============================================================================
// INTEGRATION TESTS
// =============================================================================

#[tokio::test]
async fn test_all_defaults_together() {
    let _env = ScopedEnv::remove_multiple([
        "SONGBIRD_MAX_SERVICES",
        "SONGBIRD_MAX_CACHE_SIZE",
        "SONGBIRD_BUFFER_POOL_SIZE",
        "SONGBIRD_MAX_CONNECTIONS",
        "SONGBIRD_MAX_SESSIONS",
    ])
    .await;

    assert_eq!(max_services(), 1000);
    assert_eq!(max_cache_size(), 10_000);
    assert_eq!(get_buffer_pool_size(), 100);
    assert_eq!(get_max_connections(), 1000);
    assert_eq!(get_max_sessions(), 1000);
}

#[tokio::test]
async fn test_all_from_env_together() {
    let _env = ScopedEnv::set_multiple([
        ("SONGBIRD_MAX_SERVICES", "5000"),
        ("SONGBIRD_MAX_CACHE_SIZE", "50000"),
        ("SONGBIRD_BUFFER_POOL_SIZE", "200"),
        ("SONGBIRD_MAX_CONNECTIONS", "2000"),
        ("SONGBIRD_MAX_SESSIONS", "3000"),
    ])
    .await;

    assert_eq!(max_services(), 5000);
    assert_eq!(max_cache_size(), 50_000);
    assert_eq!(get_buffer_pool_size(), 200);
    assert_eq!(get_max_connections(), 2000);
    assert_eq!(get_max_sessions(), 3000);
}

#[tokio::test]
async fn test_resource_limits_are_reasonable() {
    let _env = ScopedEnv::remove_multiple([
        "SONGBIRD_MAX_SERVICES",
        "SONGBIRD_MAX_CACHE_SIZE",
        "SONGBIRD_BUFFER_POOL_SIZE",
        "SONGBIRD_MAX_CONNECTIONS",
        "SONGBIRD_MAX_SESSIONS",
    ])
    .await;

    // Verify defaults are reasonable for production use
    assert!(max_services() >= 100, "Max services should support at least 100 services");
    assert!(max_cache_size() >= 1000, "Cache should support at least 1000 entries");
    assert!(get_buffer_pool_size() >= 10, "Buffer pool should have at least 10 buffers");
    assert!(get_max_connections() >= 100, "Should support at least 100 connections");
    assert!(get_max_sessions() >= 100, "Should support at least 100 sessions");
}
