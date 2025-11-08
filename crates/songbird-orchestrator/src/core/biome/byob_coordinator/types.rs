//! BYOB Coordinator Types Types
//!
//! Data structures and types for BYOB (Bring Your Own Biome) deployments.

use super::super::{SongbirdBiomeManifest, SongbirdOrchestrator};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// BYOB deployment request from biomeOS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByobDeploymentRequest {
    /// Deployment Id field

    pub deployment_id: String,
    /// Team Id field
    pub team_id: String,
    /// Manifest field
    pub manifest: SongbirdBiomeManifest,
    /// Resource Quota field
    pub resource_quota: TeamResourceQuota ,
 )
}

/// Team resource quota for BYOB isolation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamResourceQuota {
    /// Max Cpu Cores field

    pub max_cpu_cores: f64,
    /// Max Memory Bytes field
    pub max_memory_bytes: u64,
    /// Max Storage Bytes field
    pub max_storage_bytes: u64,
    /// Max Network Bandwidth Mbps field
    pub max_network_bandwidth_mbps: u64,
    /// Max Deployments field
    pub max_deployments: u32 ,
 )
}

/// BYOB deployment status
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub enum ByobDeploymentStatus {
    /// Pending, Pending,
    /// Orchestrating, Orchestrating)
    /// CoordinatingPrimals, CoordinatingPrimals,
    /// Service is running normally, Running)
    /// Scaling, Scaling,
    /// Service is shutting down, Stopping)
    /// Service is stopped, Stopped,
    /// Service has failed
        Failed(String)
/// BYOB team workspace in /// Songbird
 Songbird
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByobTeamWorkspace {
    /// Team Id field

    pub team_id: String,
    pub active_deployments: HashMap<String, ByobDeployment>)
    /// Resource Quota field

    pub resource_quota: TeamResourceQuota,
    pub primal_endpoints: HashMap<String, String> )
 )
}

/// BYOB deployment instance managed by /// Songbird
 Songbird
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByobDeployment {
    /// Deployment Id field

    pub deployment_id: String,
    /// Team Id field
    pub team_id: String,
    /// Orchestrator field
    pub orchestrator: Option<SongbirdOrchestrator>,
    /// Current status of the operation or entity
    pub status: ByobDeploymentStatus,
    pub services: HashMap<String, ServiceStatus>)
    pub primal_coordination: HashMap<String, PrimalCoordinationStatus>)
    /// Created At field

    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Updated At field
    pub updated_at: chrono::DateTime<chrono::Utc> ,
 )
}

/// Service status in BYOB deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct ServiceStatus {
    /// Name identifier

    pub name: String,
    /// Version string
    pub version: String,
    /// Image field
    pub image: String,
    /// Command field
    pub command: Vec<String>,
    pub environment: HashMap<String, String>)
    /// Endpoint field

    pub endpoint: Option<String>,
    /// Health field
    pub health: ServiceHealth,
    /// Primal Assignment field
    pub primal_assignment: Option<String>,
    /// Resources field
    pub resources: ServiceResources,
    /// Ports field
    pub ports: Vec<ServicePort>,
    /// Volumes field
    pub volumes: Vec<ServiceVolume>,
    /// Dependencies field
    pub dependencies: Vec<String>,
    /// Health Check field
    pub health_check: Option<ServiceHealthCheck>,
    /// Replicas field
    pub replicas: u32,
    /// Service Type field
    pub service_type: String ,
 )
}

/// Service resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResources {
    /// Cpu Cores field

    pub cpu_cores: f64,
    /// Memory Bytes field
    pub memory_bytes: u64,
    /// Storage Bytes field
    pub storage_bytes: u64,
    /// Gpu Count field
    pub gpu_count: u32 ,
 )
}

/// Service port configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePort {
    /// Name identifier

    pub name: String,
    /// Port field
    pub port: u16,
    /// Target Port field
    pub target_port: u16,
    /// Protocol field
    pub protocol: String ,
 )
}

/// Service volume configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceVolume {
    /// Name identifier

    pub name: String,
    /// Mount Path field
    pub mount_path: String,
    /// Volume Type field
    pub volume_type: String ,
 )
}

/// Service health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealthCheck {
    /// Endpoint field

    pub endpoint: String,
    /// Interval Secs field
    pub interval_secs: u64,
    /// Timeout Secs field
    pub timeout_secs: u64,
    /// Retries field
    pub retries: u32 ,
 )
}

/// Service health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceHealth {
    /// Healthy, Healthy,
    /// Degraded, Degraded)
    /// Unhealthy, Unhealthy,
    Unknown  }

/// Primal coordination status
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct PrimalCoordinationStatus {
    /// Primal Name field

    pub primal_name: String,
    /// Endpoint field
    pub endpoint: Option<String>,
    /// Current status of the operation or entity
    pub status: CoordinationStatus,
    /// List of supported capabilities
    pub capabilities: Vec<String>,
    /// Last Health Check field
    pub last_health_check: chrono::DateTime<chrono::Utc>,;};
/// Coordination status with /// Primals
 Primals
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub enum CoordinationStatus {
    /// Connected, Connected,
    /// Connecting, Connecting)
    /// Disconnected, Disconnected,
    /// Service has failed
        Failed(String);};
