use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
use std::collections::HashMap;
#[allow(dead_code, unused_imports, unused_variables)]
// Unit tests for the Songbird Orchestrator
use songbird_gaming_bridge::{errors::Result, Orchestrator, OrchestratorConfig};

mod common;
use common::{MockConfig, MockService};

#[tokio::test]
async fn test_orchestrator_initialization() -> Result<()> {
    // Create a basic configuration
    let config = OrchestratorConfig::default();

    // Create an orchestrator instance
    let orchestrator = Orchestrator::new(config).await?;

    // Verify that the orchestrator initialized correctly
    let services = orchestrator.list_services().await;
    assert!(services.is_empty()); // Initially no services

    // Test that we can access the configuration
    let _config_ref = orchestrator.config();

    Ok(())
}

#[tokio::test]
async fn test_service_metrics() -> Result<()> {
    let config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(config).await?;

    // Create and register a service - use correct MockService::new() without args
    let service = MockService::new();
    let service_config = MockConfig {
        service_id: "metrics-test".to_string(),
        port: 8080,
        timeout: 5000,
        max_connections: 100,
    };

    let service_id = orchestrator
        .register_service(service, service_config)
        .await?;

    // Start orchestrator
    orchestrator.start().await?;

    // Get orchestrator metrics
    let metrics = orchestrator.get_config().await;
    assert_eq!(metrics.total_services, 1);
    assert_eq!(metrics.healthy_services, 1);

    // Stop orchestrator
    orchestrator.stop().await?;

    Ok(())
}

#[tokio::test]
async fn test_service_registration() -> Result<()> {
    let config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(config).await?;

    // Create and register a service
    let service = MockService::new();
    let service_config = MockConfig {
        service_id: "test-service".to_string(),
        port: 8080,
        timeout: 5000,
        max_connections: 100,
    };

    let service_id = orchestrator
        .register_service(service, service_config)
        .await?;

    // Verify service was registered
    let services = orchestrator.list_services().await;
    assert_eq!(services.len(), 1);
    assert_eq!(services[0].id, service_id);

    Ok(())
}
