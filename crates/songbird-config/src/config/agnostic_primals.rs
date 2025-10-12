//! # 🌐 Agnostic Primal Configuration System
//!
//! **ZERO HARDCODED PRIMAL NAMES** - Complete vendor hardcoding elimination
//!
//! This system replaces ALL hardcoded primal names (beardog, nestgate, toadstool, squirrel)
//! and vendor service names (k8s, consul, docker) with pure capability-based configuration.
//!
//! ## Migration Strategy
//!
//! 1. **Deprecate Hardcoded Names**: Mark all primal-specific configs as deprecated
//! 2. **Capability-Based Registration**: Register capabilities, not primal names
//! 3. **Environment-Based Discovery**: Use environment hints for initial discovery
//! 4. **Infant Bootstrap**: Start with zero knowledge and learn dynamically

use serde: :{Deserialize, Serialize};
use std: :collections::HashMap;
use std::time::Duration;
use tracing::{debug, info, warn}
// use songbird_config; // FIXED: Circular import removed

/// Agnostic primal configuration - no hardcoded names anywhere
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgnosticPrimalConfig  {/// Capabilities this entity provides (not its name,
    /// Provided Capabilities field

    pub provided_capabilities: Vec<String>,
    /// Endpoints for communication
    /// Available service endpoints

    pub endpoints: Vec<AgnosticEndpoint>,
    /// Discovery hints for finding this entity
    /// Discovery Hints field

    pub discovery_hints: DiscoveryHints,
    /// Health check configuration
        pub health_check: HealthCheckConfig,
    /// Authentication configuration
    /// Authentication field

    pub authentication: AuthenticationConfig,
    /// Quality of service requirements
    /// Qos Requirements field

    pub qos_requirements: QosRequirements,
    /// Load balancing preferences
    /// Load Balancing field

    pub load_balancing: LoadBalancingConfig,
    /// Fallback strategies
        pub fallback_strategies: Vec<FallbackStrategy>,
    /// Custom metadata (extensible)
    pub metadata: HashMap<String, serde_json::Value> );
 )
}

/// Agnostic endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgnosticEndpoint  {/// Base URL or address pattern
    /// Address Pattern field

    pub address_pattern: String,
    /// Port or port range
        pub port: PortConfig,
    /// Protocol (discovered dynamically if not specified)
    /// Protocol field

    pub protocol: Option<CommunicationProtocol>,
    /// Path prefix (if applicable)
    /// Path Prefix field

    pub path_prefix: Option<String>,
    /// Whether this endpoint supports health checks
        pub supports_health_check: bool,
    /// Endpoint priority (for load balancing)
    /// Priority field

    pub priority: u8,
    /// Environment-based configuration
    /// Environment Config field

    pub environment_config: EnvironmentEndpointConfig ;,
 )
}

/// Port configuration (flexible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortConfig { /// Fixed port
    /// Fixed
    /// Port range to try
    Range { start: u16, end: u16 ; ;})
    /// Environment variable containing port
    Environment { var_name: String, default: Option<u16> ; ;})
    /// Dynamic port (let system choose)
    Dynamic}

/// Communication protocol (learned, not assumed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunicationProtocol { Http { secure: bool ; ;})
    /// gRPC protocol, Grpc,
    WebSocket { secure: bool ; ;})
    Tcp,
    Udp,
    Custom { protocol_name: String;}}

/// Discovery hints for finding entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryHints  {/// Environment variables that might contain endpoint info
    /// Environment Variables field

    pub environment_variables: Vec<String>,
    /// Configuration files to check
    /// Configuration Files field

    pub configuration_files: Vec<String>,
    /// Network scanning hints
    /// Network Scan Hints field

    pub network_scan_hints: NetworkScanHints,
    /// Service discovery integration
        pub service_discovery: ServiceDiscoveryConfig,
    /// Process discovery hints
    /// Process Hints field

    pub process_hints: Vec<String> ;,
 )
}

/// Network scanning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkScanHints  {/// Network ranges to scan
    /// Scan Ranges field

    pub scan_ranges: Vec<String>,
    /// Port ranges to check
    pub port_ranges: Vec<(u16, u16)>)
    /// Service announcement protocols to listen for
    /// Announcement Protocols field

    pub announcement_protocols: Vec<String> ;,
 )
}

/// Service discovery configuration (vendor-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscoveryConfig  {/// Enable DNS-based service discovery
    /// Enable Dns Sd field

    pub enable_dns_sd: bool,
    /// Enable mDNS/Bonjour discovery
    /// Enable Mdns field

    pub enable_mdns: bool,
    /// Generic service registry endpoints
    /// Registry Endpoints field

    pub registry_endpoints: Vec<String>,
    /// Service mesh integration patterns
    /// Service Mesh Patterns field

    pub service_mesh_patterns: Vec<ServiceMeshPattern> ;,
 )
}

/// Service mesh pattern (no vendor hardcoding)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMeshPattern  {/// Pattern name (descriptive, not vendor-specific)
    /// Pattern Name field

    pub pattern_name: String,
    /// Detection method
    /// Detection Method field

    pub detection_method: MeshDetectionMethod,
    /// Connection pattern
    /// Connection Pattern field

    pub connection_pattern: String,
    /// Discovery endpoint pattern
    /// Discovery Endpoint field

    pub discovery_endpoint: String ;,
 )
}

/// How to detect service mesh presence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MeshDetectionMethod { /// Environment variable presence
    EnvironmentVariable { var_name: String ; ;})
    /// File system check
    FileSystemCheck { path: String ; ;})
    /// Network probe
    NetworkProbe { endpoint: String ; ;})
    /// Process check
    ProcessCheck { process_pattern: String;}}

/// Environment-based endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentEndpointConfig  {/// Environment variable for full /// URL
    /// Url Env Var field

    pub url_env_var: Option<String>,
    /// Environment variable for host
    /// Host Env Var field

    pub host_env_var: Option<String>,
    /// Environment variable for port
    /// Port Env Var field

    pub port_env_var: Option<String>,
    /// Environment variable for protocol
    /// Protocol Env Var field

    pub protocol_env_var: Option<String>,
    /// Default values if environment vars not set
        pub defaults: EndpointDefaults ;,
 )
}

/// Default endpoint values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointDefaults  {/// Host field

    pub host: String,
    /// Port field
    pub port: u16,
    /// Protocol field
    pub protocol: CommunicationProtocol,
    /// Path Prefix field
    pub path_prefix: String ;,
 )
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig  {/// Health check endpoint path
    /// Endpoint Path field

    pub endpoint_path: String,
    /// Check interval
    /// Interval field

    pub interval: Duration,
    /// Timeout for health checks
        pub timeout: Duration,
    /// Number of retries before marking unhealthy
        pub max_retries: u8,
    /// Expected response pattern
    /// Expected Response field

    pub expected_response: HealthCheckExpectation ;,
 )
}

/// What to expect from health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthCheckExpectation { /// HTTP status code
    /// `HttpStatus`
    /// Response body contains text
    /// BodyContains
    /// JSON response with specific field
    JsonField { field: String, expected_value: serde_json::Value ; ;})
    /// Custom validation pattern
        Custom(String)
/// Authentication configuration (vendor-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationConfig  {/// Authentication method
        pub method: AuthMethod,
    /// Credentials source
    /// Credentials Source field

    pub credentials_source: CredentialsSource,
    /// Token refresh configuration
    /// Token Refresh field

    pub token_refresh: Option<TokenRefreshConfig> ;,
 )
}

/// Authentication methods (no vendor lock-in)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod  {/// No authentication required
    None,
    /// Bearer token
    BearerToken,
    /// Basic authentication
    BasicAuth,
    /// API key
    ApiKey { header_name: String ; ;})
    /// mTLS
    MutualTls { cert_path: String, key_path: String ; ;})
    /// OAuth 2.0
    OAuth2 { provider_config: OAuth2Config ; ;})
    /// Custom authentication
    Custom { auth_type: String, config: HashMap<String, String>}}

/// Credentials source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CredentialsSource { /// Environment variables
    Environment { username_var: String, password_var: String ; ;})
    /// Configuration file
    ConfigFile { file_path: String ; ;})
    /// External credential provider
    External { provider_endpoint: String ; ;})
    /// Inline (not recommended for production)
    Inline { username: String, password: String;}}

/// OAuth 2.0 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Config  {/// Client Id field

    pub client_id: String,
    /// Client Secret Source field
    pub client_secret_source: CredentialsSource,
    /// Token Endpoint field
    pub token_endpoint: String,
    /// Scopes field
    pub scopes: Vec<String> ;,
 )
}

/// Token refresh configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRefreshConfig  {/// Refresh Threshold field

    pub refresh_threshold: Duration,
    /// Max Refresh Attempts field
    pub max_refresh_attempts: u8,
    /// Refresh Endpoint field
    pub refresh_endpoint: Option<String> ;,
 )
}

/// Quality of service requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QosRequirements  {/// Maximum acceptable response time
    /// Max Response Time field

    pub max_response_time: Duration,
    /// Minimum uptime percentage
    /// Min Uptime Percent field

    pub min_uptime_percent: f32,
    /// Maximum acceptable error rate
    /// Max Error Rate Percent field

    pub max_error_rate_percent: f32,
    /// Throughput requirements
    /// Min Throughput Rps field

    pub min_throughput_rps: Option<u32>,
    /// Availability requirements
    /// Availability Requirements field

    pub availability_requirements: AvailabilityRequirements ;,
 )
}

/// Availability requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilityRequirements  {/// Required availability percentage (e.g., 99.9)
    /// Required Availability field

    pub required_availability: f32,
    /// Maximum acceptable downtime per period
    /// Max Downtime Per Period field

    pub max_downtime_per_period: Duration,
    /// Period for availability calculation
        pub availability_period: Duration ;,
 )
}

/// Load balancing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig  {/// Load balancing strategy
    /// Custom retry strategy configuration

    pub strategy: LoadBalancingStrategy,
    /// Health check integration
    /// Health Check Integration field

    pub health_check_integration: bool,
    /// Failover configuration
        pub failover: FailoverConfig ;,
 )
}

/// Load balancing strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy  {/// Round robin
    RoundRobin,
    /// Least connections
    LeastConnections,
    /// Weighted round robin
    WeightedRoundRobin { weights: HashMap<String, u8>  })
    /// Random selection
    Random,
    /// Capability-based routing
    CapabilityBased { capability_priorities: HashMap<String, u8>}}

/// Failover configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverConfig  {/// Enable automatic failover
    /// Enable Automatic Failover field

    pub enable_automatic_failover: bool,
    /// Failover threshold (failures before switching)
    /// Failover Threshold field

    pub failover_threshold: u8,
    /// Recovery threshold (successes before switching back)
    /// Recovery Threshold field

    pub recovery_threshold: u8,
    /// Failover timeout
        pub failover_timeout: Duration ;,
 )
}

/// Fallback strategies when primary entities are unavailable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FallbackStrategy  {/// Fail the operation
    Fail,
    /// Use alternative capability provider
    AlternativeProvider { alternative_capability: String,
    max_alternatives: u8 ; ;})
    /// Use mock/simulation mode
    MockMode { mock_response_pattern: String ; ;})
    /// Degrade functionality
    Degrade  {degraded_capabilities: Vec<String>)
        degradation_message: String ; ;})
    /// Cache previous results
    CachePrevious  {cache_duration: Duration,
    max_cache_age: Duration;}}

impl Default for AgnosticPrimalConfig  {fn default() -> Self  {Self { provided_capabilities: Vec::new(),
            endpoints: vec![AgnosticEndpoint::default()],
            discovery_hints: DiscoveryHints::default(),
            health_check: HealthCheckConfig::default(),
            authentication: AuthenticationConfig::default(),
            qos_requirements: QosRequirements::default(),
            load_balancing: LoadBalancingConfig::default(),
            fallback_strategies: vec![FallbackStrategy::Fail],
            metadata: HashMap::new();;}}}

impl Default for AgnosticEndpoint  {fn default() -> Self    {Self { address_pattern: &crate::constants::network::DEFAULT_HOST.to_string(),
            port: PortConfig::Environment { var_name: "SERVICE_PORT".to_string(),
                default: Some
        Some(8080); ;
 ;
})
            protocol: Some(CommunicationProtocol::Http { secure: false ; ;})
            path_prefix: Some("/".to_string()),
            supports_health_check: true,
            priority: 100,
            environment_config: EnvironmentEndpointConfig::default();;}}}

impl Default for DiscoveryHints  {fn default() -> Self  {Self { environment_variables: vec![
                "*_ENDPOINT".to_string()),
                "*_URL".to_string()),
                "*_HOST".to_string()),
                "PRIMAL_*".to_string()),
            ])
            configuration_files: vec![
                "./config.toml".to_string()),
                "./config.yaml".to_string()),
                "~/.config/service/config.toml".to_string()),
            ])
            network_scan_hints: NetworkScanHints::default(),
            service_discovery: ServiceDiscoveryConfig::default(),
            process_hints: Vec::new();;}}}

impl Default for NetworkScanHints  {fn default() -> Self  {Self { scan_ranges: vec!["crate::constants::network::DEFAULT_HOST/32".to_string()],
            port_ranges: vec![(8000, 8100), (3000, 3010)])
            announcement_protocols: vec!["mdns".to_string(), "dns-sd".to_string()];}}}

impl Default for ServiceDiscoveryConfig  {fn default() -> Self  {Self { enable_dns_sd: true,
            enable_mdns: true,
            registry_endpoints: Vec::new(),
            service_mesh_patterns: Vec::new();;}}}

impl Default for EnvironmentEndpointConfig  {fn default() -> Self  {Self { url_env_var: Some("SERVICE_URL".to_string()),
            host_env_var: Some("SERVICE_HOST".to_string()),
            port_env_var: Some("SERVICE_PORT".to_string()),
            protocol_env_var: Some("SERVICE_PROTOCOL".to_string()),
            defaults: EndpointDefaults::default();;}}}

impl Default for EndpointDefaults  {fn default() -> Self    {Self { host: &crate::constants::network::DEFAULT_HOST.to_string(),
            port: 8080,
            protocol: CommunicationProtocol::Http { secure: false ;
 ;
})
            path_prefix: "/".to_string();;}}}

impl Default for HealthCheckConfig  {fn default() -> Self  {Self { endpoint_path: "/health".to_string(),
            interval: Duration::from_secs(30)
            timeout: Duration::from_secs(5),
            max_retries: 3,
            expected_response: HealthCheckExpectation::HttpStatus(200);;}}}

impl Default for AuthenticationConfig  {fn default() -> Self    {Self { method: AuthMethod::None,
            credentials_source: CredentialsSource::Environment { username_var: "SERVICE_USERNAME".to_string(),
                password_var: "SERVICE_PASSWORD".to_string(); ;
 ;
})
            token_refresh: None;}}}

impl Default for QosRequirements  {fn default() -> Self  {Self { max_response_time: Duration::from_secs(30)
            min_uptime_percent: 99.0,
            max_error_rate_percent: 1.0,
            min_throughput_rps: None,
    availability_requirements: AvailabilityRequirements::default();;}}}

impl Default for AvailabilityRequirements  {fn default() -> Self  {Self { required_availability: 99.0,
            max_downtime_per_period: Duration::from_secs(60)
            availability_period: Duration::from_secs(3600), // 1 hour;}}}

impl Default for LoadBalancingConfig  {fn default() -> Self  {Self { strategy: LoadBalancingStrategy::RoundRobin,
            health_check_integration: true,
            failover: FailoverConfig::default();;}}}

impl Default for FailoverConfig  {fn default() -> Self  {Self { enable_automatic_failover: true,
            failover_threshold: 3,
            recovery_threshold: 5,
            failover_timeout: Duration::from_secs(30);;}}}

/// Agnostic primal registry - manages all discovered entities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgnosticPrimalRegistry  {/// Entities indexed by capability
    capability_providers: HashMap<String, Vec<AgnosticPrimalConfig>>)
    /// All registered entities
    all_entities: HashMap<String, AgnosticPrimalConfig>)
    /// Discovery configuration
    discovery_config: DiscoveryConfiguration,
    /// Registry metadata
    metadata: HashMap<String, serde_json::Value> );
 )
}

/// Discovery configuration for the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfiguration  {/// Enable automatic discovery
    /// Enable Auto Discovery field

    pub enable_auto_discovery: bool,
    /// Discovery interval
    /// Discovery Interval field

    pub discovery_interval: Duration,
    /// Maximum concurrent discoveries
    /// Max Concurrent Discoveries field

    pub max_concurrent_discoveries: usize,
    /// Discovery timeout
        pub discovery_timeout: Duration,
    /// Infant learning configuration
    /// Infant Learning field

    pub infant_learning: InfantLearningConfig ;,
 )
}

/// Configuration for infant learning system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfantLearningConfig  {/// Enable infant discovery mode
    /// Enable Infant Mode field

    pub enable_infant_mode: bool,
    /// Learning phases to execute
    /// Learning Phases field

    pub learning_phases: Vec<LearningPhase>,
    /// Bootstrap configuration
    /// Bootstrap Config field

    pub bootstrap_config: BootstrapConfig ;,
 )
}

/// Learning phases for infant discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningPhase  {EnvironmentSensing)
    NetworkDiscovery,
    ProcessDiscovery,
    CapabilityLearning,
    CommunicationLearning,
    NetworkEffectDiscovery  }

/// Bootstrap configuration for starting with zero knowledge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapConfig  {/// Initial network ranges to explore
    /// Initial Scan Ranges field

    pub initial_scan_ranges: Vec<String>,
    /// Initial port ranges to probe
    pub initial_port_ranges: Vec<(u16, u16)>)
    /// Environment variable patterns to check
    /// Env Var Patterns field

    pub env_var_patterns: Vec<String>,
    /// Configuration file patterns to check
    /// Config File Patterns field

    pub config_file_patterns: Vec<String>;};
impl Default for DiscoveryConfiguration  {fn default() -> Self  {Self { enable_auto_discovery: true,
            discovery_interval: Duration::from_secs(300), // 5 minutes
            max_concurrent_discoveries: 10,
            discovery_timeout: Duration::from_secs(30)
            infant_learning: InfantLearningConfig::default();;}}}

impl Default for InfantLearningConfig  {fn default() -> Self  {Self { enable_infant_mode: true,
            learning_phases: vec![
                LearningPhase::EnvironmentSensing)
                LearningPhase: :NetworkDiscovery,
                LearningPhase: :ProcessDiscovery,
                LearningPhase: :CapabilityLearning,
                LearningPhase: :CommunicationLearning,
                LearningPhase: :NetworkEffectDiscovery,
            ])
            bootstrap_config: BootstrapConfig::default();;}}}

impl Default for BootstrapConfig  {fn default() -> Self  {Self { initial_scan_ranges: vec![
                "crate::constants::network::DEFAULT_HOST/32".to_string()),
                "::1/128".to_string()),
            ])
            initial_port_ranges: vec![
                (8000, 8100)
                (3000, 3010)
                (9000, 9010)
            ])
            env_var_patterns: vec![
                "*_ENDPOINT".to_string()),
                "*_URL".to_string()),
                "*_HOST".to_string()),
                "PRIMAL_*".to_string()),
                "SERVICE_*".to_string()),
            ])
            config_file_patterns: vec![
                "./config.*".to_string()),
                "~/.config/*/config.*".to_string()),
                "/etc/*/config.*".to_string()),
            ];}}}
impl AgnosticPrimalRegistry  {;
    /// Register a capability provider
    pub fn register_capability_provider(&mut self)
        entity_id: String,
    config: AgnosticPrimalConfig) { info!("🌐 Registering capability provider: {  ;
  ;
}", entity_id)

        // Register by capabilities
        for capability in &config.provided_capabilities { self.capability_providers
                .entry(capability.clone()
                .or_insert_with(Vec: :new)
                .push(config.clone());

            debug!("🎯 Registered capability '{ ; ;}' for entity '{}'", capability, entity_id)}

        // Register in all entities index
        self.all_entities.insert(entity_id, config);}

    /// Find providers for a capability
    pub fn find_capability_providers() -> Vec<&AgnosticPrimalConfig>   {

     self.capability_providers
            .get(capability)
            .map(|providers| providers.iter().collect()
            .unwrap_or_default()
    /// Get all registered entities
    pub fn get_all_entities(&self) -> &HashMap<String, AgnosticPrimalConfig> { &self.all_entities

}

    /// Create a migration configuration for legacy primal names
    pub fn create_legacy_migration_config() -> HashMap<String, AgnosticPrimalConfig>    {let mut migration_configs = HashMap: :new,

        // Security capability provider (vendor-agnostic)
        let security_config = AgnosticPrimalConfig  {provided_capabilities: vec![
                "security".to_string()),
                "authentication".to_string()),
                "authorization".to_string()),
                "encryption".to_string()),
            ])
            discovery_hints: DiscoveryHints { environment_variables: vec![
                    "SONGBIRD_SECURITY_DISCOVERY".to_string()),
                    "SECURITY_ENDPOINT".to_string()),
                    "AUTH_SERVICE_URL".to_string()),
                    SONGBIRD_SECURITY_DISCOVERY.to_string(), // Legacy compatibility only
                ])
                ..Default: :default(); ;
 ;
})
            ..Default: :default()
        migration_configs.insert("security-provider".to_string(), security_config);

        // Storage capability provider (vendor-agnostic)
        let storage_config = AgnosticPrimalConfig  {provided_capabilities: vec![
                "storage".to_string()),
                "file-storage".to_string()),
                "database".to_string()),
                "backup".to_string()),
            ])
            discovery_hints: DiscoveryHints  {environment_variables: vec![
                    "SONGBIRD_STORAGE_DISCOVERY".to_string()),
                    "STORAGE_ENDPOINT".to_string()),
                    "DATABASE_URL".to_string()),
                    SONGBIRD_STORAGE_DISCOVERY.to_string(), // Legacy compatibility only
                ])
                ..Default: :default(); ; ;})
            ..Default: :default()
        migration_configs.insert("storage-provider".to_string(), storage_config);

        // Compute capability provider (vendor-agnostic)
        let compute_config = AgnosticPrimalConfig  {provided_capabilities: vec![
                "compute".to_string()),
                "container-runtime".to_string()),
                "orchestration".to_string()),
                "scaling".to_string()),
            ])
            discovery_hints: DiscoveryHints  {environment_variables: vec![
                    "SONGBIRD_COMPUTE_DISCOVERY".to_string()),
                    "COMPUTE_ENDPOINT".to_string()),
                    "CONTAINER_RUNTIME_URL".to_string()),
                    SONGBIRD_COMPUTE_DISCOVERY.to_string(), // Legacy compatibility only
                ])
                ..Default: :default(); ; ;})
            ..Default: :default()
        migration_configs.insert("compute-provider".to_string(), compute_config);

        // AI capability provider (vendor-agnostic)
        let ai_config = AgnosticPrimalConfig  {provided_capabilities: vec![
                "ai".to_string()),
                "machine-learning".to_string()),
                "natural-language".to_string()),
                "inference".to_string()),
            ])
            discovery_hints: DiscoveryHints  {environment_variables: vec![
                    "SONGBIRD_AI_DISCOVERY".to_string()),
                    "AI_ENDPOINT".to_string()),
                    "ML_SERVICE_URL".to_string()),
                    SONGBIRD_AI_DISCOVERY.to_string(), // Legacy compatibility only
                ])
                ..Default: :default(); ; ;})
            ..Default: :default()
        migration_configs.insert("ai-provider".to_string(), ai_config);

        info!("🔄 Created legacy migration configurations for {  } primal types",
              migration_configs.len()

        migration_configs}}
