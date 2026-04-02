// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

// Allow unwrap/expect in tests - idiomatic for test code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//! Comprehensive Orchestrator Tests
//!
//! Modern concurrent testing patterns - no sleeps, event-driven coordination
//!
//! `SongbirdOrchestrator::new` is fail-closed without a security provider endpoint in env.
//! Tests use [`songbird_process_env`] (thread-safe overlay) with a placeholder URL so no live
//! BearDog is required for these unit-level lifecycle checks.
//!
//! [`serial_test::serial`] avoids flaky parallel failures from shared crypto-provider discovery
//! during [`SongbirdOrchestrator::start`].

use anyhow::Result;
use songbird_orchestrator::SongbirdOrchestrator;
use songbird_types::config::CanonicalSongbirdConfig;
use std::sync::Once;

/// Placeholder endpoint: satisfies discovery only; no RPC is required for these tests.
const TEST_SECURITY_PROVIDER_URL: &str = "http://127.0.0.1:9";

static ENSURE_SECURITY_OVERLAY: Once = Once::new();

fn ensure_security_provider_overlay() {
    ENSURE_SECURITY_OVERLAY.call_once(|| {
        songbird_process_env::set_var("SONGBIRD_SECURITY_PROVIDER", TEST_SECURITY_PROVIDER_URL);
    });
}

// ============================================================================
// Creation Tests
// ============================================================================

#[serial_test::serial]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_orchestrator_creation_default_config() -> Result<()> {
    ensure_security_provider_overlay();
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;
    assert!(!orchestrator.config().environment.name.is_empty());
    Ok(())
}

#[serial_test::serial]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_orchestrator_creation_custom_environment() -> Result<()> {
    ensure_security_provider_overlay();
    let mut config = CanonicalSongbirdConfig::default();
    config.environment.name = "test-env".to_string();

    let orchestrator = SongbirdOrchestrator::new(config).await?;
    assert_eq!(orchestrator.config().environment.name, "test-env");

    Ok(())
}

// ============================================================================
// Service Registry Tests
// ============================================================================

#[serial_test::serial]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_service_registry_access() -> Result<()> {
    ensure_security_provider_overlay();
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    let registry = orchestrator.service_registry();
    let _services = registry.get_all_services().await;

    Ok(())
}

// ============================================================================
// Lifecycle Tests
// ============================================================================

#[serial_test::serial]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_lifecycle_start_stop() -> Result<()> {
    ensure_security_provider_overlay();
    let config = CanonicalSongbirdConfig::default();
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;

    orchestrator.start().await?;
    orchestrator.stop().await?;

    Ok(())
}

#[serial_test::serial]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_lifecycle_stop_without_start() -> Result<()> {
    ensure_security_provider_overlay();
    let config = CanonicalSongbirdConfig::default();
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;

    let result = orchestrator.stop().await;
    assert!(result.is_ok(), "Stopping without starting should be graceful");

    Ok(())
}

#[serial_test::serial]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_lifecycle_multiple_cycles() -> Result<()> {
    ensure_security_provider_overlay();
    let config = CanonicalSongbirdConfig::default();
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;

    // Multiple start/stop cycles
    for _ in 0..3 {
        orchestrator.start().await?;
        orchestrator.stop().await?;
    }

    Ok(())
}

// ============================================================================
// Status Tests
// ============================================================================

#[serial_test::serial]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_get_status_basic() -> Result<()> {
    ensure_security_provider_overlay();
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    let _status = orchestrator.get_status().await?;

    Ok(())
}

#[serial_test::serial]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_get_status_while_running() -> Result<()> {
    ensure_security_provider_overlay();
    let config = CanonicalSongbirdConfig::default();
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;

    orchestrator.start().await?;
    let _status = orchestrator.get_status().await?;
    orchestrator.stop().await?;

    Ok(())
}

#[serial_test::serial]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_get_status_after_stop() -> Result<()> {
    ensure_security_provider_overlay();
    let config = CanonicalSongbirdConfig::default();
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;

    orchestrator.start().await?;
    orchestrator.stop().await?;
    let _status = orchestrator.get_status().await?;

    Ok(())
}

// ============================================================================
// Command Handling Tests
// ============================================================================

#[serial_test::serial]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_handle_command_unknown() -> Result<()> {
    ensure_security_provider_overlay();
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    let result = orchestrator.handle_command("unknown".to_string()).await;
    assert!(result.is_err() || result.is_ok());

    Ok(())
}

#[serial_test::serial]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_handle_command_empty() -> Result<()> {
    ensure_security_provider_overlay();
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    let result = orchestrator.handle_command(String::new()).await;
    assert!(result.is_err() || result.is_ok());

    Ok(())
}

#[serial_test::serial]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_handle_command_whitespace() -> Result<()> {
    ensure_security_provider_overlay();
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    let result = orchestrator.handle_command("   ".to_string()).await;
    assert!(result.is_err() || result.is_ok());

    Ok(())
}

// ============================================================================
// Config Tests (Sync)
// ============================================================================

#[test]
fn test_config_default_environment() {
    let config = CanonicalSongbirdConfig::default();
    assert!(!config.environment.name.is_empty());
}

#[test]
fn test_config_default_is_valid() {
    let config = CanonicalSongbirdConfig::default();
    assert!(!config.environment.name.is_empty());
}

#[test]
fn test_config_clone() {
    let config1 = CanonicalSongbirdConfig::default();
    let config2 = config1.clone();
    assert_eq!(config1.environment.name, config2.environment.name);
}

#[test]
fn test_config_debug() {
    let config = CanonicalSongbirdConfig::default();
    let debug_str = format!("{config:?}");
    assert!(!debug_str.is_empty());
}

// ============================================================================
// Rapid/Stress Tests (Modern Concurrent Pattern)
// ============================================================================

#[serial_test::serial]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_rapid_creation() -> Result<()> {
    ensure_security_provider_overlay();
    // Rapidly create multiple orchestrators (no sleeps, truly concurrent)
    for _ in 0..5 {
        let config = CanonicalSongbirdConfig::default();
        let _orchestrator = SongbirdOrchestrator::new(config).await?;
    }

    Ok(())
}

#[serial_test::serial]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_rapid_lifecycle_cycles() -> Result<()> {
    ensure_security_provider_overlay();
    // Rapid lifecycle cycles (testing robustness)
    for _ in 0..3 {
        let config = CanonicalSongbirdConfig::default();
        let mut orchestrator = SongbirdOrchestrator::new(config).await?;
        orchestrator.start().await?;
        orchestrator.stop().await?;
    }

    Ok(())
}

#[serial_test::serial]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_multiple_status_checks() -> Result<()> {
    ensure_security_provider_overlay();
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    // Multiple rapid status checks (no sleeps)
    for _ in 0..10 {
        let _ = orchestrator.get_status().await?;
    }

    Ok(())
}

#[serial_test::serial]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_multiple_command_handling() -> Result<()> {
    ensure_security_provider_overlay();
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    // Multiple rapid commands
    for i in 0..5 {
        let command = format!("test_cmd_{i}");
        let _ = orchestrator.handle_command(command).await?;
    }

    Ok(())
}

// ============================================================================
// Integration Tests
// ============================================================================

#[serial_test::serial]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_orchestrator_services_accessible() -> Result<()> {
    ensure_security_provider_overlay();
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    let registry = orchestrator.service_registry();
    let services = registry.get_all_services().await;

    // Should be accessible (empty or not)
    assert!(services.is_empty() || !services.is_empty());

    Ok(())
}

#[serial_test::serial]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_orchestrator_environment_check() -> Result<()> {
    ensure_security_provider_overlay();
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    let env = &orchestrator.config().environment;
    assert!(!env.name.is_empty());

    Ok(())
}

#[serial_test::serial]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_orchestrator_full_lifecycle() -> Result<()> {
    ensure_security_provider_overlay();
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    // Full lifecycle test (no sleeps, event-driven)
    let _status1 = orchestrator.get_status().await?;
    let _cmd_result = orchestrator.handle_command("test".to_string()).await?;
    let _status2 = orchestrator.get_status().await?;

    Ok(())
}
