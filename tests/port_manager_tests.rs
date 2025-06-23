//! Port manager tests ported from the original nestgate-orchestrator
//!
//! These tests verify the basic functionality of service port management
//! and allocation within the Songbird Orchestrator.

use serde::{Deserialize, Serialize};
use songbird_orchestrator::{
    errors::SongbirdError,
    network::{NetworkConfig, NetworkManager},
    traits::service::{
        ServiceInfo, ServiceMetrics, ServiceRequest, ServiceResponse, UniversalService,
    },
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

mod common;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct MockConfig;

#[derive(Debug, Clone)]
struct MockNetworkService {
    running: Arc<AtomicBool>,
    config: MockConfig,
}

impl MockNetworkService {
    fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            config: MockConfig,
        }
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

#[async_trait::async_trait]
impl UniversalService for MockNetworkService {
    type Config = MockConfig;
    type Health = bool;
    type Error = SongbirdError;

    async fn initialize(&mut self, config: Self::Config) -> std::result::Result<(), Self::Error> {
        self.config = config;
        Ok(())
    }

    async fn start(&mut self) -> std::result::Result<(), Self::Error> {
        self.running.store(true, Ordering::Relaxed);
        Ok(())
    }

    async fn stop(&mut self) -> std::result::Result<(), Self::Error> {
        self.running.store(false, Ordering::Relaxed);
        Ok(())
    }

    async fn health_check(&self) -> std::result::Result<Self::Health, Self::Error> {
        Ok(self.is_running())
    }

    async fn handle_request(
        &self,
        request: ServiceRequest,
    ) -> std::result::Result<ServiceResponse, Self::Error> {
        if !self.is_running() {
            return Ok(ServiceResponse::error(
                request.id.clone(),
                503,
                "Service not running".to_string(),
            ));
        }

        Ok(ServiceResponse::success(
            request.id.clone(),
            serde_json::json!({"message": "Network response", "path": request.path.clone()}),
        ))
    }

    async fn update_config(&mut self, config: Self::Config) -> std::result::Result<(), Self::Error> {
        self.config = config;
        Ok(())
    }

    async fn get_metrics(&self) -> std::result::Result<ServiceMetrics, Self::Error> {
        Ok(ServiceMetrics::default())
    }

    fn service_info(&self) -> ServiceInfo {
        ServiceInfo {
            id: "network-service".to_string(),
            name: "Network Service".to_string(),
            version: "1.0.0".to_string(),
            service_type: "network".to_string(),
            description: "Network management service".to_string(),
            endpoints: vec![],
            capabilities: vec!["network".to_string()],
            tags: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
        }
    }

    async fn can_handle_load(&self) -> std::result::Result<bool, Self::Error> {
        Ok(self.is_running())
    }

    async fn get_load_factor(&self) -> std::result::Result<f64, Self::Error> {
        Ok(if self.is_running() { 0.5 } else { 1.0 })
    }
}

#[tokio::test]
async fn test_network_manager_creation() {
    let config = NetworkConfig::default();
    let manager = NetworkManager::new(config);

    let config = manager.get_config();
    assert_eq!(config.bind_address, songbird_orchestrator::config::environment::get_container_bind_address());
    assert_eq!(config.port, 8080);
    assert_eq!(config.max_connections, 1000);
}

#[tokio::test]
async fn test_network_manager_connections() {
    let config = NetworkConfig::default();
    let mut manager = NetworkManager::new(config);

    // Test connection management
    assert_eq!(manager.get_connection_count(), 0);

    manager.add_connection("conn1".to_string()).await.unwrap();
    assert_eq!(manager.get_connection_count(), 1);

    manager.add_connection("conn2".to_string()).await.unwrap();
    assert_eq!(manager.get_connection_count(), 2);

    manager.remove_connection("conn1").await.unwrap();
    assert_eq!(manager.get_connection_count(), 1);

    manager.remove_connection("conn2").await.unwrap();
    assert_eq!(manager.get_connection_count(), 0);
}

#[tokio::test]
async fn test_network_manager_lifecycle() {
    let config = NetworkConfig::default();
    let mut manager = NetworkManager::new(config);

    // Test lifecycle
    manager.start().await.unwrap();
    manager.stop().await.unwrap();
}

#[tokio::test]
async fn test_mock_network_service() {
    let mut service = MockNetworkService::new();
    let config = MockConfig::default();

    // Test initialization
    service.initialize(config).await.unwrap();

    // Test starting
    service.start().await.unwrap();
    assert!(service.is_running());

    // Test health check
    let health = service.health_check().await.unwrap();
    assert!(health);

    // Test request handling
    let request = ServiceRequest::new("GET", "/network/status");
    let response = service.handle_request(request).await.unwrap();
    assert!(matches!(
        response.status,
        songbird_orchestrator::traits::service::ResponseStatus::Success
    ));

    // Test stopping
    service.stop().await.unwrap();
    assert!(!service.is_running());
}

#[tokio::test]
async fn test_network_service_integration() {
    let service = MockNetworkService::new();
    let info = service.service_info();

    assert_eq!(info.id, "network-service");
    assert_eq!(info.service_type, "network");
    assert!(info.capabilities.contains(&"network".to_string()));
}
