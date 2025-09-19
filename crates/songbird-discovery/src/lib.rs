//! # Songbird Discovery
//!
//! Universal service discovery and network scanning capabilities providing automatic
//! detection, registration, and monitoring of services across heterogeneous networks.
//!
//! ## Features
//!
//! - **Multi-Protocol Discovery**: Support for multiple discovery protocols
//! - **Network Scanning**: Comprehensive network topology discovery
//! - **Service Registration**: Dynamic service registration and deregistration
//! - **Health Monitoring**: Continuous health monitoring of discovered services
//! - **Load Balancing Integration**: Health-aware load balancing support
//! - **Cross-Platform Support**: Works across different operating systems
//! - **Primal Coordination**: Automatic discovery and coordination with Primals
//! - **Legacy Protocol Support**: Discovery of legacy network services
//!
//! ## Architecture
//!
//! The discovery crate is organized into focused modules:
//!
//! - `discovery`: Core discovery engine and service management
//! - `traits`: Common traits and interfaces for discovery implementations
//!
//! ## Usage
//!
//! ```rust,no_run
//! use songbird_discovery::{
//!     discovery::{DiscoveryConfig, ServiceDiscovery},
//!     traits::{ServiceInfo, ServiceStatus},
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize discovery configuration
//!     let config = DiscoveryConfig::default();
//!     
//!     // Create discovery service
//!     let discovery = songbird_discovery::discovery::ServiceDiscoveryFactory::create(&config)?;
//!     
//!     // Discover services
//!     let services = discovery.discover_services(None).await?;
//!     println!("Discovered {} services", services.len());
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Discovery Backends
//!
//! Multiple discovery backends are supported:
//!
//! - **Static Configuration**: File-based service definitions
//! - **DNS-SD**: DNS Service Discovery (Bonjour/Avahi)
//! - **Consul**: HashiCorp Consul service discovery
//! - **Kubernetes**: Kubernetes service discovery
//! - **etcd**: etcd-based service registry
//! - **Network Scanning**: Active network scanning and probing
//! - **mDNS**: Multicast DNS service discovery
//!
//! ## Service Types
//!
//! Discovery supports various service types:
//!
//! - **HTTP/HTTPS Services**: Web services and APIs
//! - **TCP/UDP Services**: Network services
//! - **Database Services**: Database connections
//! - **Message Queues**: Message broker services
//! - **Gaming Services**: Gaming servers and lobbies
//! - **Primal Services**: Ecosystem Primal services
//! - **Custom Services**: User-defined service types
//!
//! ## Health Monitoring
//!
//! Continuous health monitoring capabilities:
//!
//! - **Health Checks**: Configurable health check protocols
//! - **Failure Detection**: Automatic failure detection and recovery
//! - **Status Reporting**: Real-time service status reporting
//! - **Metrics Collection**: Performance and availability metrics
//! - **Alerting**: Health-based alerting and notifications
//!
//! ## Performance
//!
//! Optimized for high-performance discovery:
//!
//! - **Concurrent Discovery**: Parallel service discovery
//! - **Caching**: Intelligent caching of discovery results
//! - **Rate Limiting**: Respectful network scanning
//! - **Incremental Updates**: Efficient delta-based updates
//!
//! ## Error Handling
//!
//! All discovery operations return `Result<T>` with detailed
//! error information including network errors, timeout handling, and recovery
//! suggestions for common discovery failures.

pub mod discovery;
pub mod traits;

// Re-export universal discovery functionality
pub use discovery::{
    DiscoveryConfig, ServiceInstance, SongbirdDiscovery, StaticServiceDiscovery,
    UniversalContainerOrchestration, UniversalDiscoveryFactory, UniversalServiceDiscovery,
};

// Re-export commonly used types from discovery
pub use discovery::{
    ComputeResources, DatasetInfo, FederationHealth, FederationMessage, FederationStats,
    NetworkMeasurement, NetworkPartition, NetworkTopology, NodeId, NodeInfo, NodeType,
    ResourceQuery, ResourceUpdate, ResourceUsage, StorageInfo, TrustLevel,
};

// Re-export traits
pub use traits::{
    CommunicationLayer, ComposablePlugin, ComposedSystem, CompositionPlan, ConfigProvider,
    HealthCheck, HealthMonitor, HealthState, HealthStatus, PluginCapability, PluginHealth,
    PluginRegistry, PluginRequirement, ServiceInfo, ServiceStatus, SystemHealth,
};

// Re-export service discovery trait
pub use traits::ServiceDiscovery;
