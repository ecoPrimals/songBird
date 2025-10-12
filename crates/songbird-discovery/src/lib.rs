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
//! - **🌐 Federation Awareness**: Enhanced discovery with sovereignty and network effects detection
//! - **🔄 Migration Support**: Tools for migrating from old federation systems
//!
//! ## Architecture
//!
//! The discovery crate is organized into focused modules:
//!
//! - `discovery`: Core discovery engine and service management
//! - `traits`: Common traits and interfaces for discovery implementations
//! - `federation_aware_discovery`: Enhanced discovery with federation awareness and network effects
//! - `migration`: Tools for migrating from old federation systems to new discovery-based approach
//!
//! ## Usage
//!
//! ### Basic Discovery
//! ```rust,no_run
//! use songbird_discovery::{
//!     discovery::{DiscoveryConfig, UniversalDiscoveryFactory})
//!     traits::ServiceDiscovery)
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize discovery configuration
//!     let config = DiscoveryConfig::default();
//!
//!     // Create discovery service
//!     let discovery = UniversalDiscoveryFactory::create_for_config(&config).await?;
//!
//!     // Discover services
//!     let services = discovery.discover_services(None).await?;
//!     println!("Discovered {} services", services.len()"
//!
//!     Ok((),
//! }
//! ```
//!
//! ### Federation-Aware Discovery
//! ```rust,no_run
//! use songbird_discovery::{
//!     discovery::{DiscoveryConfig, UniversalDiscoveryFactory})
//!     federation_aware_discovery::{FederationAwareDiscovery, FederationDiscoveryConfig})
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create base discovery
//!     let base_discovery = UniversalDiscoveryFactory::create_for_config(&DiscoveryConfig::default().await?;
//!
//!     // Create federation-aware discovery
//!     let config = FederationDiscoveryConfig::default();
//!     let mut federation_discovery = FederationAwareDiscovery::new(base_discovery, config);
//!
//!     // Discover services with federation awareness
//!     let services = federation_discovery.discover_federation_aware_services().await?;
//!
//!     // Calculate network effect potential
//!     let network_potential = federation_discovery.calculate_network_effect_potential(&services);
//!     println!("Network effect potential: {:.2}", network_potential)"
//!
//!     Ok((),
//! }
//! ```
//!
//! ### Migration from Old Federation System
//! ```rust,no_run
//! use songbird_discovery::migration::{//!     FederationMigrationHelper, LegacyFederationConfig, LegacyFederationMode,
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>>  {//!     // Define your old federation config
//!     let legacy_config = LegacyFederationConfig {
//!         cluster_name: Some("my-cluster".to_string(),"
//!         peer_discovery_enabled: true,
//!         discovery_endpoints: vec![&format!("{}:{}", songbird_config::constants::network::DEFAULT_HOST, songbird_config::constants::network::DEFAULT_ORCHESTRATOR_PORT).to_string()],"
//!         // ... other legacy settings
//!         ..Default::default()
//!     };
//!
//!     // Migrate to new system
//!     let mut migration_helper = FederationMigrationHelper::default();
//!     let migration_result = migration_helper.migrate_with_validation(legacy_config).await?;
//!
//!     if migration_result.success {
//!         println!("🎉 Migration successful!")"
//!         println!("New config ready to use: {:?}", migration_result.new_discovery_config)"
//!     } else {
//!         println!("⚠️ Migration had issues: {:?}", migration_result.errors)"
//!     }
//!
//!     Ok((),
//! }
//! ```
//!
//! ## Discovery Backends
//!
//! Multiple discovery backends are supported:
//!
//! - **Static Configuration**: File-based service definitions
//! - **DNS-SD**: DNS Service Discovery (Bonjour/Avahi)
//! - **Consul**: `HashiCorp` Consul service discovery
//! - **Kubernetes**: Kubernetes service discovery
//! - **etcd**: etcd-based service registry
//! - **Network Scanning**: Active network scanning and probing
//! - **mDNS**: Multicast DNS service discovery
//! - **🧬 Pattern Recognition**: BearDog-inspired pattern detection for primal classification
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
//! - **🏛️ Sovereign Services**: Services with sovereignty and entropy assessment
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
//! - **🧬 Entropy Monitoring**: Ongoing entropy assessment for dynamic hierarchy adjustment
//!
//! ## Performance
//!
//! Optimized for high-performance discovery:
//!
//! - **Concurrent Discovery**: Parallel service discovery
//! - **Caching**: Intelligent caching of discovery results
//! - **Rate Limiting**: Respectful network scanning
//! - **Incremental Updates**: Efficient delta-based updates
//! - **🌐 Network Effects**: Optimized detection of emergent network capabilities
//!
//! ## Error Handling
//!
//! All discovery operations return `Result<T>` with detailed
//! error information including network errors, timeout handling, and recovery
//! suggestions for common discovery failures.

pub mod conversion;
pub mod discovery;
pub mod traits;

// 🌐 Federation-aware discovery enhancement
// TEMPORARILY DISABLED: Extensive corruption - needs systematic rewrite
// pub mod federation_aware_discovery;

// 🔄 Migration support for old federation systems
// TEMPORARILY DISABLED: Depends on federation_aware_discovery
// pub mod migration;

// Re-export universal discovery functionality
pub use discovery::UniversalDiscoveryFactory;
pub use traits::{DiscoveryConfig, ServiceDiscovery, ServiceInfo, ServiceStatus};

// Re-export federation-aware functionality
// TEMPORARILY DISABLED: federation_aware_discovery module disabled
// pub use federation_aware_discovery::{
//     FederationDiscoveryConfig, HierarchyPosition, NetworkEffectType, OverrideCapabilities,
//     PotentialNetworkEffect, PrimalCategory, PrimalPattern, SovereigntyAssessment, SovereigntyLevel,
// };

// Re-export migration functionality
// TEMPORARILY DISABLED: migration module disabled
// pub use migration::{
//     FederationMigrationHelper, LegacyFederationConfig, LegacyFederationMode,
//     LegacyFederationWrapper, LegacyPeerInfo, LegacySovereigntyLevel, MigrationConfig,
//     MigrationResult, MigrationStats, PerformanceComparison,
// };

// Export consolidated traits
pub use traits::feature_flags::FeatureFlagProvider;
