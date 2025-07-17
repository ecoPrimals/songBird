//! BYOB Coordinator Types
//!
//! Data structures and types for BYOB (Bring Your Own Biome) deployments.

use super::super::{SongbirdBiomeManifest, SongbirdOrchestrator};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// BYOB deployment request from biomeOS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByobDeploymentRequest {
    pub deployment_id: String,
    pub team_id: String,
    pub manifest: SongbirdBiomeManifest,
    pub resource_quota: TeamResourceQuota,
}

/// Team resource quota for BYOB isolation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamResourceQuota {
    pub max_cpu_cores: f64,
    pub max_memory_bytes: u64,
    pub max_storage_bytes: u64,
    pub max_network_bandwidth_mbps: u64,
    pub max_deployments: u32,
}

/// BYOB deployment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ByobDeploymentStatus {
    Pending,
    Orchestrating,
    CoordinatingPrimals,
    Running,
    Scaling,
    Stopping,
    Stopped,
    Failed(String),
}

/// BYOB team workspace in Songbird
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByobTeamWorkspace {
    pub team_id: String,
    pub active_deployments: HashMap<String, ByobDeployment>,
    pub resource_quota: TeamResourceQuota,
    pub primal_endpoints: HashMap<String, String>,
}

/// BYOB deployment instance managed by Songbird
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByobDeployment {
    pub deployment_id: String,
    pub team_id: String,
    pub orchestrator: Option<SongbirdOrchestrator>,
    pub status: ByobDeploymentStatus,
    pub services: HashMap<String, ServiceStatus>,
    pub primal_coordination: HashMap<String, PrimalCoordinationStatus>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Service status in BYOB deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub name: String,
    pub version: String,
    pub image: String,
    pub command: Vec<String>,
    pub environment: HashMap<String, String>,
    pub endpoint: Option<String>,
    pub health: ServiceHealth,
    pub primal_assignment: Option<String>,
    pub resources: ServiceResources,
    pub ports: Vec<ServicePort>,
    pub volumes: Vec<ServiceVolume>,
    pub dependencies: Vec<String>,
    pub health_check: Option<ServiceHealthCheck>,
    pub replicas: u32,
    pub service_type: String,
}

/// Service resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResources {
    pub cpu_cores: f64,
    pub memory_bytes: u64,
    pub storage_bytes: u64,
    pub gpu_count: u32,
}

/// Service port configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePort {
    pub name: String,
    pub port: u16,
    pub target_port: u16,
    pub protocol: String,
}

/// Service volume configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceVolume {
    pub name: String,
    pub mount_path: String,
    pub volume_type: String,
}

/// Service health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealthCheck {
    pub endpoint: String,
    pub interval_secs: u64,
    pub timeout_secs: u64,
    pub retries: u32,
}

/// Service health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Primal coordination status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalCoordinationStatus {
    pub primal_name: String,
    pub endpoint: Option<String>,
    pub status: CoordinationStatus,
    pub capabilities: Vec<String>,
    pub last_health_check: chrono::DateTime<chrono::Utc>,
}

/// Coordination status with Primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationStatus {
    Connected,
    Connecting,
    Disconnected,
    Failed(String),
}
