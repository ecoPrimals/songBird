//! Universal Primal Discovery System
//!
//! This module provides comprehensive discovery capabilities for Universal Primals
//! across the network, including auto-detection, capability matching, and
//! real-time primal enumeration.
//!
//! ## Refactored Architecture
//!
//! The Universal Primal discovery system is organized into focused modules:
//! - `types` - Core types (PrimalNode, DiscoveredPrimal, DiscoveryMethod)
//! - `engine` - Main PrimalDiscoveryEngine and coordination logic  
//! - `network_scan` - Network scanning and probing functionality
//! - `legacy` - Legacy primal service support for backward compatibility
//! - `parsing` - Capability parsing and metadata extraction utilities
//! - `ecosystem` - Real ecosystem primal discovery (connects to ../beardog, etc.)

pub mod ecosystem;
pub mod engine;
pub mod legacy;
pub mod parsing;
pub mod types;

// Re-export main types for backward compatibility
pub use ecosystem::{create_universal_context, EcosystemDiscovery, EcosystemDiscoveryConfig};
pub use engine::{DiscoverySummary, PrimalDiscoveryEngine};
pub use legacy::{
    discover_from_well_known_locations, get_configured_service_endpoints,
    query_universal_primal_services, register_configured_primals,
};
// Network scan functionality removed - using legacy discovery methods
pub use parsing::{
    discover_capabilities_from_service, extract_metadata_from_info,
    get_default_capabilities_for_type, infer_primal_type_from_capabilities,
    parse_primal_type_from_string,
};
pub use types::{
    DiscoveredPrimal, DiscoveryConfig, DiscoveryMethod, DiscoveryResult, DiscoveryStats, PrimalNode,
};

// Legacy compatibility exports
use crate::errors::PrimalResult;

/// Legacy function for backward compatibility - creates and starts discovery engine
pub async fn discover_universal_primals(
) -> PrimalResult<Vec<crate::discovery::types::DiscoveredPrimal>> {
    use songbird_config::config::hardcoded_elimination::PrimalConfig;

    let mut engine = PrimalDiscoveryEngine::new(PrimalConfig::default());
    engine.start_discovery().await?;
    Ok(engine
        .get_discovered_primals()
        .into_iter()
        .cloned()
        .collect())
}

/// Discover ecosystem primals using the new ecosystem discovery system
pub async fn discover_ecosystem_primals(
) -> PrimalResult<Vec<crate::discovery::types::DiscoveredPrimal>> {
    let ecosystem_discovery = EcosystemDiscovery::new(EcosystemDiscoveryConfig::default());
    ecosystem_discovery.discover_ecosystem_primals().await
}
