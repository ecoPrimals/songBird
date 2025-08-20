//! Universal type enumerations and classifications

use songbird_errors::EvolvedResult;
use serde::{Deserialize, Serialize};

/// Capability type enumeration with structured data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CapabilityType {
    /// Compute capabilities (processing, calculations)
    Compute {
        /// CPU cores available
        cores: Option<u32>,
        /// Memory in GB
        memory_gb: Option<u32>,
        /// GPU acceleration available
        gpu_enabled: bool,
    },
    /// Storage capabilities (data persistence)
    Storage {
        /// Storage capacity in GB
        capacity_gb: Option<u64>,
        /// Storage type (SSD, HDD, etc.)
        storage_type: Option<String>,
        /// Replication factor
        replication: Option<u32>,
    },
    /// AI/ML capabilities (inference, training)
    AI {
        /// Supported model types
        model_types: Vec<String>,
        /// GPU memory in GB
        gpu_memory_gb: Option<u32>,
        /// Batch processing support
        batch_processing: bool,
    },
    /// Security capabilities (encryption, authentication)
    Security {
        /// Encryption algorithms supported
        encryption_types: Vec<String>,
        /// Encryption capabilities
        encryption: bool,
        /// Authentication capabilities
        authentication: bool,
        /// Authorization capabilities
        authorization: bool,
        /// Hardware security module support
        hsm_support: bool,
        /// Key management capabilities
        key_management: bool,
    },
    /// Networking capabilities (communication, routing)
    Networking {
        /// Supported protocols
        protocols: Vec<String>,
        /// Bandwidth in Mbps
        bandwidth_mbps: Option<u32>,
        /// Load balancing support
        load_balancing: bool,
    },
    /// Health monitoring capabilities
    Health {
        /// Monitoring interval in seconds
        interval_seconds: Option<u32>,
        /// Alert mechanisms supported
        alert_types: Vec<String>,
        /// Metric collection capabilities
        metrics_collection: bool,
    },
}

/// Universal message type enumeration - consolidates all MessageType variants across modules
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UniversalMessageType {
    /// Health check requests and responses
    HealthCheck,
    /// Capability discovery messages
    CapabilityDiscovery,
    /// Service registration messages
    ServiceRegistration,
    /// Load balancing coordination
    LoadBalancing,
    /// Performance metrics reporting
    PerformanceMetrics,
    /// Error and alert notifications
    ErrorAlert,
    /// Configuration updates
    ConfigUpdate,
    /// Authentication and authorization
    Authentication,
    /// Data synchronization
    DataSync,
    /// Custom application messages
    Custom(String),
}

/// Universal service type enumeration - consolidates all ServiceType variants across modules
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UniversalServiceType {
    /// Core orchestration services
    Orchestrator,
    /// Service discovery and registry
    Discovery,
    /// Load balancing services
    LoadBalancer,
    /// Health monitoring services
    HealthMonitor,
    /// Configuration management
    ConfigManager,
    /// Security and authentication
    SecurityManager,
    /// Metrics and observability
    MetricsCollector,
    /// Data storage services
    StorageService,
    /// Compute processing services
    ComputeService,
    /// AI/ML inference services
    AIService,
    /// Network communication services
    NetworkService,
    /// Federation coordination
    FederationService,
    /// Gaming-specific services
    GamingService,
    /// BearDog security integration
    BearDogSecurity,
    /// ToadStool deployment services
    ToadStoolDeployment,
    /// Custom service types
    Custom(String),
}

/// Universal connection type enumeration - consolidates all ConnectionType variants across modules
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UniversalConnectionType {
    /// HTTP/HTTPS connections
    Http {
        /// Use TLS encryption
        secure: bool,
        /// HTTP version (1.1, 2.0, 3.0)
        version: String,
    },
    /// WebSocket connections
    WebSocket {
        /// Use secure WebSocket (WSS)
        secure: bool,
        /// Subprotocols supported
        subprotocols: Vec<String>,
    },
    /// gRPC connections
    Grpc {
        /// Use TLS
        tls_enabled: bool,
        /// Compression enabled
        compression: bool,
    },
    /// TCP socket connections
    Tcp {
        /// Keep-alive enabled
        keep_alive: bool,
        /// No-delay option (Nagle algorithm)
        no_delay: bool,
    },
    /// UDP socket connections
    Udp {
        /// Broadcast support
        broadcast: bool,
        /// Multicast support
        multicast: bool,
    },
    /// Unix domain sockets
    Unix {
        /// Socket file path
        path: String,
    },
    /// Message queue connections
    MessageQueue {
        /// Queue type (RabbitMQ, Kafka, etc.)
        queue_type: String,
        /// Durable queues
        durable: bool,
    },
    /// Peer-to-peer connections
    P2P {
        /// DHT support
        dht_enabled: bool,
        /// NAT traversal support
        nat_traversal: bool,
    },
    /// Custom connection types
    Custom(String),
}

/// Universal peer type enumeration - consolidates all PeerType variants across modules
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UniversalPeerType {
    /// Core orchestrator nodes
    Orchestrator,
    /// Service provider nodes
    ServiceProvider,
    /// Client application nodes
    Client,
    /// Load balancer nodes
    LoadBalancer,
    /// Discovery service nodes
    Discovery,
    /// Health monitoring nodes
    HealthMonitor,
    /// Security gateway nodes
    SecurityGateway,
    /// Storage nodes
    StorageNode,
    /// Compute nodes
    ComputeNode,
    /// AI/ML processing nodes
    AINode,
    /// Federation coordinator nodes
    FederationCoordinator,
    /// Gaming server nodes
    GamingServer,
    /// BearDog security nodes
    BearDogNode,
    /// ToadStool deployment nodes
    ToadStoolNode,
    /// Edge computing nodes
    EdgeNode,
    /// Relay/proxy nodes
    RelayNode,
    /// Bootstrap nodes for network discovery
    BootstrapNode,
    /// Custom peer types
    Custom(String),
}

impl CapabilityType {
    /// Get the capability type name
    pub fn type_name(&self) -> &'static str {
        match self {
            CapabilityType::Compute { .. } => "compute",
            CapabilityType::Storage { .. } => "storage",
            CapabilityType::AI { .. } => "ai",
            CapabilityType::Security { .. } => "security",
            CapabilityType::Networking { .. } => "networking",
            CapabilityType::Health { .. } => "health",
        }
    }

    /// Check if capability requires GPU support
    pub fn requires_gpu(&self) -> bool {
        match self {
            CapabilityType::Compute { gpu_enabled, .. } => *gpu_enabled,
            CapabilityType::AI { .. } => true, // AI typically benefits from GPU
            _ => false,
        }
    }

    /// Get resource requirements as a descriptive string
    pub fn resource_description(&self) -> String {
        match self {
            CapabilityType::Compute {
                cores,
                memory_gb,
                gpu_enabled,
            } => {
                format!(
                    "Compute: {} cores, {} GB RAM, GPU: {}",
                    cores.map_or("unknown".to_string(), |c| c.to_string()),
                    memory_gb.map_or("unknown".to_string(), |m| m.to_string()),
                    if *gpu_enabled { "yes" } else { "no" }
                )
            }
            CapabilityType::Storage {
                capacity_gb,
                storage_type,
                replication,
            } => {
                format!(
                    "Storage: {} GB, type: {}, replication: {}x",
                    capacity_gb.map_or("unknown".to_string(), |c| c.to_string()),
                    storage_type.as_deref().unwrap_or("unknown"),
                    replication.unwrap_or(1)
                )
            }
            CapabilityType::AI {
                model_types,
                gpu_memory_gb,
                batch_processing,
            } => {
                format!(
                    "AI: models [{}], GPU RAM: {} GB, batch: {}",
                    model_types.join(", "),
                    gpu_memory_gb.map_or("unknown".to_string(), |m| m.to_string()),
                    if *batch_processing { "yes" } else { "no" }
                )
            }
            CapabilityType::Security {
                encryption_types,
                encryption: _,
                authentication: _,
                authorization: _,
                hsm_support,
                key_management,
            } => {
                format!(
                    "Security: encryption [{}], HSM: {}, key mgmt: {}",
                    encryption_types.join(", "),
                    if *hsm_support { "yes" } else { "no" },
                    if *key_management { "yes" } else { "no" }
                )
            }
            CapabilityType::Networking {
                protocols,
                bandwidth_mbps,
                load_balancing,
            } => {
                format!(
                    "Networking: protocols [{}], bandwidth: {} Mbps, LB: {}",
                    protocols.join(", "),
                    bandwidth_mbps.map_or("unknown".to_string(), |b| b.to_string()),
                    if *load_balancing { "yes" } else { "no" }
                )
            }
            CapabilityType::Health {
                interval_seconds,
                alert_types,
                metrics_collection,
            } => {
                format!(
                    "Health: interval {} sec, alerts [{}], metrics: {}",
                    interval_seconds.map_or("unknown".to_string(), |i| i.to_string()),
                    alert_types.join(", "),
                    if *metrics_collection { "yes" } else { "no" }
                )
            }
        }
    }
}

impl Default for CapabilityType {
    fn default() -> Self {
        CapabilityType::Compute {
            cores: Some(1),
            memory_gb: Some(1),
            gpu_enabled: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_type_names() {
        assert_eq!(CapabilityType::default().type_name(), "compute");

        let storage = CapabilityType::Storage {
            capacity_gb: Some(100),
            storage_type: Some("SSD".to_string()),
            replication: Some(3),
        };
        assert_eq!(storage.type_name(), "storage");

        let ai = CapabilityType::AI {
            model_types: vec!["transformer".to_string()],
            gpu_memory_gb: Some(8),
            batch_processing: true,
        };
        assert_eq!(ai.type_name(), "ai");
    }

    #[test]
    fn test_gpu_requirements() {
        let compute_with_gpu = CapabilityType::Compute {
            cores: Some(4),
            memory_gb: Some(8),
            gpu_enabled: true,
        };
        assert!(compute_with_gpu.requires_gpu());

        let compute_without_gpu = CapabilityType::Compute {
            cores: Some(2),
            memory_gb: Some(4),
            gpu_enabled: false,
        };
        assert!(!compute_without_gpu.requires_gpu());

        let ai = CapabilityType::AI {
            model_types: vec!["llm".to_string()],
            gpu_memory_gb: Some(16),
            batch_processing: true,
        };
        assert!(ai.requires_gpu());
    }

    #[test]
    fn test_resource_descriptions() {
        let compute = CapabilityType::Compute {
            cores: Some(8),
            memory_gb: Some(16),
            gpu_enabled: true,
        };
        let desc = compute.resource_description();
        assert!(desc.contains("8 cores"));
        assert!(desc.contains("16 GB RAM"));
        assert!(desc.contains("GPU: yes"));

        let storage = CapabilityType::Storage {
            capacity_gb: Some(1000),
            storage_type: Some("NVMe".to_string()),
            replication: Some(2),
        };
        let desc = storage.resource_description();
        assert!(desc.contains("1000 GB"));
        assert!(desc.contains("NVMe"));
        assert!(desc.contains("2x"));
    }

    #[test]
    fn test_universal_message_types() -> SongbirdResult<()> {
        let health_check = UniversalMessageType::HealthCheck;
        let custom = UniversalMessageType::Custom("MyCustomType".to_string());

        // Test serialization
        let serialized = serde_json::to_string(&health_check).map_err(|e| {
            Box::new(songbird_errors::SongbirdError::operation_error(format!(
                "Operation failed: {}",
                e
            ))) as Box<dyn std::error::Error>
        })?;
        assert!(serialized.contains("HealthCheck"));

        let serialized_custom = serde_json::to_string(&custom).map_err(|e| {
            Box::new(songbird_errors::SongbirdError::operation_error(format!(
                "Operation failed: {}",
                e
            ))) as Box<dyn std::error::Error>
        })?;
        assert!(serialized_custom.contains("MyCustomType"));
        Ok(SongbirdResponse::success(()))
    }

    #[test]
    fn test_universal_service_types() {
        let orchestrator = UniversalServiceType::Orchestrator;
        let custom_service = UniversalServiceType::Custom("MyService".to_string());

        assert_ne!(orchestrator, custom_service);
        assert_eq!(orchestrator, UniversalServiceType::Orchestrator);
    }

    #[test]
    fn test_universal_connection_types() -> SongbirdResult<()> {
        let http = UniversalConnectionType::Http {
            secure: true,
            version: "2.0".to_string(),
        };

        let websocket = UniversalConnectionType::WebSocket {
            secure: true,
            subprotocols: vec!["chat".to_string(), "data".to_string()],
        };

        assert_ne!(http, websocket);

        // Test pattern matching
        match http {
            UniversalConnectionType::Http { secure, version } => {
                assert!(secure);
                assert_eq!(version, "2.0");
            }
            _ => {
                return Err(Box::new(songbird_errors::SongbirdError::validation_error(
                    "Expected HTTP connection type",
                )))
            }
        }
        Ok(SongbirdResponse::success(()))
    }

    #[test]
    fn test_universal_peer_types() {
        let orchestrator = UniversalPeerType::Orchestrator;
        let custom_peer = UniversalPeerType::Custom("SpecialNode".to_string());

        // Test equality
        assert_eq!(orchestrator, UniversalPeerType::Orchestrator);
        assert_ne!(orchestrator, custom_peer);

        // Test in collections
        let mut peer_set = std::collections::HashSet::new();
        peer_set.insert(orchestrator.clone());
        peer_set.insert(custom_peer.clone());

        assert_eq!(peer_set.len(), 2);
        assert!(peer_set.contains(&UniversalPeerType::Orchestrator));
    }
}
