//! Capability definitions for Universal Primals
//!
//! Provides comprehensive capability definitions, resource requirements, and performance
//! specifications for the Universal Primals system with modern Rust idioms.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Alias for capability dependency (now using CapabilityDependency)
pub use CapabilityDependency as PrimalDependency;

/// Core capability enumeration for Universal Primals
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PrimalCapability {
    /// AI model inference capability
    ModelInference { models: Vec<String> },

    /// Agent framework support
    AgentFramework { mcp_support: bool },

    /// Natural language processing
    NaturalLanguage { languages: Vec<String> },

    /// Network discovery capability
    NetworkDiscovery { protocols: Vec<String> },

    /// Container orchestration
    ContainerOrchestration { platforms: Vec<String> },

    /// Service mesh management
    ServiceMesh { protocols: Vec<String> },

    /// Service discovery
    ServiceDiscovery { protocols: Vec<String> },

    /// Load balancing
    LoadBalancing { algorithms: Vec<String> },

    /// Security and authentication (general)
    Security { methods: Vec<String> },

    /// Authentication capability
    Authentication { methods: Vec<String> },

    /// Authorization capability
    Authorization { rbac_support: bool },

    /// Encryption capability
    Encryption { algorithms: Vec<String> },

    /// Key management capability
    KeyManagement { key_types: Vec<String> },

    /// Threat detection capability
    ThreatDetection { ml_enabled: bool },

    /// File system storage
    FileSystem { supports_zfs: bool },

    /// Object storage capability
    ObjectStorage { backends: Vec<String> },

    /// Data replication capability
    DataReplication { consistency: String },

    /// Backup capability
    Backup { incremental: bool },

    /// Container runtime capability
    ContainerRuntime { orchestrators: Vec<String> },

    /// Serverless execution capability
    ServerlessExecution { languages: Vec<String> },

    /// GPU acceleration capability
    GpuAcceleration { gpu_types: Vec<String> },

    /// Auto scaling capability
    AutoScaling { strategies: Vec<String> },

    /// Machine learning capability
    MachineLearning { frameworks: Vec<String> },

    /// Orchestration capability
    Orchestration { platforms: Vec<String> },

    /// Manifests capability
    Manifests { formats: Vec<String> },

    /// Network routing capability
    NetworkRouting { protocols: Vec<String> },

    /// Proxy services capability
    ProxyServices { protocols: Vec<String> },

    /// VPN services capability
    VpnServices { protocols: Vec<String> },

    /// Storage capability (general)
    Storage { storage_types: Vec<String> },

    /// Compute capability (general)
    Compute { compute_types: Vec<String> },

    /// AI capability (general)
    AI { capabilities: Vec<String> },

    /// Custom capability with arbitrary parameters
    Custom {
        name: String,
        properties: Vec<(String, String)>,
    },
}

impl PrimalCapability {
    /// Get the capability name as a string
    pub fn name(&self) -> &str {
        match self {
            Self::ModelInference { .. } => "model_inference",
            Self::AgentFramework { .. } => "agent_framework",
            Self::NaturalLanguage { .. } => "natural_language",
            Self::NetworkDiscovery { .. } => "network_discovery",
            Self::ContainerOrchestration { .. } => "container_orchestration",
            Self::ServiceMesh { .. } => "service_mesh",
            Self::ServiceDiscovery { .. } => "service_discovery",
            Self::LoadBalancing { .. } => "load_balancing",
            Self::Security { .. } => "security",
            Self::Authentication { .. } => "authentication",
            Self::Authorization { .. } => "authorization",
            Self::Encryption { .. } => "encryption",
            Self::KeyManagement { .. } => "key_management",
            Self::ThreatDetection { .. } => "threat_detection",
            Self::FileSystem { .. } => "file_system",
            Self::ObjectStorage { .. } => "object_storage",
            Self::DataReplication { .. } => "data_replication",
            Self::Backup { .. } => "backup",
            Self::ContainerRuntime { .. } => "container_runtime",
            Self::ServerlessExecution { .. } => "serverless_execution",
            Self::GpuAcceleration { .. } => "gpu_acceleration",
            Self::AutoScaling { .. } => "auto_scaling",
            Self::MachineLearning { .. } => "machine_learning",
            Self::Orchestration { .. } => "orchestration",
            Self::Manifests { .. } => "manifests",
            Self::NetworkRouting { .. } => "network_routing",
            Self::ProxyServices { .. } => "proxy_services",
            Self::VpnServices { .. } => "vpn_services",
            Self::Storage { .. } => "storage",
            Self::Compute { .. } => "compute",
            Self::AI { .. } => "ai",
            Self::Custom { name, .. } => name,
        }
    }

    /// Check if this capability is compatible with another
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ModelInference { .. }, Self::AgentFramework { .. }) => true,
            (Self::NetworkDiscovery { .. }, Self::ServiceDiscovery { .. }) => true,
            (Self::ContainerOrchestration { .. }, Self::ServiceMesh { .. }) => true,
            (Self::ServiceDiscovery { .. }, Self::LoadBalancing { .. }) => true,
            _ => false,
        }
    }
}

/// Performance metrics for capability monitoring
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapabilityPerformance {
    /// Average response time in milliseconds
    pub response_time_ms: f64,

    /// Throughput in requests per second
    pub throughput_rps: f64,

    /// Error rate as a percentage (0.0 to 100.0)
    pub error_rate: f64,

    /// Resource utilization percentage (0.0 to 100.0)
    pub resource_utilization: f64,

    /// Availability percentage (0.0 to 100.0)
    pub availability: f64,

    /// Custom metrics
    pub custom_metrics: HashMap<String, f64>,
}

impl Default for CapabilityPerformance {
    fn default() -> Self {
        Self {
            response_time_ms: 100.0,
            throughput_rps: 10.0,
            error_rate: 1.0,
            resource_utilization: 50.0,
            availability: 99.9,
            custom_metrics: HashMap::new(),
        }
    }
}

impl Eq for CapabilityPerformance {}

impl Hash for CapabilityPerformance {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Convert floats to integer representations for hashing
        (self.response_time_ms as u64).hash(state);
        (self.throughput_rps as u64).hash(state);
        (self.error_rate as u64).hash(state);
        (self.resource_utilization as u64).hash(state);
        (self.availability as u64).hash(state);

        // Hash custom metrics in a consistent order
        let mut metrics: Vec<_> = self.custom_metrics.iter().collect();
        metrics.sort_by_key(|(k, _)| *k);
        for (key, value) in metrics {
            key.hash(state);
            (*value as u64).hash(state);
        }
    }
}

/// Resource requirements for capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceRequirements {
    /// Minimum CPU cores required
    pub min_cpu_cores: Option<u32>,

    /// Minimum memory in MB
    pub min_memory_mb: Option<u64>,

    /// Minimum disk space in MB
    pub min_disk_mb: Option<u64>,

    /// Network requirements
    pub network: Option<NetworkRequirements>,

    /// Custom resource requirements
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            min_cpu_cores: Some(1),
            min_memory_mb: Some(512),
            min_disk_mb: Some(1024),
            network: None,
            custom: HashMap::new(),
        }
    }
}

impl Eq for ResourceRequirements {}

impl Hash for ResourceRequirements {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.min_cpu_cores.hash(state);
        self.min_memory_mb.hash(state);
        self.min_disk_mb.hash(state);
        self.network.hash(state);

        // Hash custom requirements in a consistent order
        let mut custom_vec: Vec<_> = self.custom.iter().collect();
        custom_vec.sort_by_key(|(k, _)| *k);
        for (key, value) in custom_vec {
            key.hash(state);
            // Hash JSON value as string for consistency
            value.to_string().hash(state);
        }
    }
}

/// Network requirements for capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NetworkRequirements {
    /// Minimum bandwidth in Mbps
    pub min_bandwidth_mbps: Option<u32>,

    /// Maximum acceptable latency in milliseconds
    pub max_latency_ms: Option<u32>,

    /// Required network protocols
    pub protocols: Vec<String>,

    /// Port requirements
    pub ports: Vec<u16>,

    /// Whether TLS is required
    pub tls_required: bool,
}

impl Default for NetworkRequirements {
    fn default() -> Self {
        Self {
            min_bandwidth_mbps: Some(10),
            max_latency_ms: Some(100),
            protocols: vec!["http".to_string()],
            ports: vec![80, 443],
            tls_required: false,
        }
    }
}

/// Capability dependency specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityDependency {
    /// Name of the required capability
    pub capability_name: String,

    /// Minimum version required
    pub min_version: Option<String>,

    /// Whether this dependency is optional
    pub optional: bool,

    /// Custom dependency parameters
    pub parameters: HashMap<String, String>,
}

impl CapabilityDependency {
    /// Create a new required capability dependency
    pub fn required(capability_name: &str) -> Self {
        Self {
            capability_name: capability_name.to_string(),
            min_version: None,
            optional: false,
            parameters: HashMap::new(),
        }
    }

    /// Create a new optional capability dependency
    pub fn optional(capability_name: &str) -> Self {
        Self {
            capability_name: capability_name.to_string(),
            min_version: None,
            optional: true,
            parameters: HashMap::new(),
        }
    }

    /// Set minimum version requirement
    pub fn with_min_version(mut self, version: &str) -> Self {
        self.min_version = Some(version.to_string());
        self
    }

    /// Add a parameter to the dependency
    pub fn with_parameter(mut self, key: &str, value: &str) -> Self {
        self.parameters.insert(key.to_string(), value.to_string());
        self
    }
}

/// Capability configuration and metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityConfig {
    /// Capability identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Description of the capability
    pub description: String,

    /// Version of the capability implementation
    pub version: String,

    /// Whether the capability is enabled
    pub enabled: bool,

    /// Configuration parameters
    pub parameters: HashMap<String, String>,

    /// Tags for categorization
    pub tags: Vec<String>,
}

impl CapabilityConfig {
    /// Create a new capability configuration
    pub fn new(id: &str, name: &str, version: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: String::new(),
            version: version.to_string(),
            enabled: true,
            parameters: HashMap::new(),
            tags: Vec::new(),
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// Add a parameter
    pub fn with_parameter(mut self, key: &str, value: &str) -> Self {
        self.parameters.insert(key.to_string(), value.to_string());
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    /// Enable or disable the capability
    pub fn set_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}
