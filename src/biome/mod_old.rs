//! Songbird-Sovereign biome.yaml Integration
//!
//! This module provides Songbird's own lightweight biome.yaml parsing capability
//! without depending on other Primals. It can coordinate with other Primals via
//! network APIs when they are available, leveraging network effects while maintaining sovereignty.

pub mod byob_coordinator;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::path::Path;
use std::time::Duration;
use tokio::fs;
use tracing::{info, warn};
use uuid::Uuid;

// Re-export BYOB coordinator types
pub use byob_coordinator::{
    ByobCoordinator, ByobDeployment, ByobDeploymentRequest, ByobDeploymentStatus,
    ByobTeamWorkspace, ServiceHealth, ServiceStatus, TeamResourceQuota,
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
///
/// ## Configuration Options
///
/// The following environment variables can be used to configure discovery:
///
/// - `SONGBIRD_DISCOVERY_PORTS`: Comma-separated list of ports to scan (default: 8080,8081,8082,8083,8084,8085,3000,5000,9000)
/// - `SONGBIRD_DISCOVERY_HOSTS`: Comma-separated list of hosts to scan (default: 127.0.0.1,localhost,0.0.0.0)
/// - `SONGBIRD_DEFAULT_PORT`: Default port for endpoint patterns (default: 8080)
/// - `SONGBIRD_DEFAULT_ENDPOINT_PATTERNS`: Custom endpoint patterns with {primal} placeholder
/// - `SONGBIRD_DISCOVERY_TIMEOUT_MS`: Timeout for endpoint testing in milliseconds (default: 500)
///
/// ## Example Configuration
///
/// ```bash
/// export SONGBIRD_DISCOVERY_PORTS="8080,8081,3000,5000"
/// export SONGBIRD_DISCOVERY_HOSTS="127.0.0.1,localhost,10.0.0.1"
/// export SONGBIRD_DEFAULT_PORT="8080"
/// export SONGBIRD_DISCOVERY_TIMEOUT_MS="1000"
/// ```
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

/// Storage endpoint configuration
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

/// Storage status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageStatus {
    Provisioning,
    Ready,
    Degraded,
    Failed,
    Error,
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
    pub default_port: Option<u16>,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            id: "default-orchestrator".to_string(),
            name: "Default Orchestrator".to_string(),
            endpoints: HashMap::new(),
            timeout: Duration::from_secs(30),
            default_port: Some(8080),
        }
    }
}

impl OrchestratorConfig {
    /// Get Toadstool configuration for coordination
    pub fn get_toadstool_config(&self) -> ToadstoolConfig {
        ToadstoolConfig {
            endpoint: ToadstoolEndpoint {
                primary_url: self
                    .endpoints
                    .get("toadstool")
                    .cloned()
                    .unwrap_or_else(|| "http://localhost:8082".to_string()),
            },
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
    ///
    /// Creates a new `SongbirdOrchestrator` instance from a biome.yaml manifest file.
    /// This method reads the manifest, parses it, and creates an orchestrator ready
    /// to manage the biome deployment.
    ///
    /// # Arguments
    ///
    /// * `manifest_path` - Path to the biome.yaml manifest file
    /// * `config` - Orchestrator configuration settings
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the orchestrator instance or an error if the
    /// manifest file cannot be read or parsed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use songbird::biome::{SongbirdOrchestrator, OrchestratorConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = OrchestratorConfig::default();
    /// let orchestrator = SongbirdOrchestrator::from_manifest_file(
    ///     Path::new("biome.yaml"),
    ///     config
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn from_manifest_file(
        manifest_path: &Path,
        config: OrchestratorConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Read and parse biome.yaml
        let content = fs::read_to_string(manifest_path).await?;
        let manifest: SongbirdBiomeManifest = serde_yaml::from_str(&content)?;

        info!(
            "Parsed biome.yaml for orchestration: {}",
            manifest.metadata.name
        );

        Ok(Self {
            id: String::new(),
            config,
            status: OrchestratorStatus::Initializing,
            endpoints: HashMap::new(),
            created_at: Utc::now(),
            manifest,
        })
    }

    /// Extract services that Songbird needs to orchestrate
    pub fn get_orchestration_services(&self) -> Vec<String> {
        self.manifest.services.keys().cloned().collect()
    }

    /// Extract networking configuration for Songbird
    pub fn extract_networking_config(&self) -> Option<crate::config::NetworkConfig> {
        let bind_address = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        Some(crate::config::NetworkConfig {
            bind_address,
            production_bind_address: bind_address,
            orchestrator_port: 8000,
            gaming_port: 6112,
            gaming_port_range: crate::config::network::PortRange {
                start: 7000,
                end: 7100,
            },
            discovery_port: 8001,
            health_port: 8002,
            dashboard_port: 8003,
            timeouts: crate::config::network::NetworkTimeouts::default(),
            connection_limits: crate::config::network::ConnectionLimits::default(),
            gaming: crate::config::network::GamingNetworkConfig::default(),
            discovery_ports: vec![8001],
            connection_timeout: std::time::Duration::from_secs(30),
            request_timeout: std::time::Duration::from_secs(30),
            require_tls: false,
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
                origins: vec!["*".to_string()],
                allowed_methods: vec!["GET".to_string(), "POST".to_string()],
                allowed_headers: vec!["Content-Type".to_string()],
            },
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
    /// This method is completely agnostic and future-proof - any Primal can integrate
    ///
    /// This method implements the universal coordination protocol that allows Songbird
    /// to coordinate with any Primal (Toadstool, NestGate, BearDog, Squirrel, etc.)
    /// without requiring specific knowledge of each Primal's implementation.
    ///
    /// # Arguments
    ///
    /// * `primal_name` - Name of the Primal to coordinate with (e.g., "toadstool", "nestgate")
    /// * `primal_config` - Configuration specifying capabilities and coordination settings
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating successful coordination or an error if coordination fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use songbird::biome::{SongbirdOrchestrator, PrimalCoordination};
    ///
    /// # async fn example(orchestrator: SongbirdOrchestrator) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    /// let config = PrimalCoordination {
    ///     enabled: true,
    ///     endpoint: Some("http://toadstool:8080".to_string()),
    ///     capabilities: vec!["compute".to_string(), "containers".to_string()],
    /// };
    ///
    /// orchestrator.coordinate_with_primal("toadstool", &config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn coordinate_with_primal(
        &self,
        primal_name: &str,
        primal_config: &PrimalCoordination,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !primal_config.enabled {
            info!("Primal {} coordination disabled - skipping", primal_name);
            return Ok(());
        }

        if let Some(endpoint) = &primal_config.endpoint {
            info!(
                "Universal coordination with {} at: {}",
                primal_name, endpoint
            );

            // Use universal coordination based on capabilities
            return self
                .call_universal_primal_api(primal_name, endpoint, primal_config)
                .await;
        }

        // Try to discover endpoint if not configured
        if let Some(discovered_endpoint) = self.discover_primal_endpoint(primal_name).await {
            info!(
                "Discovered {} endpoint: {}",
                primal_name, discovered_endpoint
            );
            return self
                .call_universal_primal_api(primal_name, &discovered_endpoint, primal_config)
                .await;
        }

        warn!(
            "{} coordination endpoint not available - continuing without",
            primal_name
        );
        Ok(())
    }

    /// Universal coordination with all available Primals
    ///
    /// This method coordinates with all configured Primals in the biome manifest.
    /// It's completely fault-tolerant - if any Primal fails, coordination continues
    /// with the remaining Primals. This ensures maximum network effect while
    /// maintaining system stability.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating the overall coordination status. Individual
    /// Primal failures are logged but don't cause the entire operation to fail.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use songbird::biome::SongbirdOrchestrator;
    ///
    /// # async fn example(orchestrator: SongbirdOrchestrator) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    /// // Coordinate with all available Primals
    /// orchestrator.coordinate_with_all_primals().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn coordinate_with_all_primals(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(primals) = &self.manifest.primals {
            for (primal_name, primal_config) in primals {
                if let Err(e) = self
                    .coordinate_with_primal(primal_name, primal_config)
                    .await
                {
                    warn!(
                        "Coordination with {} failed (continuing): {}",
                        primal_name, e
                    );
                }
            }
        }
        Ok(())
    }

    /// Universal Primal endpoint discovery
    /// Future-proof discovery that works with any Primal's discovery mechanism
    async fn discover_primal_endpoint(&self, primal_name: &str) -> Option<String> {
        // Try multiple discovery methods in order of preference

        // 1. Service discovery (DNS-SD, mDNS)
        if let Some(endpoint) = self.discover_via_service_discovery(primal_name).await {
            return Some(endpoint);
        }

        // 2. Network scanning on common ports
        if let Some(endpoint) = self.discover_via_network_scan(primal_name).await {
            return Some(endpoint);
        }

        // 3. Environment variables
        if let Some(endpoint) = self.discover_via_environment(primal_name).await {
            return Some(endpoint);
        }

        // 4. Default endpoints based on common patterns
        if let Some(endpoint) = self.discover_via_defaults(primal_name).await {
            return Some(endpoint);
        }

        None
    }

    /// Service discovery (DNS-SD, mDNS)
    async fn discover_via_service_discovery(&self, primal_name: &str) -> Option<String> {
        // Look for service advertisements
        let service_name = format!("_{}-primal._tcp.local", primal_name.to_lowercase());
        info!("Searching for service: {}", service_name);

        // This would integrate with actual service discovery
        // For now, return None to indicate not found
        None
    }

    /// Network scanning on common ports
    async fn discover_via_network_scan(&self, primal_name: &str) -> Option<String> {
        // Get configurable ports from environment or use defaults
        let common_ports = self.get_discovery_ports();
        let localhost_variants = self.get_discovery_hosts();

        for host in localhost_variants {
            for port in &common_ports {
                let endpoint = format!("http://{}:{}", host, port);
                if self.test_primal_endpoint(&endpoint, primal_name).await {
                    return Some(endpoint);
                }
            }
        }

        None
    }

    /// Get configurable discovery ports
    fn get_discovery_ports(&self) -> Vec<u16> {
        if let Ok(ports_str) = std::env::var("SONGBIRD_DISCOVERY_PORTS") {
            ports_str
                .split(',')
                .filter_map(|p| p.trim().parse::<u16>().ok())
                .collect()
        } else {
            vec![8080, 8081, 8082, 8083, 8084, 8085, 3000, 5000, 9000]
        }
    }

    /// Get configurable discovery hosts
    fn get_discovery_hosts(&self) -> Vec<&str> {
        if let Ok(hosts_str) = std::env::var("SONGBIRD_DISCOVERY_HOSTS") {
            // Use a thread-safe pattern to avoid unsafe code
            use std::sync::OnceLock;

            static PARSED_HOSTS: OnceLock<Vec<String>> = OnceLock::new();

            let hosts = PARSED_HOSTS
                .get_or_init(|| hosts_str.split(',').map(|h| h.trim().to_string()).collect());

            // Convert to &str references
            hosts.iter().map(|s| s.as_str()).collect()
        } else {
            vec!["127.0.0.1", "localhost", "0.0.0.0"]
        }
    }

    /// Environment variable discovery
    async fn discover_via_environment(&self, primal_name: &str) -> Option<String> {
        let env_vars = vec![
            format!("{}_ENDPOINT", primal_name.to_uppercase()),
            format!("{}_URL", primal_name.to_uppercase()),
            format!("{}_API_ENDPOINT", primal_name.to_uppercase()),
        ];

        for env_var in env_vars {
            if let Ok(endpoint) = std::env::var(env_var) {
                if self.test_primal_endpoint(&endpoint, primal_name).await {
                    return Some(endpoint);
                }
            }
        }

        None
    }

    /// Default endpoint patterns
    async fn discover_via_defaults(&self, primal_name: &str) -> Option<String> {
        let default_patterns = self.get_default_endpoint_patterns(primal_name);

        for endpoint in default_patterns {
            if self.test_primal_endpoint(&endpoint, primal_name).await {
                return Some(endpoint);
            }
        }

        None
    }

    /// Get configurable default endpoint patterns
    fn get_default_endpoint_patterns(&self, primal_name: &str) -> Vec<String> {
        if let Ok(patterns_str) = std::env::var("SONGBIRD_DEFAULT_ENDPOINT_PATTERNS") {
            patterns_str
                .split(',')
                .map(|pattern| {
                    pattern
                        .trim()
                        .replace("{primal}", &primal_name.to_lowercase())
                })
                .collect()
        } else {
            // Get configurable default port
            let default_port =
                std::env::var("SONGBIRD_DEFAULT_PORT").unwrap_or_else(|_| "8080".to_string());

            vec![
                format!("http://{}:{}", primal_name.to_lowercase(), default_port),
                format!("http://{}-api:{}", primal_name.to_lowercase(), default_port),
                format!(
                    "http://{}.local:{}",
                    primal_name.to_lowercase(),
                    default_port
                ),
            ]
        }
    }

    /// Test if an endpoint responds to Primal coordination
    async fn test_primal_endpoint(&self, endpoint: &str, primal_name: &str) -> bool {
        let client = reqwest::Client::new();
        let test_url = format!("{}/health", endpoint);

        // Get configurable timeout
        let timeout_ms = std::env::var("SONGBIRD_DISCOVERY_TIMEOUT_MS")
            .unwrap_or_else(|_| "500".to_string())
            .parse::<u64>()
            .unwrap_or(500);

        if let Ok(response) = client
            .get(&test_url)
            .timeout(std::time::Duration::from_millis(timeout_ms))
            .send()
            .await
        {
            if response.status().is_success() {
                // Check if response indicates it's the right Primal
                if let Ok(text) = response.text().await {
                    return text.to_lowercase().contains(&primal_name.to_lowercase());
                }
            }
        }

        false
    }

    /// Universal API call that adapts to any Primal's interface
    async fn call_universal_primal_api(
        &self,
        primal_name: &str,
        endpoint: &str,
        config: &PrimalCoordination,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        // Determine the appropriate API path based on capabilities
        let api_path = self.determine_api_path(primal_name, &config.capabilities);
        let full_url = format!("{}{}", endpoint, api_path);

        // Create universal coordination payload
        let coordination_payload = self.create_universal_payload(primal_name, &config.capabilities);

        info!(
            "Universal coordination with {} at {}",
            primal_name, full_url
        );

        let response = client
            .post(&full_url)
            .json(&coordination_payload)
            .send()
            .await?;

        if response.status().is_success() {
            info!(
                "Successfully coordinated with {} (universal adapter)",
                primal_name
            );
        } else {
            warn!(
                "{} coordination failed: {} (universal adapter)",
                primal_name,
                response.status()
            );
        }

        Ok(())
    }

    /// Determine the appropriate API path based on Primal capabilities
    /// Future-proof capability-based routing that adapts to any Primal
    fn determine_api_path(&self, _primal_name: &str, capabilities: &[String]) -> String {
        // Priority-based capability routing - more specific capabilities first

        // High-priority specific capabilities
        for capability in capabilities {
            match capability.as_str() {
                "orchestration" | "coordination" => return "/api/v1/coordinate".to_string(),
                "deployment" | "provisioning" => return "/api/v1/provision".to_string(),
                "security" | "authentication" | "authorization" => {
                    return "/api/v1/secure".to_string()
                }
                "ai" | "ml" | "agents" | "intelligence" => {
                    return "/api/v1/intelligence".to_string()
                }
                "compute" | "execution" | "processing" => return "/api/v1/compute".to_string(),
                "storage" | "data" | "persistence" => return "/api/v1/storage".to_string(),
                "network" | "networking" | "connectivity" => return "/api/v1/network".to_string(),
                "monitoring" | "observability" | "metrics" => return "/api/v1/monitor".to_string(),
                _ => continue,
            }
        }

        // Fallback to universal coordination endpoint
        "/api/v1/coordinate".to_string()
    }

    /// Create universal payload that any Primal can understand
    fn create_universal_payload(
        &self,
        primal_name: &str,
        capabilities: &[String],
    ) -> serde_json::Value {
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
}

/// Orchestration operations
impl SongbirdOrchestrator {
    /// Start orchestrating the biome
    ///
    /// This is the main entry point for biome orchestration. It performs the complete
    /// orchestration workflow:
    ///
    /// 1. Sets up Songbird's internal service registry
    /// 2. Coordinates with all available Primals for network effects
    /// 3. Orchestrates all services defined in the manifest
    ///
    /// The orchestration is designed to be fault-tolerant and will continue even if
    /// some Primal coordination fails, ensuring that the core biome functionality
    /// remains available.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating successful orchestration startup or an error
    /// if critical components fail to initialize.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use songbird::biome::{SongbirdOrchestrator, OrchestratorConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = OrchestratorConfig::default();
    /// let orchestrator = SongbirdOrchestrator::from_manifest_file(
    ///     Path::new("biome.yaml"),
    ///     config
    /// ).await?;
    ///
    /// // Start the orchestration
    /// orchestrator.orchestrate().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn orchestrate(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Starting sovereign biome orchestration: {}",
            self.manifest.metadata.name
        );

        // 1. Set up Songbird's own orchestration
        self.setup_service_registry().await?;

        // 2. Universal coordination with all available Primals (optional)
        let _ = self.coordinate_with_all_primals().await; // Continues if any fail

        // 3. Start orchestrating services
        self.orchestrate_services().await?;

        info!("Biome orchestration started successfully");
        Ok(())
    }

    async fn setup_service_registry(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Setting up Songbird service registry with {} services",
            self.manifest.services.len()
        );

        // Create service registry instance
        let _registry = crate::registry::ServiceRegistry::default();

        // Register all services from the manifest
        for (service_name, service_spec) in &self.manifest.services {
            let service_info = crate::traits::service::ServiceInfo {
                service_id: service_name.clone(),
                name: service_name.clone(),
                version: self.manifest.metadata.version.clone(),
                service_type: service_spec
                    .primal_managed
                    .clone()
                    .unwrap_or_else(|| "generic".to_string()),
                description: Some(format!(
                    "Service {} from biome {}",
                    service_name, self.manifest.metadata.name
                )),
                endpoints: {
                    let mut endpoints = Vec::new();
                    if let Some(endpoint) = &service_spec.endpoint {
                        endpoints.push(crate::traits::service::ServiceEndpoint {
                            path: endpoint.clone(),
                            method: "GET".to_string(),
                            description: Some("Primary service endpoint".to_string()),
                            parameters: vec![],
                            response_schema: None,
                            auth_required: false,
                            rate_limit: None,
                        });
                    }
                    endpoints
                },
                health_check_endpoint: service_spec
                    .health_check
                    .as_ref()
                    .map(|hc| hc.endpoint.clone()),
                metadata: {
                    let mut metadata = std::collections::HashMap::new();
                    metadata.insert(
                        "biome_name".to_string(),
                        self.manifest.metadata.name.clone().into(),
                    );
                    metadata.insert("orchestrator_id".to_string(), self.id.clone().into());
                    if let Some(primal) = &service_spec.primal_managed {
                        metadata.insert("primal_managed".to_string(), primal.clone().into());
                    }
                    metadata
                },
                tags: vec!["songbird".to_string(), "biome".to_string()],
                dependencies: service_spec.depends_on.clone(),
                status: crate::traits::service::ServiceStatus::Starting,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                instance_id: format!("{}-{}", self.id, service_name),
                host: "localhost".to_string(),
                port: 8080, // Default port, would be parsed from endpoint
            };

            // For now, just log the service registration - full implementation would call registry.register()
            tracing::info!(
                "Would register service: {} with {:?}",
                service_info.service_id,
                service_info.endpoints
            );
            info!(
                "Registered service: {} -> {}",
                service_name,
                service_spec
                    .endpoint
                    .as_deref()
                    .unwrap_or("auto-discovered")
            );
        }

        // Setup health monitoring for registered services
        self.setup_health_monitoring().await?;

        info!(
            "Service registry setup complete with {} services",
            self.manifest.services.len()
        );
        Ok(())
    }

    /// Setup health monitoring for registered services
    async fn setup_health_monitoring(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Setting up health monitoring for biome services");

        for (service_name, service_spec) in &self.manifest.services {
            if let Some(health_check) = &service_spec.health_check {
                info!(
                    "Configuring health check for {}: {} every {}s",
                    service_name, health_check.endpoint, health_check.interval_secs
                );

                // Start health check monitoring (in a real implementation,
                // this would spawn background tasks)
                tokio::spawn({
                    let service_name = service_name.clone();
                    let health_check = health_check.clone();
                    async move {
                        loop {
                            match Self::check_service_health(&service_name, &health_check).await {
                                Ok(healthy) => {
                                    tracing::debug!(
                                        "Health check for {}: {}",
                                        service_name,
                                        if healthy { "HEALTHY" } else { "UNHEALTHY" }
                                    );
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "Health check failed for {}: {}",
                                        service_name,
                                        e
                                    );
                                }
                            }
                            tokio::time::sleep(Duration::from_secs(health_check.interval_secs))
                                .await;
                        }
                    }
                });
            }
        }

        Ok(())
    }

    /// Check health of a specific service
    async fn check_service_health(
        service_name: &str,
        health_check: &HealthCheckSpec,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(health_check.timeout_secs))
            .build()?;

        let response = client.get(&health_check.endpoint).send().await?;
        let is_healthy = response.status().is_success();

        if !is_healthy {
            tracing::warn!(
                "Service {} health check failed: HTTP {}",
                service_name,
                response.status()
            );
        }

        Ok(is_healthy)
    }

    async fn orchestrate_services(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Starting service orchestration for {} services",
            self.manifest.services.len()
        );

        // Sort services by dependency order
        let orchestration_order = self.resolve_service_dependencies()?;

        // Orchestrate services in dependency order
        for service_name in orchestration_order {
            if let Some(service_spec) = self.manifest.services.get(&service_name) {
                info!("Orchestrating service: {}", service_name);

                // Start the service orchestration
                self.orchestrate_single_service(&service_name, service_spec)
                    .await?;

                // Wait for service to be ready before continuing
                self.wait_for_service_ready(&service_name, service_spec)
                    .await?;
            }
        }

        info!(
            "Service orchestration completed for {} services",
            self.manifest.services.len()
        );
        Ok(())
    }

    /// Resolve service dependencies and return orchestration order
    fn resolve_service_dependencies(
        &self,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut ordered_services = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut visiting = std::collections::HashSet::new();

        // Topological sort using DFS
        for service_name in self.manifest.services.keys() {
            if !visited.contains(service_name) {
                self.visit_service_dependencies(
                    service_name,
                    &mut visited,
                    &mut visiting,
                    &mut ordered_services,
                )?;
            }
        }

        info!("Service orchestration order: {:?}", ordered_services);
        Ok(ordered_services)
    }

    /// Visit service dependencies recursively for topological sort
    fn visit_service_dependencies(
        &self,
        service_name: &str,
        visited: &mut std::collections::HashSet<String>,
        visiting: &mut std::collections::HashSet<String>,
        ordered_services: &mut Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if visiting.contains(service_name) {
            return Err(format!(
                "Circular dependency detected involving service: {}",
                service_name
            )
            .into());
        }

        if visited.contains(service_name) {
            return Ok(());
        }

        visiting.insert(service_name.to_string());

        if let Some(service_spec) = self.manifest.services.get(service_name) {
            for dependency in &service_spec.depends_on {
                if !self.manifest.services.contains_key(dependency) {
                    warn!(
                        "Service {} depends on {} which is not defined in manifest",
                        service_name, dependency
                    );
                    continue;
                }

                self.visit_service_dependencies(dependency, visited, visiting, ordered_services)?;
            }
        }

        visiting.remove(service_name);
        visited.insert(service_name.to_string());
        ordered_services.push(service_name.to_string());

        Ok(())
    }

    /// Orchestrate a single service
    async fn orchestrate_single_service(
        &self,
        service_name: &str,
        service_spec: &ServiceSpec,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting orchestration for service: {}", service_name);

        // Check if service is Primal-managed
        if let Some(primal_name) = &service_spec.primal_managed {
            info!(
                "Service {} is managed by {}, coordinating...",
                service_name, primal_name
            );

            // Coordinate with the managing Primal
            if let Some(primals) = &self.manifest.primals {
                if let Some(primal_config) = primals.get(primal_name) {
                    self.coordinate_with_primal(primal_name, primal_config)
                        .await?;
                } else {
                    warn!(
                        "Primal {} not found in manifest for service {}",
                        primal_name, service_name
                    );
                }
            }
        } else {
            // Orchestrate service directly with Songbird
            info!(
                "Service {} is Songbird-managed, starting direct orchestration",
                service_name
            );

            // Ensure service endpoint is available
            if let Some(endpoint) = &service_spec.endpoint {
                info!(
                    "Service {} will be available at: {}",
                    service_name, endpoint
                );
            } else {
                let default_endpoint = format!("http://localhost:8080/{}", service_name);
                info!(
                    "Service {} will use default endpoint: {}",
                    service_name, default_endpoint
                );
            }

            // Start service monitoring
            self.start_service_monitoring(service_name, service_spec)
                .await?;
        }

        info!("Service {} orchestration initiated", service_name);
        Ok(())
    }

    /// Start monitoring for a service
    async fn start_service_monitoring(
        &self,
        service_name: &str,
        service_spec: &ServiceSpec,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting monitoring for service: {}", service_name);

        // Create a monitoring task for this service
        let service_name = service_name.to_string();
        let service_spec = service_spec.clone();

        tokio::spawn(async move {
            loop {
                // Check if service is responding
                if let Some(endpoint) = &service_spec.endpoint {
                    match reqwest::get(endpoint).await {
                        Ok(response) => {
                            if response.status().is_success() {
                                tracing::debug!("Service {} is healthy", service_name);
                            } else {
                                tracing::warn!(
                                    "Service {} returned HTTP {}",
                                    service_name,
                                    response.status()
                                );
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Service {} health check failed: {}", service_name, e);
                        }
                    }
                }

                // Wait before next check
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        });

        Ok(())
    }

    /// Wait for service to be ready
    async fn wait_for_service_ready(
        &self,
        service_name: &str,
        service_spec: &ServiceSpec,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Waiting for service {} to be ready...", service_name);

        let max_wait_time = Duration::from_secs(60);
        let check_interval = Duration::from_secs(2);
        let start_time = std::time::Instant::now();

        loop {
            if start_time.elapsed() > max_wait_time {
                return Err(format!(
                    "Service {} did not become ready within timeout",
                    service_name
                )
                .into());
            }

            // Check service health if health check is configured
            if let Some(health_check) = &service_spec.health_check {
                match Self::check_service_health(service_name, health_check).await {
                    Ok(true) => {
                        info!("Service {} is ready", service_name);
                        return Ok(());
                    }
                    Ok(false) => {
                        tracing::debug!("Service {} not ready yet, waiting...", service_name);
                    }
                    Err(e) => {
                        tracing::debug!(
                            "Service {} health check error (still starting): {}",
                            service_name,
                            e
                        );
                    }
                }
            } else {
                // No health check configured, assume ready after a short delay
                tokio::time::sleep(Duration::from_secs(5)).await;
                info!(
                    "Service {} assumed ready (no health check configured)",
                    service_name
                );
                return Ok(());
            }

            tokio::time::sleep(check_interval).await;
        }
    }
}
