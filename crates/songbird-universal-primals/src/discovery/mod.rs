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

pub mod engine;
pub mod legacy;
pub mod network_scan;
pub mod parsing;
pub mod types;

// Re-export main types for backward compatibility
pub use engine::{DiscoverySummary, PrimalDiscoveryEngine};
pub use legacy::{
    discover_from_well_known_locations, get_configured_service_endpoints,
    query_universal_primal_services, register_configured_primals,
};
pub use network_scan::{
    get_common_primal_ports, perform_network_scan, probe_primal_endpoint, scan_for_primal_type,
    scan_network_range, test_endpoint_connectivity,
};
pub use parsing::{
    extract_metadata_from_info, get_default_capabilities_for_type, 
    infer_primal_type_from_capabilities, parse_primal_type_from_string,
    discover_capabilities_from_service,
};
pub use types::{
    DiscoveredPrimal, DiscoveryConfig, DiscoveryMethod, DiscoveryResult, DiscoveryStats, PrimalNode,
};

// Legacy compatibility exports
use crate::errors::PrimalResult;

/// Legacy function for backward compatibility - creates and starts discovery engine
pub async fn start_comprehensive_primal_discovery(
    config: songbird_config::config::hardcoded_elimination::PrimalConfig,
) -> PrimalResult<Vec<DiscoveredPrimal>> {
    let mut engine = PrimalDiscoveryEngine::new(config);
    engine.start_discovery().await?;
    Ok(engine
        .get_discovered_primals()
        .into_iter()
        .cloned()
        .collect())
}
