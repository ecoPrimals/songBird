//! Biome Types and Data Structures
//!
//! This module contains all the data structures, enums, and type definitions
//! used throughout the biome management system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

/// Songbird's sovereign biome manifest structure
/// This is Songbird's own interpretation of biome.yaml focused on orchestration needs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdBiomeManifest {
    /// Basic metadata
    pub metadata: BiomeMetadata,

    /// Services that need orchestration
    pub services: HashMap<String, ServiceSpec>,

    /// Networking configuration
    pub networking: Option<NetworkingSpec>,

    /// Primal coordination (optional network effects)
    pub primals: Option<HashMap<String, PrimalCoordination>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSpec {
    /// Service endpoint for orchestration
    pub endpoint: Option<String>,

    /// Dependencies on other services
    pub depends_on: Vec<String>,

    /// Health check configuration
    pub health_check: Option<HealthCheckSpec>,

    /// Whether this service is managed by a Primal
    pub primal_managed: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkingSpec {
    /// Service discovery configuration
    pub discovery: Option<DiscoverySpec>,

    /// Port configurations
    pub ports: Option<Vec<u16>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverySpec {
    /// Discovery method (mDNS, consul, etc.)
    pub method: String,

    /// Configuration for discovery
    pub config: Option<serde_yaml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalCoordination {
    /// Whether this Primal is enabled for coordination
    pub enabled: bool,

    /// Network endpoint for coordination (discovered or configured)
    pub endpoint: Option<String>,

    /// Coordination capabilities this Primal provides
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckSpec {
    pub endpoint: String,
    pub interval_secs: u64,
    pub timeout_secs: u64,
}

/// BYOB-specific error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ByobError {
    Storage(String),
    Network(String),
    Coordination(String),
    Deployment(String),
    Configuration(String),
}

impl std::fmt::Display for ByobError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ByobError::Storage(msg) => write!(f, "Storage error: {msg}"),
            ByobError::Network(msg) => write!(f, "Network error: {msg}"),
            ByobError::Coordination(msg) => write!(f, "Coordination error: {msg}"),
            ByobError::Deployment(msg) => write!(f, "Deployment error: {msg}"),
            ByobError::Configuration(msg) => write!(f, "Configuration error: {msg}"),
        }
    }
}

impl std::error::Error for ByobError {}

/// NestGate configuration for coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestGateConfig {
    pub api_endpoint: String,
    pub api_key: String,
    pub default_pool: String,
    pub default_quotas: StorageQuotas,
    pub connection_timeout: u64,
}

/// Storage quotas for teams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageQuotas {
    pub max_storage_bytes: u64,
    pub max_snapshots: u32,
    pub max_volumes: u32,
}

/// Team storage requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamStorageRequirements {
    pub storage_size_bytes: u64,
    pub storage_tier: StorageTier,
    pub backup_enabled: bool,
    pub encryption_enabled: bool,
    pub service_storage: HashMap<String, ServiceStorageSpec>,
    pub persistence: bool,
    pub total_storage_quota: u64,
}

/// Storage specification for individual services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStorageSpec {
    pub size_bytes: u64,
    pub tier: StorageTier,
    pub backup_enabled: bool,
    pub name: String,
    pub mount_path: String,
    pub read_only: bool,
}

/// Storage tier levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageTier {
    Hot,
    Warm,
    Cold,
    Cache,
    Archive,
}

/// Storage deployment response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDeploymentResponse {
    pub deployment_id: Uuid,
    pub team_id: String,
    pub endpoints: HashMap<String, StorageEndpoint>,
    pub mounts: HashMap<String, VolumeMount>,
    pub usage: StorageUsage,
    pub status: StorageStatus,
    pub created_at: DateTime<Utc>,
    pub manifest: SongbirdBiomeManifest,
}

/// Storage endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageEndpoint {
    pub endpoint_url: String,
    pub tier: StorageTier,
    pub endpoint_type: String,
    pub mount_instructions: String,
    pub url: String,
    pub port: u16,
    pub protocol: String,
    pub is_secure: bool,
}

/// Volume mount configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    pub volume_id: String,
    pub mount_path: String,
    pub read_only: bool,
    pub size_bytes: u64,
    pub name: String,
    pub tier: StorageTier,
}

/// Storage usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageUsage {
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub total_bytes: u64,
    pub snapshots_count: u32,
    pub total_allocated: u64,
    pub total_used: u64,
    pub service_usage: HashMap<String, u64>,
}

/// Storage system status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageStatus {
    Provisioning,
    Ready,
    Degraded,
    Failed,
    Error,
    Maintenance,
}

/// Team deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamDeployment {
    pub deployment_id: String,
    pub team_id: String,
    pub manifest: SongbirdBiomeManifest,
    pub requirements: TeamStorageRequirements,
}

/// Deployment operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResult {
    pub deployment_id: String,
    pub status: DeploymentStatus,
    pub endpoints: HashMap<String, String>,
    pub service_endpoints: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub manifest: SongbirdBiomeManifest,
}

/// Deployment status tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStatus {
    Pending,
    Running,
    Stopped,
    Failed,
    Scaling,
}

/// Orchestrator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub id: String,
    pub name: String,
    pub endpoints: HashMap<String, String>,
    pub timeout: Duration,
    pub default_port: Option<u16>,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Songbird Orchestrator".to_string(),
            endpoints: HashMap::new(),
            timeout: Duration::from_secs(30),
            default_port: Some(8080),
        }
    }
}

/// Toadstool configuration for mushroom coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToadstoolConfig {
    pub endpoint: ToadstoolEndpoint,
}

/// Toadstool endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToadstoolEndpoint {
    pub primary_url: String,
}

/// Orchestrator operational status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrchestratorStatus {
    Initializing,
    Running,
    Stopped,
    Failed,
}

/// Main orchestrator structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdOrchestrator {
    pub id: String,
    pub config: OrchestratorConfig,
    pub status: OrchestratorStatus,
    pub endpoints: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub manifest: SongbirdBiomeManifest,
}

// Helper implementations for common operations
impl OrchestratorConfig {
    /// Get toadstool configuration from orchestrator config
    pub fn get_toadstool_config(&self) -> ToadstoolConfig {
        let toadstool_endpoint = self
            .endpoints
            .get("toadstool")
            .unwrap_or(&"http://localhost:8080".to_string())
            .clone();

        ToadstoolConfig {
            endpoint: ToadstoolEndpoint {
                primary_url: toadstool_endpoint,
            },
        }
    }

    /// Create a new config with default settings
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            endpoints: HashMap::new(),
            timeout: Duration::from_secs(30),
            default_port: Some(8080),
        }
    }

    /// Add an endpoint to the configuration
    pub fn add_endpoint(mut self, key: String, endpoint: String) -> Self {
        self.endpoints.insert(key, endpoint);
        self
    }

    /// Set the default port
    pub fn with_default_port(mut self, port: u16) -> Self {
        self.default_port = Some(port);
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

impl StorageTier {
    /// Get the performance characteristics of this storage tier
    pub fn performance_level(&self) -> u8 {
        match self {
            StorageTier::Hot => 5,     // Highest performance
            StorageTier::Cache => 4,   // Very high performance
            StorageTier::Warm => 3,    // Medium performance
            StorageTier::Cold => 2,    // Lower performance
            StorageTier::Archive => 1, // Lowest performance
        }
    }

    /// Check if this tier supports real-time access
    pub fn supports_realtime(&self) -> bool {
        matches!(self, StorageTier::Hot | StorageTier::Cache)
    }
}

impl StorageStatus {
    /// Check if the storage is in a healthy state
    pub fn is_healthy(&self) -> bool {
        matches!(self, StorageStatus::Ready)
    }

    /// Check if the storage is in a failed state
    pub fn is_failed(&self) -> bool {
        matches!(self, StorageStatus::Failed | StorageStatus::Error)
    }

    /// Check if the storage is in a transitional state
    pub fn is_transitional(&self) -> bool {
        matches!(
            self,
            StorageStatus::Provisioning | StorageStatus::Maintenance
        )
    }
}

impl DeploymentStatus {
    /// Check if the deployment is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, DeploymentStatus::Stopped | DeploymentStatus::Failed)
    }

    /// Check if the deployment is active
    pub fn is_active(&self) -> bool {
        matches!(self, DeploymentStatus::Running | DeploymentStatus::Scaling)
    }

    /// Check if the deployment is pending or starting
    pub fn is_starting(&self) -> bool {
        matches!(self, DeploymentStatus::Pending)
    }
}

impl OrchestratorStatus {
    /// Check if the orchestrator is operational
    pub fn is_operational(&self) -> bool {
        matches!(self, OrchestratorStatus::Running)
    }

    /// Check if the orchestrator is in a failed state
    pub fn is_failed(&self) -> bool {
        matches!(self, OrchestratorStatus::Failed)
    }

    /// Check if the orchestrator is starting up
    pub fn is_starting(&self) -> bool {
        matches!(self, OrchestratorStatus::Initializing)
    }
}
