//! Compute Capability Provider (Primal-Agnostic)
//!
//! Provides compute, orchestration, and container management capabilities through
//! pure capability-based discovery. No hardcoded primal names.
//!
//! # Philosophy
//!
//! This module requests "compute" capability without knowing or caring which
//! primal provides it. Could be toadstool, could be something else. We only
//! care about the CAPABILITY, not the PROVIDER.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use songbird_types::{SongbirdError, SongbirdResult};

/// Compute capability configuration (vendor/primal agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeCapabilityConfig {
    /// Required compute capabilities
    pub required_capabilities: Vec<String>,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Discovery hints (environment variables to check)
    pub discovery_hints: Vec<String>,
}

/// Compute capability request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeRequest {
    /// Operation to perform
    pub operation: ComputeOperation,
    /// Request parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Request ID for tracking
    pub request_id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Compute operations (capability-based)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputeOperation {
    /// Deploy container
    DeployContainer { deployment: ContainerDeployment },
    /// Stop container
    StopContainer { container_id: String },
    /// Get container status
    GetContainerStatus { container_id: String },
    /// List containers
    ListContainers { filter: Option<HashMap<String, String>> },
    /// Configure service mesh
    ConfigureServiceMesh { config: ServiceMeshConfig },
    /// Discover network services
    DiscoverServices { network_range: Option<String> },
    /// Get network topology
    GetNetworkTopology,
    /// Scale deployment
    ScaleDeployment { deployment_id: String, replicas: u32 },
}

/// Container deployment specification (vendor-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerDeployment {
    /// Container name
    pub name: String,
    /// Container image
    pub image: String,
    /// Exposed ports
    pub ports: Vec<u16>,
    /// Environment variables
    pub environment: HashMap<String, String>,
    /// Resource requirements
    pub resources: ResourceRequirements,
    /// Volume mounts
    pub volumes: Vec<VolumeMount>,
    /// Network mode
    pub network_mode: Option<String>,
}

/// Resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// CPU limit (e.g., "1.0", "500m")
    pub cpu_limit: Option<String>,
    /// Memory limit (e.g., "1Gi", "512Mi")
    pub memory_limit: Option<String>,
    /// Storage limit
    pub storage_limit: Option<String>,
}

/// Volume mount
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    /// Source path
    pub source: String,
    /// Destination path in container
    pub destination: String,
    /// Read-only flag
    pub read_only: bool,
}

/// Service mesh configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMeshConfig {
    /// Mesh name
    pub name: String,
    /// Services to include
    pub services: Vec<String>,
    /// Routing rules
    pub routing_rules: HashMap<String, String>,
    /// Security policies
    pub security_policies: Vec<String>,
}

/// Compute capability response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeResponse {
    /// Request ID
    pub request_id: String,
    /// Success status
    pub success: bool,
    /// Response data
    pub data: serde_json::Value,
    /// Error message if failed
    pub error: Option<String>,
    /// Provider ID (learned through discovery)
    pub provider_id: String,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Network service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkService {
    /// Service name
    pub name: String,
    /// Endpoint URL
    pub endpoint: String,
    /// Protocol
    pub protocol: String,
    /// Status
    pub status: String,
    /// Last seen timestamp
    pub last_seen: DateTime<Utc>,
}

/// Container status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStatus {
    /// Container ID
    pub id: String,
    /// Container name
    pub name: String,
    /// Current status
    pub status: String,
    /// CPU usage percentage
    pub cpu_usage: Option<f64>,
    /// Memory usage in bytes
    pub memory_usage: Option<f64>,
    /// Uptime in seconds
    pub uptime_seconds: Option<u64>,
}

impl Default for ComputeCapabilityConfig {
    fn default() -> Self {
        Self {
            required_capabilities: vec![
                "container_orchestration".to_string(),
                "network_discovery".to_string(),
            ],
            timeout_secs: std::env::var("COMPUTE_TIMEOUT_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            max_retries: std::env::var("COMPUTE_MAX_RETRIES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3),
            discovery_hints: vec![
                "SONGBIRD_COMPUTE_DISCOVERY".to_string(),
                "COMPUTE_ENDPOINT".to_string(),
                "ORCHESTRATION_URL".to_string(),
                // Legacy compatibility (for migration period only)
                "TOADSTOOL_ENDPOINT".to_string(),
            ],
        }
    }
}

/// Request compute capability from discovered provider
///
/// This function uses the infant discovery engine to find a provider that
/// offers "compute" capability. It doesn't know or care about primal names.
pub async fn request_compute_capability(
    request: ComputeRequest,
) -> SongbirdResult<ComputeResponse> {
    // Import the infant discovery engine
    use songbird_universal::InfantDiscoveryEngine;

    // Get or create discovery engine
    let discovery = InfantDiscoveryEngine::new();

    // Request compute capability (no primal name needed!)
    let response = discovery
        .request_capability(
            "compute",
            &serde_json::to_string(&request.operation)
                .map_err(|e| SongbirdError::internal_error(&format!("Serialization failed: {}", e)))?,
            &serde_json::to_value(&request.parameters)
                .map_err(|e| SongbirdError::internal_error(&format!("Value conversion failed: {}", e)))?,
        )
        .await?;

    // Parse response
    let compute_response: ComputeResponse = serde_json::from_value(response.response_data)
        .map_err(|e| SongbirdError::internal_error(&format!("Failed to parse compute response: {}", e)))?;

    Ok(compute_response)
}

/// Helper: Deploy container
pub async fn deploy_container(deployment: ContainerDeployment) -> SongbirdResult<String> {
    let request = ComputeRequest {
        operation: ComputeOperation::DeployContainer { deployment },
        parameters: HashMap::new(),
        request_id: uuid::Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
    };

    let response = request_compute_capability(request).await?;

    if response.success {
        response.data
            .get("deployment_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| SongbirdError::internal_error("No deployment ID in response"))
    } else {
        Err(SongbirdError::internal_error(
            &response.error.unwrap_or_else(|| "Deployment failed".to_string()),
        ))
    }
}

/// Helper: Discover network services
pub async fn discover_services() -> SongbirdResult<Vec<NetworkService>> {
    let request = ComputeRequest {
        operation: ComputeOperation::DiscoverServices { network_range: None },
        parameters: HashMap::new(),
        request_id: uuid::Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
    };

    let response = request_compute_capability(request).await?;

    if response.success {
        serde_json::from_value(
            response.data.get("services")
                .cloned()
                .unwrap_or(serde_json::Value::Array(vec![]))
        )
        .map_err(|e| SongbirdError::internal_error(&format!("Failed to parse services: {}", e)))
    } else {
        Err(SongbirdError::internal_error(
            &response.error.unwrap_or_else(|| "Service discovery failed".to_string()),
        ))
    }
}

/// Helper: Get container status
pub async fn get_container_status(container_id: String) -> SongbirdResult<ContainerStatus> {
    let request = ComputeRequest {
        operation: ComputeOperation::GetContainerStatus { container_id },
        parameters: HashMap::new(),
        request_id: uuid::Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
    };

    let response = request_compute_capability(request).await?;

    if response.success {
        serde_json::from_value(response.data)
            .map_err(|e| SongbirdError::internal_error(&format!("Failed to parse container status: {}", e)))
    } else {
        Err(SongbirdError::internal_error(
            &response.error.unwrap_or_else(|| "Failed to get container status".to_string()),
        ))
    }
}

/// Helper: Scale deployment
pub async fn scale_deployment(deployment_id: String, replicas: u32) -> SongbirdResult<ComputeResponse> {
    let request = ComputeRequest {
        operation: ComputeOperation::ScaleDeployment { deployment_id, replicas },
        parameters: HashMap::new(),
        request_id: uuid::Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
    };

    request_compute_capability(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_config_default() {
        let config = ComputeCapabilityConfig::default();
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.max_retries, 3);
        assert!(!config.required_capabilities.is_empty());
    }

    #[test]
    fn test_container_deployment_creation() {
        let deployment = ContainerDeployment {
            name: "test-container".to_string(),
            image: "nginx:latest".to_string(),
            ports: vec![80, 443],
            environment: HashMap::new(),
            resources: ResourceRequirements {
                cpu_limit: Some("1.0".to_string()),
                memory_limit: Some("512Mi".to_string()),
                storage_limit: None,
            },
            volumes: vec![],
            network_mode: None,
        };

        assert_eq!(deployment.name, "test-container");
        assert_eq!(deployment.ports.len(), 2);
    }
}
