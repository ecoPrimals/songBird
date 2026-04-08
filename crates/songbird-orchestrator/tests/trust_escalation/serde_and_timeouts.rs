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

use songbird_orchestrator::trust::{HardwareAttestation, TowerIdentity, TrustTimeouts};
use std::time::SystemTime;

// ═══════════════════════════════════════════════════════════════════════════
// TowerIdentity tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_tower_identity_serde_roundtrip() {
    let identity = TowerIdentity {
        node_id: "node-1".to_string(),
        hostname: "host-1".to_string(),
        organization: Some("org".to_string()),
        public_key: Some("key".to_string()),
    };
    let json = serde_json::to_string(&identity).expect("serialize");
    let deserialized: TowerIdentity = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.node_id, "node-1");
    assert_eq!(deserialized.organization, Some("org".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════════
// HardwareAttestation tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_hardware_attestation_serde_roundtrip() {
    let attestation = HardwareAttestation {
        hardware_key: "hw-key-12345".to_string(),
        genetic_proof: Some("genetic-data".to_string()),
        attested_at: SystemTime::now(),
        signature: "sig-abc".to_string(),
    };
    let json = serde_json::to_string(&attestation).expect("serialize");
    let deserialized: HardwareAttestation = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.hardware_key, "hw-key-12345");
    assert_eq!(deserialized.genetic_proof, Some("genetic-data".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════════
// TrustTimeouts tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_trust_timeouts_default() {
    let timeouts = TrustTimeouts::default();
    assert_eq!(timeouts.anonymous, 3600);
    assert_eq!(timeouts.capability, 86400);
    assert_eq!(timeouts.identity, 604800);
    assert_eq!(timeouts.hardware, 0);
}

#[test]
fn test_trust_timeouts_custom() {
    let timeouts = TrustTimeouts {
        anonymous: 300,
        capability: 1800,
        identity: 7200,
        hardware: 86400,
    };
    assert_eq!(timeouts.anonymous, 300);
    assert_eq!(timeouts.hardware, 86400);
}

#[test]
fn test_trust_timeouts_clone() {
    let timeouts = TrustTimeouts::default();
    let cloned = timeouts.clone();
    assert_eq!(cloned.anonymous, timeouts.anonymous);
}
