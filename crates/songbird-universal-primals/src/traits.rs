//! Universal Primal Traits
//!
//! Core traits and types that define the universal primal interface

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;

use crate::errors::PrimalResult;
use crate::types::{PrimalRequest, PrimalResponse};

// Re-export common types
pub use songbird_universal::PrimalType;

/// Universal trait that any primal can implement
#[async_trait]
pub trait PrimalProvider: Send + Sync {
    /// Unique primal identifier (e.g., "beardog", "nestgate", "toadstool", "squirrel")
    fn primal_id(&self) -> &str;

    /// Instance identifier for multi-instance support (e.g., "beardog-user123", "beardog-device456")
    fn instance_id(&self) -> &str;

    /// User/device context this primal instance serves
    fn context(&self) -> &PrimalContext;

    /// Primal type category (e.g., Security, Storage, Compute, AI)
    fn primal_type(&self) -> PrimalType;

    /// Capabilities this primal provides
    fn capabilities(&self) -> Vec<PrimalCapability>;

    /// What this primal needs from other primals
    fn dependencies(&self) -> Vec<PrimalDependency>;

    /// Health check for this primal
    async fn health_check(&self) -> PrimalHealth;

    /// Get primal API endpoints
    fn endpoints(&self) -> Vec<String>;

    /// Handle inter-primal communication
    async fn handle_primal_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse>;

    /// Initialize the primal with configuration
    async fn initialize(&mut self, config: serde_json::Value) -> PrimalResult<()>;

    /// Shutdown the primal gracefully
    async fn shutdown(&mut self) -> PrimalResult<()>;

    /// Check if this primal can serve the given context
    fn can_serve_context(&self, context: &PrimalContext) -> bool;

    /// Get dynamic port information
    fn dynamic_port_info(&self) -> Option<DynamicPortInfo>;
}

/// Dynamic port information for songbird-managed ports
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DynamicPortInfo {
    /// Port assigned by songbird
    pub assigned_port: u16,
    /// Port type (HTTP, HTTPS, WebSocket, etc.)
    pub port_type: PortType,
    /// Port status
    pub status: PortStatus,
    /// Port assignment timestamp
    pub assigned_at: chrono::DateTime<chrono::Utc>,
    /// Port lease duration
    pub lease_duration: chrono::Duration,
}

/// Port type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortType {
    /// HTTP port
    Http,
    /// HTTPS port
    Https,
    /// WebSocket port
    WebSocket,
    /// gRPC port
    Grpc,
    /// Custom port type
    Custom(String),
}

/// Port status
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortStatus {
    /// Port is active and available
    Active,
    /// Port is reserved but not yet active
    Reserved,
    /// Port is being released
    Releasing,
    /// Port is expired and should be cleaned up
    Expired,
}

/// Primal dependency information - modernized to be capability-based
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PrimalDependency {
    /// The type of primal this depends on
    pub primal_type: PrimalType,
    /// Required capabilities from the dependency
    pub required_capabilities: Vec<PrimalCapability>,
    /// Whether this dependency is optional
    pub optional: bool,
    /// Dependency name for identification
    pub name: String,
}

impl PrimalDependency {
    /// Create a new primal dependency
    pub fn new(dependency_id: String, primal_type: PrimalType) -> Self {
        Self {
            name: dependency_id,
            primal_type,
            required_capabilities: Vec::new(),
            optional: false,
        }
    }

    /// Add required capability
    pub fn with_capability(mut self, capability: PrimalCapability) -> Self {
        self.required_capabilities.push(capability);
        self
    }

    /// Make this dependency optional
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    // Legacy compatibility methods
    /// Create authentication dependency (legacy compatibility)
    pub fn requires_authentication(methods: Vec<String>) -> Self {
        Self::new("authentication".to_string(), PrimalType::new("security"))
            .with_capability(PrimalCapability::Authentication { methods })
    }

    /// Create encryption dependency (legacy compatibility)
    pub fn requires_encryption(algorithms: Vec<String>) -> Self {
        Self::new("encryption".to_string(), PrimalType::new("security"))
            .with_capability(PrimalCapability::Encryption { algorithms })
    }

    /// Create storage dependency (legacy compatibility)
    pub fn requires_storage(types: Vec<String>) -> Self {
        Self::new("storage".to_string(), PrimalType::new("storage"))
            .with_capability(PrimalCapability::Storage { types })
    }

    /// Create compute dependency (legacy compatibility)
    pub fn requires_compute(types: Vec<String>) -> Self {
        Self::new("compute".to_string(), PrimalType::new("compute"))
            .with_capability(PrimalCapability::Compute { types })
    }

    /// Create AI dependency (legacy compatibility)
    pub fn requires_ai(models: Vec<String>) -> Self {
        Self::new("ai".to_string(), PrimalType::new("ai"))
            .with_capability(PrimalCapability::AI { models })
    }

    /// Create custom dependency (legacy compatibility)
    pub fn custom(name: String, requirements: HashMap<String, String>) -> Self {
        Self::new(name.clone(), PrimalType::new("custom")).with_capability(
            PrimalCapability::Custom {
                name,
                properties: requirements.into_iter().collect(),
            },
        )
    }
}

/// Health status of a primal
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalHealth {
    /// Healthy and operational
    Healthy,
    /// Degraded but functional with issues
    Degraded {
        /// List of issues affecting the service
        issues: Vec<String>,
    },
    /// Unhealthy - service issues
    Unhealthy {
        /// Reason for unhealthy status
        reason: String,
    },
    /// Unknown status
    Unknown,
}

impl Default for PrimalHealth {
    fn default() -> Self {
        Self::Unknown
    }
}

/// API endpoints for a primal
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimalEndpoints {
    /// Primary API endpoint
    pub primary: String,
    /// Health check endpoint
    pub health: String,
    /// Metrics endpoint
    pub metrics: Option<String>,
    /// Admin interface endpoint
    pub admin: Option<String>,
    /// WebSocket endpoint
    pub websocket: Option<String>,
    /// Custom endpoints
    pub custom: HashMap<String, String>,
}

/// Universal primal context for any primal implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalContext {
    /// Unique identifier for the primal instance
    pub primal_id: String,
    /// User ID that owns this primal instance
    pub user_id: String,
    /// Device ID where this primal is running
    pub device_id: String,
    /// Security level for this primal instance
    pub security_level: SecurityLevel,
    /// Session identifier for request tracking
    pub session_id: String,
    /// Network location information
    pub network_location: NetworkLocation,
    /// Additional metadata for extensibility
    pub metadata: HashMap<String, String>,
}

impl Default for PrimalContext {
    fn default() -> Self {
        Self {
            primal_id: "unknown".to_string(),
            user_id: "default_user".to_string(),
            device_id: "localhost".to_string(),
            security_level: SecurityLevel::User,
            session_id: uuid::Uuid::new_v4().to_string(),
            network_location: NetworkLocation::default(),
            metadata: HashMap::new(),
        }
    }
}

/// Security levels for primal operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Public access - no authentication required
    Public,
    /// User access - requires user authentication
    User,
    /// Admin access - requires admin privileges
    Admin,
    /// System access - requires system-level privileges
    System,
    /// Standard security (legacy compatibility)
    Standard,
}

impl Default for SecurityLevel {
    fn default() -> Self {
        Self::User
    }
}

/// Capabilities that a primal can provide - enhanced for universal discovery
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalCapability {
    /// Storage capabilities
    Storage {
        /// Storage types supported (object, block, file)
        types: Vec<String>, // object, block, file
    },
    /// Compute capabilities  
    Compute {
        /// Compute types supported (container, vm, serverless)
        types: Vec<String>, // container, vm, serverless
    },
    /// AI/ML capabilities
    AI {
        /// Models supported (llm, embedding, classification)
        models: Vec<String>, // llm, embedding, classification
    },
    /// Authentication capabilities
    Authentication {
        /// Authentication methods supported (jwt, oauth2, api_key)
        methods: Vec<String>, // jwt, oauth2, api_key
    },
    /// Networking capabilities
    Networking {
        /// Network protocols supported (tcp, udp, websocket)
        protocols: Vec<String>, // tcp, udp, websocket
    },
    /// Service discovery capabilities
    ServiceDiscovery {
        /// Discovery protocols supported (http, grpc, websocket)
        protocols: Vec<String>, // http, grpc, websocket
    },
    /// Security capabilities
    Security {
        /// Security protocols supported (tls, zero_trust)
        protocols: Vec<String>, // tls, zero_trust
    },
    /// Encryption capabilities
    Encryption {
        /// Encryption algorithms supported (aes256, chacha20)
        algorithms: Vec<String>, // aes256, chacha20
    },
    /// Orchestration capabilities
    Orchestration {
        /// Orchestration features supported (federation, load_balancing, health_monitoring)
        features: Vec<String>, // federation, load_balancing, health_monitoring
    },
    /// Database capabilities
    Database {
        /// Database types supported (relational, document, kv)
        types: Vec<String>, // relational, document, kv
    },
    /// Messaging capabilities
    Messaging {
        /// Messaging protocols supported (mqtt, amqp, kafka)
        protocols: Vec<String>, // mqtt, amqp, kafka
    },

    // Legacy compatibility variants (can be deprecated later)
    /// File system support (legacy - use Storage instead)
    FileSystem {
        /// Whether ZFS is supported
        supports_zfs: bool,
    },
    /// Container runtime support (legacy - use Compute instead)
    ContainerRuntime {
        /// Orchestrators supported
        orchestrators: Vec<String>,
    },
    /// Serverless execution (legacy - use Compute instead)  
    ServerlessExecution {
        /// Languages supported
        languages: Vec<String>,
    },
    /// Model inference (legacy - use AI instead)
    ModelInference {
        /// Models supported
        models: Vec<String>,
    },
    /// Agent framework (legacy - use AI instead)
    AgentFramework {
        /// Whether MCP is supported
        mcp_support: bool,
    },
    /// Natural language processing (legacy - use AI instead)
    NaturalLanguage {
        /// Languages supported
        languages: Vec<String>,
    },
    /// Object storage (legacy - use Storage instead)
    ObjectStorage {
        /// Storage backends supported
        backends: Vec<String>,
    },
    /// Load balancing (legacy - use Orchestration instead)
    LoadBalancing {
        /// Load balancing algorithms supported
        algorithms: Vec<String>,
    },
    /// Auto-scaling (legacy - use Orchestration instead)
    AutoScaling {
        /// Scaling metrics supported
        metrics: Vec<String>,
    },
    /// Data replication (legacy - use Storage instead)
    DataReplication {
        /// Consistency level
        consistency: String,
    },
    /// Backup capabilities (legacy - use Storage instead)
    Backup {
        /// Whether incremental backup is supported
        incremental: bool,
    },
    /// Data archiving (legacy - use Storage instead)  
    DataArchiving {
        /// Compression algorithms supported
        compression: Vec<String>,
    },
    /// Key management (legacy - use Security instead)
    KeyManagement {
        /// Whether HSM is supported
        hsm_support: bool,
    },
    /// Threat detection (legacy - use Security instead)
    ThreatDetection {
        /// Whether ML-based detection is enabled
        ml_enabled: bool,
    },
    /// Authorization (legacy - use Authentication instead)
    Authorization {
        /// Whether RBAC is supported
        rbac_support: bool,
    },
    /// GPU acceleration (legacy - use Compute instead)
    GpuAcceleration {
        /// Whether CUDA is supported
        cuda_support: bool,
    },
    /// Machine learning (legacy - use AI instead)
    MachineLearning {
        /// Whether training is supported
        training_support: bool,
    },
    /// Computer vision (legacy - use AI instead)
    ComputerVision {
        /// Vision models supported
        models: Vec<String>,
    },
    /// Network routing (legacy - use Networking instead)
    NetworkRouting {
        /// Routing protocols supported
        protocols: Vec<String>,
    },
    /// Proxy services (legacy - use Networking instead)
    ProxyServices {
        /// Proxy types supported
        types: Vec<String>,
    },
    /// VPN services (legacy - use Networking instead)
    VpnServices {
        /// VPN protocols supported
        protocols: Vec<String>,
    },
    /// Manifest management (legacy - use Orchestration instead)
    Manifests {
        /// Manifest formats supported
        formats: Vec<String>,
    },
    /// Backup and restore (legacy - use Storage instead)
    BackupRestore {
        /// Whether incremental backup is supported
        incremental: bool,
    },

    /// Custom capability (without HashMap to fix Hash issue)
    Custom {
        /// Capability name
        name: String,
        /// Capability properties as key-value pairs
        properties: Vec<(String, String)>, // Changed from HashMap to Vec for Hash compatibility
    },
}

// Manual Hash implementation to handle the Custom variant properly
impl Hash for PrimalCapability {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Storage { types } => {
                "Storage".hash(state);
                types.hash(state);
            }
            Self::Compute { types } => {
                "Compute".hash(state);
                types.hash(state);
            }
            Self::AI { models } => {
                "AI".hash(state);
                models.hash(state);
            }
            Self::Authentication { methods } => {
                "Authentication".hash(state);
                methods.hash(state);
            }
            Self::Networking { protocols } => {
                "Networking".hash(state);
                protocols.hash(state);
            }
            Self::ServiceDiscovery { protocols } => {
                "ServiceDiscovery".hash(state);
                protocols.hash(state);
            }
            Self::Security { protocols } => {
                "Security".hash(state);
                protocols.hash(state);
            }
            Self::Encryption { algorithms } => {
                "Encryption".hash(state);
                algorithms.hash(state);
            }
            Self::Orchestration { features } => {
                "Orchestration".hash(state);
                features.hash(state);
            }
            Self::Database { types } => {
                "Database".hash(state);
                types.hash(state);
            }
            Self::Messaging { protocols } => {
                "Messaging".hash(state);
                protocols.hash(state);
            }
            // Legacy variants
            Self::FileSystem { supports_zfs } => {
                "FileSystem".hash(state);
                supports_zfs.hash(state);
            }
            Self::ContainerRuntime { orchestrators } => {
                "ContainerRuntime".hash(state);
                orchestrators.hash(state);
            }
            Self::ServerlessExecution { languages } => {
                "ServerlessExecution".hash(state);
                languages.hash(state);
            }
            Self::ModelInference { models } => {
                "ModelInference".hash(state);
                models.hash(state);
            }
            Self::AgentFramework { mcp_support } => {
                "AgentFramework".hash(state);
                mcp_support.hash(state);
            }
            Self::NaturalLanguage { languages } => {
                "NaturalLanguage".hash(state);
                languages.hash(state);
            }
            Self::ObjectStorage { backends } => {
                "ObjectStorage".hash(state);
                backends.hash(state);
            }
            Self::LoadBalancing { algorithms } => {
                "LoadBalancing".hash(state);
                algorithms.hash(state);
            }
            Self::AutoScaling { metrics } => {
                "AutoScaling".hash(state);
                metrics.hash(state);
            }
            Self::DataReplication { consistency } => {
                "DataReplication".hash(state);
                consistency.hash(state);
            }
            Self::Backup { incremental } => {
                "Backup".hash(state);
                incremental.hash(state);
            }
            Self::DataArchiving { compression } => {
                "DataArchiving".hash(state);
                compression.hash(state);
            }
            Self::KeyManagement { hsm_support } => {
                "KeyManagement".hash(state);
                hsm_support.hash(state);
            }
            Self::ThreatDetection { ml_enabled } => {
                "ThreatDetection".hash(state);
                ml_enabled.hash(state);
            }
            Self::Authorization { rbac_support } => {
                "Authorization".hash(state);
                rbac_support.hash(state);
            }
            Self::GpuAcceleration { cuda_support } => {
                "GpuAcceleration".hash(state);
                cuda_support.hash(state);
            }
            Self::MachineLearning { training_support } => {
                "MachineLearning".hash(state);
                training_support.hash(state);
            }
            Self::ComputerVision { models } => {
                "ComputerVision".hash(state);
                models.hash(state);
            }
            Self::NetworkRouting { protocols } => {
                "NetworkRouting".hash(state);
                protocols.hash(state);
            }
            Self::ProxyServices { types } => {
                "ProxyServices".hash(state);
                types.hash(state);
            }
            Self::VpnServices { protocols } => {
                "VpnServices".hash(state);
                protocols.hash(state);
            }
            Self::Manifests { formats } => {
                "Manifests".hash(state);
                formats.hash(state);
            }
            Self::BackupRestore { incremental } => {
                "BackupRestore".hash(state);
                incremental.hash(state);
            }
            Self::Custom { name, properties } => {
                "Custom".hash(state);
                name.hash(state);
                properties.hash(state);
            }
        }
    }
}

impl PrimalCapability {
    /// Check if this capability matches another capability requirement
    pub fn matches(&self, requirement: &PrimalCapability) -> bool {
        match (self, requirement) {
            (Self::Storage { types: a }, Self::Storage { types: b }) => {
                b.iter().all(|req_type| a.contains(req_type))
            }
            (Self::Compute { types: a }, Self::Compute { types: b }) => {
                b.iter().all(|req_type| a.contains(req_type))
            }
            (Self::AI { models: a }, Self::AI { models: b }) => {
                b.iter().all(|req_model| a.contains(req_model))
            }
            (Self::Authentication { methods: a }, Self::Authentication { methods: b }) => {
                b.iter().all(|req_method| a.contains(req_method))
            }
            (Self::Networking { protocols: a }, Self::Networking { protocols: b }) => {
                b.iter().all(|req_protocol| a.contains(req_protocol))
            }
            (Self::ServiceDiscovery { protocols: a }, Self::ServiceDiscovery { protocols: b }) => {
                b.iter().all(|req_protocol| a.contains(req_protocol))
            }
            (Self::Security { protocols: a }, Self::Security { protocols: b }) => {
                b.iter().all(|req_protocol| a.contains(req_protocol))
            }
            (Self::Encryption { algorithms: a }, Self::Encryption { algorithms: b }) => {
                b.iter().all(|req_algo| a.contains(req_algo))
            }
            (Self::Orchestration { features: a }, Self::Orchestration { features: b }) => {
                b.iter().all(|req_feature| a.contains(req_feature))
            }
            (Self::Database { types: a }, Self::Database { types: b }) => {
                b.iter().all(|req_type| a.contains(req_type))
            }
            (Self::Messaging { protocols: a }, Self::Messaging { protocols: b }) => {
                b.iter().all(|req_protocol| a.contains(req_protocol))
            }
            // Legacy compatibility
            (Self::FileSystem { .. }, Self::Storage { .. })
            | (Self::ObjectStorage { .. }, Self::Storage { .. }) => true,
            (Self::ContainerRuntime { .. }, Self::Compute { .. })
            | (Self::ServerlessExecution { .. }, Self::Compute { .. }) => true,
            (Self::ModelInference { .. }, Self::AI { .. })
            | (Self::AgentFramework { .. }, Self::AI { .. })
            | (Self::MachineLearning { .. }, Self::AI { .. }) => true,
            _ => std::mem::discriminant(self) == std::mem::discriminant(requirement),
        }
    }

    /// Get the capability type name
    pub fn capability_type(&self) -> String {
        match self {
            Self::Storage { .. } => "storage".to_string(),
            Self::Compute { .. } => "compute".to_string(),
            Self::AI { .. } => "ai".to_string(),
            Self::Authentication { .. } => "authentication".to_string(),
            Self::Networking { .. } => "networking".to_string(),
            Self::ServiceDiscovery { .. } => "service_discovery".to_string(),
            Self::Security { .. } => "security".to_string(),
            Self::Encryption { .. } => "encryption".to_string(),
            Self::Orchestration { .. } => "orchestration".to_string(),
            Self::Database { .. } => "database".to_string(),
            Self::Messaging { .. } => "messaging".to_string(),
            // Legacy variants map to modern types
            Self::FileSystem { .. } => "storage".to_string(),
            Self::ObjectStorage { .. } => "storage".to_string(),
            Self::ContainerRuntime { .. } => "compute".to_string(),
            Self::ServerlessExecution { .. } => "compute".to_string(),
            Self::ModelInference { .. } => "ai".to_string(),
            Self::AgentFramework { .. } => "ai".to_string(),
            Self::MachineLearning { .. } => "ai".to_string(),
            Self::LoadBalancing { .. } => "orchestration".to_string(),
            Self::Custom { name, .. } => name.clone(),
            _ => "unknown".to_string(),
        }
    }

    /// Check if this is a core capability (required for basic operation)
    pub fn is_core_capability(&self) -> bool {
        matches!(
            self,
            Self::ServiceDiscovery { .. } | Self::Networking { .. } | Self::Security { .. }
        )
    }
}

/// Capability-based service matcher for name-agnostic discovery
#[derive(Debug, Clone)]
pub struct CapabilityMatcher {
    /// Required capabilities
    pub required: Vec<PrimalCapability>,
    /// Optional capabilities (nice to have)
    pub optional: Vec<PrimalCapability>,
    /// Capabilities to avoid/exclude
    pub excluded: Vec<PrimalCapability>,
}

impl CapabilityMatcher {
    /// Create a new capability matcher
    pub fn new() -> Self {
        Self {
            required: Vec::new(),
            optional: Vec::new(),
            excluded: Vec::new(),
        }
    }

    /// Add a required capability
    pub fn require(mut self, capability: PrimalCapability) -> Self {
        self.required.push(capability);
        self
    }

    /// Add an optional capability
    pub fn prefer(mut self, capability: PrimalCapability) -> Self {
        self.optional.push(capability);
        self
    }

    /// Add an excluded capability
    pub fn exclude(mut self, capability: PrimalCapability) -> Self {
        self.excluded.push(capability);
        self
    }

    /// Check if a set of capabilities matches this matcher
    pub fn matches(&self, capabilities: &[PrimalCapability]) -> bool {
        // All required capabilities must be present
        for required in &self.required {
            if !capabilities.iter().any(|cap| cap.matches(required)) {
                return false;
            }
        }

        // No excluded capabilities should be present
        for excluded in &self.excluded {
            if capabilities.iter().any(|cap| cap.matches(excluded)) {
                return false;
            }
        }

        true
    }

    /// Calculate a compatibility score (0.0 to 1.0)
    pub fn compatibility_score(&self, capabilities: &[PrimalCapability]) -> f64 {
        if !self.matches(capabilities) {
            return 0.0;
        }

        let mut score = 1.0;

        // Bonus for optional capabilities
        let optional_matches = self
            .optional
            .iter()
            .filter(|opt| capabilities.iter().any(|cap| cap.matches(opt)))
            .count();

        if !self.optional.is_empty() {
            score += (optional_matches as f64 / self.optional.len() as f64) * 0.5;
        }

        // Cap at 1.0
        score.min(1.0)
    }
}

impl Default for CapabilityMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of primal integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationResult {
    /// Success status
    pub success: bool,
    /// Integration ID
    pub integration_id: String,
    /// Shared capabilities after integration
    pub shared_capabilities: Vec<PrimalCapability>,
    /// Configuration updates needed
    pub configuration_updates: Option<serde_json::Value>,
    /// Error message if integration failed
    pub error_message: Option<String>,
}

/// Network location information
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetworkLocation {
    /// IP address
    pub ip_address: String,
    /// Subnet
    pub subnet: Option<String>,
    /// Local network identifier
    pub network_id: Option<String>,
    /// Geographic location
    pub geo_location: Option<String>,
}

impl Default for NetworkLocation {
    fn default() -> Self {
        Self {
            ip_address: "127.0.0.1".to_string(),
            subnet: None,
            network_id: None,
            geo_location: None,
        }
    }
}

/// Configuration for primal instances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalConfiguration {
    /// Primal identifier
    pub primal_id: String,
    /// Instance identifier
    pub instance_id: String,
    /// Context for this primal
    pub context: PrimalContext,
    /// Endpoint configuration
    pub endpoints: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}
