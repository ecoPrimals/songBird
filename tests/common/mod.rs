use std::collections::HashMap;
#[allow(dead_code, unused_imports, unused_variables)]
// Common test utilities for Songbird Orchestrator
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use songbird_gaming_bridge::{
    errors::{Result, SongbirdError},
    orchestrator::Orchestrator,
    traits::{
        config::OrchestratorConfig,
        service_id::{ServiceInfo, ServiceMetrics, ServiceRequest, ServiceResponse, UniversalService},
    },
};

use tempfile::TempDir;
use tokio::time::Duration;

/// Test fixture for orchestrator testing
pub struct TestOrchestrator {
    pub orchestrator: Orchestrator,
    pub temp_dir: TempDir,
    pub config: OrchestratorConfig,
}

impl TestOrchestrator {
    /// Create a new test orchestrator instance
    pub async fn new() -> Result<Self> {
        let temp_dir = TempDir::new().map_err(|e| SongbirdError::Configuration {
            field: "temp_directory".to_string(),
            message: format!("Failed to create temp dir: {}", e),
        })?;

        let config = OrchestratorConfig::default();
        let orchestrator = Orchestrator::new(config.clone())?;

        Ok(Self {
            orchestrator,
            temp_dir,
            config,
        })
    }

    /// Create a test orchestrator with custom configuration
    pub async fn with_config(config: OrchestratorConfig) -> Result<Self> {
        let temp_dir = TempDir::new().map_err(|e| SongbirdError::Configuration {
            field: "temp_directory".to_string(),
            message: format!("Failed to create temp dir: {}", e),
        })?;

        let orchestrator = Orchestrator::new(config.clone())?;

        Ok(Self {
            orchestrator,
            temp_dir,
            config,
        })
    }

    /// Get the orchestrator instance
    pub fn orchestrator(&self) -> &Orchestrator {
        &self.orchestrator
    }

    /// Get the configuration
    pub fn config(&self) -> &OrchestratorConfig {
        &self.config
    }

    /// Get the temporary directory path
    pub fn temp_path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }

    /// Create a test service info
    pub fn create_test_service(&self, id: &str, service_type: &str) -> ServiceInfo {
        use chrono::Utc;
        ServiceInfo {
            service_id: id.to_string(),
            name: format!("Test {}", id),
            service_type: service_type.to_string(),
            version: "1.0.0".to_string(),
            description: Some(format!("Test service {}", id)),
            health_check_endpoint: Some("/health".to_string()),
            endpoints: vec![],
            tags: std::collections::HashMap::new(),
            
            dependencies: vec![],
            status: songbird_gaming_bridge::traits::service_id::ServiceStatus::Running,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            host: "localhost".to_string(),
            instance_id: format!("{}-instance", id),
            port: 8080,
        }
    }

    /// Create a test request
    pub fn create_test_request(&self, method: &str, path: &str) -> ServiceRequest {
        ServiceRequest::new(method.to_string(), path.to_string())
    }

    /// Wait for a condition to be true with timeout
    pub async fn wait_for_condition<F, Fut>(&self, condition: F, timeout: Duration) -> Result<()>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            if condition().await {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        Err(SongbirdError::Configuration {
            field: "timeout".to_string(),
            message: "Timeout waiting for condition".to_string(),
        })
    }

    /// Cleanup resources (automatically called on drop)
    pub async fn cleanup(self) -> Result<()> {
        // Perform any necessary cleanup
        drop(self.temp_dir);
        Ok(())
    }
}

/// Create a test service info with default values
pub fn create_test_service_info(id: &str, service_type: &str) -> ServiceInfo {
    use chrono::Utc;
    ServiceInfo {
        service_id: id.to_string(),
        name: format!("Test {}", id),
        service_type: service_type.to_string(),
        version: "1.0.0".to_string(),
        description: Some(format!("Test service {}", id)),
        health_check_endpoint: Some("/health".to_string()),
        endpoints: vec![],
        tags: std::collections::HashMap::new(),
        
        dependencies: vec![],
        status: songbird_gaming_bridge::traits::service_id::ServiceStatus::Running,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        host: "localhost".to_string(),
        instance_id: format!("{}-instance", id),
        port: 8080,
    }
}

/// Create a test request with default values
pub fn create_test_request(method: &str, path: &str) -> ServiceRequest {
    ServiceRequest::new(method.to_string(), path.to_string())
}

/// Create a success response
pub fn create_success_response(request_id: String, data: serde_json::Value) -> ServiceResponse {
    ServiceResponse::success(request_id).with_body(data)
}

/// Create an error response
pub fn create_error_response(request_id: String, error: String) -> ServiceResponse {
    ServiceResponse::error(request_id, error)
}

/// Assert that a result is ok and return the value
#[macro_export]
macro_rules! assert_ok {
    ($result:expr) => {
        match $result {
            Ok(val) => val,
            Err(e) => panic!("Expected Ok, got Err: {:?}", e),
        }
    };
}

/// Assert that a result is an error
#[macro_export]
macro_rules! assert_err {
    ($result:expr) => {
        match $result {
            Ok(val) => panic!("Expected Err, got Ok: {:?}", val),
            Err(_) => {}
        }
    };
}

/// Mock configuration for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockConfig {
    pub service_id: String,
    pub port: u16,
    pub timeout: u64,
    pub max_connections: usize,
}

impl Default for MockConfig {
    fn default() -> Self {
        Self {
            service_id: "mock-service".to_string(),
            port: 8080,
            timeout: 30,
            max_connections: 100,
        }
    }
}

/// Mock service health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockHealth {
    pub status: String,
    pub uptime: u64,
    pub connections: usize,
}

impl Default for MockHealth {
    fn default() -> Self {
        Self {
            status: "healthy".to_string(),
            uptime: 0,
            connections: 0,
        }
    }
}

/// Mock service for testing
#[derive(Debug)]
pub struct MockService {
    config: MockConfig,
    running: std::sync::Arc<std::sync::atomic::AtomicBool>,
    health: std::sync::Arc<tokio::sync::RwLock<MockHealth>>,
}

impl MockService {
    pub fn new() -> Self {
        Self {
            config: MockConfig::default(),
            running: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            health: std::sync::Arc::new(tokio::sync::RwLock::new(MockHealth::default())),
        }
    }

    pub fn with_config(config: MockConfig) -> Self {
        Self {
            config,
            running: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            health: std::sync::Arc::new(tokio::sync::RwLock::new(MockHealth::default())),
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[async_trait]
impl UniversalService for MockService {
    type Error = SongbirdError;

    async fn start(&mut self) -> std::result::Result<()> {
        self.running
            .store(true, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    async fn stop(&mut self) -> std::result::Result<()> {
        self.running
            .store(false, std::sync::atomic::Ordering::Relaxed);
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
            serde_json::json!({"message": "Mock response", "path": request.path.clone()}),
        ))
    }

    async fn update_config(
        &mut self,
        config: serde_json::Value,
    ) -> std::result::Result<()> {
        // For testing purposes, we'll ignore the config update
        Ok(())
    }

    async fn get_metrics(&self) -> std::result::Result<ServiceMetrics> {
        Ok(ServiceMetrics {
            request_count: 0,
            error_count: 0,
            average_response_time: 10.0,
            cpu_usage: Some()Some(0.1),
            memory_usage: Some()Some(1024 * 1024), // 1MB
            active_connections: 0,
            uptime: Duration::from_secs(0),
            custom_metrics: HashMap::new(),
        })
    }

    fn service_info(&self) -> ServiceInfo {
        use chrono::Utc;
        ServiceInfo {
            service_id: self.config.service_id.clone(),
            name: "Mock Service".to_string(),
            version: "1.0.0".to_string(),
            service_type: "mock".to_string(),
            description: Some("Mock service for testing".to_string()),
            health_check_endpoint: Some("/health".to_string()),
            endpoints: vec![],
            tags: std::collections::HashMap::new(),
            
            dependencies: vec![],
            status: songbird_gaming_bridge::traits::service_id::ServiceStatus::Running,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            host: "localhost".to_string(),
            instance_id: format!("{}-instance", self.config.service_id),
            port: 8080,
        }
    }

    fn is_running(&self) -> bool {
        self.running.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// Test environment setup helper
pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub config: OrchestratorConfig,
}

pub async fn setup_test_environment() -> Result<TestEnvironment> {
    let temp_dir = tempfile::tempdir().map_err(|e| SongbirdError::Configuration {
        field: "temp_directory".to_string(),
        message: format!("Failed to create temp dir: {}", e),
    })?;

    let config = OrchestratorConfig::default();

    Ok(TestEnvironment { temp_dir, config })
}

pub async fn create_test_temp_dir() -> Result<tempfile::TempDir> {
    tempfile::tempdir().map_err(|e| SongbirdError::Configuration {
        field: "temp_directory".to_string(),
        message: format!("Failed to create temp dir: {}", e),
    })
}

async fn wait_for_condition<F, Fut>(mut condition: F, timeout: Duration) -> Result<()>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let start = tokio::time::Instant::now();
    while start.elapsed() < timeout {
        if condition().await {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Err(SongbirdError::Configuration {
        field: "timeout".to_string(),
        message: "Timeout waiting for condition".to_string(),
    })
}
