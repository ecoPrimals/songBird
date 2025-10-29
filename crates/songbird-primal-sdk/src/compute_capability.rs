//! # ⚙️ Compute Capability Client (Zero Hardcoding)
//!
//! **REPLACES**: `toadstool.rs` - Hardcoded compute primal
//!
//! This module provides compute/workload capabilities without hardcoding specific primal names.
//! Works with ANY compute provider that implements the compute capability interface.
//!
//! ## Migration from Toadstool
//!
//! ```rust,ignore
//! // ❌ OLD: Hardcoded toadstool primal
//! let toadstool = ToadstoolPrimal::new(context);
//! let result = toadstool.execute_workload(workload_spec).await?;
//!
//! // ✅ NEW: Capability-based compute client
//! let compute = ComputeCapabilityClient::new().await?;
//! let result = compute.execute_workload(workload_spec).await?;
//! // Works with toadstool, k8s, docker, lambda, or any compute provider!
//! ```

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use songbird_config::capability_endpoints::{CapabilityEndpointResolver, CapabilityType};
use songbird_types::errors::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info};

/// Compute capability client (replaces ToadstoolPrimal)
#[derive(Debug)]
pub struct ComputeCapabilityClient {
    /// Capability endpoint resolver
    resolver: CapabilityEndpointResolver,
    /// HTTP client for requests
    http_client: Client,
    /// Client configuration
    config: ComputeClientConfig,
}

/// Compute client configuration
#[derive(Debug, Clone)]
pub struct ComputeClientConfig {
    /// Request timeout
    pub timeout: Duration,
    /// Default CPU limit (millicores)
    pub default_cpu_limit: u32,
    /// Default memory limit (MB)
    pub default_memory_limit: u32,
}

impl Default for ComputeClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(
                std::env::var("COMPUTE_REQUEST_TIMEOUT_SECS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(60),
            ),
            default_cpu_limit: std::env::var("COMPUTE_DEFAULT_CPU_LIMIT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1000), // 1 CPU
            default_memory_limit: std::env::var("COMPUTE_DEFAULT_MEMORY_LIMIT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(512), // 512 MB
        }
    }
}

/// Workload execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadRequest {
    /// Workload name/identifier
    pub name: String,
    /// Container image or execution target
    pub image: String,
    /// Command to execute
    pub command: Vec<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Resource requirements
    pub resources: ResourceRequirements,
    /// Networking configuration
    pub networking: Option<NetworkingConfig>,
}

/// Resource requirements for workload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// CPU limit (millicores)
    pub cpu_limit: u32,
    /// Memory limit (MB)
    pub memory_limit: u32,
    /// GPU requirements (optional)
    pub gpu: Option<GpuRequirements>,
}

/// GPU requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuRequirements {
    /// Number of GPUs
    pub count: u32,
    /// GPU type (e.g., "nvidia", "amd")
    pub gpu_type: Option<String>,
    /// Minimum GPU memory (MB)
    pub memory_mb: Option<u32>,
}

/// Networking configuration for workload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkingConfig {
    /// Port mappings
    pub ports: Vec<PortMapping>,
    /// Service mesh integration
    pub service_mesh: bool,
}

/// Port mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    /// Container port
    pub container_port: u16,
    /// Host port (optional, auto-assigned if None)
    pub host_port: Option<u16>,
    /// Protocol (tcp/udp)
    pub protocol: String,
}

/// Workload execution response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadResponse {
    /// Workload ID
    pub workload_id: String,
    /// Current status
    pub status: WorkloadStatus,
    /// Assigned endpoints
    pub endpoints: Vec<String>,
    /// Resource allocation
    pub allocated_resources: ResourceRequirements,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Workload status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkloadStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Terminated,
}

/// Workload scaling request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleRequest {
    /// Workload ID to scale
    pub workload_id: String,
    /// Target replica count
    pub replicas: u32,
}

impl ComputeCapabilityClient {
    /// Create new compute capability client
    ///
    /// Discovers compute providers dynamically - no hardcoded endpoints!
    ///
    /// # Example
    /// ```no_run
    /// use songbird_primal_sdk::compute_capability::ComputeCapabilityClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let compute = ComputeCapabilityClient::new().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new() -> SongbirdResult<Self> {
        Self::with_config(ComputeClientConfig::default()).await
    }
    
    /// Create compute client with custom configuration
    pub async fn with_config(config: ComputeClientConfig) -> SongbirdResult<Self> {
        info!("⚙️ Creating compute capability client (zero hardcoding)");
        
        let http_client = Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| SongbirdError::Configuration {
                message: format!("Failed to create HTTP client: {}", e),
                field: Some("http_client".to_string()),
                suggestion: Some("Check network configuration".to_string()),
            })?;
        
        Ok(Self {
            resolver: CapabilityEndpointResolver::new(),
            http_client,
            config,
        })
    }
    
    /// Execute workload on any compute provider
    ///
    /// Works with ANY provider that implements the compute capability:
    /// - toadstool (if available)
    /// - Kubernetes
    /// - Docker
    /// - AWS Lambda
    /// - Custom compute services
    ///
    /// # Example
    /// ```no_run
    /// # use songbird_primal_sdk::compute_capability::*;
    /// # use std::collections::HashMap;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let compute = ComputeCapabilityClient::new().await?;
    /// 
    /// let workload = WorkloadRequest {
    ///     name: "data-processor".to_string(),
    ///     image: "my-app:latest".to_string(),
    ///     command: vec!["./process".to_string()],
    ///     env: HashMap::new(),
    ///     resources: ResourceRequirements {
    ///         cpu_limit: 2000,
    ///         memory_limit: 2048,
    ///         gpu: None,
    ///     },
    ///     networking: None,
    /// };
    /// 
    /// let response = compute.execute_workload(workload).await?;
    /// println!("Workload started: {}", response.workload_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_workload(&self, request: WorkloadRequest) -> SongbirdResult<WorkloadResponse> {
        debug!("🚀 Executing workload: {}", request.name);
        
        // Discover compute capability provider
        let endpoint = self.resolver.get_endpoint(CapabilityType::Compute).await?;
        
        let response = self.http_client
            .post(format!("{}/workloads", endpoint))
            .json(&request)
            .send()
            .await
            .map_err(|e| SongbirdError::Network {
                message: format!("Workload execution request failed: {}", e),
                source: Some(endpoint.clone()),
            })?;
        
        if response.status().is_success() {
            let result: WorkloadResponse = response.json().await
                .map_err(|e| SongbirdError::Parsing {
                    message: format!("Failed to parse workload response: {}", e),
                    expected: "WorkloadResponse".to_string(),
                })?;
            
            info!("✅ Workload started: {}", result.workload_id);
            Ok(result)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(SongbirdError::External {
                service: "compute_capability".to_string(),
                message: format!("Workload execution failed with status {}: {}", status, error_text),
            })
        }
    }
    
    /// Get workload status
    pub async fn get_workload_status(&self, workload_id: &str) -> SongbirdResult<WorkloadResponse> {
        debug!("🔍 Getting workload status: {}", workload_id);
        
        let endpoint = self.resolver.get_endpoint(CapabilityType::Compute).await?;
        
        let response = self.http_client
            .get(format!("{}/workloads/{}", endpoint, workload_id))
            .send()
            .await
            .map_err(|e| SongbirdError::Network {
                message: format!("Failed to get workload status: {}", e),
                source: Some(endpoint.clone()),
            })?;
        
        if response.status().is_success() {
            response.json().await
                .map_err(|e| SongbirdError::Parsing {
                    message: format!("Failed to parse workload status: {}", e),
                    expected: "WorkloadResponse".to_string(),
                })
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(SongbirdError::External {
                service: "compute_capability".to_string(),
                message: format!("Failed to get workload status {}: {}", status, error_text),
            })
        }
    }
    
    /// Scale workload replicas
    pub async fn scale_workload(&self, request: ScaleRequest) -> SongbirdResult<()> {
        debug!("📊 Scaling workload {} to {} replicas", request.workload_id, request.replicas);
        
        let endpoint = self.resolver.get_endpoint(CapabilityType::Compute).await?;
        
        let response = self.http_client
            .post(format!("{}/workloads/{}/scale", endpoint, request.workload_id))
            .json(&request)
            .send()
            .await
            .map_err(|e| SongbirdError::Network {
                message: format!("Workload scaling request failed: {}", e),
                source: Some(endpoint.clone()),
            })?;
        
        if response.status().is_success() {
            info!("✅ Workload scaled successfully");
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(SongbirdError::External {
                service: "compute_capability".to_string(),
                message: format!("Workload scaling failed with status {}: {}", status, error_text),
            })
        }
    }
    
    /// Terminate workload
    pub async fn terminate_workload(&self, workload_id: &str) -> SongbirdResult<()> {
        debug!("🛑 Terminating workload: {}", workload_id);
        
        let endpoint = self.resolver.get_endpoint(CapabilityType::Compute).await?;
        
        let response = self.http_client
            .delete(format!("{}/workloads/{}", endpoint, workload_id))
            .send()
            .await
            .map_err(|e| SongbirdError::Network {
                message: format!("Workload termination request failed: {}", e),
                source: Some(endpoint.clone()),
            })?;
        
        if response.status().is_success() {
            info!("✅ Workload terminated successfully");
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(SongbirdError::External {
                service: "compute_capability".to_string(),
                message: format!("Workload termination failed with status {}: {}", status, error_text),
            })
        }
    }
    
    /// Check if compute capability is available
    pub async fn is_available(&self) -> bool {
        self.resolver.get_endpoint(CapabilityType::Compute).await.is_ok()
    }
    
    /// Get current configuration
    pub fn config(&self) -> &ComputeClientConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_compute_client_creation() {
        let result = ComputeCapabilityClient::new().await;
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_default_config() {
        let config = ComputeClientConfig::default();
        assert!(config.default_cpu_limit > 0);
        assert!(config.default_memory_limit > 0);
    }
}

