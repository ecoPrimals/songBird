//! Integration tests for service lifecycle management
//!
//! These tests verify end-to-end service registration, lifecycle management,
//! and orchestrator integration functionality.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use songbird_orchestrator::{
    config::OrchestratorConfig,
    errors::{Result, SongbirdError},
    traits::service::{
        ServiceInfo, ServiceMetrics, ServiceRequest, ServiceResponse, UniversalService,
    },
    Orchestrator,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Test service implementation for integration testing
#[derive(Debug)]
struct TestIntegrationService {
    id: String,
    config: Option<TestServiceConfig>,
    started: Arc<AtomicBool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestServiceConfig {
    name: String,
    port: u16,
    enabled: bool,
}

#[derive(Debug, Serialize)]
struct TestServiceHealth {
    status: String,
    uptime_seconds: u64,
    requests_handled: u64,
}

impl TestIntegrationService {
    fn new(id: String) -> Self {
        Self {
            id,
            config: None,
            started: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[async_trait]
impl UniversalService for TestIntegrationService {
    type Config = TestServiceConfig;
    type Health = TestServiceHealth;
    type Error = SongbirdError;

    async fn initialize(&mut self, config: Self::Config) -> std::result::Result<(), Self::Error> {
        self.config = Some(config);
        Ok(())
    }

    async fn start(&mut self) -> std::result::Result<(), Self::Error> {
        self.started.store(true, Ordering::Relaxed);
        Ok(())
    }

    async fn stop(&mut self) -> std::result::Result<(), Self::Error> {
        self.started.store(false, Ordering::Relaxed);
        Ok(())
    }

    async fn health_check(&self) -> std::result::Result<Self::Health, Self::Error> {
        Ok(TestServiceHealth {
            status: if self.started.load(Ordering::Relaxed) {
                "running".to_string()
            } else {
                "stopped".to_string()
            },
            uptime_seconds: 42,
            requests_handled: 100,
        })
    }

    async fn handle_request(
        &self,
        request: ServiceRequest,
    ) -> std::result::Result<ServiceResponse, Self::Error> {
        if !self.started.load(Ordering::Relaxed) {
            return Ok(ServiceResponse::error(
                request.id,
                503,
                "Service not running".to_string(),
            ));
        }

        match request.path.as_str() {
            "/ping" => Ok(ServiceResponse::success(
                request.id,
                serde_json::json!({"status": "pong", "service": self.id}),
            )),
            "/info" => Ok(ServiceResponse::success(
                request.id,
                serde_json::json!({
                    "id": self.id,
                    "config": self.config,
                    "started": self.started.load(Ordering::Relaxed)
                }),
            )),
            _ => Ok(ServiceResponse::error(
                request.id,
                404,
                format!("Unknown endpoint: {}", request.path),
            )),
        }
    }

    async fn get_metrics(&self) -> std::result::Result<ServiceMetrics, Self::Error> {
        Ok(ServiceMetrics::default())
    }

    fn service_info(&self) -> ServiceInfo {
        ServiceInfo {
            id: self.id.clone(),
            name: format!("Test Integration Service {}", self.id),
            version: "1.0.0".to_string(),
            service_type: "test".to_string(),
            description: "Test service for integration testing".to_string(),
            endpoints: vec![],
            capabilities: vec!["integration-test".to_string()],
            tags: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    async fn can_handle_load(&self) -> std::result::Result<bool, Self::Error> {
        Ok(self.started.load(Ordering::Relaxed))
    }

    async fn get_load_factor(&self) -> std::result::Result<f64, Self::Error> {
        Ok(0.5)
    }

    async fn update_config(
        &mut self,
        config: Self::Config,
    ) -> std::result::Result<(), Self::Error> {
        self.config = Some(config);
        Ok(())
    }
}

#[tokio::test]
async fn test_service_registration_and_lifecycle() -> Result<()> {
    let config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(config).await?;

    // Create and register a test service
    let service = TestIntegrationService::new("test-service-1".to_string());
    let service_config = TestServiceConfig {
        name: "Test Service 1".to_string(),
        port: 8080,
        enabled: true,
    };

    orchestrator
        .register_service(service, service_config)
        .await?;

    // Start the orchestrator
    orchestrator.start().await?;

    // Verify orchestrator metrics
    let metrics = orchestrator.get_metrics().await;
    assert!(metrics.total_services >= 1);

    // Stop the orchestrator
    orchestrator.stop().await?;

    Ok(())
}

#[tokio::test]
async fn test_multiple_service_registration() -> Result<()> {
    let config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(config).await?;

    // Register multiple services
    for i in 1..=3 {
        let service = TestIntegrationService::new(format!("multi-service-{}", i));
        let service_config = TestServiceConfig {
            name: format!("Multi Service {}", i),
            port: 8080 + i as u16,
            enabled: true,
        };
        orchestrator
            .register_service(service, service_config)
            .await?;
    }

    // Start orchestrator
    orchestrator.start().await?;

    // Verify metrics
    let metrics = orchestrator.get_metrics().await;
    assert!(metrics.total_services >= 3);

    // Verify services are listed
    let services = orchestrator.list_services().await;
    assert!(services.len() >= 3);

    // Stop orchestrator
    orchestrator.stop().await?;

    Ok(())
}

#[tokio::test]
async fn test_configuration_management() -> Result<()> {
    let mut config = OrchestratorConfig::default();
    config.orchestrator.port = 9999;
    config.orchestrator.bind_address = "0.0.0.0".to_string();
    config.network.port_range = (9000, 9100);

    let orchestrator = Orchestrator::new(config).await?;

    // Verify configuration
    let config_ref = orchestrator.config();
    assert_eq!(config_ref.orchestrator.port, 9999);
    assert_eq!(config_ref.orchestrator.bind_address, "0.0.0.0");
    assert_eq!(config_ref.network.port_range, (9000, 9100));

    Ok(())
}

#[tokio::test]
async fn test_orchestrator_error_handling() -> Result<()> {
    let config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(config).await?;

    // Test graceful startup/shutdown
    orchestrator.start().await?;
    orchestrator.stop().await?;

    // Test double start/stop (should be idempotent)
    orchestrator.start().await?;
    orchestrator.start().await?; // Should not error
    orchestrator.stop().await?;
    orchestrator.stop().await?; // Should not error

    Ok(())
}

#[tokio::test]
async fn test_service_integration_flow() -> Result<()> {
    let config = OrchestratorConfig::default();
    let _orchestrator = Orchestrator::new(config).await?;

    // Create service with configuration
    let mut service = TestIntegrationService::new("integration-flow-test".to_string());
    let service_config = TestServiceConfig {
        name: "Integration Test Service".to_string(),
        port: 8080,
        enabled: true,
    };

    // Initialize service
    service.initialize(service_config).await?;

    // Test health check before start
    let health = service.health_check().await?;
    assert_eq!(health.status, "stopped");

    // Start service
    service.start().await?;

    // Test health check after start
    let health = service.health_check().await?;
    assert_eq!(health.status, "running");

    // Test service request handling
    let request = ServiceRequest::new("GET", "/ping");
    let _response = service.handle_request(request).await?;

    // Stop service
    service.stop().await?;

    // Test health check after stop
    let health = service.health_check().await?;
    assert_eq!(health.status, "stopped");

    Ok(())
}

#[tokio::test]
async fn test_concurrent_orchestrator_operations() -> Result<()> {
    let config = OrchestratorConfig::default();
    let orchestrator = Arc::new(Orchestrator::new(config).await?);

    let mut handles = vec![];

    // Test concurrent service registrations
    for i in 0..5 {
        let orch = orchestrator.clone();
        let handle = tokio::spawn(async move {
            let service = TestIntegrationService::new(format!("concurrent-{}", i));
            let service_config = TestServiceConfig {
                name: format!("Concurrent Service {}", i),
                port: 8080 + i as u16,
                enabled: true,
            };
            orch.register_service(service, service_config).await
        });
        handles.push(handle);
    }

    // Wait for all concurrent operations
    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    // Verify all services were registered
    let services = orchestrator.list_services().await;
    assert!(services.len() >= 5);

    Ok(())
}
