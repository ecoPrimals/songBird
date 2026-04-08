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

use songbird_orchestrator::trust::CapabilityProof;
use std::time::SystemTime;

// ═══════════════════════════════════════════════════════════════════════════
// CapabilityProof verification edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_capability_proof_empty_proof_fails() {
    let proof = CapabilityProof {
        capabilities: vec!["orchestration".to_string()],
        proof: String::new(),
        timestamp: SystemTime::now(),
    };
    assert!(!proof.verify());
}

#[test]
fn test_capability_proof_too_short_fails() {
    let proof = CapabilityProof {
        capabilities: vec!["compute".to_string()],
        proof: "abc".to_string(), // < 32 chars
        timestamp: SystemTime::now(),
    };
    assert!(!proof.verify());
}

#[test]
fn test_capability_proof_future_timestamp_fails() {
    let future = SystemTime::now() + std::time::Duration::from_secs(7200);
    let proof = CapabilityProof {
        capabilities: vec!["orchestration".to_string()],
        proof: "0123456789abcdef0123456789abcdef".to_string(),
        timestamp: future,
    };
    assert!(!proof.verify());
}

#[test]
fn test_capability_proof_empty_capabilities_fails() {
    let proof = CapabilityProof {
        capabilities: vec![],
        proof: "0123456789abcdef0123456789abcdef".to_string(),
        timestamp: SystemTime::now(),
    };
    assert!(!proof.verify());
}

#[test]
fn test_capability_proof_valid_multiple_capabilities() {
    let proof = CapabilityProof {
        capabilities: vec!["compute".to_string(), "storage".to_string(), "networking".to_string()],
        proof: "0123456789abcdef0123456789abcdef".to_string(),
        timestamp: SystemTime::now(),
    };
    assert!(proof.verify());
}

#[test]
fn test_capability_proof_exactly_32_chars() {
    let proof = CapabilityProof {
        capabilities: vec!["test".to_string()],
        proof: "a".repeat(32),
        timestamp: SystemTime::now(),
    };
    assert!(proof.verify());
}

#[test]
fn test_capability_proof_serde_roundtrip() {
    let proof = CapabilityProof {
        capabilities: vec!["compute".to_string()],
        proof: "0123456789abcdef0123456789abcdef".to_string(),
        timestamp: SystemTime::now(),
    };
    let json = serde_json::to_string(&proof).expect("serialize");
    let deserialized: CapabilityProof = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.capabilities, proof.capabilities);
    assert_eq!(deserialized.proof, proof.proof);
}
