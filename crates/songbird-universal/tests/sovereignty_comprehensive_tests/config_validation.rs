// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

// ============================================================================
// CONFIGURATION VALIDATION TESTS
// ============================================================================

use crate::imports::*;

#[test]
fn test_config_all_features_enabled() {
    let config = SovereigntyAdapterConfig {
        enable_sovereignty_routing: true,
        enable_federation_routing: true,
        enable_network_optimization: true,
        sovereignty_timeout: Duration::from_secs(5),
        sovereignty_preference_weight: 0.8,
    };

    assert!(config.enable_sovereignty_routing);
    assert!(config.enable_federation_routing);
    assert!(config.enable_network_optimization);
}

#[test]
fn test_config_all_features_disabled() {
    let config = SovereigntyAdapterConfig {
        enable_sovereignty_routing: false,
        enable_federation_routing: false,
        enable_network_optimization: false,
        sovereignty_timeout: Duration::from_secs(1),
        sovereignty_preference_weight: 0.0,
    };

    assert!(!config.enable_sovereignty_routing);
    assert!(!config.enable_federation_routing);
    assert!(!config.enable_network_optimization);
    assert_eq!(config.sovereignty_preference_weight, 0.0);
}

#[test]
fn test_config_mixed_features() {
    let config = SovereigntyAdapterConfig {
        enable_sovereignty_routing: true,
        enable_federation_routing: false,
        enable_network_optimization: true,
        sovereignty_timeout: Duration::from_millis(2500),
        sovereignty_preference_weight: 0.6,
    };

    assert!(config.enable_sovereignty_routing);
    assert!(!config.enable_federation_routing);
    assert!(config.enable_network_optimization);
}
