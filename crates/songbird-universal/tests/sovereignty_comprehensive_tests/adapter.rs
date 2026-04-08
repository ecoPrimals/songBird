// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

// ============================================================================
// SOVEREIGNTY ADAPTER TESTS
// ============================================================================

use crate::imports::*;

#[tokio::test]
async fn test_sovereignty_adapter_creation() {
    let adapter = SovereigntyAwareAdapter::new().await;
    assert!(adapter.is_ok(), "Sovereignty adapter should be created successfully");
}

#[tokio::test]
async fn test_sovereignty_adapter_with_default_config() {
    let config = SovereigntyAdapterConfig::default();
    let adapter = SovereigntyAwareAdapter::with_config(config).await;
    assert!(adapter.is_ok(), "Adapter should be created with default config");
}

#[tokio::test]
async fn test_sovereignty_adapter_with_custom_config() {
    let config = SovereigntyAdapterConfig {
        enable_sovereignty_routing: true,
        enable_federation_routing: true,
        enable_network_optimization: true,
        sovereignty_timeout: Duration::from_secs(5),
        sovereignty_preference_weight: 0.9,
    };

    let adapter = SovereigntyAwareAdapter::with_config(config).await;
    assert!(adapter.is_ok(), "Adapter should be created with custom config");
}

#[tokio::test]
async fn test_sovereignty_adapter_disabled_routing() {
    let config = SovereigntyAdapterConfig {
        enable_sovereignty_routing: false,
        enable_federation_routing: false,
        enable_network_optimization: false,
        sovereignty_timeout: Duration::from_secs(1),
        sovereignty_preference_weight: 0.0,
    };

    let adapter = SovereigntyAwareAdapter::with_config(config).await;
    assert!(adapter.is_ok(), "Adapter should work with sovereignty features disabled");
}

#[tokio::test]
async fn test_multiple_sovereignty_adapters() {
    let adapter1 = SovereigntyAwareAdapter::new().await;
    let adapter2 = SovereigntyAwareAdapter::new().await;
    let adapter3 = SovereigntyAwareAdapter::new().await;

    assert!(adapter1.is_ok());
    assert!(adapter2.is_ok());
    assert!(adapter3.is_ok());
}
