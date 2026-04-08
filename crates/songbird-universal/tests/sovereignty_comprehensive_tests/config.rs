// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

// ============================================================================
// SOVEREIGNTY CONFIG TESTS
// ============================================================================

use crate::imports::*;

#[test]
fn test_sovereignty_config_default_values() {
    let config = SovereigntyAdapterConfig::default();

    assert!(config.enable_sovereignty_routing, "Sovereignty routing should be enabled by default");
    assert!(config.enable_federation_routing, "Federation routing should be enabled by default");
    assert!(
        config.enable_network_optimization,
        "Network optimization should be enabled by default"
    );
    assert_eq!(
        config.sovereignty_timeout,
        Duration::from_secs(3),
        "Default timeout should be 3 seconds"
    );
    assert!(
        (config.sovereignty_preference_weight - 0.8).abs() < 0.01,
        "Default sovereignty weight should heavily prefer sovereign paths"
    );
}

#[test]
fn test_sovereignty_config_high_preference() {
    let config = SovereigntyAdapterConfig {
        sovereignty_preference_weight: 1.0,
        ..Default::default()
    };

    assert_eq!(config.sovereignty_preference_weight, 1.0);
}

#[test]
fn test_sovereignty_config_balanced_preference() {
    let config = SovereigntyAdapterConfig {
        sovereignty_preference_weight: 0.5,
        ..Default::default()
    };

    assert_eq!(config.sovereignty_preference_weight, 0.5);
}

#[test]
fn test_sovereignty_config_efficiency_preference() {
    let config = SovereigntyAdapterConfig {
        sovereignty_preference_weight: 0.2,
        ..Default::default()
    };

    assert!(config.sovereignty_preference_weight < 0.5);
}

#[test]
fn test_sovereignty_config_custom_timeout() -> SongbirdResult<()> {
    let config = SovereigntyAdapterConfig {
        sovereignty_timeout: Duration::from_millis(500),
        ..Default::default()
    };

    assert_eq!(config.sovereignty_timeout, Duration::from_millis(500));
    Ok(())
}
