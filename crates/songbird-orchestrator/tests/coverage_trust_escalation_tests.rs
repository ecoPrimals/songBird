// SPDX-License-Identifier: AGPL-3.0-only
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

//! Comprehensive tests for trust escalation and types
//!
//! Covers edge cases not hit by existing tests:
//! - `TrustLevel` Display trait
//! - `TrustLevel` `description()`
//! - `TrustRelationship` `can_perform()` with expiration
//! - `TrustEscalationManager`: role verification edge cases
//! - `TrustEscalationManager`: `get_all_relationships`, `get_trust_level_counts`
//! - `BearDogClient` creation and defaults
//! - `CapabilityProof` verification edge cases
//! - `IdentityProof` verification edge cases
//! - `PeerMetadata` serde

use songbird_orchestrator::trust::escalation::BearDogClient;
use songbird_orchestrator::trust::{
    CapabilityProof, HardwareAttestation, IdentityProof, TowerIdentity, TrustEscalationManager,
    TrustLevel, TrustRelationship, TrustTimeouts,
};
use std::time::SystemTime;

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

// ═══════════════════════════════════════════════════════════════════════════
// BearDogClient tests
// ═══════════════════════════════════════════════════════════════════════════

static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[test]
fn test_beardog_client_default_no_endpoint() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_SECURITY_PROVIDER");
    songbird_process_env::remove_var("SECURITY_ENDPOINT");
    songbird_process_env::remove_var("BEARDOG_URL");
    let _client = BearDogClient::default();
}

#[test]
fn test_beardog_client_new_with_security_provider() {
    let _g = lock_env();
    songbird_process_env::set_var("SONGBIRD_SECURITY_PROVIDER", "http://localhost:9090");
    let _client = BearDogClient::new();
    songbird_process_env::remove_var("SONGBIRD_SECURITY_PROVIDER");
}

#[test]
fn test_beardog_client_new_with_security_endpoint() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_SECURITY_PROVIDER");
    songbird_process_env::set_var("SECURITY_ENDPOINT", "http://localhost:9091");
    let _client = BearDogClient::new();
    songbird_process_env::remove_var("SECURITY_ENDPOINT");
}

#[test]
fn test_beardog_client_new_with_deprecated_url() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_SECURITY_PROVIDER");
    songbird_process_env::remove_var("SECURITY_ENDPOINT");
    songbird_process_env::set_var("BEARDOG_URL", "http://localhost:9092");
    let _client = BearDogClient::new();
    songbird_process_env::remove_var("BEARDOG_URL");
}

// ═══════════════════════════════════════════════════════════════════════════
// TrustEscalationManager advanced tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_trust_manager_debug() {
    let manager = TrustEscalationManager::with_defaults();
    let debug = format!("{manager:?}");
    assert!(debug.contains("TrustEscalationManager"));
    assert!(debug.contains("trust_timeouts"));
}

#[tokio::test]
async fn test_trust_manager_custom_timeouts() {
    let timeouts = TrustTimeouts {
        anonymous: 60,
        capability: 300,
        identity: 600,
        hardware: 1200,
    };
    let manager = TrustEscalationManager::new(timeouts, None);
    let counts = manager.get_trust_level_counts().await;
    assert!(counts.is_empty());
}

#[tokio::test]
async fn test_trust_manager_get_all_relationships_empty() {
    let manager = TrustEscalationManager::with_defaults();
    let rels = manager.get_all_relationships().await;
    assert!(rels.is_empty());
}

#[tokio::test]
async fn test_trust_manager_get_all_relationships_multiple() {
    let manager = TrustEscalationManager::with_defaults();
    manager.establish_anonymous("sess-1".to_string()).await.unwrap();
    manager.establish_anonymous("sess-2".to_string()).await.unwrap();
    manager.establish_anonymous("sess-3".to_string()).await.unwrap();
    let rels = manager.get_all_relationships().await;
    assert_eq!(rels.len(), 3);
}

#[tokio::test]
async fn test_trust_manager_get_trust_level_counts_mixed() {
    let manager = TrustEscalationManager::with_defaults();
    manager.establish_anonymous("sess-1".to_string()).await.unwrap();
    manager.establish_anonymous("sess-2".to_string()).await.unwrap();

    // Escalate one to capability
    let proof = CapabilityProof {
        capabilities: vec!["orchestration".to_string()],
        proof: "0123456789abcdef0123456789abcdef".to_string(),
        timestamp: SystemTime::now(),
    };
    manager.verify_capabilities("sess-1", proof).await.unwrap();

    let counts = manager.get_trust_level_counts().await;
    assert_eq!(*counts.get(&TrustLevel::Anonymous).unwrap_or(&0), 1);
    assert_eq!(*counts.get(&TrustLevel::CapabilityVerified).unwrap_or(&0), 1);
}

#[tokio::test]
async fn test_trust_manager_get_relationship() {
    let manager = TrustEscalationManager::with_defaults();
    assert!(manager.get_relationship("nonexistent").await.is_none());

    manager.establish_anonymous("sess-1".to_string()).await.unwrap();
    let rel = manager.get_relationship("sess-1").await;
    assert!(rel.is_some());
    assert_eq!(rel.unwrap().trust_level, TrustLevel::Anonymous);
}

#[tokio::test]
async fn test_trust_manager_verify_capabilities_session_not_found() {
    let manager = TrustEscalationManager::with_defaults();
    let proof = CapabilityProof {
        capabilities: vec!["orchestration".to_string()],
        proof: "0123456789abcdef0123456789abcdef".to_string(),
        timestamp: SystemTime::now(),
    };
    let result = manager.verify_capabilities("nonexistent", proof).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Session not found"));
}

#[tokio::test]
async fn test_trust_manager_verify_capabilities_invalid_proof() {
    let manager = TrustEscalationManager::with_defaults();
    manager.establish_anonymous("sess-1".to_string()).await.unwrap();

    let bad_proof = CapabilityProof {
        capabilities: vec![],
        proof: "too-short".to_string(),
        timestamp: SystemTime::now(),
    };
    let result = manager.verify_capabilities("sess-1", bad_proof).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_trust_manager_verify_role_empty_role() {
    let manager = TrustEscalationManager::with_defaults();
    manager.establish_anonymous("sess-1".to_string()).await.unwrap();

    let proof = CapabilityProof {
        capabilities: vec!["orchestration".to_string()],
        proof: "0123456789abcdef0123456789abcdef".to_string(),
        timestamp: SystemTime::now(),
    };
    manager.verify_capabilities("sess-1", proof).await.unwrap();

    let result = manager.verify_role("sess-1", String::new()).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("empty"));
}

#[tokio::test]
async fn test_trust_manager_verify_role_admin_requires_identity() {
    let manager = TrustEscalationManager::with_defaults();
    manager.establish_anonymous("sess-1".to_string()).await.unwrap();

    let proof = CapabilityProof {
        capabilities: vec!["orchestration".to_string()],
        proof: "0123456789abcdef0123456789abcdef".to_string(),
        timestamp: SystemTime::now(),
    };
    manager.verify_capabilities("sess-1", proof).await.unwrap();

    let result = manager.verify_role("sess-1", "admin".to_string()).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("identity"));
}

#[tokio::test]
async fn test_trust_manager_verify_role_unknown_role_accepted() {
    let manager = TrustEscalationManager::with_defaults();
    manager.establish_anonymous("sess-1".to_string()).await.unwrap();

    let proof = CapabilityProof {
        capabilities: vec!["orchestration".to_string()],
        proof: "0123456789abcdef0123456789abcdef".to_string(),
        timestamp: SystemTime::now(),
    };
    manager.verify_capabilities("sess-1", proof).await.unwrap();

    // Unknown roles should still be accepted (logged but not rejected)
    let result = manager.verify_role("sess-1", "custom_role".to_string()).await;
    assert!(result.is_ok());
    let level = manager.get_trust_level("sess-1").await.unwrap();
    assert_eq!(level, TrustLevel::RoleVerified);
}

#[tokio::test]
async fn test_trust_manager_verify_role_requires_capability_first() {
    let manager = TrustEscalationManager::with_defaults();
    manager.establish_anonymous("sess-1".to_string()).await.unwrap();

    // Try to verify role without capability first
    let result = manager.verify_role("sess-1", "coordinator".to_string()).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Cannot escalate"));
}

#[tokio::test]
async fn test_trust_manager_verify_identity_requires_role_first() {
    let manager = TrustEscalationManager::with_defaults();
    manager.establish_anonymous("sess-1".to_string()).await.unwrap();

    let proof = CapabilityProof {
        capabilities: vec!["orchestration".to_string()],
        proof: "0123456789abcdef0123456789abcdef".to_string(),
        timestamp: SystemTime::now(),
    };
    manager.verify_capabilities("sess-1", proof).await.unwrap();

    // Try to verify identity without role first
    let identity = TowerIdentity {
        node_id: "test-node-12345".to_string(),
        hostname: "test-host".to_string(),
        organization: None,
        public_key: None,
    };
    let id_proof = IdentityProof {
        identity,
        proof: "0123456789abcdef0123456789abcdef".to_string(),
        proof_type: "jwt".to_string(),
        timestamp: SystemTime::now(),
    };
    let result = manager.verify_identity("sess-1", id_proof).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Cannot escalate"));
}

#[tokio::test]
async fn test_trust_manager_revoke_nonexistent() {
    let manager = TrustEscalationManager::with_defaults();
    let result = manager.revoke_trust("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_trust_manager_get_trust_level_nonexistent() {
    let manager = TrustEscalationManager::with_defaults();
    let result = manager.get_trust_level("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_trust_manager_check_permission_nonexistent() {
    let manager = TrustEscalationManager::with_defaults();
    let result = manager.check_permission("nonexistent", TrustLevel::Anonymous).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_trust_manager_cleanup_none_expired() {
    let manager = TrustEscalationManager::with_defaults();
    manager.establish_anonymous("sess-1".to_string()).await.unwrap();
    manager.establish_anonymous("sess-2".to_string()).await.unwrap();

    let removed = manager.cleanup_expired().await;
    assert_eq!(removed, 0);

    let rels = manager.get_all_relationships().await;
    assert_eq!(rels.len(), 2);
}

#[tokio::test]
async fn test_trust_manager_full_escalation_chain() {
    let manager = TrustEscalationManager::with_defaults();
    let session = "full-chain".to_string();

    // Step 1: Anonymous
    manager.establish_anonymous(session.clone()).await.unwrap();
    assert_eq!(manager.get_trust_level(&session).await.unwrap(), TrustLevel::Anonymous);

    // Step 2: Capability
    let cap_proof = CapabilityProof {
        capabilities: vec!["orchestration".to_string(), "compute".to_string()],
        proof: "0123456789abcdef0123456789abcdef".to_string(),
        timestamp: SystemTime::now(),
    };
    manager.verify_capabilities(&session, cap_proof).await.unwrap();
    assert_eq!(manager.get_trust_level(&session).await.unwrap(), TrustLevel::CapabilityVerified);

    // Step 3: Role
    manager.verify_role(&session, "coordinator".to_string()).await.unwrap();
    assert_eq!(manager.get_trust_level(&session).await.unwrap(), TrustLevel::RoleVerified);

    // Step 4: Identity
    let identity = TowerIdentity {
        node_id: "production-node-42".to_string(),
        hostname: "prod-01".to_string(),
        organization: Some("EcoPrimals".to_string()),
        public_key: Some("ed25519:key123456".to_string()),
    };
    let id_proof = IdentityProof {
        identity,
        proof: "0123456789abcdef0123456789abcdef".to_string(),
        proof_type: "jwt".to_string(),
        timestamp: SystemTime::now(),
    };
    manager.verify_identity(&session, id_proof).await.unwrap();
    assert_eq!(manager.get_trust_level(&session).await.unwrap(), TrustLevel::IdentityVerified);

    // Verify permissions
    assert!(manager.check_permission(&session, TrustLevel::Anonymous).await.unwrap());
    assert!(manager.check_permission(&session, TrustLevel::IdentityVerified).await.unwrap());
    assert!(!manager.check_permission(&session, TrustLevel::HardwareVerified).await.unwrap());
}

// ═══════════════════════════════════════════════════════════════════════════
// PeerMetadata serde tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_peer_metadata_serde_roundtrip() {
    use songbird_orchestrator::app::connection_manager::PeerMetadata;
    use std::time::SystemTime;

    let metadata = PeerMetadata {
        peer_id: "peer-42".to_string(),
        endpoint: "https://10.0.0.1:8443".to_string(),
        trust_level: songbird_types::TrustLevel::Elevated,
        discovery_method: "mdns".to_string(),
        capabilities: vec!["compute".to_string(), "storage".to_string()],
        established_at: SystemTime::now(),
    };

    let json = serde_json::to_string(&metadata).expect("serialize");
    let deserialized: PeerMetadata = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.peer_id, "peer-42");
    assert_eq!(deserialized.endpoint, "https://10.0.0.1:8443");
    assert_eq!(deserialized.capabilities.len(), 2);
}
