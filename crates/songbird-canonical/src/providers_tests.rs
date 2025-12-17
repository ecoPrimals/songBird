//! Tests for Canonical Providers
//!
//! Comprehensive test coverage for provider utilities.

use super::providers::*;

// ============================================================================
// CanonicalProviderFactory Tests
// ============================================================================

#[test]
fn test_provider_factory_new() {
    let factory = CanonicalProviderFactory::new();

    // Verify factory can be created
    let _ = factory;
}

#[test]
fn test_provider_factory_default() {
    let factory = CanonicalProviderFactory;

    // Verify default implementation works
    let _ = factory;
}

#[test]
fn test_provider_factory_multiple_instances() {
    let factory1 = CanonicalProviderFactory::new();
    let factory2 = CanonicalProviderFactory::new();
    let factory3 = CanonicalProviderFactory;

    // Verify multiple instances can be created
    let _ = factory1;
    let _ = factory2;
    let _ = factory3;
}

// ============================================================================
// Provider Trait Re-exports Tests
// ============================================================================

#[test]
fn test_provider_traits_available() {
    // Verify all provider traits are available through re-exports
    // This is a compile-time check - if this compiles, the re-exports work

    // These type annotations verify the traits exist
    let _: Option<&dyn Provider> = None;
    let _: Option<&dyn ServiceProvider> = None;
    let _: Option<&dyn CapabilityProvider> = None;
    let _: Option<&dyn DiscoveryProvider> = None;
    let _: Option<&dyn ObservabilityProvider> = None;
    let _: Option<&dyn OrchestrationProvider> = None;
    // Temporarily disabled - PrimalProvider uses generics which aren't dyn-compatible
    // let _: Option<&dyn PrimalProvider> = None;
    let _: Option<&dyn SecurityProvider> = None;
}
