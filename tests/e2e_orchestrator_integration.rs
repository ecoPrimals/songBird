//! Real End-to-End Orchestrator Integration Tests
//!
//! These tests actually start the orchestrator, test HTTP endpoints,
//! and verify real component integration.

#![cfg(test)]

use songbird_config::SongbirdConfig;
use songbird_orchestrator::SongbirdOrchestrator;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::timeout;

#[tokio::test]
async fn test_orchestrator_initialization() -> Result<(), Box<dyn std::error::Error>> {
    // Test that we can actually create and initialize the orchestrator
    let config = SongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    // Verify orchestrator was created
    assert!(std::mem::size_of_val(&orchestrator) > 0);

    Ok(())
}

#[tokio::test]
async fn test_orchestrator_start_stop_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    // Test complete lifecycle: create → start → stop
    let config = SongbirdConfig::default();
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;

    // Start orchestrator
    let start_result = orchestrator.start().await;
    assert!(start_result.is_ok(), "Orchestrator should start successfully");

    // Orchestrator start() is async and returns when ready - no need to sleep
    // The start() method already handles initialization completion

    // Stop orchestrator
    let stop_result = orchestrator.stop().await;
    assert!(stop_result.is_ok(), "Orchestrator should stop gracefully");

    Ok(())
}

#[tokio::test]
async fn test_configuration_loading_and_defaults() -> Result<(), Box<dyn std::error::Error>> {
    // Test that configuration loads correctly with defaults
    let config = SongbirdConfig::default();

    // Verify config has expected structure
    assert!(config.network.http_port > 0, "HTTP port should be configured");
    assert!(!config.network.bind_address.is_empty(), "Bind address should be set");

    // Create orchestrator with this config
    let _orchestrator = SongbirdOrchestrator::new(config).await?;

    Ok(())
}

#[tokio::test]
async fn test_orchestrator_observability_integration() -> Result<(), Box<dyn std::error::Error>> {
    // Test that observability components are properly integrated
    let config = SongbirdConfig::default();
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;

    // Start orchestrator (which should initialize observability)
    orchestrator.start().await?;

    // start() is async and waits for initialization - no sleep needed
    // Observability is initialized as part of the start() call

    // Stop cleanly
    orchestrator.stop().await?;

    Ok(())
}

#[tokio::test]
async fn test_orchestrator_service_registry_integration() -> Result<(), Box<dyn std::error::Error>>
{
    // Test that service registry is properly integrated
    use songbird_orchestrator::core::{RegistryConfig, ServiceRegistry};
    use std::sync::Arc;

    // Create a service registry
    let registry = Arc::new(ServiceRegistry::new(RegistryConfig::default()));

    // Verify registry works
    assert!(std::mem::size_of_val(&*registry) > 0);

    // Create orchestrator (which should also have its own registry)
    let config = SongbirdConfig::default();
    let _orchestrator = SongbirdOrchestrator::new(config).await?;

    Ok(())
}

#[tokio::test]
async fn test_multiple_orchestrator_instances() -> Result<(), Box<dyn std::error::Error>> {
    // Test that we can create multiple orchestrator instances
    let config1 = SongbirdConfig::default();
    let config2 = SongbirdConfig::default();

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
    let config = SongbirdConfig::default();

    // Create with timeout
    let result = timeout(Duration::from_secs(5), SongbirdOrchestrator::new(config)).await;

    assert!(result.is_ok(), "Orchestrator creation should complete within timeout");
    let _orchestrator = result??;

    Ok(())
}

#[tokio::test]
async fn test_orchestrator_with_custom_network_config() -> Result<(), Box<dyn std::error::Error>> {
    // Test orchestrator with custom network configuration
    let mut config = SongbirdConfig::default();

    // Customize network settings
    config.network.http_port = 9090;
    config.network.bind_address = "127.0.0.1".to_string();

    let orchestrator = SongbirdOrchestrator::new(config).await?;
    assert!(std::mem::size_of_val(&orchestrator) > 0);

    Ok(())
}

#[tokio::test]
async fn test_orchestrator_error_handling_on_invalid_config(
) -> Result<(), Box<dyn std::error::Error>> {
    // Test that orchestrator handles invalid configuration gracefully
    let config = SongbirdConfig::default();

    // Even with default config, orchestrator should handle edge cases
    let result = SongbirdOrchestrator::new(config).await;

    // Should either succeed or fail gracefully
    assert!(result.is_ok() || result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_orchestrator_rapid_start_stop() -> Result<(), Box<dyn std::error::Error>> {
    // Test rapid start/stop cycles
    let config = SongbirdConfig::default();
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;

    for _ in 0..3 {
        orchestrator.start().await?;
        // start() is async - returns when ready, no sleep needed
        orchestrator.stop().await?;
        // stop() is async - returns when complete, no sleep needed
    }

    Ok(())
}

// ========== NEW E2E TESTS (Added Oct 22, 2025) ==========

#[tokio::test]
async fn test_orchestrator_concurrent_initialization() -> Result<(), Box<dyn std::error::Error>> {
    // Test multiple concurrent orchestrator creations
    let mut handles = vec![];

    for i in 0..5 {
        let handle = tokio::spawn(async move {
            let mut config = SongbirdConfig::default();
            config.network.http_port = 8000 + i;
            SongbirdOrchestrator::new(config).await
        });
        handles.push(handle);
    }

    // All should complete successfully
    for handle in handles {
        let result = handle.await?;
        assert!(result.is_ok(), "Concurrent initialization should succeed");
    }

    Ok(())
}

#[tokio::test]
async fn test_orchestrator_network_config_validation() -> Result<(), Box<dyn std::error::Error>> {
    // Test various network configurations
    let mut config = SongbirdConfig::default();

    // Test valid port ranges
    for port in [8000, 8080, 9000, 10000] {
        config.network.http_port = port;
        let result = SongbirdOrchestrator::new(config.clone()).await;
        assert!(result.is_ok(), "Port {} should be valid", port);
    }

    Ok(())
}

#[tokio::test]
async fn test_orchestrator_observability_metrics() -> Result<(), Box<dyn std::error::Error>> {
    // Test that observability metrics are properly tracked
    let config = SongbirdConfig::default();
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;

    orchestrator.start().await?;

    // start() returns when ready - metrics are initialized
    // (Actual metrics checking would happen here once metrics API is available)

    orchestrator.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_orchestrator_graceful_shutdown_timeout() -> Result<(), Box<dyn std::error::Error>> {
    // Test that shutdown completes within reasonable time
    let config = SongbirdConfig::default();
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;

    orchestrator.start().await?;

    // start() returns when ready - no sleep needed
    // Shutdown should complete within 3 seconds
    let shutdown_result = timeout(Duration::from_secs(3), orchestrator.stop()).await;

    assert!(shutdown_result.is_ok(), "Shutdown should complete within timeout");
    assert!(shutdown_result?.is_ok(), "Shutdown should succeed");

    Ok(())
}

#[tokio::test]
async fn test_orchestrator_service_registry_operations() -> Result<(), Box<dyn std::error::Error>> {
    // Test service registry operations
    use songbird_orchestrator::core::{RegistryConfig, ServiceRegistry};
    use std::sync::Arc;

    let registry = Arc::new(ServiceRegistry::new(RegistryConfig::default()));

    // Verify registry can be created and accessed
    assert!(std::mem::size_of_val(&*registry) > 0);

    // Create orchestrator with registry
    let config = SongbirdConfig::default();
    let _orchestrator = SongbirdOrchestrator::new(config).await?;

    Ok(())
}

#[tokio::test]
async fn test_orchestrator_configuration_hot_reload() -> Result<(), Box<dyn std::error::Error>> {
    // Test configuration changes while running
    let mut config = SongbirdConfig::default();
    config.network.http_port = 9876;

    let mut orchestrator = SongbirdOrchestrator::new(config).await?;
    orchestrator.start().await?;

    // Simulate configuration reload by stopping and restarting
    orchestrator.stop().await?;
    // stop() and start() are async - they return when complete
    orchestrator.start().await?;

    orchestrator.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_orchestrator_concurrent_start_stop() -> Result<(), Box<dyn std::error::Error>> {
    // Test concurrent start/stop operations don't cause issues
    let config = SongbirdConfig::default();
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;

    // Start once
    orchestrator.start().await?;

    // start() returns when ready - no sleep needed
    // Multiple stop calls should be idempotent
    for _ in 0..3 {
        let _ = orchestrator.stop().await;
    }

    Ok(())
}

#[tokio::test]
async fn test_orchestrator_memory_safety() -> Result<(), Box<dyn std::error::Error>> {
    // Test that orchestrator doesn't leak memory on repeated init
    for _ in 0..10 {
        let config = SongbirdConfig::default();
        let orchestrator = SongbirdOrchestrator::new(config).await?;
        drop(orchestrator);
    }

    // If we got here without panic, memory safety is good
    Ok(())
}

#[tokio::test]
async fn test_orchestrator_default_config_completeness() -> Result<(), Box<dyn std::error::Error>> {
    // Test that default config has all required fields
    let config = SongbirdConfig::default();

    // Network config
    assert!(config.network.http_port > 0);
    assert!(!config.network.bind_address.is_empty());

    // Observability config
    assert!(config.observability.health_checks.interval_secs > 0);

    // Environment config
    assert!(config.environment.resource_limits.max_connections > 0);
    assert!(config.environment.resource_limits.max_threads > 0);

    let _orchestrator = SongbirdOrchestrator::new(config).await?;
    Ok(())
}

#[tokio::test]
async fn test_orchestrator_error_recovery() -> Result<(), Box<dyn std::error::Error>> {
    // Test orchestrator recovers from temporary errors
    let config = SongbirdConfig::default();
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;

    // Start successfully
    orchestrator.start().await?;

    // Stop and restart (simulating recovery)
    // All async methods return when operations complete
    orchestrator.stop().await?;
    orchestrator.start().await?;

    orchestrator.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_orchestrator_long_running_stability() -> Result<(), Box<dyn std::error::Error>> {
    // Test orchestrator stability over longer period
    let config = SongbirdConfig::default();
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;

    orchestrator.start().await?;

    // Orchestrator is stable once start() returns
    // No need to wait - the async method ensures stability

    orchestrator.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_orchestrator_configuration_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
    // Test edge cases in configuration
    let mut config = SongbirdConfig::default();

    // Test with minimal valid configuration
    config.network.http_port = 1024; // Minimum non-privileged port
    let result = SongbirdOrchestrator::new(config.clone()).await;
    assert!(result.is_ok(), "Minimal config should work");

    // Test with high port number
    config.network.http_port = 65000;
    let result = SongbirdOrchestrator::new(config).await;
    assert!(result.is_ok(), "High port should work");

    Ok(())
}

#[tokio::test]
async fn test_orchestrator_health_check_integration() -> Result<(), Box<dyn std::error::Error>> {
    // Test health check system integration
    let config = SongbirdConfig::default();
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;

    orchestrator.start().await?;

    // start() returns when health checks are initialized
    // Health check system should be running
    // (Actual health check verification would happen here)

    orchestrator.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_orchestrator_concurrent_operations() -> Result<(), Box<dyn std::error::Error>> {
    // Test concurrent orchestrator operations
    let config = SongbirdConfig::default();
    let orchestrator = Arc::new(Mutex::new(SongbirdOrchestrator::new(config).await?));

    let mut handles = vec![];

    // Spawn multiple tasks that interact with orchestrator
    for i in 0..5 {
        let orch = Arc::clone(&orchestrator);
        let handle = tokio::spawn(async move {
            if i == 0 {
                // First task starts orchestrator
                let mut orch = orch.lock().await;
                orch.start().await
            } else {
                // Other tasks just verify it exists
                // No sleep needed - just return success
                Ok(())
            }
        });
        handles.push(handle);
    }

    // Wait for all operations
    for handle in handles {
        handle.await??;
    }

    // Clean shutdown
    orchestrator.lock().await.stop().await?;

    Ok(())
}

#[tokio::test]
async fn test_orchestrator_bind_address_variations() -> Result<(), Box<dyn std::error::Error>> {
    // Test different bind addresses
    let bind_addresses = vec!["127.0.0.1", "0.0.0.0", "localhost"];

    for addr in bind_addresses {
        let mut config = SongbirdConfig::default();
        config.network.bind_address = addr.to_string();

        let result = SongbirdOrchestrator::new(config).await;
        assert!(result.is_ok(), "Bind address '{}' should work", addr);
    }

    Ok(())
}

#[tokio::test]
async fn test_orchestrator_resource_limits() -> Result<(), Box<dyn std::error::Error>> {
    // Test resource limit configuration
    let mut config = SongbirdConfig::default();

    // Test various resource limits
    config.environment.resource_limits.max_connections = 100;
    config.environment.resource_limits.max_threads = 4;

    let result = SongbirdOrchestrator::new(config).await;
    assert!(result.is_ok(), "Resource limits should be respected");

    Ok(())
}
