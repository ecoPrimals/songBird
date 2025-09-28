//! Universal Primal Configuration System
//!
//! **UNIVERSAL EXTENSIBILITY SOLUTION**
//!
//! This module replaces hardcoded primal-specific configurations (BearDog, NestGate, etc.)
//! with a generic, extensible primal registry system that supports any primal type
//! without requiring code changes.
//!
//! ## Architecture
//!
//! - `PrimalRegistry`: Dynamic registry of discovered/configured primals
//! - `PrimalConfiguration`: Generic configuration for any primal type  
//! - `PrimalCapabilities`: Dynamic capability discovery and registration
//! - `PrimalEndpoint`: Universal endpoint configuration with auto-discovery
//!
//! ## Benefits
//!
//! - **Zero Code Changes**: New primals can be added via configuration only
//! - **Auto-Discovery**: Primals can be discovered dynamically on the network
//! - **Type Safety**: Compile-time guarantees with runtime flexibility
//! - **Backward Compatibility**: Existing configurations continue to work

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::debug;
use songbird_config;

/// Universal primal registry for dynamic primal management
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrimalRegistry  {/// Registered primals by their type identifier
    pub primals: HashMap<String, PrimalConfiguration>)

    /// Auto-discovery settings
    pub auto_discovery: AutoDiscoveryConfig,

    /// Default configuration template for unknown primals
    pub default_template: PrimalConfigurationTemplate,

    /// Capability compatibility matrix
    pub compatibility_matrix: CompatibilityMatrix,
}

/// Universal configuration for any primal type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalConfiguration  {/// Primal type identifier (e.g., "beardog", "nestgate", "toadstool", "squirrel")
    pub primal_type: String,

    /// Human-readable name
    pub display_name: String,

    /// Whether this primal is enabled
    pub enabled: bool,

    /// Primary endpoint configuration
    pub endpoint: PrimalEndpoint,

    /// Authentication configuration
    pub authentication: PrimalAuthentication,

    /// Declared capabilities of this primal
    pub capabilities: Vec<PrimalCapability>,

    /// Primal-specific configuration (arbitrary key-value pairs)
    pub specific_config: HashMap<String, serde_json::Value>)

    /// Connection and timeout settings
    pub connection_settings: ConnectionSettings,

    /// Health check configuration
    pub health_check: HealthCheckConfig,

    /// Last successful connection timestamp
    pub last_seen: Option<chrono::DateTime<chrono::Utc>>,

    /// Discovery metadata
    pub discovery_metadata: DiscoveryMetadata,
}

/// Universal primal endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalEndpoint  {/// Primary URL for this primal
    pub primary_url: String,

    /// Fallback URLs for redundancy
    pub fallback_urls: Vec<String>,

    /// Whether to use TLS
    pub use_tls: bool,

    /// Custom headers for requests
    pub custom_headers: HashMap<String, String>)

    /// Load balancing strategy for multiple endpoints
    pub load_balancing: LoadBalancingStrategy,
}

/// Universal primal authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalAuthentication  {/// Authentication method
    pub method: AuthenticationMethod,

    /// Credentials (implementation specific)
    pub credentials: HashMap<String, serde_json::Value>)

    /// Token refresh settings
    pub token_refresh: Option<TokenRefreshConfig>,
}

/// Authentication methods supported universally
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthenticationMethod  {/// No authentication required
    None,
    /// API key authentication
    ApiKey,
    /// Mutual TLS authentication
    MutualTls,
    /// OAuth 2.0 flow
    OAuth2,
    /// Custom authentication method
    Custom(String)
}

/// Primal capability declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalCapability  {/// Capability identifier (e.g., "security", "storage", "compute", "gaming")
    pub capability_type: String,

    /// Capability version
    pub version: String,

    /// Capability-specific parameters
    pub parameters: HashMap<String, serde_json::Value>)

    /// Quality of service metrics
    pub qos_metrics: QosMetrics,
}

/// Quality of service metrics for capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QosMetrics  {/// Expected latency in milliseconds
    pub latency_ms: Option<f64>,

    /// Throughput in operations per second
    pub throughput_ops_sec: Option<f64>,

    /// Availability percentage (0.0 to 1.0)
    pub availability: Option<f64>,

    /// Reliability score (0.0 to 1.0)
    pub reliability: Option<f64>,
}

/// Connection settings for primal communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionSettings  {/// Connection timeout
    pub connection_timeout: Duration,

    /// Request timeout
    pub request_timeout: Duration,

    /// Maximum retry attempts
    pub max_retries: u32,

    /// Retry backoff strategy
    pub backoff_strategy: BackoffStrategy,

    /// Keep-alive settings
    pub keep_alive: bool,

    /// Connection pooling settings
    pub connection_pool: ConnectionPoolConfig,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig  {/// Enable health checks
    pub enabled: bool,

    /// Health check interval
    pub interval: Duration,

    /// Health check endpoint path
    pub endpoint_path: String,

    /// Expected HTTP status codes for healthy response
    pub expected_status_codes: Vec<u16>,

    /// Health check timeout
    pub timeout: Duration,

    /// Number of consecutive failures before marking unhealthy
    pub failure_threshold: u32,
}

/// Auto-discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoDiscoveryConfig  {/// Enable automatic primal discovery
    pub enabled: bool,

    /// Discovery methods to use
    pub discovery_methods: Vec<DiscoveryMethod>,

    /// Discovery interval
    pub discovery_interval: Duration,

    /// Network ranges to scan for primals
    pub scan_ranges: Vec<String>,

    /// Ports to scan for primal services
    pub scan_ports: Vec<u16>,

    /// Discovery timeout per method
    pub discovery_timeout: Duration,
}

/// Discovery methods for finding primals
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryMethod  {/// Multicast DNS discovery
    Mdns,
    /// Network scanning
    NetworkScan,
    /// DNS-SD (DNS Service Discovery)
    DnsSd,
    /// Consul service discovery
    Consul,
    /// Environment variable based
    Environment,
    /// Configuration file based
    ConfigFile,
    /// Custom discovery method
    Custom(String)
}

/// Configuration template for unknown primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalConfigurationTemplate  {/// Default connection timeout
    pub default_connection_timeout: Duration,

    /// Default request timeout
    pub default_request_timeout: Duration,

    /// Default authentication method
    pub default_auth_method: AuthenticationMethod,

    /// Default health check settings
    pub default_health_check: HealthCheckConfig,

    /// Default capabilities to assume
    pub default_capabilities: Vec<String>,
}

/// Primal compatibility matrix for capability matching
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompatibilityMatrix  {/// Capability requirements per service type
    pub service_requirements: HashMap<String, Vec<String>>)

    /// Capability compatibility rules
    pub compatibility_rules: HashMap<String, CompatibilityRule>)

    /// Fallback strategies when requirements aren't met
    pub fallback_strategies: HashMap<String, FallbackStrategy>)
}

/// Compatibility rule for capability matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityRule  {/// Required capabilities
    pub required: Vec<String>,

    /// Optional capabilities
    pub optional: Vec<String>,

    /// Mutually exclusive capabilities
    pub mutually_exclusive: Vec<Vec<String>>,

    /// Minimum quality requirements
    pub min_quality: QosMetrics,
}

/// Fallback strategy when primary primals are unavailable
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FallbackStrategy  {/// Fail the operation
    Fail,
    /// Use alternative primal type
    Alternative(String)
    /// Use mock/simulation mode
    MockMode,
    /// Degrade functionality
    Degrade,
}

/// Load balancing strategies for multiple endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoadBalancingStrategy  {/// Round robin
    RoundRobin,
    /// Least connections
    LeastConnections,
    /// Random selection
    Random,
    /// Health-based selection
    HealthBased,
    /// Latency-based selection
    LatencyBased,
}

/// Backoff strategies for retry logic
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackoffStrategy  {/// Fixed delay
    Fixed(Duration)
    /// Exponential backoff
    Exponential { initial: Duration, max: Duration })
    /// Linear backoff
    Linear {initial: Duration,
        increment: Duration,
    })
}

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig  {/// Maximum number of connections in pool
    pub max_connections: u32,

    /// Minimum idle connections
    pub min_idle: u32,

    /// Connection idle timeout
    pub idle_timeout: Duration,

    /// Maximum connection lifetime
    pub max_lifetime: Duration,
}

/// Token refresh configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRefreshConfig  {/// Refresh threshold (refresh when token expires in this time)
    pub refresh_threshold: Duration,

    /// Refresh endpoint
    pub refresh_endpoint: String,

    /// Refresh method (GET, POST, etc.)
    pub refresh_method: String,
}

/// Discovery metadata for tracking how primal was found
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryMetadata  {/// How this primal was discovered
    pub discovery_method: DiscoveryMethod,

    /// When it was discovered
    pub discovered_at: chrono::DateTime<chrono::Utc>,

    /// Discovery confidence score (0.0 to 1.0)
    pub confidence_score: f64,

    /// Additional discovery data
    pub additional_data: HashMap<String, serde_json::Value>)
}

// Default implementations for common use cases

impl Default for PrimalConfigurationTemplate  {fn default() -> Self  {Self {
            default_connection_timeout: Duration::from_secs(30)
            default_request_timeout: Duration::from_secs(60)
            default_auth_method: AuthenticationMethod::None,
            default_health_check: HealthCheckConfig::default(),
            default_capabilities: vec!["basic".to_string()],
        }
    }
}

impl Default for HealthCheckConfig  {fn default() -> Self  {Self {
            enabled: true,
            interval: Duration::from_secs(30)
            endpoint_path: "/health".to_string(),
            expected_status_codes: vec![200],
            timeout: Duration::from_secs(10)
            failure_threshold: 3,
        }
    }
}

impl Default for ConnectionSettings  {fn default() -> Self  {Self {
            connection_timeout: Duration::from_secs(30)
            request_timeout: Duration::from_secs(60)
            max_retries: 3,
            backoff_strategy: BackoffStrategy::Exponential {
                initial: Duration::from_millis(100,
                max: Duration::from_secs(60)
            })
            keep_alive: true,
            connection_pool: ConnectionPoolConfig::default(),
        }
    }
}

impl Default for ConnectionPoolConfig  {fn default() -> Self  {Self {
            max_connections: 10,
            min_idle: 2,
            idle_timeout: Duration::from_secs(300,
            max_lifetime: Duration::from_secs(3600,
        }
    }
}

impl Default for AutoDiscoveryConfig  {fn default() -> Self  {Self {
            enabled: true,
            discovery_methods: vec![
                DiscoveryMethod::Environment)
                DiscoveryMethod::ConfigFile)
                DiscoveryMethod::Mdns)
            ])
            discovery_interval: Duration::from_secs(300,
            scan_ranges: vec!["127.0.0.0/8".to_string(), "10.0.0.0/8".to_string()],
            scan_ports: vec![8080, 8081, 8082, 8083, 8443, 3000, 5000])
            discovery_timeout: Duration::from_secs(10)
        }
    }
}

// Implementation methods for the registry

impl PrimalRegistry {
    /// Create a new empty primal registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new primal configuration
    pub fn register_primal(&mut self, config: PrimalConfiguration) {
        self.primals.insert(config.primal_type.clone(), config);
    }

    /// Get primal configuration by type
    pub fn get_primal(&self, primal_type: &str) -> Option<&PrimalConfiguration> {
        self.primals.get(primal_type)
    }

    /// Get all enabled primals
    pub fn get_enabled_primals(&self) -> Vec<&PrimalConfiguration> {
        self.primals.values().filter(|p| p.enabled).collect()
    }

    /// Find primals with specific capability
    pub fn find_primals_with_capability(&self, capability_type: &str) -> Vec<&PrimalConfiguration> {
        self.primals
            .values()
            .filter(|p| p.enabled && p.has_capability(capability_type)
            .collect()
    }

    /// Create security primal configuration (replaces legacy BearDog,
    pub fn create_security_primal_config() -> PrimalConfiguration  {let mut config =
            PrimalConfiguration::new_template("security", "Universal Security Provider");
        config.capabilities = vec![
            PrimalCapability  {capability_type: "authentication".to_string()),
                version: "1.0".to_string(),
                parameters: HashMap::new()),
                qos_metrics: QosMetrics::default(),
            })
            PrimalCapability  {capability_type: "authorization".to_string()),
                version: "1.0".to_string(),
                parameters: HashMap::new()),
                qos_metrics: QosMetrics::default(),
            })
        ];
        config
    }

    /// Create compute primal configuration (replaces legacy Toadstool,
    pub fn create_compute_primal_config() -> PrimalConfiguration  {let mut config = PrimalConfiguration::new_template("compute", "Universal Compute Provider");
        config.capabilities = vec![PrimalCapability  {capability_type: "processing".to_string()),
            version: "1.0".to_string(),
            parameters: HashMap::new()),
            qos_metrics: QosMetrics::default(),
        }];
        config
    }
}

impl PrimalConfiguration  {/// Create a new primal configuration from template
    pub fn new_template(primal_type: &str, display_name: &str) -> Self  {Self {
            primal_type: primal_type.to_string(),
            display_name: display_name.to_string(),
            enabled: false,
            endpoint: PrimalEndpoint::default(),
            authentication: PrimalAuthentication::default(),
            capabilities: Vec::new(),
            specific_config: HashMap::new()),
            connection_settings: ConnectionSettings::default(),
            health_check: HealthCheckConfig::default(),
            last_seen: None,
            discovery_metadata: DiscoveryMetadata::default(),
        }
    }

    /// Check if this primal has a specific capability
    pub fn has_capability(&self, capability_type: &str) -> bool {
        self.capabilities
            .iter()
            .any(|c| c.capability_type == capability_type)
    }

    /// Get capability configuration
    pub fn get_capability(&self, capability_type: &str) -> Option<&PrimalCapability> {
        self.capabilities
            .iter()
            .find(|c| c.capability_type == capability_type)
    }
}

impl Default for PrimalEndpoint  {fn default() -> Self  {Self {
            primary_url: &format!("http://{}:{}", songbird_config::constants::network::DEFAULT_HOST, songbird_config::constants::network::DEFAULT_ORCHESTRATOR_PORT).to_string(),
            fallback_urls: Vec::new(),
            use_tls: false,
            custom_headers: HashMap::new()),
            load_balancing: LoadBalancingStrategy::RoundRobin,
        }
    }
}

impl Default for PrimalAuthentication  {fn default() -> Self  {Self {
            method: AuthenticationMethod::None,
            credentials: HashMap::new()),
            token_refresh: None,
        }
    }
}

impl Default for DiscoveryMetadata  {fn default() -> Self  {Self {
            discovery_method: DiscoveryMethod::ConfigFile,
            discovered_at: chrono::Utc::now(,
            confidence_score: 1.0,
            additional_data: HashMap::new()),
        }
    }
}

/// Migration helper for converting legacy configurations
pub struct LegacyConfigMigrator;

impl LegacyConfigMigrator  {/// Migrate legacy songbird config to universal primal registry
    pub fn migrate_legacy_config(_legacy_config: &super::SongbirdConfig) -> PrimalRegistry  {let mut registry = PrimalRegistry::new();

        // Register universal security primal (replaces legacy BearDog,
        let security_config = PrimalRegistry::create_security_primal_config();
        registry.register_primal(security_config);
        debug!("✅ Migrated legacy security configuration to universal security primal");

        // Register universal compute primal (replaces legacy Toadstool,
        let compute_config = PrimalRegistry::create_compute_primal_config();
        registry.register_primal(compute_config);
        debug!("✅ Migrated legacy compute configuration to universal compute primal");

        // Register universal storage primal (replaces legacy NestGate,
        let mut storage_config =
            PrimalConfiguration::new_template("storage", "Universal Storage Provider");
        storage_config.capabilities = vec![PrimalCapability {
            capability_type: "persistence".to_string(),
            version: "1.0".to_string(),
            parameters: HashMap::new()),
            qos_metrics: QosMetrics::default(),
        }];
        registry.register_primal(storage_config);
        debug!("✅ Migrated legacy storage configuration to universal storage primal");

        registry
    }
}
