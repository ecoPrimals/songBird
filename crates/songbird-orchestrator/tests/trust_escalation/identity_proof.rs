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

use songbird_orchestrator::trust::{IdentityProof, TowerIdentity};
use std::time::SystemTime;

// ═══════════════════════════════════════════════════════════════════════════
// IdentityProof verification edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_identity_proof_short_node_id_fails() {
    let identity = TowerIdentity {
        node_id: "short".to_string(), // < 8 chars
        hostname: "host".to_string(),
        organization: None,
        public_key: None,
    };
    let proof = IdentityProof {
        identity,
        proof: "0123456789abcdef0123456789abcdef".to_string(),
        proof_type: "jwt".to_string(),
        timestamp: SystemTime::now(),
    };
    assert!(!proof.verify());
}

#[test]
fn test_identity_proof_empty_node_id_fails() {
    let identity = TowerIdentity {
        node_id: String::new(),
        hostname: "host".to_string(),
        organization: None,
        public_key: None,
    };
    let proof = IdentityProof {
        identity,
        proof: "0123456789abcdef0123456789abcdef".to_string(),
        proof_type: "jwt".to_string(),
        timestamp: SystemTime::now(),
    };
    assert!(!proof.verify());
}

#[test]
fn test_identity_proof_short_proof_fails() {
    let identity = TowerIdentity {
        node_id: "long-enough-node-id".to_string(),
        hostname: "host".to_string(),
        organization: None,
        public_key: None,
    };
    let proof = IdentityProof {
        identity,
        proof: "too-short".to_string(), // < 32 chars
        proof_type: "jwt".to_string(),
        timestamp: SystemTime::now(),
    };
    assert!(!proof.verify());
}

#[test]
fn test_identity_proof_empty_proof_fails() {
    let identity = TowerIdentity {
        node_id: "long-enough-node-id".to_string(),
        hostname: "host".to_string(),
        organization: None,
        public_key: None,
    };
    let proof = IdentityProof {
        identity,
        proof: String::new(),
        proof_type: "jwt".to_string(),
        timestamp: SystemTime::now(),
    };
    assert!(!proof.verify());
}

#[test]
fn test_identity_proof_future_timestamp_fails() {
    let identity = TowerIdentity {
        node_id: "long-enough-node-id".to_string(),
        hostname: "host".to_string(),
        organization: None,
        public_key: None,
    };
    let future = SystemTime::now() + std::time::Duration::from_secs(86400 * 2);
    let proof = IdentityProof {
        identity,
        proof: "0123456789abcdef0123456789abcdef".to_string(),
        proof_type: "certificate".to_string(),
        timestamp: future,
    };
    assert!(!proof.verify());
}

#[test]
fn test_identity_proof_valid_with_org_and_pubkey() {
    let identity = TowerIdentity {
        node_id: "production-node-42".to_string(),
        hostname: "prod-01.example.com".to_string(),
        organization: Some("EcoPrimals Inc.".to_string()),
        public_key: Some("ed25519:abcdef123456".to_string()),
    };
    let proof = IdentityProof {
        identity,
        proof: "0123456789abcdef0123456789abcdef".to_string(),
        proof_type: "certificate".to_string(),
        timestamp: SystemTime::now(),
    };
    assert!(proof.verify());
}
