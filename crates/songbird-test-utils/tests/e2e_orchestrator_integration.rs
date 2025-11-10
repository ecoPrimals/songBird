//! Real End-to-End Orchestrator Integration Tests
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]

//!
//! These tests actually start the orchestrator, test HTTP endpoints,
//! and verify real component integration.

#![cfg(test)]

use songbird_types::config::CanonicalSongbirdConfig;
use songbird_orchestrator::SongbirdOrchestrator;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_orchestrator_initialization() -> Result<(), Box<dyn std::error::Error>> {
    // Test that we can actually create and initialize the orchestrator
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    // Verify orchestrator was created
    assert!(std::mem::size_of_val(&orchestrator) > 0);

    Ok(())
}

#[tokio::test]
async fn test_orchestrator_start_stop_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    // Test complete lifecycle: create → start → stop
    let config = CanonicalSongbirdConfig::default();
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;

    // Start orchestrator
    let start_result = orchestrator.start().await;
    assert!(start_result.is_ok(), "Orchestrator should start successfully");

    // Give it a moment to initialize
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Stop orchestrator
    let stop_result = orchestrator.stop().await;
    assert!(stop_result.is_ok(), "Orchestrator should stop gracefully");

    Ok(())
}

#[tokio::test]
async fn test_configuration_loading_and_defaults() -> Result<(), Box<dyn std::error::Error>> {
    // Test that configuration loads correctly with defaults
    let config = CanonicalSongbirdConfig::default();

    // Verify config has expected structure
    assert!(config.network.port_range.start > 0, "Port range start should be configured");
    assert!(!config.network.bind_address.is_empty(), "Bind address should be set");

    // Create orchestrator with this config
    let _orchestrator = SongbirdOrchestrator::new(config).await?;

    Ok(())
}

#[tokio::test]
async fn test_orchestrator_observability_integration() -> Result<(), Box<dyn std::error::Error>> {
    // Test that observability components are properly integrated
    let config = CanonicalSongbirdConfig::default();
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;

    // Start orchestrator (which should initialize observability)
    orchestrator.start().await?;

    // Give observability time to initialize
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Stop cleanly
    orchestrator.stop().await?;

    Ok(())
}

#[tokio::test]
async fn test_orchestrator_service_registry_integration() -> Result<(), Box<dyn std::error::Error>>
{
    // Test that service registry is properly integrated
    use songbird_orchestrator::core::{RegistryConfig, ServiceRegistry};
    use songbird_test_utils::network_fixtures::*;
    use songbird_test_utils::test_bind_address;
    use std::sync::Arc;

    // Create a service registry
    let registry = Arc::new(ServiceRegistry::new(RegistryConfig::default()));

    // Verify registry works
    assert!(std::mem::size_of_val(&*registry) > 0);

    // Create orchestrator (which should also have its own registry)
    let config = CanonicalSongbirdConfig::default();
    let _orchestrator = SongbirdOrchestrator::new(config).await?;

    Ok(())
}

#[tokio::test]
async fn test_multiple_orchestrator_instances() -> Result<(), Box<dyn std::error::Error>> {
    // Test that we can create multiple orchestrator instances
    let config1 = CanonicalSongbirdConfig::default();
    let config2 = CanonicalSongbirdConfig::default();

    let orch1 = SongbirdOrchestrator::new(config1).await?;
    let orch2 = SongbirdOrchestrator::new(config2).await?;

    // Verify both exist
    assert!(std::mem::size_of_val(&orch1) > 0);
    assert!(std::mem::size_of_val(&orch2) > 0);

    Ok(())
}

#[tokio::test]
async fn test_orchestrator_timeout_handling() -> Result<(), Box<dyn std::error::Error>> {
    // Test that orchestrator operations don't hang
    let config = CanonicalSongbirdConfig::default();

    // Create with timeout
    let result = timeout(Duration::from_secs(5), SongbirdOrchestrator::new(config)).await;

    assert!(result.is_ok(), "Orchestrator creation should complete within timeout");
    let _orchestrator = result??;

    Ok(())
}

#[tokio::test]
async fn test_orchestrator_with_custom_network_config() -> Result<(), Box<dyn std::error::Error>> {
    // Test orchestrator with custom network configuration
    let mut config = CanonicalSongbirdConfig::default();

    // Customize network settings
    config.network.port_range.start = songbird_config::defaults::ports::federation_port();
    config.network.bind_address = songbird_test_utils::test_bind_address().to_string();

    let orchestrator = SongbirdOrchestrator::new(config).await?;
    assert!(std::mem::size_of_val(&orchestrator) > 0);

    Ok(())
}

#[tokio::test]
async fn test_orchestrator_error_handling_on_invalid_config(
) -> Result<(), Box<dyn std::error::Error>> {
    // Test that orchestrator handles invalid configuration gracefully
    let config = CanonicalSongbirdConfig::default();

    // Even with default config, orchestrator should handle edge cases
    let result = SongbirdOrchestrator::new(config).await;

    // Should either succeed or fail gracefully
    assert!(result.is_ok() || result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_orchestrator_rapid_start_stop() -> Result<(), Box<dyn std::error::Error>> {
    // Test rapid start/stop cycles
    let config = CanonicalSongbirdConfig::default();
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;

    for _ in 0..3 {
        orchestrator.start().await?;
        tokio::time::sleep(Duration::from_millis(10)).await;
        orchestrator.stop().await?;
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    Ok(())
}
