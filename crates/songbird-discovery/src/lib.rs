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

#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)] // Allow DarkForestBeacon naming
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
//! use songbird_discovery::UniversalDiscoveryFactory;
//! use songbird_discovery::traits::{ServiceDiscovery, ServiceQuery};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create discovery service with auto-detection
//!     let discovery = UniversalDiscoveryFactory::create_auto_detect().await?;
//!
//!     // Discover services
//!     let query = ServiceQuery::default();
//!     let services = discovery.discover_services(&query).await?;
//!     println!("Discovered {} services", services.len());
//!
//!     Ok(())
//! # }
//! ```
//!
//! ### Federation-Aware Discovery
//! ```rust,no_run
//! use songbird_discovery::UniversalDiscoveryFactory;
//! // Note: Federation-aware discovery capabilities are built into the discovery system
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create discovery with auto-detection (includes federation capabilities)
//!     let discovery = UniversalDiscoveryFactory::create_auto_detect().await?;
//!     println!("Discovery created with federation awareness");
//!
//!     Ok(())
//! # }
//! ```
//!
//! ### Migration from Old Federation System
//! ```rust,no_run
//! use songbird_discovery::UniversalDiscoveryFactory;
//! // Note: Migration helpers for legacy federation systems
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Modern discovery with federation built-in
//!     let discovery = UniversalDiscoveryFactory::create_auto_detect().await?;
//!     println!("Migrated to modern discovery system with federation capabilities");
//!
//!     Ok(())
//! # }
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

pub mod beardog_birdsong_provider; // NEW: BearDog BirdSong encryption provider (Jan 3, 2026)
pub mod birdsong_integration; // NEW: BirdSong encrypted discovery
pub mod conversion;
pub mod dark_forest_beacon; // ✅ NEW (Feb 3, 2026): Dark Forest Beacon Genetics (TRUE encrypted discovery, zero metadata leakage)
pub mod discovery;
pub mod discovery_packet; // NEW: Enhanced discovery with genetic lineage
pub mod discovery_stats;
pub mod lineage_discovery; // NEW: Lineage-aware mDNS backend
pub mod traits; // NEW: Discovery statistics and observability (Jan 5, 2026)

/// Anonymous discovery protocol (v3.22.1) - Refactored modules
///
/// Implements secure anonymous discovery with UDP multicast.
/// Split into domain-driven modules for maintainability and testability.
///
/// ## Modules
/// - `messages` - Message types and serialization (✅ Complete)
/// - `peer` - Peer discovery and management (✅ Complete)
/// - `broadcaster` - Broadcasting logic (✅ Complete)
/// - `listener` - Listening and processing (✅ Complete)
///
/// **Refactoring Complete (v3.22.1)**: 1402 lines → 4 focused modules
/// - Old `anonymous_discovery.rs` removed (Jan 12, 2026)
/// - All imports updated to use `anonymous::` module
/// - File size compliance achieved (<1000 lines per file)
pub mod anonymous;

/// Unit and integration tests for self-filtering (v3.10.2 - Jan 5, 2026)
#[cfg(test)]
mod tests_self_filtering;

// 🌐 Federation-aware discovery (Phase 2)
// Deferred: Optional P2 feature for advanced federation capabilities
// pub mod federation_aware_discovery;
// pub mod migration;

// Re-export universal discovery functionality
pub use beardog_birdsong_provider::BearDogBirdSongProvider; // NEW (Jan 3, 2026)
pub use birdsong_integration::{BirdSongConfig, BirdSongEncryption, BirdSongProcessor}; // NEW
pub use discovery::UniversalDiscoveryFactory;
pub use discovery_packet::{DiscoveryError, DiscoveryPacket, IdentityAttestation}; // NEW
pub use discovery_stats::{
    DiscoveryStats, DiscoveryStatsSnapshot, DiscoveryStatus, DiscoveryStatusManager, NetworkInfo,
};
pub use lineage_discovery::LineageServiceDiscovery; // NEW
pub use traits::{DiscoveryConfig, ServiceDiscovery, ServiceInfo, ServiceStatus}; // NEW (Jan 5, 2026)

// Note: federation_aware_discovery and migration modules deferred to Phase 2
// These are optional P2 features for advanced federation capabilities

// Export consolidated traits
pub use traits::feature_flags::FeatureFlagProvider;
