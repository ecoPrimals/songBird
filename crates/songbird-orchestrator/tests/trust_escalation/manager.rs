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

use songbird_orchestrator::trust::{
    CapabilityProof, IdentityProof, TowerIdentity, TrustEscalationManager, TrustLevel,
    TrustTimeouts,
};
use std::time::SystemTime;

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
