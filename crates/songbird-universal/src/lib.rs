//! # 🌐 Songbird Universal Orchestration
//!
//! **UNIFIED UNIVERSAL ADAPTER SYSTEM** ✅
//!
//! This crate provides universal, vendor-agnostic orchestration capabilities
//! that can work with any primal or service provider through capability-based discovery.

#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![allow(clippy::significant_drop_tightening)] // Will optimize in Phase 2
#![allow(clippy::cognitive_complexity)] // Will refactor in Phase 2
#![allow(clippy::if_same_then_else)] // Will refactor in Phase 2
#![allow(clippy::needless_borrow)] // Will optimize in Phase 2
#![allow(clippy::missing_fields_in_debug)] // Will complete Debug impls in Phase 2

// ============================================================================
// UNIFIED ADAPTER SYSTEM - Primary Interface
// ============================================================================

pub mod adapters;
pub mod capabilities;
pub mod circuit_breaker;
pub mod discovery;
pub mod federated_capability_adapter;
pub mod load_balancer;
pub mod sovereignty;
pub mod traits;
pub mod types;
pub mod unified_adapter;

// NOTE: zero_knowledge_bootstrap removed - will be properly reimplemented in Phase 2B
// with modern idiomatic Rust patterns. Feature tracked in IMPLEMENTATION_CHECKLIST.md

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

// Explicitly re-export capability types to avoid confusion
pub use capabilities::Capability as CapabilityDefinition;
pub use types::DiscoveredCapability;

/// Create a new unified universal adapter with default configuration
#[must_use]
pub fn create_universal_adapter() -> UnifiedUniversalAdapter {
    UnifiedUniversalAdapter::new()
}

/// Create a new unified universal adapter with custom configuration
#[must_use]
pub fn create_universal_adapter_with_config(
    config: UnifiedAdapterConfig,
) -> UnifiedUniversalAdapter {
    UnifiedUniversalAdapter::with_config(config)
}
