// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

// ============================================================================
// SECURITY CAPABILITY TESTS
// ============================================================================

use crate::imports::*;

#[test]
fn test_security_capability_variants() -> SongbirdResult<()> {
    let capabilities = [
        SecurityCapability::Encryption,
        SecurityCapability::Authentication,
        SecurityCapability::Authorization,
        SecurityCapability::FederationAware,
        SecurityCapability::NetworkOptimized,
        SecurityCapability::SovereigntyCompliant,
    ];

    assert_eq!(capabilities.len(), 6, "Should have 6 security capabilities");

    // Ensure all are distinct
    for (i, cap1) in capabilities.iter().enumerate() {
        for (j, cap2) in capabilities.iter().enumerate() {
            if i != j {
                assert_ne!(format!("{cap1:?}"), format!("{:?}", cap2));
            }
        }
    }
    Ok(())
}

#[test]
fn test_security_level_variants() -> SongbirdResult<()> {
    let levels = [
        SecurityLevel::Maximum,
        SecurityLevel::High,
        SecurityLevel::Medium,
        SecurityLevel::Low,
        SecurityLevel::Minimal,
    ];

    assert_eq!(levels.len(), 5, "Should have 5 security levels");

    // Ensure all are distinct
    for (i, level1) in levels.iter().enumerate() {
        for (j, level2) in levels.iter().enumerate() {
            if i != j {
                assert_ne!(format!("{level1:?}"), format!("{:?}", level2));
            }
        }
    }
    Ok(())
}
