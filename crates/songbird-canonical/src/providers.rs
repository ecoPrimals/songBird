//! # 🔧 Canonical Providers
//!
//! **CANONICAL PROVIDER SYSTEM** ✅
//!
//! This module provides canonical provider definitions that have been
//! migrated to songbird-types. All provider traits are now centralized.

// ============================================================================
// CANONICAL PROVIDER SYSTEM - Migrated to songbird-types
// ============================================================================

// All provider traits have been consolidated into songbird-types::traits::canonical
// Use the canonical traits directly:

pub use songbird_types::traits::canonical::{
    CapabilityProvider, DiscoveryProvider, ObservabilityProvider, OrchestrationProvider,
    PrimalProvider, Provider, SecurityProvider, ServiceProvider,
};

// ============================================================================
// PROVIDER UTILITIES
// ============================================================================

/// Provider factory for creating canonical providers
pub struct CanonicalProviderFactory;

impl CanonicalProviderFactory {
    /// Create a new provider factory
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for CanonicalProviderFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_provider_factory_new() {
        let factory = CanonicalProviderFactory::new();
        // Factory is a ZST (zero-sized type), just verify creation
        let _ = factory;
    }

    #[test]
    fn test_canonical_provider_factory_default() {
        let factory = CanonicalProviderFactory::default();
        let _ = factory;
    }
}
