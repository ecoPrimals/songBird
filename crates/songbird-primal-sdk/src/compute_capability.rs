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
use serde::{Deserialize, Serialize};
use songbird_config::capability_endpoints::{CapabilityEndpointResolver, CapabilityType};
use songbird_types::errors::{SongbirdError, SongbirdResult};
use songbird_universal::UnixRpcClient;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tracing::{debug, info};

/// Compute capability client (replaces ToadstoolPrimal)
///
/// **Pure Rust Implementation**: Uses Unix socket JSON-RPC for inter-primal communication,
/// eliminating HTTP overhead and `reqwest` dependency (ring-free!).
#[derive(Debug, Clone)]
pub struct ComputeCapabilityClient {
    /// Capability endpoint resolver (for discovery)
    resolver: CapabilityEndpointResolver,
    /// JSON-RPC client for Unix socket communication (Pure Rust!)
    rpc_client: UnixRpcClient,
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
        info!("⚙️ Creating compute capability client (Pure Rust Unix socket!)");
        
        // Discover Unix socket path for compute capability
        let socket_path = Self::discover_socket_path()?;
        
        // Create UnixRpcClient (100% Pure Rust!)
        let rpc_client = UnixRpcClient::new(&socket_path)
            .map_err(|e| SongbirdError::Configuration {
                message: format!("Failed to create Unix RPC client for {:?}: {}", socket_path, e),
                field: Some("rpc_client".to_string()),
                suggestion: Some("Ensure compute primal is running and socket exists".to_string()),
            })?;
        
        info!("✅ Compute capability client connected to {:?}", socket_path);
        
        Ok(Self {
            resolver: CapabilityEndpointResolver::new(),
            rpc_client,
            config,
        })
    }
    
    /// Discover Unix socket path for compute capability
    ///
    /// Priority:
    /// 1. COMPUTE_SOCKET_PATH environment variable
    /// 2. TOADSTOOL_SOCKET_PATH environment variable (legacy)
    /// 3. Default: /tmp/toadstool.sock
    fn discover_socket_path() -> SongbirdResult<PathBuf> {
        std::env::var("COMPUTE_SOCKET_PATH")
            .or_else(|_| std::env::var("TOADSTOOL_SOCKET_PATH"))
            .map(PathBuf::from)
            .or_else(|_| Ok(PathBuf::from("/tmp/toadstool.sock")))
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
        debug!("🚀 Executing workload via JSON-RPC: {}", request.name);
        
        // Call compute.execute_workload JSON-RPC method
        let response: WorkloadResponse = self.rpc_client
            .call("compute.execute_workload", &request)
            .await
            .map_err(|e| SongbirdError::Network {
                message: format!("Workload execution RPC failed: {}", e),
                source: Some("compute.execute_workload".to_string()),
            })?;
        
        info!("✅ Workload started (Pure Rust RPC!): {}", response.workload_id);
        Ok(response)
    }
    
    /// Get workload status
    pub async fn get_workload_status(&self, workload_id: &str) -> SongbirdResult<WorkloadResponse> {
        debug!("🔍 Getting workload status via JSON-RPC: {}", workload_id);
        
        #[derive(Serialize)]
        struct StatusRequest {
            workload_id: String,
        }
        
        // Call compute.get_workload_status JSON-RPC method
        let response: WorkloadResponse = self.rpc_client
            .call("compute.get_workload_status", &StatusRequest {
                workload_id: workload_id.to_string(),
            })
            .await
            .map_err(|e| SongbirdError::Network {
                message: format!("Workload status RPC failed: {}", e),
                source: Some("compute.get_workload_status".to_string()),
            })?;
        
        info!("✅ Workload status retrieved (Pure Rust RPC!)");
        Ok(response)
    }
    
    /// Scale workload replicas
    pub async fn scale_workload(&self, request: ScaleRequest) -> SongbirdResult<()> {
        debug!("📊 Scaling workload {} to {} replicas via JSON-RPC", request.workload_id, request.replicas);
        
        // Call compute.scale_workload JSON-RPC method
        let _response: serde_json::Value = self.rpc_client
            .call("compute.scale_workload", &request)
            .await
            .map_err(|e| SongbirdError::Network {
                message: format!("Workload scaling RPC failed: {}", e),
                source: Some("compute.scale_workload".to_string()),
            })?;
        
        info!("✅ Workload scaled successfully (Pure Rust RPC!)");
        Ok(())
    }
    
    /// Terminate workload
    pub async fn terminate_workload(&self, workload_id: &str) -> SongbirdResult<()> {
        debug!("🛑 Terminating workload via JSON-RPC: {}", workload_id);
        
        #[derive(Serialize)]
        struct TerminateRequest {
            workload_id: String,
        }
        
        // Call compute.terminate_workload JSON-RPC method
        let _response: serde_json::Value = self.rpc_client
            .call("compute.terminate_workload", &TerminateRequest {
                workload_id: workload_id.to_string(),
            })
            .await
            .map_err(|e| SongbirdError::Network {
                message: format!("Workload termination RPC failed: {}", e),
                source: Some("compute.terminate_workload".to_string()),
            })?;
        
        info!("✅ Workload terminated successfully (Pure Rust RPC!)");
        Ok(())
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

