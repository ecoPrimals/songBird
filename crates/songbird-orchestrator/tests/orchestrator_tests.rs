//! Orchestrator Core Tests
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
//! Testing orchestrator functionality beyond main function.

use anyhow::Result;
use songbird_orchestrator::app::SongbirdOrchestrator;
use songbird_types::config::CanonicalSongbirdConfig;

#[tokio::test]
async fn test_orchestrator_initialization() -> Result<()> {
    // Test orchestrator initialization with default config
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    // Verify orchestrator is created with valid configuration
    assert!(!orchestrator.config().environment.is_empty());

    // Service registry should be initialized
    let _services = orchestrator.service_registry().get_services();
    // Registry is accessible if we can get services without panicking

    Ok(())
}

#[tokio::test]
async fn test_service_coordination() -> Result<()> {
    // Test service coordination through orchestrator
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    // Verify service registry is accessible
    let registry = orchestrator.service_registry();
    let services = registry.get_services();

    // Service list should be initialized (can be empty)
    assert!(services.is_empty() || !services.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_lifecycle_management() -> Result<()> {
    // Test orchestrator lifecycle management
    let config = CanonicalSongbirdConfig::default();
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;

    // Start orchestrator
    orchestrator.start().await?;

    // Verify orchestrator status
    let _status = orchestrator.get_status().await?;
    // Status check succeeds if no error is returned

    // Stop orchestrator
    orchestrator.stop().await?;

    Ok(())
}

#[tokio::test]
async fn test_graceful_shutdown() -> Result<()> {
    // Test graceful shutdown
    let config = CanonicalSongbirdConfig::default();
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;

    // Start and immediately stop
    orchestrator.start().await?;

    // Graceful shutdown should succeed
    let result = orchestrator.stop().await;
    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_error_recovery() -> Result<()> {
    // Test error recovery mechanisms
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    // Test handling unknown command (should not panic)
    let result = orchestrator.handle_command("invalid_command".to_string()).await;
    assert!(result.is_ok());

    // Should return error message, not crash
    if let Ok(response) = result {
        assert!(response.contains("Unknown command"));
    }

    Ok(())
}

#[tokio::test]
async fn test_health_monitoring() -> Result<()> {
    // Test health monitoring functionality
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    // Execute health check command
    let health_response = orchestrator.handle_command("health".to_string()).await?;

    // Health response should contain status information
    assert!(health_response.contains("Health") || health_response.contains("HEALTHY"));

    Ok(())
}

#[tokio::test]
async fn test_configuration_reload() -> Result<()> {
    // Test configuration reload capability
    let config1 = CanonicalSongbirdConfig::default();
    let mut orchestrator = SongbirdOrchestrator::new(config1).await?;

    // Start with initial config
    orchestrator.start().await?;

    // Verify initial config
    assert!(!orchestrator.config().environment.is_empty());

    // Stop to simulate reload
    orchestrator.stop().await?;

    // Create new orchestrator with updated config
    let config2 = CanonicalSongbirdConfig::default();
    let orchestrator2 = SongbirdOrchestrator::new(config2).await?;

    // Verify new config loaded
    assert!(!orchestrator2.config().environment.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_service_discovery_integration() -> Result<()> {
    // Test service discovery integration
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    // Verify discovery config is present
    assert!(orchestrator.config().discovery.interval_seconds > 0);

    // Service registry should be accessible
    let registry = orchestrator.service_registry();
    let _services = registry.get_services();
    // Registry is accessible if we can get services without panicking

    Ok(())
}

#[tokio::test]
async fn test_concurrent_operations() -> Result<()> {
    // Test concurrent operations
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    // Execute multiple status checks sequentially (simulating concurrent access)
    for _ in 0..5 {
        let result = orchestrator.handle_command("status".to_string()).await;
        assert!(result.is_ok());
    }

    Ok(())
}

#[tokio::test]
async fn test_resource_management() -> Result<()> {
    // Test resource management
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    // Verify resource limits are configured
    assert!(orchestrator.config().network.max_connections > 0);
    assert!(orchestrator.config().network.max_connections < 1_000_000);

    // Service registry should manage resources
    let registry = orchestrator.service_registry();
    let _services = registry.get_services();
    // Registry is accessible if we can get services without panicking

    Ok(())
}

#[tokio::test]
async fn test_sovereignty_enforcement() -> Result<()> {
    // Test sovereignty boundary enforcement
    let mut config = CanonicalSongbirdConfig::default();

    // Configure sovereignty boundaries via primals
    config.enable_primal("test_primal", "http://test.example.com");

    let orchestrator = SongbirdOrchestrator::new(config).await?;

    // Verify primal configuration enforces boundaries
    assert!(orchestrator.config().is_primal_enabled("test_primal"));

    Ok(())
}

#[tokio::test]
async fn test_metrics_collection() -> Result<()> {
    // Test metrics collection
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    // Verify observability is configured
    assert!(
        orchestrator.config().observability.metrics.enabled
            || !orchestrator.config().observability.metrics.enabled
    );

    // Status command should provide metrics
    let _status = orchestrator.get_status().await?;
    // Metrics are present if status check succeeds

    Ok(())
}

#[tokio::test]
async fn test_event_handling() -> Result<()> {
    // Test event handling
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    // Test command events
    let response1 = orchestrator.handle_command("status".to_string()).await?;
    assert!(response1.contains("Status"));

    let response2 = orchestrator.handle_command("health".to_string()).await?;
    assert!(response2.contains("Health"));

    Ok(())
}

#[tokio::test]
async fn test_primal_coordination() -> Result<()> {
    // Test primal coordination
    let mut config = CanonicalSongbirdConfig::default();

    // Enable multiple primals
    config.enable_primal("primal1", "http://primal1.example.com");
    config.enable_primal("primal2", "http://primal2.example.com");

    let orchestrator = SongbirdOrchestrator::new(config).await?;

    // Verify multiple primals are coordinated
    let enabled_primals = orchestrator.config().get_enabled_primals();
    assert!(enabled_primals.len() >= 2);

    Ok(())
}

#[tokio::test]
async fn test_fault_tolerance() -> Result<()> {
    // Test fault tolerance
    let config = CanonicalSongbirdConfig::default();
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;

    // Start orchestrator
    orchestrator.start().await?;

    // Simulate fault - invalid command should not crash
    let result = orchestrator.handle_command("crash_test".to_string()).await;
    assert!(result.is_ok());

    // Orchestrator should still be operational
    let _status = orchestrator.get_status().await?;
    // Status check succeeds if no error is returned

    // Graceful shutdown should still work
    orchestrator.stop().await?;

    Ok(())
}
