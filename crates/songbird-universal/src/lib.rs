//! # 🌐 Songbird Universal Orchestration
//!
//! **UNIFIED UNIVERSAL ADAPTER SYSTEM** ✅
//!
//! This crate provides universal, vendor-agnostic orchestration capabilities
//! that can work with any primal or service provider through capability-based discovery.

#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]

// ============================================================================
// UNIFIED ADAPTER SYSTEM - Primary Interface
// ============================================================================

pub mod capabilities;
pub mod discovery;
pub mod sovereignty;
pub mod traits;
pub mod types;
pub mod unified_adapter;

// Re-export the unified adapter as the primary interface
pub use unified_adapter::{
    CapabilityRegistry, RegistryStats, ServiceConnection, UnifiedAdapterConfig,
    UnifiedUniversalAdapter, UniversalAdapterError,
};

// Re-export modernized sovereignty system
pub use sovereignty::{
    PathSegment, RoutingPath, SovereigntyAdapterConfig, SovereigntyAwareAdapter,
    SovereigntyAwareRoutingDecision,
};

// Re-export core types
pub use types::*;

/// Create a new unified universal adapter with default configuration
pub fn create_universal_adapter() -> UnifiedUniversalAdapter {
    UnifiedUniversalAdapter::new()
}

/// Create a new unified universal adapter with custom configuration
pub fn create_universal_adapter_with_config(
    config: UnifiedAdapterConfig,
) -> UnifiedUniversalAdapter {
    UnifiedUniversalAdapter::with_config(config)
}
