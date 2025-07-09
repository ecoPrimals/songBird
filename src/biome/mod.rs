//! Songbird-Sovereign biome.yaml Integration
//!
//! This module provides Songbird's own lightweight biome.yaml parsing capability
//! without depending on other Primals. It can coordinate with other Primals via
//! network APIs when they are available, leveraging network effects while maintaining sovereignty.

pub mod byob_coordinator;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tracing::{info, warn};
use std::time::Duration;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Re-export BYOB coordinator types
pub use byob_coordinator::{
    ByobCoordinator, ByobDeployment, ByobDeploymentRequest, ByobDeploymentStatus,
    ByobTeamWorkspace, ServiceHealth, ServiceStatus, TeamResourceQuota
};

// All coordination types are defined in this module and available by default

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

/// Songbird orchestrator for managing biome deployments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdOrchestrator {
    pub id: String,
    pub config: OrchestratorConfig,
    pub status: OrchestratorStatus,
    pub endpoints: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub manifest: SongbirdBiomeManifest,
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
            ByobError::Storage(msg) => write!(f, "Storage error: {}", msg),
            ByobError::Network(msg) => write!(f, "Network error: {}", msg),
            ByobError::Coordination(msg) => write!(f, "Coordination error: {}", msg),
            ByobError::Deployment(msg) => write!(f, "Deployment error: {}", msg),
            ByobError::Configuration(msg) => write!(f, "Configuration error: {}", msg),
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

/// Storage endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageEndpoint {
    pub endpoint_url: String,
    pub protocol: String,
    pub port: u16,
    pub tier: StorageTier,
    pub endpoint_type: String,
    pub url: String,
    pub mount_instructions: String,
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

/// Storage status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageStatus {
    Provisioning,
    Ready,
    Degraded,
    Failed,
    Maintenance,
}

/// Team deployment for processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamDeployment {
    pub deployment_id: String,
    pub team_id: String,
    pub manifest: SongbirdBiomeManifest,
    pub requirements: TeamStorageRequirements,
}

/// Deployment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResult {
    pub deployment_id: String,
    pub status: DeploymentStatus,
    pub endpoints: HashMap<String, String>,
    pub service_endpoints: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub manifest: SongbirdBiomeManifest,
}

/// Deployment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStatus {
    Pending,
    Running,
    Stopped,
    Failed,
    Scaling,
}

/// Configuration for orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub id: String,
    pub name: String,
    pub endpoints: HashMap<String, String>,
    pub timeout: Duration,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            id: "default-orchestrator".to_string(),
            name: "Default Orchestrator".to_string(),
            endpoints: HashMap::new(),
            timeout: Duration::from_secs(30),
        }
    }
}

impl OrchestratorConfig {
    /// Get Toadstool configuration for coordination
    pub fn get_toadstool_config(&self) -> ToadstoolConfig {
        ToadstoolConfig {
            endpoint: ToadstoolEndpoint {
                primary_url: self.endpoints.get("toadstool")
                    .cloned()
                    .unwrap_or_else(|| "http://localhost:8082".to_string()),
            }
        }
    }
}

/// Toadstool configuration for coordination
#[derive(Debug, Clone)]
pub struct ToadstoolConfig {
    pub endpoint: ToadstoolEndpoint,
}

/// Toadstool endpoint configuration
#[derive(Debug, Clone)]
pub struct ToadstoolEndpoint {
    pub primary_url: String,
}

/// Orchestrator status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestratorStatus {
    Initializing,
    Running,
    Stopped,
    Failed,
}

impl SongbirdOrchestrator {
    /// Parse biome.yaml using Songbird's sovereign parser
    pub async fn from_manifest_file(
        manifest_path: &Path,
        config: crate::config::OrchestratorConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Read and parse biome.yaml
        let content = fs::read_to_string(manifest_path).await?;
        let manifest: SongbirdBiomeManifest = serde_yaml::from_str(&content)?;
        
        info!("Parsed biome.yaml for orchestration: {}", manifest.metadata.name);
        
        Ok(Self { id: String::new(), config, status: OrchestratorStatus::Initializing, endpoints: HashMap::new(), created_at: Utc::now(), manifest })
    }

    /// Extract services that Songbird needs to orchestrate
    pub fn get_orchestration_services(&self) -> Vec<String> {
        self.manifest.services.keys().cloned().collect()
    }

    /// Extract networking configuration for Songbird
    pub fn extract_networking_config(&self) -> Option<crate::config::NetworkConfig> {
        self.manifest.networking.as_ref().map(|net_spec| {
            // Parse bind address from discovery config
            let bind_address = net_spec.discovery.as_ref()
                .and_then(|d| d.config.as_ref())
                .and_then(|c| c.get("bind_address"))
                .and_then(|v| v.as_str())
                .unwrap_or("127.0.0.1")
                .parse()
                .unwrap_or("127.0.0.1".parse().unwrap());

            // Get orchestrator port from ports config
            let orchestrator_port = net_spec.ports.as_ref()
                .and_then(|ports| ports.first())
                .copied()
                .unwrap_or(8080);

            crate::config::NetworkConfig {
                bind_address,
                orchestrator_port,
                gaming_port_range: crate::config::network::PortRange { start: 7000, end: 7100 },
                discovery_port: 8001,
                health_port: 8002,
                dashboard_port: 8003,
                timeouts: crate::config::network::TimeoutConfig::default(),
                connection_limits: crate::config::network::ConnectionLimits::default(),
                gaming: crate::config::network::GamingNetworkConfig::default(),
                discovery_ports: vec![8001],
                connection_timeout: std::time::Duration::from_secs(30),
                request_timeout: std::time::Duration::from_secs(30),
                enable_tls: false,
                allowed_networks: vec!["0.0.0.0/0".to_string()],
                max_connections: 1000,
                max_bandwidth_mbps: 1000,
                worker_threads: 4,
                federation_endpoints: vec![],
                stun_servers: vec![],
                websocket_port: 8004,
                metrics_bind_address: bind_address,
                metrics_port: 8005,
                federation_bind_address: bind_address,
                federation_port: 8006,
                cors: crate::config::network::CorsConfig {
                    enabled: true,
                    allowed_origins: vec!["*".to_string()],
                    allowed_methods: vec!["GET".to_string(), "POST".to_string()],
                    allowed_headers: vec!["Content-Type".to_string()],
                },
            }
        })
    }

    /// Check if coordination with other Primals is available (network effects)
    pub fn get_primal_coordination(&self) -> HashMap<String, PrimalCoordination> {
        self.manifest.primals.clone().unwrap_or_default()
    }
}

/// Network Effects - Universal Primal Coordination
impl SongbirdOrchestrator {
    /// Universal coordination method that works with any Primal
    pub async fn coordinate_with_primal(&self, primal_name: &str, primal_config: &PrimalCoordination) -> Result<(), Box<dyn std::error::Error>> {
        if !primal_config.enabled {
            info!("Primal {} coordination disabled - skipping", primal_name);
            return Ok(());
        }

        if let Some(endpoint) = &primal_config.endpoint {
            info!("Coordinating with {} at: {}", primal_name, endpoint);
            
            // Use universal coordination based on capabilities
            return self.call_universal_primal_api(primal_name, endpoint, primal_config).await;
        }

        warn!("{} coordination endpoint not available - continuing without", primal_name);
        Ok(())
    }

    /// Universal coordination with all available Primals
    pub async fn coordinate_with_all_primals(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(primals) = &self.manifest.primals {
            for (primal_name, primal_config) in primals {
                if let Err(e) = self.coordinate_with_primal(primal_name, primal_config).await {
                    warn!("Coordination with {} failed (continuing): {}", primal_name, e);
                }
            }
        }
        Ok(())
    }

    /// Universal API call that adapts to any Primal's interface
    async fn call_universal_primal_api(
        &self, 
        primal_name: &str, 
        endpoint: &str, 
        config: &PrimalCoordination
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        
        // Determine the appropriate API path based on capabilities
        let api_path = self.determine_api_path(primal_name, &config.capabilities);
        let full_url = format!("{}{}", endpoint, api_path);
        
        // Create universal coordination payload
        let coordination_payload = self.create_universal_payload(primal_name, &config.capabilities);
        
        info!("Universal coordination with {} at {}", primal_name, full_url);
        
        let response = client
            .post(&full_url)
            .json(&coordination_payload)
            .send()
            .await?;
        
        if response.status().is_success() {
            info!("Successfully coordinated with {} (universal adapter)", primal_name);
        } else {
            warn!("{} coordination failed: {} (universal adapter)", primal_name, response.status());
        }
        
        Ok(())
    }

    /// Determine the appropriate API path based on Primal capabilities
    fn determine_api_path(&self, primal_name: &str, capabilities: &[String]) -> String {
        // Universal API path detection based on capabilities
        for capability in capabilities {
            match capability.as_str() {
                "compute" | "execution" => return "/api/v1/orchestrate".to_string(),
                "storage" | "data" => return "/api/v1/provision".to_string(),
                "security" | "authentication" => return "/api/v1/authenticate".to_string(),
                "ai" | "ml" | "agents" => return "/api/v1/deploy-agents".to_string(),
                "custom" => return "/api/v1/coordinate".to_string(),
                _ => continue,
            }
        }
        
        // Fallback to standard coordination endpoint
        "/api/v1/coordinate".to_string()
    }

    /// Create universal payload that any Primal can understand
    fn create_universal_payload(&self, primal_name: &str, capabilities: &[String]) -> serde_json::Value {
        serde_json::json!({
            "coordination_request": {
                "from": "songbird",
                "to": primal_name,
                "manifest": self.manifest,
                "capabilities_requested": capabilities,
                "api_version": "universal/v1",
                "timestamp": chrono::Utc::now().to_rfc3339()
            },
            "songbird_context": {
                "orchestrator_id": self.id,
                "biome_name": self.manifest.metadata.name,
                "services_count": self.manifest.services.len(),
                "networking_enabled": self.manifest.networking.is_some()
            }
        })
    }

    /// Legacy method for backward compatibility with Toadstool
    pub async fn coordinate_with_toadstool(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(primals) = &self.manifest.primals {
            if let Some(toadstool) = primals.get("toadstool") {
                return self.coordinate_with_primal("toadstool", toadstool).await;
            }
        }
        warn!("Toadstool configuration not found - skipping");
        Ok(())
    }

    /// Legacy method for backward compatibility with NestGate
    pub async fn coordinate_with_nestgate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(primals) = &self.manifest.primals {
            if let Some(nestgate) = primals.get("nestgate") {
                return self.coordinate_with_primal("nestgate", nestgate).await;
            }
        }
        warn!("NestGate configuration not found - skipping");
        Ok(())
    }

    /// Legacy method for backward compatibility with BearDog
    pub async fn coordinate_with_beardog(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(primals) = &self.manifest.primals {
            if let Some(beardog) = primals.get("beardog") {
                return self.coordinate_with_primal("beardog", beardog).await;
            }
        }
        warn!("BearDog configuration not found - skipping");
        Ok(())
    }

    /// Legacy method for backward compatibility with Squirrel
    pub async fn coordinate_with_squirrel(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(primals) = &self.manifest.primals {
            if let Some(squirrel) = primals.get("squirrel") {
                return self.coordinate_with_primal("squirrel", squirrel).await;
            }
        }
        warn!("Squirrel configuration not found - skipping");
        Ok(())
    }
}

/// Orchestration operations
impl SongbirdOrchestrator {
    /// Start orchestrating the biome
    pub async fn orchestrate(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting sovereign biome orchestration: {}", self.manifest.metadata.name);
        
        // 1. Set up Songbird's own orchestration
        self.setup_service_registry().await?;
        
        // 2. Universal coordination with all available Primals (optional)
        let _ = self.coordinate_with_all_primals().await; // Continues if any fail
        
        // 3. Start orchestrating services
        self.orchestrate_services().await?;
        
        info!("Biome orchestration started successfully");
        Ok(())
    }

    async fn setup_service_registry(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Setting up Songbird service registry");
        // TODO: Implement Songbird's service registry
        Ok(())
    }

    async fn orchestrate_services(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting service orchestration for {} services", self.manifest.services.len());
        
        for (service_name, _service_spec) in &self.manifest.services {
            info!("Orchestrating service: {}", service_name);
            // TODO: Implement service orchestration logic
        }
        
        Ok(())
    }
} 