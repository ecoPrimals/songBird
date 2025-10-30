//! # 🔧 Canonical Adapters System
//!
//! **SINGLE SOURCE OF TRUTH FOR ALL ADAPTERS** ✅
//!
//! This module provides the consolidated adapter system that replaces
//! all fragmented adapter implementations across the Songbird ecosystem.

pub mod canonical; // NEW: Consolidated canonical adapter

// Re-export the canonical adapter system (PREFERRED,
pub use canonical::{
    create_adapter_request,
    // Convenience functions
    create_canonical_adapter,
    // Configuration types
    CanonicalAdapterConfig,
    CanonicalAdapterMetrics,

    CanonicalAdapterRequest,
    CanonicalAdapterResponse,
    CanonicalCircuitBreakerConfig,
    CanonicalCircuitState,
    CanonicalDiscoveryConfig,
    CanonicalHealthCheckConfig,
    CanonicalLoadBalancingConfig,
    CanonicalLoadBalancingStrategy,
    CanonicalMonitoringConfig,

    // Protocol handling
    CanonicalProtocolHandler,

    // Supporting types
    CanonicalRegisteredService,
    CanonicalRequestPriority,
    CanonicalRetryConfig,
    CanonicalServicePerformance,
    CanonicalTimeoutConfig,
    // Main adapter
    CanonicalUniversalAdapter,
};

// ============================================================================
// CANONICAL ADAPTERS - Use these directly
// ============================================================================

// Migration compatibility aliases have been removed.
// Use the canonical types directly:
// - CanonicalUniversalAdapter instead of UniversalAdapter
// - CanonicalAdapterConfig instead of AdapterConfig
// - CanonicalAdapterRequest instead of AdapterRequest
// - CanonicalAdapterResponse instead of AdapterResponse
// - CanonicalRequestPriority instead of RequestPriority
// - CanonicalLoadBalancingStrategy instead of LoadBalancingStrategy

// ============================================================================
// ADAPTERS PRELUDE
// ============================================================================

/// Prelude module for easy importing of canonical adapter types
pub mod prelude {
    pub use super::canonical::{
        create_adapter_request, create_canonical_adapter, CanonicalAdapterConfig,
        CanonicalAdapterRequest, CanonicalAdapterResponse, CanonicalLoadBalancingStrategy,
        CanonicalRequestPriority, CanonicalUniversalAdapter,
    };
}
