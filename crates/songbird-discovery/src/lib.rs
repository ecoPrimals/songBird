// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![warn(missing_docs)]

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
#![allow(
    clippy::module_name_repetitions,
    clippy::items_after_statements,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::unused_async,
    clippy::unused_self,
    clippy::too_many_lines,
    clippy::manual_let_else,
    clippy::struct_excessive_bools,
    reason = "discovery crate: large surface; doc and style exceptions during consolidation"
)]
#![cfg_attr(
    test,
    allow(
        deprecated,
        dead_code,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::await_holding_lock,
        clippy::float_cmp,
        clippy::absurd_extreme_comparisons,
        clippy::nonminimal_bool,
        clippy::needless_collect,
        clippy::used_underscore_binding,
        clippy::overly_complex_bool_expr,
        clippy::assertions_on_constants,
        clippy::unreadable_literal,
        clippy::empty_line_after_doc_comments,
        clippy::field_reassign_with_default,
        clippy::unnecessary_wraps,
        clippy::no_effect_underscore_binding,
        clippy::return_self_not_must_use,
        clippy::duplicated_attributes,
        clippy::needless_pass_by_value,
        clippy::must_use_candidate,
        clippy::missing_panics_doc,
        clippy::missing_errors_doc,
        clippy::doc_markdown,
        clippy::wildcard_imports,
        clippy::enum_glob_use,
        unused_imports,
        unused_variables,
        clippy::unused_self,
        clippy::unnecessary_cast,
        clippy::items_after_test_module,
        clippy::clone_on_ref_ptr,
        clippy::default_trait_access,
        clippy::needless_range_loop,
        clippy::similar_names,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::unnecessary_unwrap,
        clippy::ignore_without_reason,
        clippy::case_sensitive_file_extension_comparisons,
    )
)]
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

#[allow(missing_docs, reason = "discovery submodule; primary docs on crate root and re-exports")]
pub mod beardog_birdsong_provider; // NEW: BearDog BirdSong encryption provider (Jan 3, 2026)
#[allow(missing_docs, reason = "discovery submodule; primary docs on crate root and re-exports")]
pub mod birdsong; // NEW: BirdSong encrypted discovery (REFACTORED v3.22.0 - Feb 5, 2026)
#[allow(missing_docs, reason = "discovery submodule; primary docs on crate root and re-exports")]
pub mod conversion;
#[allow(missing_docs, reason = "discovery submodule; primary docs on crate root and re-exports")]
pub mod dark_forest_beacon; // ✅ NEW (Feb 3, 2026): Dark Forest Beacon Genetics (TRUE encrypted discovery, zero metadata leakage)
#[allow(missing_docs, reason = "discovery submodule; primary docs on crate root and re-exports")]
pub mod discovery;
#[allow(missing_docs, reason = "discovery submodule; primary docs on crate root and re-exports")]
pub mod discovery_packet; // NEW: Enhanced discovery with genetic lineage
#[allow(missing_docs, reason = "discovery submodule; primary docs on crate root and re-exports")]
pub mod discovery_stats;
#[allow(missing_docs, reason = "discovery submodule; primary docs on crate root and re-exports")]
pub mod lineage_discovery; // NEW: Lineage-aware mDNS backend
#[allow(missing_docs, reason = "discovery submodule; primary docs on crate root and re-exports")]
pub mod primal_self_knowledge; // Primal self-knowledge architecture (capability-first)
#[allow(missing_docs, reason = "discovery submodule; primary docs on crate root and re-exports")]
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
#[allow(missing_docs, reason = "anonymous discovery submodule; module docs above")]
pub mod anonymous;

/// Unit and integration tests for self-filtering (v3.10.2 - Jan 5, 2026)
#[cfg(test)]
mod tests_self_filtering;

// 🌐 Federation-aware discovery (Phase 2)
// Deferred: Optional P2 feature for advanced federation capabilities
// pub mod federation_aware_discovery;
// pub mod migration;

// Re-export universal discovery functionality
/// BearDog-backed `BirdSong` provider for encrypted discovery.
pub use beardog_birdsong_provider::BearDogBirdSongProvider; // NEW (Jan 3, 2026)
/// `BirdSong` encryption types and processor for discovery traffic.
pub use birdsong::{BirdSongConfig, BirdSongEncryption, BirdSongProcessor}; // REFACTORED v3.22.0
/// Factory for creating discovery backends from configuration or auto-detection.
pub use discovery::UniversalDiscoveryFactory;
/// Discovery packet wire format, errors, and identity attestations.
pub use discovery_packet::{DiscoveryError, DiscoveryPacket, IdentityAttestation}; // NEW
/// Runtime discovery statistics, status, and network snapshot types.
pub use discovery_stats::{
    DiscoveryStats, DiscoveryStatsSnapshot, DiscoveryStatus, DiscoveryStatusManager, NetworkInfo,
};
/// mDNS discovery backend that respects genetic lineage.
pub use lineage_discovery::LineageServiceDiscovery; // NEW
/// Core discovery traits, service metadata, and configuration.
pub use traits::{DiscoveryConfig, ServiceDiscovery, ServiceInfo, ServiceStatus}; // NEW (Jan 5, 2026)

// Note: federation_aware_discovery and migration modules deferred to Phase 2
// These are optional P2 features for advanced federation capabilities

// Export consolidated traits
/// Provider hook for feature-flag driven discovery behavior.
pub use traits::feature_flags::FeatureFlagProvider;
