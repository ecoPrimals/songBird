//! # 🔧 Canonical Adapters System
//!
//! **SINGLE SOURCE OF TRUTH FOR ALL ADAPTERS** ✅
//!
//! This module provides the consolidated adapter system that replaces
//! all fragmented adapter implementations across the Songbird ecosystem.

pub mod canonical; // NEW: Consolidated canonical adapter

// Re-export the canonical adapter system (PREFERRED,
pub use canonical::{
    // Main adapter
    CanonicalUniversalAdapter,
    
    // Configuration types
    CanonicalAdapterConfig,
    CanonicalDiscoveryConfig,
    CanonicalLoadBalancingConfig,
    CanonicalCircuitBreakerConfig,
    CanonicalRetryConfig,
    CanonicalTimeoutConfig,
    CanonicalHealthCheckConfig,
    CanonicalMonitoringConfig,
    
    // Supporting types
    CanonicalRegisteredService,
    CanonicalServicePerformance,
    CanonicalLoadBalancingStrategy,
    CanonicalCircuitState,
    CanonicalAdapterRequest,
    CanonicalAdapterResponse,
    CanonicalRequestPriority,
    CanonicalAdapterMetrics,
    
    // Protocol handling
    CanonicalProtocolHandler,
    
    // Convenience functions
    create_canonical_adapter,
    create_adapter_request,
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
pub mod prelude  {
    pub use super::canonical::{CanonicalUniversalAdapter,
        CanonicalAdapterConfig,
        CanonicalAdapterRequest,
        CanonicalAdapterResponse,
        CanonicalRequestPriority,
        CanonicalLoadBalancingStrategy,
        create_canonical_adapter,
        create_adapter_request,
    };
} 