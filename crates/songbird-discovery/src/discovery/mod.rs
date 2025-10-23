//! Service Discovery Module
//!
//! **MODERNIZED UNIVERSAL DISCOVERY SYSTEM** ✅
//!
//! ## Universal Architecture
//!
//! The service discovery system now uses universal capability-based adapters:
//! - `UniversalDiscoveryFactory` - Auto-detects environment and creates appropriate universal adapters
//! - `UniversalServiceDiscoveryAdapter` - Replaces hardcoded backend implementations
//! - `FederationAwareDiscovery` - Enhanced discovery with built-in federation capabilities
//! - `StaticServiceDiscovery` - Still available for development and testing
//!
//! ## Migration from Legacy Backends
//!
//! **DEPRECATED** (use universal factory instead):
//! - ~~`KubernetesServiceDiscovery`~~ → `UniversalDiscoveryFactory::create_kubernetes_universal()`
//! - ~~`ConsulServiceDiscovery`~~ → `UniversalDiscoveryFactory::create_consul_universal()`
//! - ~~`StaticServiceDiscovery`~~ → Use via `UniversalDiscoveryFactory::create_for_config()`
//!
//! ## Federation Consolidation
//!
//! **CONSOLIDATED** (federation capabilities integrated):
//! - ~~`songbird-federation` crate~~ → `FederationAwareDiscovery` system
//! - Multi-node coordination, leader election, distributed locking
//! - Sovereignty-aware routing and network effects optimization
//! - All federation features available through enhanced discovery

// Type alias for convenience
type Result<T> = songbird_types::SongbirdResult<T>;

// Core discovery types and traits
pub mod core;

// Backend implementations
pub mod backends;

// Universal factory for creating discovery instances (MODERNIZED)
pub mod factory;

// Enhanced discovery with federation capabilities (NEW)
// TEMP DISABLED: enhanced_discovery has extensive string corruption (622 lines), needs full rewrite
// pub mod enhanced_discovery;

// Existing submodules (already well-organized)
pub mod config;
// TEMP DISABLED: monitoring, network, resources have persistent string corruption (needs systematic rewrite)
// pub mod monitoring;
// pub mod network;
// pub mod resources;
// TEMP DISABLED: songbird_discovery depends on the above broken modules
// pub mod songbird_discovery;
pub mod types;

// UNIVERSAL DISCOVERY ARCHITECTURE:
// =================================
// Discovery services now use universal capability-based adapters that can:
// - Auto-detect Kubernetes, Consul, Docker, and other environments
// - Dynamically register environment-specific capabilities
// - Provide unified interface regardless of backend
// - Eliminate hardcoded backend implementations
// - Include federation capabilities (multi-node coordination, sovereignty)
//
// Security and federation discovery operations are handled by:
// - songbird-security crate for trust verification and certificate validation
// - Enhanced discovery system for multi-node coordination and federation
// - songbird-universal crate for universal capability adapters

// ============================================================================
// MODERN EXPORTS - Use these instead of deprecated ones
// ============================================================================

pub use factory::UniversalDiscoveryFactory;

// ============================================================================
// MIGRATION GUIDE - CANONICAL PROVIDER SYSTEM
// ============================================================================

/// Migration examples for updating to canonical Provider traits
pub mod migration_examples {
    use super::{Result, UniversalDiscoveryFactory};

    /// Example: Migrating to canonical Provider-based discovery
    ///
    /// ```rust
    /// // OLD (deprecated):
    /// // let discovery = KubernetesServiceDiscovery::new().await?;
    /// // let discovery = ConsulServiceDiscovery::new().await?;
    ///
    /// // NEW (canonical Provider system):
    /// use songbird_types::traits::canonical::DiscoveryProvider;
    /// let discovery = UniversalDiscoveryFactory::create_auto_detect().await?;
    /// ```
    pub async fn migrate_to_canonical_providers() -> Result<Box<dyn crate::traits::ServiceDiscovery>>
    {
        UniversalDiscoveryFactory::create_auto_detect().await
    }

    // DISABLED: federation_aware_discovery module temporarily disabled
    /*
    /// Example: Using federation-aware discovery with canonical traits
    ///
    /// ```rust
    /// use songbird_discovery::{FederationAwareDiscovery, FederationConfig};
    /// use songbird_types::traits::canonical::DiscoveryProvider;
    ///
    /// let config = FederationConfig::default();
    /// let discovery = FederationAwareDiscovery::new(config).await?;
    /// ```
    pub async fn migrate_federation_discovery() -> crate::Result<Box<dyn songbird_types::traits::canonical::DiscoveryProvider>> {
        use crate::federation_aware_discovery::{FederationAwareDiscovery, FederationConfig};
        let config = FederationConfig::default();
        FederationAwareDiscovery::new(config).await.map(|d| Box::new(d) as Box<dyn songbird_types::traits::canonical::DiscoveryProvider>)
    }
    */
}
