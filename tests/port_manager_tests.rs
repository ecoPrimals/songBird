use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
#[allow(dead_code, unused_imports, unused_variables)]
// Port manager tests ported from the original nestgate-orchestrator
//
// These tests verify the basic functionality of service port management
// and allocation within the Songbird Orchestrator.
use serde::{Deserialize, Serialize};
use songbird_gaming_bridge::{
    errors::SongbirdError,
    network::{ConnectionInfo, NetworkConfig, NetworkManager},
    traits::service_id::{
        HealthStatus, ServiceInfo, ServiceMetrics, ServiceRequest, ServiceResponse,
        UniversalService,
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
    type Error = SongbirdError;

    async fn start(&mut self) -> std::result::Result<()> {
        self.running.store(true, Ordering::Relaxed);
        Ok(())
    }

    async fn stop(&mut self) -> std::result::Result<()> {
        self.running.store(false, Ordering::Relaxed);
        Ok(())
    }

    async fn health_check(
        &self,
    ) -> std::result::Result<songbird_gaming_bridge::traits::service_id::HealthStatus>
    {
        Ok(if self.is_running() {
            songbird_gaming_bridge::traits::service_id::HealthStatus::Healthy
        } else {
            songbird_gaming_bridge::traits::service_id::HealthStatus::Unhealthy
        })
    }

    async fn handle_request(
        &self,
        request: ServiceRequest,
    ) -> std::result::Result<ServiceResponse> {
        if !self.is_running() {
            return Ok(ServiceResponse::error(
                request.id.clone(),
                "Service not running".to_string(),
            ));
        }

        Ok(ServiceResponse::success(request.id.clone()).with_body(
            serde_json::json!({"message": "Network response", "path": request.path.clone()}),
        ))
    }

    async fn update_config(
        &mut self,
        config: serde_json::Value,
    ) -> std::result::Result<()> {
        // For testing purposes, we'll just ignore the config update
        Ok(())
    }

    async fn get_metrics(&self) -> std::result::Result<ServiceMetrics> {
        use std::time::Duration;
        Ok(ServiceMetrics {
            request_count: 0,
            error_count: 0,
            average_response_time: 10.0,
            cpu_usage: Some()Some(0.1),
            memory_usage: Some()Some(1024 * 1024), // 1MB
            active_connections: 0,
            uptime: Duration::from_secs(0),
            custom_metrics: std::collections::HashMap::new(),
        })
    }

    fn service_info(&self) -> ServiceInfo {
        use chrono::Utc;
        use std::collections::HashMap;
        ServiceInfo {
            service_id: "network-service".to_string(),
            name: "Network Service".to_string(),
            version: "1.0.0".to_string(),
            service_type: "network".to_string(),
            description: Some("Network management service".to_string()),
            endpoints: vec![],
            health_check_endpoint: Some("/health".to_string()),
            
            tags: std::collections::HashMap::new(),
            dependencies: vec![],
            status: songbird_gaming_bridge::traits::service_id::ServiceStatus::Running,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            instance_id: "network-service-instance".to_string(),
            host: "localhost".to_string(),
            port: 8080,
        }
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

#[tokio::test]
async fn test_network_manager_creation() {
    let config = NetworkConfig::default();
    let manager = NetworkManager::new(config);

    let config = manager.get_config();
    // Verify basic config structure without environment-dependent values
    assert!(
        config.bind_address.to_string() == "0.0.0.0"
            || config.bind_address.to_string() == "127.0.0.1"
    );
    assert_eq!(config.bind_port, 8080);
    assert_eq!(config.max_connections, 1000);
}

#[tokio::test]
async fn test_network_manager_connections() {
    let config = NetworkConfig::default();
    let mut manager = NetworkManager::new(config);

    // Test connection management
    assert_eq!(manager.get_active_connection_count(), 0);

    let conn_info = ConnectionInfo {
        remote_addr: "127.0.0.1:8080".to_string(),
        connected_at: chrono::Utc::now(),
        last_activity: chrono::Utc::now(),
        bytes_sent: 0,
        bytes_received: 0,
        connection_type: "tcp".to_string(),
    };

    manager.add_connection("conn1".to_string(), conn_info.clone());
    assert_eq!(manager.get_active_connection_count(), 1);

    manager.add_connection("conn2".to_string(), conn_info);
    assert_eq!(manager.get_active_connection_count(), 2);

    manager.remove_connection("conn1");
    assert_eq!(manager.get_active_connection_count(), 1);

    manager.remove_connection("conn2");
    assert_eq!(manager.get_active_connection_count(), 0);
}

#[tokio::test]
async fn test_network_manager_lifecycle() {
    let config = NetworkConfig::default();
    let manager = NetworkManager::new(config);

    // NetworkManager doesn't have start/stop methods, it's ready to use
    assert_eq!(manager.get_active_connection_count(), 0);
}

#[tokio::test]
async fn test_mock_network_service() {
    let mut service = MockNetworkService::new();
    let config = MockConfig::default();

    // Service is ready to use

    // Test starting
    service.start().await.expect("Test assertion failed");
    assert!(service.is_running());

    // Test health check
    let health = service.health_check().await.expect("Test assertion failed");
    assert!(matches!(
        health,
        songbird_gaming_bridge::traits::service_id::HealthStatus::Healthy
    ));

    // Test request handling
    let request = ServiceRequest::new("GET".to_string(), "/network/status".to_string());
    let response = service
        .handle_request(request)
        .await
        .expect("Test assertion failed");
    assert!(matches!(
        response.status,
        songbird_gaming_bridge::traits::service_id::ResponseStatus::Success
    ));

    // Test stopping
    service.stop().await.expect("Test assertion failed");
    assert!(!service.is_running());
}

#[tokio::test]
async fn test_network_service_integration() {
    let service = MockNetworkService::new();
    let info = service.service_info();

    assert_eq!(info.service_id, "network-service");
    assert_eq!(info.service_type, "network");
    assert!(info.tags.contains(&"network".to_string()));
}
