// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::ignore_without_reason,
    clippy::unreadable_literal,
    clippy::no_effect_underscore_binding,
    clippy::float_cmp,
    clippy::default_trait_access,
    clippy::needless_collect,
    clippy::unused_async,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::items_after_statements,
    clippy::unnecessary_wraps,
    clippy::used_underscore_binding,
    clippy::struct_excessive_bools,
    clippy::similar_names,
    clippy::significant_drop_tightening,
    clippy::struct_field_names,
    clippy::match_same_arms,
    clippy::future_not_send,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: strict clippy matches crate [lints] policy"
)]

use songbird_orchestrator::trust::TrustLevel;

// ═══════════════════════════════════════════════════════════════════════════
// TrustLevel tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_trust_level_display() {
    assert_eq!(format!("{}", TrustLevel::Anonymous), "Anonymous (discovery only)");
    assert_eq!(
        format!("{}", TrustLevel::CapabilityVerified),
        "Capability-Verified (task coordination)"
    );
    assert_eq!(format!("{}", TrustLevel::RoleVerified), "Role-Verified (registry access)");
    assert_eq!(
        format!("{}", TrustLevel::IdentityVerified),
        "Identity-Verified (infrastructure access)"
    );
    assert_eq!(format!("{}", TrustLevel::HardwareVerified), "Hardware-Verified (full admin)");
}

#[test]
fn test_trust_level_description_all_variants() {
    assert!(!TrustLevel::Anonymous.description().is_empty());
    assert!(!TrustLevel::CapabilityVerified.description().is_empty());
    assert!(!TrustLevel::RoleVerified.description().is_empty());
    assert!(!TrustLevel::IdentityVerified.description().is_empty());
    assert!(!TrustLevel::HardwareVerified.description().is_empty());
}

#[test]
fn test_trust_level_can_perform_all_combinations() {
    let levels = [
        TrustLevel::Anonymous,
        TrustLevel::CapabilityVerified,
        TrustLevel::RoleVerified,
        TrustLevel::IdentityVerified,
        TrustLevel::HardwareVerified,
    ];

    for (i, level) in levels.iter().enumerate() {
        for (j, required) in levels.iter().enumerate() {
            if i >= j {
                assert!(
                    level.can_perform(*required),
                    "{level:?} should be able to perform {required:?}"
                );
            } else {
                assert!(
                    !level.can_perform(*required),
                    "{level:?} should NOT be able to perform {required:?}"
                );
            }
        }
    }
}

#[test]
fn test_trust_level_ordering_complete() {
    assert!(TrustLevel::Anonymous < TrustLevel::CapabilityVerified);
    assert!(TrustLevel::CapabilityVerified < TrustLevel::RoleVerified);
    assert!(TrustLevel::RoleVerified < TrustLevel::IdentityVerified);
    assert!(TrustLevel::IdentityVerified < TrustLevel::HardwareVerified);
    assert!(TrustLevel::Anonymous < TrustLevel::HardwareVerified);
}

#[test]
fn test_trust_level_eq() {
    assert_eq!(TrustLevel::Anonymous, TrustLevel::Anonymous);
    assert_ne!(TrustLevel::Anonymous, TrustLevel::HardwareVerified);
}

#[test]
fn test_trust_level_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(TrustLevel::Anonymous);
    set.insert(TrustLevel::CapabilityVerified);
    set.insert(TrustLevel::Anonymous); // duplicate
    assert_eq!(set.len(), 2);
}

#[test]
fn test_trust_level_serde_roundtrip() {
    let levels = [
        TrustLevel::Anonymous,
        TrustLevel::CapabilityVerified,
        TrustLevel::RoleVerified,
        TrustLevel::IdentityVerified,
        TrustLevel::HardwareVerified,
    ];
    for level in levels {
        let json = serde_json::to_string(&level).expect("serialize");
        let deserialized: TrustLevel = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized, level);
    }
}
