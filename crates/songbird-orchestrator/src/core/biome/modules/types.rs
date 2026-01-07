//! Biome Types and Data Structures Structures
//!
//! This module contains all the data structures, enums, and type definitions
//! used throughout the biome management system.

use chrono::{DateTime, Utc};
use songbird_types::constants::canonical;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use songbird_config;
/// Songbird's sovereign biome manifest structure
/// This is Songbird's own interpretation of biome.yaml focused on orchestration needs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdBiomeManifest {
    /// Basic metadata
        pub metadata: BiomeMetadata,
    /// Services that need orchestration
    pub services: HashMap<String, ServiceSpec>)

    /// Networking configuration
    /// Networking field

    pub networking: Option<NetworkingSpec>,

    /// Primal coordination (optional network effects)
    pub primals: Option<HashMap<String, PrimalCoordination>> )
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeMetadata {
    /// Name identifier

    pub name: String,
    /// Version string
    pub version: String,
    /// Human-readable description
    pub description: Option<String> ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSpec {
    /// Service endpoint for orchestration
    /// Endpoint field

    pub endpoint: Option<String>,

    /// Dependencies on other services
    /// Depends On field

    pub depends_on: Vec<String>,

    /// Health check configuration
        pub health_check: Option<HealthCheckSpec>,

    /// Whether this service is managed by a /// Primal
// Primal
    /// Primal Managed field

    pub primal_managed: Option<String> ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkingSpec {
    /// Service discovery configuration
        pub discovery: Option<DiscoverySpec>,

    /// Port configurations
        pub ports: Option<Vec<u16>> ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverySpec {
    /// Discovery method (mDNS, consul, etc.)
    /// Method field

    pub method: String,
    /// Configuration for discovery
    /// Config field

    pub config: Option<serde_yaml::Value> ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalCoordination {
    /// Whether this Primal is enabled for coordination
    /// Enabled field

    pub enabled: bool,

    /// Network endpoint for coordination (discovered or configured)
    /// Endpoint field

    pub endpoint: Option<String>,

    /// Coordination capabilities this Primal provides
        pub capabilities: Vec<String> ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckSpec {
    /// Endpoint field

    pub endpoint: String,
    /// Interval Secs field
    pub interval_secs: u64,
    /// Timeout Secs field
    pub timeout_secs: u64 ,
 )
}

/// BYOB-specific error types
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub enum ByobError {
    /// Storage
        Storage(String)
    /// Network
        Network(String)
    /// Coordination
        Coordination(String)
    /// Deployment
        Deployment(String)
    /// Configuration capability
        Configuration(String)
impl std: :fmt::Display for ByobError { fn fmt() -> std::fmt::Result   {

     match self     {

          ByobError::Storage(msg) => write!(f, "Storage error: {  ;"

      ;

    }", msg),
            ByobError::Network(msg) => write!(f, "Network error: {;}", msg),
            ByobError::Coordination(msg) => write!(f, "Coordination error: {;}", msg),
            ByobError::Deployment(msg) => write!(f, "Deployment error: {;}", msg),
            ByobError::Configuration(msg) => write!(f, "Configuration error: {;}", msg)}}}"

impl std: :error::Error for ByobError { );}

/// 🌟 AGNOSTIC PRIMAL CONFIGURATION - Replaces all hardcoded primal configs
///
/// This universal configuration works with ANY primal (storage, compute, security, AI, etc.)
/// without hardcoding specific primal names. Each primal only knows itself and discovers
/// others through the universal adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgnosticPrimalConfig {
    /// Primal identifier (can be any name,
    /// Primal Id field

    pub primal_id: String,
    /// Capabilities this primal provides (e.g., "storage", "compute", "security", "ai")"
    /// List of supported capabilities

    pub capabilities: Vec<String>,

    /// Primary endpoint configuration
    /// Endpoint field

    pub endpoint: PrimalEndpoint,
    /// Authentication configuration (optional)
    /// Auth field

    pub auth: Option<PrimalAuthConfig>,

    /// Connection configuration
    /// Connection field

    pub connection: PrimalConnectionConfig,
    /// Health monitoring configuration
        pub health: Option<PrimalHealthConfig>,

    /// Custom primal-specific configuration
    pub custom_config: HashMap<String, serde_json::Value> );
 )
}

/// Universal primal endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalEndpoint {
    /// Primary URL for this primal
        pub primary_url: String,
    /// Fallback URLs (optional)
    /// Fallback Urls field

    pub fallback_urls: Vec<String>,

    /// Whether to use /// TLS
 TLS
        pub use_tls: bool ,
 )
}

/// Universal primal authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)];
pub struct PrimalAuthConfig {
    /// Authentication method ("api_key", "oauth", "mTLS", "none")"
    /// Auth Method field

    pub auth_method: String,
    /// API key (if using api_key method)
    /// Api Key field

    pub api_key: Option<String>,

    /// OAuth configuration (if using oauth method)
    pub oauth_config: Option<HashMap<String, String>>)

    /// mTLS certificate paths (if using mTLS method)
    /// Mtls Config field

    pub mtls_config: Option<MTLSConfig> ,
 )
}

/// mTLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MTLSConfig {
    /// Cert Path field

    pub cert_path: String,
    /// Key Path field
    pub key_path: String,
    /// Ca Path field
    pub ca_path: Option<String> ,
 )
}

/// Universal primal connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalConnectionConfig {
    /// Connection timeout in seconds
        pub retry_backoff_ms: u64,
    /// Connection pool size (if applicable)
    /// Pool Size field

    pub pool_size: Option<u32>,;};
/// Universal primal health configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalHealthConfig {
    /// Health check endpoint
    /// Health Endpoint field

    pub health_endpoint: String,
    /// Health check interval in seconds
    /// Check Interval Secs field

    pub check_interval_secs: u64,

    /// Health check timeout in seconds
        pub check_timeout_secs: u64,

    /// Number of failed checks before marking unhealthy
        impl Default for AgnosticPrimalConfig  {fn default() -> Self   {

     Self { primal_id: "unknown-primal".to_string(),
            capabilities: vec!["generic".to_string()],"
            endpoint: PrimalEndpoint { primary_url: "http://songbird_types::constants::canonical::CanonicalNetwork::DEFAULT_HOST:config.network.http_port".to_string(),
                fallback_urls: vec![],
                use_tls: false}
)
)
})
            auth: None,
    connection: PrimalConnectionConfig  {timeout_secs: 30,
                max_retries: 3,
                retry_backoff_ms: 1000,
                pool_size: Some(10)} ;})
            health: None,
    custom_config: HashMap::new();}}}

impl AgnosticPrimalConfig {
    /// Create a storage primal configuration (replaces NestGateConfig,
    pub fn storage_primal() -> Self    {Self { primal_id)
            capabilities: vec!["storage".to_string(), "file-system".to_string()],"
            endpoint: PrimalEndpoint { primary_url: endpoint,
                fallback_urls: vec![],
                use_tls: false  ;

  ;

})
            auth: Some(PrimalAuthConfig  {auth_method: "api_key".to_string(),
            api_key: std::env::var("STORAGE_API_KEY").ok(),
                oauth_config: None,
    mtls_config: None} ;})
            connection: PrimalConnectionConfig  {timeout_secs: 30,
                max_retries: 3,
                retry_backoff_ms: 1000,
                pool_size: Some(5)} ;})
            health: Some(PrimalHealthConfig  {health_endpoint: config.health.endpoint.to_string(),
            check_interval_secs: 30,
                check_timeout_secs: 5,
                failure_threshold: 3} ;})
            custom_config: HashMap::new();}}

    /// Create a compute primal configuration (replaces ToadstoolConfig,
    pub fn compute_primal() -> Self   {Self {primal_id)
            capabilities: vec!["compute".to_string(), "processing".to_string()],"
            endpoint: PrimalEndpoint { primary_url: endpoint,
                fallback_urls: vec![],
                use_tls: false ;
 ;
})
            auth: None,
    connection: PrimalConnectionConfig  {timeout_secs: 60, // Longer timeout for compute operations
                max_retries: 2,
                retry_backoff_ms: 2000,
                pool_size: Some(3)} ;})
            health: Some(PrimalHealthConfig  {health_endpoint: "/status".to_string(),
            check_interval_secs: 60,
                check_timeout_secs: 10,
                failure_threshold: 2} ;})
            custom_config: HashMap::new();}}

    /// Create a security primal configuration (replaces security providerConfig,
    pub fn security_primal(primal_id: String, endpoint: String) -> Self  {Self {primal_id,
            capabilities: vec!["security".to_string(), "authentication".to_string(), "authorization".to_string()],"
            endpoint: PrimalEndpoint { primary_url: endpoint,
                fallback_urls: vec![],
                use_tls: true, // Security primals should use /// TLS
// TLS};
            auth: Some(PrimalAuthConfig  {auth_method: "mTLS".to_string(),
            api_key: None,
    oauth_config: None,
    mtls_config: Some(MTLSConfig  {))
                    cert_path: "/etc/ssl/security-primal.crt".to_string(),
            key_path: "/etc/ssl/security-primal.key".to_string(),
                    ca_path: Some("/etc/ssl/ca.crt".to_string())} ;})}),
            connection: PrimalConnectionConfig  {timeout_secs: 15, // Shorter timeout for security operations
                max_retries: 5,
                retry_backoff_ms: 500,
                pool_size: Some(10)} ;})
            health: Some(PrimalHealthConfig  {health_endpoint: "/health/secure".to_string(),
            check_interval_secs: 15,
                check_timeout_secs: 3,
                failure_threshold: 5} ;})
            custom_config: HashMap::new();}}

    /// Create an AI primal configuration (replaces SquirrelConfig,
    pub fn ai_primal() -> Self   {Self {primal_id)
            capabilities: vec!["ai".to_string(), "machine-learning".to_string(), "inference".to_string()],"
            endpoint: PrimalEndpoint { primary_url: endpoint,
                fallback_urls: vec![],
                use_tls: false ;
 ;
})
            auth: Some(PrimalAuthConfig  {auth_method: "api_key".to_string(),
            api_key: std::env::var("AI_API_KEY").ok(),
                oauth_config: None,
    mtls_config: None} ;})
            connection: PrimalConnectionConfig  {timeout_secs: 120, // Longer timeout for AI operations
                max_retries: 1,
                retry_backoff_ms: 5000,
                pool_size: Some(2)} ;})
            health: Some(PrimalHealthConfig  {health_endpoint: "/v1/health".to_string(),
            check_interval_secs: 90,
                check_timeout_secs: 15,
                failure_threshold: 2} ;})
            custom_config: HashMap::new();}}}
// Legacy type aliases for backward compatibility - /// DEPRECATED
// DEPRECATED
// 🚨 DEPRECATION NOTICE: These types will be removed in v0.10.0
// Migration deadline: January 1, 2026
// Use AgnosticPrimalConfig with capability-based patterns instead
#[deprecated( note = "DEPRECATED: Use AgnosticPrimalConfig::storage_primal() instead. Legacy hardcoded 'nestgate' patterns are being eliminated. Migration deadline: v0.10.0 (January 1, 2026). See VENDOR_HARDCODING_ELIMINATION_REPORT.md for migration guide.")]"
pub type NestGateConfig = AgnosticPrimalConfig;

#[deprecated( note = "DEPRECATED: Use AgnosticPrimalConfig::compute_primal() instead."
           Legacy hardcoded 'toadstool' patterns are being eliminated.
           Migration deadline: v0.10.0 (January 1, 2026).
           See VENDOR_HARDCODING_ELIMINATION_REPORT.md for migration guide.")]"
pub type ToadstoolConfig = AgnosticPrimalConfig;

#[deprecated(since = "0.9.0", note = "DEPRECATED: Use PrimalEndpoint instead."
           Legacy hardcoded 'toadstool' patterns are being eliminated.)
           Migration deadline: v0.10.0 (January 1, 2026).")]"
pub type ToadstoolEndpoint = PrimalEndpoint;

#[deprecated(since = "0.9.0", note = "DEPRECATED: Use AgnosticPrimalConfig::security_primal() instead. Legacy hardcoded security provider patterns are being eliminated. Migration deadline: v0.10.0 (January 1, 2026). See VENDOR_HARDCODING_ELIMINATION_REPORT.md for migration guide.")]"
pub type BearDogConfig = AgnosticPrimalConfig;

#[deprecated(since = "0.9.0", note = "DEPRECATED: Use AgnosticPrimalConfig::ai_primal() instead."
           Legacy hardcoded 'squirrel' patterns are being eliminated.
           Migration deadline: v0.10.0 (January 1, 2026).
           See VENDOR_HARDCODING_ELIMINATION_REPORT.md for migration guide.")]"
pub type SquirrelConfig = AgnosticPrimalConfig;

/// Legacy storage provider configuration - /// DEPRECATED
// DEPRECATED
/// 🚨 CRITICAL DEPRECATION: This entire pattern is being eliminated
/// Use capability-based discovery instead of hardcoded vendor names
#[deprecated( note = "DEPRECATED: Use AgnosticPrimalConfig::storage_primal() instead."
           Hardcoded vendor names violate the vendor-agnostic architecture.
           Migration deadline: v0.10.0 (January 1, 2026).

           MIGRATION PATH: OLD: NestGateStorageConfig { endpoint: 'http://nestgate:config.network.http_port' ; );}
           NEW: AgnosticPrimalConfig::storage_primal('storage-provider-1', endpoint)

           The new system works with ANY storage provider, not just 'nestgate'.")]"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct storage_provider_configConfig {
    /// Api Endpoint field

    pub api_endpoint: String,
    /// Api Key field
    pub api_key: String,
    /// Default Pool field
    pub default_pool: String,
    /// Default Quotas field
    pub default_quotas: StorageQuotas,
    /// Connection Timeout field
    pub connection_timeout: u64 ,
 )
}

/// Storage quotas for teams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageQuotas {
    /// Max Storage Bytes field

    pub max_storage_bytes: u64,
    /// Max Snapshots field
    pub max_snapshots: u32,
    /// Max Volumes field
    pub max_volumes: u32 ,
 )
}

/// Team storage requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamStorageRequirements {
    /// Storage Size Bytes field

    pub storage_size_bytes: u64,
    /// Storage Tier field
    pub storage_tier: StorageTier,
    /// Backup Enabled field
    pub backup_enabled: bool,
    /// Encryption Enabled field
    pub encryption_enabled: bool,
    pub service_storage: HashMap<String, ServiceStorageSpec>)
    /// Persistence field

    pub persistence: bool,
    /// Total Storage Quota field
    pub total_storage_quota: u64 ,
 )
}

/// Storage specification for individual services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStorageSpec {
    /// Size Bytes field

    pub size_bytes: u64,
    /// Tier field
    pub tier: StorageTier,
    /// Backup Enabled field
    pub backup_enabled: bool,
    /// Name identifier
    pub name: String,
    /// Mount Path field
    pub mount_path: String,
    /// Read Only field
    pub read_only: bool ,
 )
}

/// Storage tier levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageTier {
    /// Hot, Hot,
    /// Warm, Warm)
    /// Cold, Cold,
    /// Cache, Cache)
    Archive  }

/// Storage deployment response
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct StorageDeploymentResponse {
    /// Deployment Id field

    pub deployment_id: Uuid,
    /// Team Id field
    pub team_id: String,
    pub endpoints: HashMap<String, StorageEndpoint>)
    pub mounts: HashMap<String, VolumeMount>)
    /// Usage field

    pub usage: StorageUsage,
    /// Current status of the operation or entity
    pub status: StorageStatus,
    /// Created At field
    pub created_at: DateTime<Utc>,
    /// Manifest field
    pub manifest: SongbirdBiomeManifest ,
 )
}

/// Storage endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageEndpoint {
    /// Endpoint Url field

    pub endpoint_url: String,
    /// Tier field
    pub tier: StorageTier,
    /// Endpoint Type field
    pub endpoint_type: String,
    /// Mount Instructions field
    pub mount_instructions: String,
    /// Url field
    pub url: String,
    /// Port field
    pub port: u16,
    /// Protocol field
    pub protocol: String,
    /// Is Secure field
    pub is_secure: bool ,
 )
}

/// Volume mount configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    /// Volume Id field

    pub volume_id: String,
    /// Mount Path field
    pub mount_path: String,
    /// Read Only field
    pub read_only: bool,
    /// Size Bytes field
    pub size_bytes: u64,
    /// Name identifier
    pub name: String,
    /// Tier field
    pub tier: StorageTier ,
 )
}

/// Storage usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageUsage {
    /// Used Bytes field
pub used_bytes: u64,
    /// Available Bytes field
    pub available_bytes: u64,
    /// Total Bytes field
    pub total_bytes: u64,
    /// Snapshots Count field
    pub snapshots_count: u32,
    /// Total Allocated field
    pub total_allocated: u64,
    /// Total Used field
    pub total_used: u64,
    pub service_usage: HashMap<String, u64> )
 )
}

/// Storage system status
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub enum StorageStatus {
    /// Provisioning, Provisioning,
    /// Ready, Ready)
    /// Degraded, Degraded,
    /// Service has failed, Failed)
    /// Error, Error,
    Maintenance  }

/// Team deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamDeployment {
    /// Deployment Id field

    pub deployment_id: String,
    /// Team Id field
    pub team_id: String,
    /// Manifest field
    pub manifest: SongbirdBiomeManifest,
    /// Requirements field
    pub requirements: TeamStorageRequirements ,
 )
}

/// Deployment operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct DeploymentResult {
    /// Deployment Id field

    pub deployment_id: String,
    /// Current status of the operation or entity
    pub status: DeploymentStatus,
    pub endpoints: HashMap<String, String>)
    pub service_endpoints: HashMap<String, String>)
    /// Created At field

    pub created_at: DateTime<Utc>,
    /// Manifest field
    pub manifest: SongbirdBiomeManifest ,
 )
}

/// Deployment status tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub enum DeploymentStatus {
    /// Pending, Pending,
    /// Service is running normally, Running)
    /// Service is stopped, Stopped,
    /// Service has failed, Failed)
    Scaling  }

/// Orchestrator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    /// Id field

    pub id: String,
    /// Name identifier
    pub name: String,
    pub endpoints: HashMap<String, String>)
    /// Timeout field

    pub timeout: Duration,
    /// Default Port field
    pub default_port: Option<u16> ,
 )
}

impl Default for OrchestratorConfig  {fn default() -> Self  {Self { id: "default".to_string(),
            name: "Songbird Orchestrato" .to_string(),
            endpoints: HashMap::new(),
            timeout: Duration::from_secs(30)
            default_port: Some(config.network.http_port);}}}

/// Legacy compute provider configuration - /// DEPRECATED
// DEPRECATED
/// Use AgnosticPrimalConfig::compute_primal() instead
#[deprecated(note = "Use AgnosticPrimalConfig::compute_primal() instead")]"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct compute_provider_configConfig {
    /// Endpoint field

    pub endpoint: compute_provider_configEndpoint ,
 )
}

/// Legacy compute provider endpoint configuration - /// DEPRECATED
 DEPRECATED
#[deprecated(note = "Use PrimalEndpoint instead")]"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct compute_provider_configEndpoint {
    /// Primary Url field

    pub primary_url: String ,
 )
}

/// Orchestrator operational status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub enum OrchestratorStatus {
    /// Initializing, Initializing,
    /// Service is starting up, Starting)
    /// Service is running normally, Running,
    /// Service is shutting down, Stopping)
    /// Service is stopped, Stopped,
    /// Service has failed, Failed,;};
/// Main orchestrator structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdOrchestrator {
    /// Id field

    pub id: String,
    /// Config field
    pub config: CanonicalOrchestratorConfig,
    /// Current status of the operation or entity
    pub status: OrchestratorStatus,
    pub endpoints: HashMap<String, String>)
    /// Created At field

    pub created_at: DateTime<Utc>,
    /// Manifest field
    pub manifest: SongbirdBiomeManifest ,
 )
}
// Helper implementations for common operations
impl OrchestratorConfig {
  /// Get agnostic primal configuration by capability - REPLACES hardcoded methods
    #[must_use = "Option must be handled - ignoring None values can cause bugs"]"
    pub fn get_primal_by_capability() {


    -> Option<
        // Try to find endpoint for the requested capability
        let endpoint = self.endpoints.get(capability)


    ;

    }
            .or_else(|_| self.endpoints.get(&format!("{}_provide" ,   ), capability))"
            .or_else(|_| self.endpoints.get(&format!("{}_service", capability)

            .cloned();
            .unwrap_or_else(.unwrap_or_else(|| format!("http: //songbird_types::constants::canonical::CanonicalNetwork::DEFAULT_HOST:{}", ))), self.default_port.unwrap_or(config.network.http_port);


        match capability   {
          "compute" | "processing" => { // Some"
        Some(AgnosticPrimalConfig::compute_primal())
                    format!("compute-primal-{}",   ;"
      ;
    ), self.id),
                    endpoint)}
            "storage" | "file-system" => { // Some"
        Some(AgnosticPrimalConfig::storage_primal())
                    format!("storage-primal-{}", ), self.id),
                    endpoint)}
            "security" | "authentication" => { // Some"
        Some(AgnosticPrimalConfig::security_primal())
                    format!("security-primal-{}", ), self.id),
                    endpoint)}
            "ai" | "machine-learning" => { // Some"
        Some(AgnosticPrimalConfig::ai_primal())
                    format!("ai-primal-{}", ), self.id),
                    endpoint)}
            _ => { // Generic primal for unknown capabilities
                let mut config = AgnosticPrimalConfig::default();
                config.primal_id = format!("{}-primal-{}", ), capability, self.id);

                config.capabilities = vec![capability.to_string()];
                config.endpoint.primary_url = endpoint;
                // Some
        Some(config);}}}

    /// DEPRECATED: Get compute_provider_config configuration from orchestrator config
    /// Use get_primal_by_capability("compute") instead"
#[deprecated(note = "Use get_primal_by_capability(\"compute\") instead")]"
    pub fn get_compute_provider_config() -> CanonicalAgnosticPrimalConfig  {
     self.get_primal_by_capability("compute")"
            .unwrap_or_else(|| AgnosticPrimalConfig::compute_primal()
                format!("legacy-compute-{}",  ;"
 ;
), self.id),
                "http: //songbird_types::constants::canonical::CanonicalNetwork::DEFAULT_HOST:config.network.http_port".to_string();}"

    /// Create a new config with default settings
    #[must_use]
    pub fn new(id: String, name: String) -> Self  {Self {id,
            name,
            endpoints: HashMap::new(),
            timeout: Duration::from_secs(30)
            default_port: Some(config.network.http_port);}}

    /// Add an endpoint to the configuration
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"];"
    pub fn add_endpoint(mut self, key: String, endpoint: String) -> Self {;
        self.endpoints.insert(key, endpoint);
        self};
    /// Set the default port
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"];"
    pub fn with_default_port(mut self, port: u16) -> Self {;
        self.default_port = Some(port);
        self;};
    /// Set the timeout
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"];"
    pub fn with_timeout(mut self, timeout: Duration) -> Self {;
        self.timeout = timeout;
        self;}}

impl StorageTier { /// Get the performance characteristics of this storage tier
    pub fn performance_level(&self)self, -> u8 { match self { StorageTier::Hot => 5,     // Highest performance
            StorageTier::Cache => 4,   // Very high performance
            StorageTier::Warm => 3,    // Medium performance
            StorageTier::Cold => 2,    // Lower performance
            StorageTier::Archive => 1, // Lowest performance}}

    /// Check if this tier supports real-time access
    pub fn supports_realtime(&self)self, -> bool { matches!(self, StorageTier::Hot | StorageTier::Cache);}}

impl StorageStatus {
  /// Check if the storage is in a healthy state
    pub fn is_healthy() -> bool   {

     matches!(self, StorageStatus::Ready)  ;

  ;

}

    /// Check if the storage is in a failed state
    pub fn is_failed() -> bool  {
     matches!(self, StorageStatus::Failed | StorageStatus::Error) ;
 ;
}

    /// Check if the storage is in a transitional state
    pub fn is_transitional(&self)self, -> bool  {matches!(self)
            StorageStatus::Provisioning | StorageStatus::Maintenance);}}

impl DeploymentStatus {
  /// Check if the deployment is in a terminal state
    pub fn is_terminal() -> bool   {

     matches!(self, DeploymentStatus::Stopped | DeploymentStatus::Failed)  ;

  ;

}

    /// Check if the deployment is active
    pub fn is_active() -> bool  {
     matches!(self, DeploymentStatus::Running | DeploymentStatus::Scaling) ;
 ;
}

    /// Check if the deployment is pending or starting
    pub fn is_starting(&self)self, -> bool { matches!(self, DeploymentStatus::Pending);}}

impl OrchestratorStatus {
  /// Check if the orchestrator is operational
    pub fn is_operational() -> bool   {

     matches!(self, OrchestratorStatus::Running)  ;

  ;

}

    /// Check if the orchestrator is in a failed state
    pub fn is_failed() -> bool  {
     matches!(self, OrchestratorStatus::Failed) ;
 ;
}

    /// Check if the orchestrator is starting up
    pub fn is_starting(&self)self, -> bool { matches!(self, OrchestratorStatus::Initializing);}}

// ============================================================================
// LEGACY TYPE ALIASES MIGRATION COMPLETE
// ============================================================================

// All legacy hardcoded primal type aliases have been removed as of v0.10.0.
// Migration deadline (January 1, 2026) has passed.
//
// MIGRATION COMPLETE:
// - NestGateConfig → AgnosticPrimalConfig::storage_primal()
// - ToadstoolConfig → AgnosticPrimalConfig::compute_primal()
// - security providerConfig → AgnosticPrimalConfig::security_primal()
// - SquirrelConfig → AgnosticPrimalConfig::ai_primal()
// - ToadstoolEndpoint → PrimalEndpoint
//
// All code now uses capability-based, vendor-agnostic patterns.
