//! Universal primal traits and interfaces

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::errors::PrimalResult;
use crate::types::{PrimalRequest, PrimalResponse};

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
    fn endpoints(&self) -> PrimalEndpoints;

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

/// Context for user/device-specific primal routing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimalContext {
    /// User identifier
    pub user_id: String,
    /// Device identifier
    pub device_id: String,
    /// Session identifier
    pub session_id: String,
    /// Network location (IP, subnet, etc.)
    pub network_location: NetworkLocation,
    /// Security level required
    pub security_level: SecurityLevel,
    /// Additional context metadata
    pub metadata: HashMap<String, String>,
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

/// Security level requirements
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Basic security
    Basic,
    /// Standard security
    Standard,
    /// High security
    High,
    /// Maximum security
    Maximum,
}

impl Default for SecurityLevel {
    fn default() -> Self {
        Self::Basic
    }
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

/// Primal type categories
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimalType {
    /// Security primal (BearDog)
    Security,
    /// Storage primal (NestGate)
    Storage,
    /// Compute primal (Toadstool)
    Compute,
    /// AI primal (Squirrel)
    AI,
    /// Network primal
    Network,
    /// Custom primal type
    Custom(String),
}

impl std::fmt::Display for PrimalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimalType::Security => write!(f, "Security"),
            PrimalType::Storage => write!(f, "Storage"),
            PrimalType::Compute => write!(f, "Compute"),
            PrimalType::AI => write!(f, "AI"),
            PrimalType::Network => write!(f, "Network"),
            PrimalType::Custom(name) => write!(f, "Custom({name})"),
        }
    }
}

/// Universal capabilities that any primal can provide
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalCapability {
    // Security capabilities (BearDog)
    /// Authentication with supported methods
    Authentication {
        /// List of supported authentication methods
        methods: Vec<String>,
    },
    /// Encryption with supported algorithms
    Encryption {
        /// List of supported encryption algorithms
        algorithms: Vec<String>,
    },
    /// Key management with HSM support
    KeyManagement {
        /// Whether HSM (Hardware Security Module) is supported
        hsm_support: bool,
    },
    /// Threat detection with ML capabilities
    ThreatDetection {
        /// Whether machine learning is enabled for threat detection
        ml_enabled: bool,
    },
    /// Audit logging with compliance standards
    AuditLogging {
        /// List of supported compliance standards
        compliance: Vec<String>,
    },
    /// Authorization and access control
    Authorization {
        /// Whether RBAC (Role-Based Access Control) is supported
        rbac_support: bool,
    },

    // Storage capabilities (NestGate)
    /// File system support
    FileSystem {
        /// Whether ZFS file system is supported
        supports_zfs: bool,
    },
    /// Object storage with backends
    ObjectStorage {
        /// List of supported storage backends
        backends: Vec<String>,
    },
    /// Data replication
    DataReplication {
        /// Consistency model for data replication
        consistency: String,
    },
    /// Backup capabilities
    Backup {
        /// Whether incremental backups are supported
        incremental: bool,
    },
    /// Data archiving
    DataArchiving {
        /// List of supported compression algorithms
        compression: Vec<String>,
    },

    // Compute capabilities (Toadstool)
    /// Container runtime support
    ContainerRuntime {
        /// List of supported container orchestrators
        orchestrators: Vec<String>,
    },
    /// Serverless execution
    ServerlessExecution {
        /// List of supported programming languages
        languages: Vec<String>,
    },
    /// GPU acceleration
    GpuAcceleration {
        /// Whether CUDA is supported
        cuda_support: bool,
    },
    /// Load balancing
    LoadBalancing {
        /// List of supported load balancing algorithms
        algorithms: Vec<String>,
    },
    /// Auto-scaling
    AutoScaling {
        /// List of supported scaling metrics
        metrics: Vec<String>,
    },

    // AI capabilities (Squirrel)
    /// Model inference
    ModelInference {
        /// List of supported AI models
        models: Vec<String>,
    },
    /// Agent framework
    AgentFramework {
        /// Whether MCP (Model Context Protocol) is supported
        mcp_support: bool,
    },
    /// Machine learning
    MachineLearning {
        /// Whether training is supported
        training_support: bool,
    },
    /// Natural language processing
    NaturalLanguage {
        /// List of supported languages
        languages: Vec<String>,
    },
    /// Computer vision
    ComputerVision {
        /// List of supported computer vision models
        models: Vec<String>,
    },

    // Networking capabilities
    /// Service discovery
    ServiceDiscovery {
        /// List of supported discovery protocols
        protocols: Vec<String>,
    },
    /// Network routing
    NetworkRouting {
        /// List of supported routing protocols
        protocols: Vec<String>,
    },
    /// Proxy services
    ProxyServices {
        /// List of supported proxy types
        types: Vec<String>,
    },
    /// VPN capabilities
    VpnServices {
        /// List of supported VPN protocols
        protocols: Vec<String>,
    },

    // Generic capabilities
    /// Custom capability
    Custom {
        /// Name of the custom capability
        name: String,
        /// Custom attributes for the capability
        attributes: HashMap<String, String>,
    },
}

impl Hash for PrimalCapability {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            PrimalCapability::Authentication { methods } => {
                "Authentication".hash(state);
                methods.hash(state);
            }
            PrimalCapability::Encryption { algorithms } => {
                "Encryption".hash(state);
                algorithms.hash(state);
            }
            PrimalCapability::KeyManagement { hsm_support } => {
                "KeyManagement".hash(state);
                hsm_support.hash(state);
            }
            PrimalCapability::ThreatDetection { ml_enabled } => {
                "ThreatDetection".hash(state);
                ml_enabled.hash(state);
            }
            PrimalCapability::AuditLogging { compliance } => {
                "AuditLogging".hash(state);
                compliance.hash(state);
            }
            PrimalCapability::Authorization { rbac_support } => {
                "Authorization".hash(state);
                rbac_support.hash(state);
            }
            PrimalCapability::FileSystem { supports_zfs } => {
                "FileSystem".hash(state);
                supports_zfs.hash(state);
            }
            PrimalCapability::ObjectStorage { backends } => {
                "ObjectStorage".hash(state);
                backends.hash(state);
            }
            PrimalCapability::DataReplication { consistency } => {
                "DataReplication".hash(state);
                consistency.hash(state);
            }
            PrimalCapability::Backup { incremental } => {
                "Backup".hash(state);
                incremental.hash(state);
            }
            PrimalCapability::DataArchiving { compression } => {
                "DataArchiving".hash(state);
                compression.hash(state);
            }
            PrimalCapability::ContainerRuntime { orchestrators } => {
                "ContainerRuntime".hash(state);
                orchestrators.hash(state);
            }
            PrimalCapability::ServerlessExecution { languages } => {
                "ServerlessExecution".hash(state);
                languages.hash(state);
            }
            PrimalCapability::GpuAcceleration { cuda_support } => {
                "GpuAcceleration".hash(state);
                cuda_support.hash(state);
            }
            PrimalCapability::LoadBalancing { algorithms } => {
                "LoadBalancing".hash(state);
                algorithms.hash(state);
            }
            PrimalCapability::AutoScaling { metrics } => {
                "AutoScaling".hash(state);
                metrics.hash(state);
            }
            PrimalCapability::ModelInference { models } => {
                "ModelInference".hash(state);
                models.hash(state);
            }
            PrimalCapability::AgentFramework { mcp_support } => {
                "AgentFramework".hash(state);
                mcp_support.hash(state);
            }
            PrimalCapability::MachineLearning { training_support } => {
                "MachineLearning".hash(state);
                training_support.hash(state);
            }
            PrimalCapability::NaturalLanguage { languages } => {
                "NaturalLanguage".hash(state);
                languages.hash(state);
            }
            PrimalCapability::ComputerVision { models } => {
                "ComputerVision".hash(state);
                models.hash(state);
            }
            PrimalCapability::ServiceDiscovery { protocols } => {
                "ServiceDiscovery".hash(state);
                protocols.hash(state);
            }
            PrimalCapability::NetworkRouting { protocols } => {
                "NetworkRouting".hash(state);
                protocols.hash(state);
            }
            PrimalCapability::ProxyServices { types } => {
                "ProxyServices".hash(state);
                types.hash(state);
            }
            PrimalCapability::VpnServices { protocols } => {
                "VpnServices".hash(state);
                protocols.hash(state);
            }
            PrimalCapability::Custom {
                name,
                attributes: _,
            } => {
                "Custom".hash(state);
                name.hash(state);
                // Skip hashing attributes since HashMap doesn't implement Hash
            }
        }
    }
}

/// Dependencies that a primal needs from other primals
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalDependency {
    /// Requires authentication
    RequiresAuthentication {
        /// List of required authentication methods
        methods: Vec<String>,
    },
    /// Requires encryption
    RequiresEncryption {
        /// List of required encryption algorithms
        algorithms: Vec<String>,
    },
    /// Requires storage
    RequiresStorage {
        /// List of required storage types
        types: Vec<String>,
    },
    /// Requires compute
    RequiresCompute {
        /// List of required compute types
        types: Vec<String>,
    },
    /// Requires AI
    RequiresAI {
        /// List of required AI capabilities
        capabilities: Vec<String>,
    },
    /// Custom dependency
    Custom {
        /// Name of the custom dependency
        name: String,
        /// Custom requirements for the dependency
        requirements: HashMap<String, String>,
    },
}

/// Health status of a primal
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalHealth {
    /// Primal is healthy and operational
    Healthy,
    /// Primal is degraded but operational
    Degraded {
        /// List of issues causing degradation
        issues: Vec<String>,
    },
    /// Primal is unhealthy and not operational
    Unhealthy {
        /// Reason why the primal is unhealthy
        reason: String,
    },
}

/// Primal API endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalEndpoints {
    /// Primary API endpoint
    pub primary: String,
    /// Health check endpoint
    pub health: String,
    /// Metrics endpoint
    pub metrics: Option<String>,
    /// Admin endpoint
    pub admin: Option<String>,
    /// WebSocket endpoint
    pub websocket: Option<String>,
    /// Additional custom endpoints
    pub custom: HashMap<String, String>,
}

impl Default for PrimalEndpoints {
    fn default() -> Self {
        Self {
            primary: "http://localhost:8080".to_string(),
            health: "http://localhost:8080/health".to_string(),
            metrics: None,
            admin: None,
            websocket: None,
            custom: HashMap::new(),
        }
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
