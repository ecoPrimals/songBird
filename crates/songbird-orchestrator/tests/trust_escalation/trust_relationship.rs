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

use songbird_orchestrator::trust::{TrustLevel, TrustRelationship};
use std::time::SystemTime;

// ═══════════════════════════════════════════════════════════════════════════
// TrustRelationship tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_trust_relationship_new_anonymous_defaults() {
    let rel = TrustRelationship::new_anonymous("sess-1".to_string(), 3600);
    assert_eq!(rel.session_id, "sess-1");
    assert_eq!(rel.trust_level, TrustLevel::Anonymous);
    assert!(rel.verified_capabilities.is_empty());
    assert!(rel.identity.is_none());
    assert!(rel.hardware_proof.is_none());
    assert!(!rel.is_expired());
}

#[test]
fn test_trust_relationship_expired_cannot_perform() {
    let mut rel = TrustRelationship::new_anonymous("sess-2".to_string(), 0);
    rel.expires_at = SystemTime::now() - std::time::Duration::from_secs(10);
    assert!(rel.is_expired());
    assert!(!rel.can_perform(TrustLevel::Anonymous));
}

#[test]
fn test_trust_relationship_active_can_perform() {
    let rel = TrustRelationship::new_anonymous("sess-3".to_string(), 3600);
    assert!(rel.can_perform(TrustLevel::Anonymous));
    // Anonymous cannot perform higher levels
    assert!(!rel.can_perform(TrustLevel::CapabilityVerified));
}

#[test]
fn test_trust_relationship_clone() {
    let rel = TrustRelationship::new_anonymous("clone-test".to_string(), 3600);
    let cloned = rel.clone();
    assert_eq!(cloned.session_id, rel.session_id);
    assert_eq!(cloned.trust_level, rel.trust_level);
}
