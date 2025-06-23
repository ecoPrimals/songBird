//! NestGate Integration Example
//!
//! This example demonstrates how to integrate existing NestGate services
//! with the Songbird Orchestrator, showing the migration from project-specific
//! patterns to universal, reusable ones.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use songbird_orchestrator::prelude::*;
use std::collections::HashMap;

/// NestGate NAS Service Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NasServiceConfig {
    pub storage_path: String,
    pub max_storage_gb: u64,
    pub enable_encryption: bool,
    pub backup_schedule: String,
    pub network_interfaces: Vec<String>,
    pub protocols: Vec<String>,
}

impl Default for NasServiceConfig {
    fn default() -> Self {
        Self {
            storage_path: "/data".to_string(),
            max_storage_gb: 1000,
            enable_encryption: true,
            backup_schedule: "0 2 * * *".to_string(), // Daily at 2 AM
            network_interfaces: vec!["eth0".to_string()],
            protocols: vec!["nfs".to_string(), "smb".to_string()],
        }
    }
}

/// Health status information for NAS service
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NasHealthStatus {
    pub storage_available_gb: u64,
    pub storage_used_gb: u64,
    pub storage_usage_percent: f64,
    pub active_connections: usize,
    pub protocols_status: HashMap<String, String>,
    pub last_backup: Option<chrono::DateTime<chrono::Utc>>,
    pub system_load: f64,
    pub temperature_celsius: Option<f64>,
}

/// NestGate NAS Service Adapter
///
/// This adapter wraps the existing NestGate NAS functionality
/// and makes it compatible with the Songbird Orchestrator
pub struct NestGateNasService {
    config: Option<NasServiceConfig>,
    started: bool,
}

impl NestGateNasService {
    pub fn new() -> Self {
        Self {
            config: None,
            started: false,
        }
    }
}

impl Default for NestGateNasService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UniversalService for NestGateNasService {
    type Config = NasServiceConfig;
    type Health = NasHealthStatus;
    type Error = SongbirdError;

    async fn initialize(&mut self, config: Self::Config) -> std::result::Result<(), Self::Error> {
        if config.storage_path.is_empty() {
            return Err(SongbirdError::Configuration {
                field: "storage_path".to_string(),
                message: "Storage path cannot be empty".to_string(),
            });
        }

        self.config = Some(config);
        println!("NestGate NAS Service initialized");
        Ok(())
    }

    async fn start(&mut self) -> std::result::Result<(), Self::Error> {
        if self.config.is_none() {
            return Err(SongbirdError::Configuration {
                field: "service_initialization".to_string(),
                message: "Service not initialized".to_string(),
            });
        }

        self.started = true;
        println!("NestGate NAS Service started");

        // Start background tasks for NAS operations
        // This would include:
        // - Storage monitoring
        // - Backup scheduling
        // - Performance optimization
        // - Health checks

        Ok(())
    }

    async fn stop(&mut self) -> std::result::Result<(), Self::Error> {
        self.started = false;
        println!("NestGate NAS Service stopped");

        // Clean shutdown:
        // - Save state
        // - Complete ongoing operations
        // - Release resources

        Ok(())
    }

    async fn health_check(&self) -> std::result::Result<Self::Health, Self::Error> {
        let _config = self
            .config
            .as_ref()
            .ok_or_else(|| SongbirdError::Configuration {
                field: "service_configuration".to_string(),
                message: "Service not configured".to_string(),
            })?;

        // Simulate health check logic
        let storage_used = 3200; // GB
        let storage_available = 6800; // GB
        let total = storage_used + storage_available;
        let usage_percent = (storage_used as f64 / total as f64) * 100.0;

        let mut protocols_status = HashMap::new();
        protocols_status.insert("nfs".to_string(), "running".to_string());
        protocols_status.insert("smb".to_string(), "running".to_string());
        protocols_status.insert("ftp".to_string(), "stopped".to_string());

        Ok(NasHealthStatus {
            storage_available_gb: storage_available,
            storage_used_gb: storage_used,
            storage_usage_percent: usage_percent,
            active_connections: 42,
            protocols_status,
            last_backup: Some(chrono::Utc::now() - chrono::Duration::hours(6)),
            system_load: 0.45,
            temperature_celsius: Some(42.5),
        })
    }

    async fn handle_request(
        &self,
        request: ServiceRequest,
    ) -> std::result::Result<ServiceResponse, Self::Error> {
        if !self.started {
            return Ok(ServiceResponse::error(
                request.id,
                503,
                "Service not running".to_string(),
            ));
        }

        // Handle different types of requests
        match request.path.as_str() {
            "/health" => {
                let health = self.health_check().await?;
                Ok(ServiceResponse::success(
                    request.id,
                    serde_json::to_value(health).map_err(SongbirdError::from)?,
                ))
            }
            "/storage/info" => {
                // Return storage information
                let info = serde_json::json!({
                    "total_capacity": "10TB",
                    "used_capacity": "3.2TB",
                    "available_capacity": "6.8TB"
                });
                Ok(ServiceResponse::success(request.id, info))
            }
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
            id: "nestgate-nas".to_string(),
            name: "NestGate NAS Service".to_string(),
            version: "1.0.0".to_string(),
            service_type: "storage".to_string(),
            description: "Network Attached Storage service for NestGate".to_string(),
            endpoints: vec![
                ServiceEndpoint {
                    path: "/health".to_string(),
                    method: "GET".to_string(),
                    description: "Service health status".to_string(),
                    parameters: vec![],
                    response_schema: None,
                },
                ServiceEndpoint {
                    path: "/storage/info".to_string(),
                    method: "GET".to_string(),
                    description: "Current storage usage and capacity".to_string(),
                    parameters: vec![],
                    response_schema: None,
                },
            ],
            capabilities: vec![
                "storage".to_string(),
                "backup".to_string(),
                "nas-protocols".to_string(),
                "health-monitoring".to_string(),
            ],
            tags: {
                let mut tags = HashMap::new();
                tags.insert("component".to_string(), "storage".to_string());
                tags.insert("protocol".to_string(), "nas".to_string());
                tags.insert("tier".to_string(), "backend".to_string());
                tags
            },
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("storage_type".to_string(), serde_json::json!("network"));
                metadata.insert("backup_enabled".to_string(), serde_json::json!(true));
                metadata.insert(
                    "protocols".to_string(),
                    serde_json::json!(["nfs", "smb", "ftp"]),
                );
                metadata
            },
        }
    }

    async fn can_handle_load(&self) -> std::result::Result<bool, Self::Error> {
        Ok(self.started)
    }

    async fn get_load_factor(&self) -> std::result::Result<f64, Self::Error> {
        // Return load factor (0.0 = no load, 1.0 = full load)
        Ok(0.3) // Simulate 30% load
    }

    async fn update_config(
        &mut self,
        config: Self::Config,
    ) -> std::result::Result<(), Self::Error> {
        self.config = Some(config);
        println!("NestGate NAS Service configuration updated");
        Ok(())
    }
}

/// Example demonstrating the full integration
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("Starting NestGate Integration Example...");

    // Create and initialize the service
    let nas_service = NestGateNasService::new();
    let nas_config = NasServiceConfig {
        storage_path: "/data/nestgate".to_string(),
        max_storage_gb: 1000,
        enable_encryption: true,
        backup_schedule: "0 2 * * *".to_string(),
        network_interfaces: vec!["eth0".to_string()],
        protocols: vec!["nfs".to_string(), "smb".to_string()],
    };

    // Create orchestrator configuration
    let mut orchestrator_config = OrchestratorConfig::default();
    orchestrator_config.orchestrator.bind_address = "127.0.0.1".to_string();
    orchestrator_config.orchestrator.port = 8080;

    // Create the orchestrator
    let orchestrator = Orchestrator::new(orchestrator_config).await?;

    // Register the NAS service with both service and config
    orchestrator
        .register_service(nas_service, nas_config)
        .await?;

    // Start the orchestrator
    orchestrator.start().await?;

    println!("NestGate Orchestrator started successfully!");
    println!("Running with NAS service integration...");

    // Keep the service running
    tokio::signal::ctrl_c()
        .await
        .map_err(|e| SongbirdError::Configuration {
            field: "signal_handling".to_string(),
            message: format!("Signal handling error: {}", e),
        })?;

    println!("Shutting down...");
    orchestrator.stop().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nas_service_lifecycle() {
        let mut service = NestGateNasService::new();
        let config = NasServiceConfig::default();

        // Test initialization
        service.initialize(config).await.unwrap();

        // Test starting
        service.start().await.unwrap();
        assert!(service.started);

        // Test health check
        let health = service.health_check().await.unwrap();
        assert!(health.storage_available_gb > 0);

        // Test request handling
        let request = ServiceRequest::new("GET", "/storage/info");
        let response = service.handle_request(request).await.unwrap();
        assert!(response.status_code == 200);

        // Test metrics
        let metrics = service.get_metrics().await.unwrap();
        assert!(metrics.uptime_seconds >= 0);

        // Test stopping
        service.stop().await.unwrap();
        assert!(!service.started);
    }

    #[tokio::test]
    async fn test_service_info() {
        let service = NestGateNasService::new();
        let info = service.service_info();

        assert_eq!(info.id, "nestgate-nas");
        assert_eq!(info.service_type, "storage");
        assert!(!info.endpoints.is_empty());
        assert!(info.capabilities.contains(&"storage".to_string()));
    }

    #[tokio::test]
    async fn test_load_management() {
        let mut service = NestGateNasService::new();
        let config = NasServiceConfig::default();

        service.initialize(config).await.unwrap();
        service.start().await.unwrap();

        // Test load capabilities
        let can_handle = service.can_handle_load().await.unwrap();
        assert!(can_handle); // Should be able to handle load when not overloaded

        let load_factor = service.get_load_factor().await.unwrap();
        assert!(load_factor >= 0.0 && load_factor <= 1.0);
    }
}
