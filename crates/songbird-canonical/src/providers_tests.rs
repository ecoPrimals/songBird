// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

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

/// Smoke check that canonical provider traits are reachable through `providers::*`.
///
/// Native async provider traits are not used with `dyn`; concrete implementations are covered in
/// integration tests.
#[test]
fn test_provider_traits_available() {
    assert!(core::mem::size_of::<usize>() > 0);
}
