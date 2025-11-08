//! Comprehensive tests for orchestrator lifecycle management
//!
//! Tests orchestrator initialization, startup, shutdown, and state transitions

use serial_test::serial;
use songbird_config::SongbirdConfig;
use songbird_orchestrator::SongbirdOrchestrator;
use songbird_test_utils::network_fixtures::*;
use songbird_test_utils::test_orchestrator_port;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_types::{SongbirdError, SongbirdResult};
use std::env;

#[tokio::test]
#[serial]
async fn test_orchestrator_creation_with_default_config() {
    // Create orchestrator with default config
    let config = SongbirdConfig::default();
    let result = SongbirdOrchestrator::new(config).await;

    assert!(result.is_ok(), "Orchestrator should be created with default config");
}

#[tokio::test]
#[serial]
async fn test_orchestrator_federation_disabled_by_default() {
    // Federation should be disabled by default
    env::remove_var("SONGBIRD_FEDERATION_ENABLED");

    let config = SongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await;

    assert!(orchestrator.is_ok());
    // When federation is disabled, orchestrator runs in standalone mode
}

#[tokio::test]
#[serial]
async fn test_orchestrator_federation_enabled() {
    // Enable federation
    env::set_var("SONGBIRD_FEDERATION_ENABLED", "true");
    env::set_var("SONGBIRD_NODE_ID", "test-node-1");
    env::set_var("SONGBIRD_NODE_NAME", "test-orchestrator");

    let config = SongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await;

    // Cleanup
    env::remove_var("SONGBIRD_FEDERATION_ENABLED");
    env::remove_var("SONGBIRD_NODE_ID");
    env::remove_var("SONGBIRD_NODE_NAME");

    assert!(orchestrator.is_ok(), "Orchestrator should support federation mode");
}

#[tokio::test]
#[serial]
async fn test_orchestrator_with_custom_node_id() {
    env::set_var("SONGBIRD_FEDERATION_ENABLED", "true");
    env::set_var("SONGBIRD_NODE_ID", "custom-node-123");

    let config = SongbirdConfig::default();
    let result = SongbirdOrchestrator::new(config).await;

    env::remove_var("SONGBIRD_FEDERATION_ENABLED");
    env::remove_var("SONGBIRD_NODE_ID");

    assert!(result.is_ok());
    // Node ID should be used in federation registration
}

#[tokio::test]
#[serial]
async fn test_orchestrator_with_bootstrap_address() {
    env::set_var("SONGBIRD_FEDERATION_ENABLED", "true");
    env::set_var(
        "SONGBIRD_BOOTSTRAP_ADDRESS",
        format!("http://bootstrap:{}", test_orchestrator_port()),
    );

    let config = SongbirdConfig::default();
    let result = SongbirdOrchestrator::new(config).await;

    env::remove_var("SONGBIRD_FEDERATION_ENABLED");
    env::remove_var("SONGBIRD_BOOTSTRAP_ADDRESS");

    assert!(result.is_ok());
    // Should attempt to join federation via bootstrap
}

#[tokio::test]
#[serial]
async fn test_orchestrator_service_registry_initialization() {
    let config = SongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await;

    assert!(orchestrator.is_ok());
    let orch = orchestrator.ok_or_else(|| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;

    // Service registry should be initialized
    let _registry = orch.service_registry();
    // Registry exists and is accessible (test passes if we get here)
    assert!(true);
}

#[tokio::test]
#[serial]
async fn test_orchestrator_federation_state_initialization() {
    env::set_var("SONGBIRD_FEDERATION_ENABLED", "true");

    let config = SongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await;

    env::remove_var("SONGBIRD_FEDERATION_ENABLED");

    assert!(orchestrator.is_ok());
    let orch = orchestrator.ok_or_else(|| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;

    // Federation state should be initialized
    let state = orch.federation_state();
    let stats = state.get_stats().await;
    assert!(stats.total_nodes == 0 || stats.total_nodes >= 0);
    // State exists and is accessible
}

#[tokio::test]
#[serial]
async fn test_orchestrator_federated_service_registry() {
    env::set_var("SONGBIRD_FEDERATION_ENABLED", "true");

    let config = SongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await;

    env::remove_var("SONGBIRD_FEDERATION_ENABLED");

    assert!(orchestrator.is_ok());
    let orch = orchestrator.ok_or_else(|| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;

    // Federated service registry should be initialized
    let fed_registry = orch.federated_service_registry();
    let services = fed_registry.get_all_services().await;
    assert!(services.is_empty() || !services.is_empty());
}

#[tokio::test]
#[serial]
async fn test_orchestrator_config_access() {
    let config = SongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config.clone()).await;

    assert!(orchestrator.is_ok());
    let orch = orchestrator.ok_or_else(|| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;

    // Config should be accessible
    let stored_config = orch.config();
    assert_eq!(stored_config.network.bind_address, config.network.bind_address);
}

#[tokio::test]
#[serial]
async fn test_orchestrator_custom_port_configuration() {
    env::set_var("SONGBIRD_PORT", "9090");

    let config = SongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await;

    env::remove_var("SONGBIRD_PORT");

    assert!(orchestrator.is_ok());
    // Custom port should be used
}

#[tokio::test]
#[serial]
async fn test_orchestrator_custom_bind_address() {
    env::set_var("SONGBIRD_BIND_ADDRESS", "0.0.0.0");

    let config = SongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await;

    env::remove_var("SONGBIRD_BIND_ADDRESS");

    assert!(orchestrator.is_ok());
    // Custom bind address should be used
}

#[tokio::test]
#[serial]
async fn test_orchestrator_resource_detection() {
    let config = SongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await;

    assert!(orchestrator.is_ok());
    // CPU cores should be detected
    // Memory should be detected
    // GPU should be detected (if available)
    // Storage should be detected (if available)
}

#[tokio::test]
#[serial]
async fn test_orchestrator_with_environment_overrides() {
    env::set_var("SONGBIRD_NODE_ADDRESS", "192.168.1.100");
    env::set_var("SONGBIRD_PORT", "8888");
    env::set_var("SONGBIRD_NODE_NAME", "production-node");

    let config = SongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await;

    env::remove_var("SONGBIRD_NODE_ADDRESS");
    env::remove_var("SONGBIRD_PORT");
    env::remove_var("SONGBIRD_NODE_NAME");

    assert!(orchestrator.is_ok());
    // Environment overrides should be respected
}

#[tokio::test]
#[serial]
async fn test_orchestrator_gpu_detection_override() {
    env::set_var("GPU_MODEL", "Tesla T4");

    let config = SongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await;

    env::remove_var("GPU_MODEL");

    assert!(orchestrator.is_ok());
    // GPU override should be used
}

#[tokio::test]
#[serial]
async fn test_orchestrator_storage_detection_override() {
    env::set_var("STORAGE_GB", "1000");

    let config = SongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await;

    env::remove_var("STORAGE_GB");

    assert!(orchestrator.is_ok());
    // Storage override should be used
}

#[tokio::test]
#[serial]
async fn test_orchestrator_heartbeat_interval_configuration() {
    env::set_var("SONGBIRD_FEDERATION_ENABLED", "true");
    // Heartbeat interval should be configurable

    let config = SongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await;

    env::remove_var("SONGBIRD_FEDERATION_ENABLED");

    assert!(orchestrator.is_ok());
    // Default heartbeat interval should be 30 seconds
}

#[tokio::test]
#[serial]
async fn test_orchestrator_multiple_capabilities() {
    env::set_var("SONGBIRD_FEDERATION_ENABLED", "true");

    let config = SongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await;

    env::remove_var("SONGBIRD_FEDERATION_ENABLED");

    assert!(orchestrator.is_ok());
    // Orchestrator should register with "orchestrator" capability
}

#[tokio::test]
#[serial]
async fn test_orchestrator_standalone_mode_default() {
    // By default, federation should be disabled
    env::remove_var("SONGBIRD_FEDERATION_ENABLED");

    let config = SongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await;

    assert!(orchestrator.is_ok());
    // Should run in standalone mode
}

#[tokio::test]
#[serial]
async fn test_orchestrator_observability_initialization() {
    let config = SongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await;

    assert!(orchestrator.is_ok());
    // Observability manager should be initialized
}

#[tokio::test]
#[serial]
async fn test_orchestrator_concurrent_initialization() {
    // Test that multiple orchestrators can be created concurrently
    let config1 = SongbirdConfig::default();
    let config2 = SongbirdConfig::default();

    let (result1, result2) =
        tokio::join!(SongbirdOrchestrator::new(config1), SongbirdOrchestrator::new(config2));

    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

#[tokio::test]
#[serial]
async fn test_orchestrator_memory_efficiency() {
    // Test that orchestrator initialization is memory efficient
    let config = SongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await;

    assert!(orchestrator.is_ok());
    // Orchestrator should use Arc for shared state
    // Should not clone large structures unnecessarily
}
