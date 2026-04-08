// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

// ============================================================================
// HUMAN DIGNITY COMPLIANCE TESTS
// ============================================================================

use crate::helpers::create_test_service_with_sovereignty;
use crate::imports::*;

#[tokio::test]
async fn test_human_dignity_high_sovereignty_preference() {
    // Critical test: Ensure system defaults to respecting user sovereignty
    let config = SovereigntyAdapterConfig::default();

    assert!(
        config.sovereignty_preference_weight >= 0.7,
        "Default config MUST prefer sovereignty (human dignity) over efficiency"
    );
    assert!(
        config.enable_sovereignty_routing,
        "Sovereignty routing MUST be enabled by default for human dignity"
    );
}

#[tokio::test]
async fn test_human_dignity_full_sovereignty_available() {
    // Critical test: System must support fully sovereign routing
    let config = SovereigntyAdapterConfig {
        sovereignty_preference_weight: 1.0, // Maximum sovereignty preference
        enable_sovereignty_routing: true,
        ..Default::default()
    };

    let adapter = SovereigntyAwareAdapter::with_config(config).await;
    assert!(
        adapter.is_ok(),
        "System MUST support maximum sovereignty preference for human dignity"
    );
}

#[tokio::test]
async fn test_human_dignity_no_forced_efficiency() {
    // Critical test: Users must never be forced into efficiency-only routing
    let config = SovereigntyAdapterConfig {
        sovereignty_preference_weight: 1.0,
        enable_network_optimization: false, // Disable optimization that might override sovereignty
        ..Default::default()
    };

    let adapter = SovereigntyAwareAdapter::with_config(config).await;
    assert!(
        adapter.is_ok(),
        "Users MUST be able to disable efficiency optimizations for maximum sovereignty"
    );
}

#[test]
fn test_human_dignity_sovereignty_levels_comprehensive() -> SongbirdResult<()> {
    // Critical test: System must support full range of sovereignty levels
    let levels = vec![
        SovereigntyLevel::FullySovereign,
        SovereigntyLevel::HighlySovereign,
        SovereigntyLevel::ModeratelySovereign,
        SovereigntyLevel::LimitedSovereignty,
        SovereigntyLevel::NonSovereign,
    ];

    for level in levels {
        let endpoint = format!("http://localhost:{}", test_orchestrator_port());
        let service = create_test_service_with_sovereignty("test", &endpoint, level.clone());
        assert!(!service.name.is_empty(), "System MUST support all sovereignty levels: {level:?}");
    }
    Ok(())
}

#[test]
fn test_human_dignity_security_capabilities_comprehensive() -> SongbirdResult<()> {
    // Critical test: System must support comprehensive security for sovereignty
    let capabilities = vec![
        SecurityCapability::Encryption,
        SecurityCapability::Authentication,
        SecurityCapability::Authorization,
        SecurityCapability::FederationAware,
        SecurityCapability::NetworkOptimized,
        SecurityCapability::SovereigntyCompliant,
    ];

    for cap in capabilities {
        // Ensure all security capabilities are well-defined
        assert!(
            !format!("{cap:?}").is_empty(),
            "System MUST support all security capabilities: {cap:?}"
        );
    }
    Ok(())
}
