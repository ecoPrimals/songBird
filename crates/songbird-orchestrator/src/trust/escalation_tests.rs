// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::*;
use crate::trust::{HardwareAttestation, IdentityProof, TowerIdentity};

#[tokio::test]
async fn test_trust_escalation_manager_creation() {
    let manager = TrustEscalationManager::with_defaults();
    let counts = manager.get_trust_level_counts().await;
    assert!(counts.is_empty());
}

#[tokio::test]
async fn test_establish_anonymous_trust() {
    let manager = TrustEscalationManager::with_defaults();
    let session_id = "test-session".to_string();

    manager.establish_anonymous(session_id.clone()).await.expect("anonymous");

    let level = manager.get_trust_level(&session_id).await.expect("level");
    assert_eq!(level, TrustLevel::Anonymous);
}

#[tokio::test]
async fn test_verify_capabilities() {
    let manager = TrustEscalationManager::with_defaults();
    let session_id = "test-session".to_string();

    // Establish anonymous trust first
    manager.establish_anonymous(session_id.clone()).await.expect("establish");

    // Create capability proof (must be >= 32 chars)
    let proof = CapabilityProof {
        capabilities: vec!["orchestration".to_string()],
        proof: "0123456789abcdef0123456789abcdef".to_string(), // 32 chars minimum
        timestamp: SystemTime::now(),
    };

    // Verify capabilities
    manager.verify_capabilities(&session_id, proof).await.expect("cap");

    let level = manager.get_trust_level(&session_id).await.expect("level");
    assert_eq!(level, TrustLevel::CapabilityVerified);
}

#[tokio::test]
async fn test_verify_identity() {
    let manager = TrustEscalationManager::with_defaults();
    let session_id = "test-session".to_string();

    // Establish anonymous trust
    manager.establish_anonymous(session_id.clone()).await.expect("establish");

    // Escalate to capability-verified (proof must be >= 32 chars)
    let cap_proof = CapabilityProof {
        capabilities: vec!["orchestration".to_string()],
        proof: "0123456789abcdef0123456789abcdef".to_string(), // 32 chars minimum
        timestamp: SystemTime::now(),
    };
    manager.verify_capabilities(&session_id, cap_proof).await.expect("cap");

    // Escalate to role-verified (use "coordinator" not "admin" - admin requires identity first)
    manager.verify_role(&session_id, "coordinator".to_string()).await.expect("role");

    // Escalate to identity-verified
    let identity = TowerIdentity {
        node_id: "test-node".to_string(),
        hostname: "test-host".to_string(),
        organization: None,
        public_key: None,
    };

    let identity_proof = IdentityProof {
        identity,
        proof: "0123456789abcdef0123456789abcdef".to_string(), // 32 chars minimum
        proof_type: "jwt".to_string(),
        timestamp: SystemTime::now(),
    };

    manager.verify_identity(&session_id, identity_proof).await.expect("identity");

    let level = manager.get_trust_level(&session_id).await.expect("level");
    assert_eq!(level, TrustLevel::IdentityVerified);
}

#[tokio::test]
async fn test_check_permission() {
    let manager = TrustEscalationManager::with_defaults();
    let session_id = "test-session".to_string();

    // Establish capability-verified trust (proof must be >= 32 chars)
    manager.establish_anonymous(session_id.clone()).await.expect("establish");
    let proof = CapabilityProof {
        capabilities: vec!["orchestration".to_string()],
        proof: "0123456789abcdef0123456789abcdef".to_string(), // 32 chars minimum
        timestamp: SystemTime::now(),
    };
    manager.verify_capabilities(&session_id, proof).await.expect("cap");

    // Should be able to perform anonymous and capability operations
    assert!(manager.check_permission(&session_id, TrustLevel::Anonymous).await.expect("anon"));
    assert!(
        manager.check_permission(&session_id, TrustLevel::CapabilityVerified).await.expect("cap")
    );

    // Should NOT be able to perform identity operations
    assert!(
        !manager.check_permission(&session_id, TrustLevel::IdentityVerified).await.expect("id")
    );
}

#[tokio::test]
async fn test_revoke_trust() {
    let manager = TrustEscalationManager::with_defaults();
    let session_id = "test-session".to_string();

    manager.establish_anonymous(session_id.clone()).await.expect("establish");
    manager.revoke_trust(&session_id).await.expect("revoke");

    // Should fail to get trust level after revocation
    assert!(manager.get_trust_level(&session_id).await.is_err());
}

#[tokio::test(start_paused = true)]
async fn test_cleanup_expired() {
    let mut timeouts = TrustTimeouts::default();
    timeouts.anonymous = 0; // Expire immediately

    let manager = TrustEscalationManager::new(timeouts, None);
    let session_id = "test-session".to_string();

    manager.establish_anonymous(session_id.clone()).await.expect("establish");

    // Wait a moment to ensure expiration
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let removed = manager.cleanup_expired().await;
    assert_eq!(removed, 1);
}

#[tokio::test]
async fn verify_capabilities_rejects_bad_proof() {
    let manager = TrustEscalationManager::with_defaults();
    let session_id = "bad-proof".to_string();
    manager.establish_anonymous(session_id.clone()).await.expect("establish");
    let proof = CapabilityProof {
        capabilities: vec!["orchestration".to_string()],
        proof: "short".to_string(),
        timestamp: SystemTime::now(),
    };
    let err = manager.verify_capabilities(&session_id, proof).await;
    assert!(err.is_err());
}

#[tokio::test]
async fn verify_role_rejects_empty_role() {
    let manager = TrustEscalationManager::with_defaults();
    let session_id = "role-empty".to_string();
    manager.establish_anonymous(session_id.clone()).await.expect("establish");
    let cap = CapabilityProof {
        capabilities: vec!["orchestration".to_string()],
        proof: "0123456789abcdef0123456789abcdef".to_string(),
        timestamp: SystemTime::now(),
    };
    manager.verify_capabilities(&session_id, cap).await.expect("cap");
    let err = manager.verify_role(&session_id, String::new()).await;
    assert!(err.is_err());
}

#[tokio::test]
async fn verify_role_admin_rejected_without_identity_chain() {
    let manager = TrustEscalationManager::with_defaults();
    let session_id = "admin-chain".to_string();
    manager.establish_anonymous(session_id.clone()).await.expect("establish");
    let cap = CapabilityProof {
        capabilities: vec!["orchestration".to_string()],
        proof: "0123456789abcdef0123456789abcdef".to_string(),
        timestamp: SystemTime::now(),
    };
    manager.verify_capabilities(&session_id, cap).await.expect("cap");
    let err = manager.verify_role(&session_id, "admin".to_string()).await;
    assert!(err.is_err());
}

#[tokio::test]
async fn verify_role_requires_capability_first() {
    let manager = TrustEscalationManager::with_defaults();
    let session_id = "role-order".to_string();
    manager.establish_anonymous(session_id.clone()).await.expect("establish");
    let err = manager.verify_role(&session_id, "worker".to_string()).await;
    assert!(err.is_err());
}

#[tokio::test]
async fn verify_identity_requires_role_first() {
    let manager = TrustEscalationManager::with_defaults();
    let session_id = "id-order".to_string();
    manager.establish_anonymous(session_id.clone()).await.expect("establish");
    let cap = CapabilityProof {
        capabilities: vec!["orchestration".to_string()],
        proof: "0123456789abcdef0123456789abcdef".to_string(),
        timestamp: SystemTime::now(),
    };
    manager.verify_capabilities(&session_id, cap).await.expect("cap");
    let identity = TowerIdentity {
        node_id: "n".to_string(),
        hostname: "h".to_string(),
        organization: None,
        public_key: None,
    };
    let id_proof = IdentityProof {
        identity,
        proof: "0123456789abcdef0123456789abcdef".to_string(),
        proof_type: "jwt".to_string(),
        timestamp: SystemTime::now(),
    };
    let err = manager.verify_identity(&session_id, id_proof).await;
    assert!(err.is_err());
}

#[tokio::test]
async fn get_relationship_clones_store_entry() {
    let manager = TrustEscalationManager::with_defaults();
    let session_id = "rel-clone".to_string();
    manager.establish_anonymous(session_id.clone()).await.expect("establish");
    let rel = manager.get_relationship(&session_id).await;
    assert!(rel.is_some());
    assert_eq!(rel.expect("rel").trust_level, TrustLevel::Anonymous);
}

#[tokio::test]
async fn verify_hardware_fails_without_security_client() {
    let manager = TrustEscalationManager::with_defaults();
    let hw = HardwareAttestation {
        hardware_key: "0123456789abcdef0123456789abcdef".to_string(),
        genetic_proof: None,
        attested_at: SystemTime::now(),
        signature: "sig".to_string(),
    };
    let err = manager.verify_hardware("any-session", hw).await;
    assert!(err.is_err());
    let msg = err.expect_err("expected err").to_string();
    assert!(msg.contains("security") || msg.contains("Session"), "unexpected message: {msg}");
}

#[tokio::test(start_paused = true)]
async fn get_trust_level_reports_anonymous_when_expired() {
    let mut timeouts = TrustTimeouts::default();
    timeouts.anonymous = 0;
    let manager = TrustEscalationManager::new(timeouts, None);
    let session_id = "exp-anon".to_string();
    manager.establish_anonymous(session_id.clone()).await.expect("establish");
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let level = manager.get_trust_level(&session_id).await.expect("level");
    assert_eq!(level, TrustLevel::Anonymous);
}

#[tokio::test(start_paused = true)]
async fn check_permission_false_when_session_expired() {
    let mut timeouts = TrustTimeouts::default();
    timeouts.anonymous = 0;
    let manager = TrustEscalationManager::new(timeouts, None);
    let session_id = "exp-check".to_string();
    manager.establish_anonymous(session_id.clone()).await.expect("establish");
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let allowed =
        manager.check_permission(&session_id, TrustLevel::Anonymous).await.expect("check");
    assert!(!allowed);
}

#[test]
fn trust_timeouts_defaults_are_ordered_by_increasing_duration() {
    let t = TrustTimeouts::default();
    assert!(t.anonymous < t.capability);
    assert!(t.capability < t.identity);
    assert_eq!(t.hardware, 0);
}

#[tokio::test]
async fn revoke_unknown_session_errors() {
    let m = TrustEscalationManager::with_defaults();
    let err = m.revoke_trust("no-such").await;
    assert!(err.is_err());
}

#[tokio::test]
async fn get_trust_level_missing_session_errors() {
    let m = TrustEscalationManager::with_defaults();
    assert!(m.get_trust_level("missing").await.is_err());
}

#[tokio::test]
async fn check_permission_missing_session_errors() {
    let m = TrustEscalationManager::with_defaults();
    assert!(m.check_permission("missing", TrustLevel::Anonymous).await.is_err());
}

#[tokio::test]
async fn verify_capabilities_missing_session() {
    let m = TrustEscalationManager::with_defaults();
    let proof = CapabilityProof {
        capabilities: vec!["x".to_string()],
        proof: "0123456789abcdef0123456789abcdef".to_string(),
        timestamp: SystemTime::now(),
    };
    assert!(m.verify_capabilities("nope", proof).await.is_err());
}

#[tokio::test]
async fn verify_identity_bad_proof_fails() {
    let m = TrustEscalationManager::with_defaults();
    m.establish_anonymous("s1".to_string()).await.expect("establish");
    let cap = CapabilityProof {
        capabilities: vec!["orchestration".to_string()],
        proof: "0123456789abcdef0123456789abcdef".to_string(),
        timestamp: SystemTime::now(),
    };
    m.verify_capabilities("s1", cap).await.expect("cap");
    m.verify_role("s1", "worker".to_string()).await.expect("role");
    let id = TowerIdentity {
        node_id: "n".to_string(),
        hostname: "h".to_string(),
        organization: None,
        public_key: None,
    };
    let bad = IdentityProof {
        identity: id,
        proof: "short".to_string(),
        proof_type: "jwt".to_string(),
        timestamp: SystemTime::now(),
    };
    assert!(m.verify_identity("s1", bad).await.is_err());
}

#[tokio::test]
async fn get_all_relationships_lists_sessions() {
    let m = TrustEscalationManager::with_defaults();
    m.establish_anonymous("a".to_string()).await.expect("establish");
    m.establish_anonymous("b".to_string()).await.expect("establish");
    let all = m.get_all_relationships().await;
    assert_eq!(all.len(), 2);
}

#[tokio::test]
async fn get_trust_level_counts_after_escalation() {
    let m = TrustEscalationManager::with_defaults();
    m.establish_anonymous("s".to_string()).await.expect("establish");
    let proof = CapabilityProof {
        capabilities: vec!["orchestration".to_string()],
        proof: "0123456789abcdef0123456789abcdef".to_string(),
        timestamp: SystemTime::now(),
    };
    m.verify_capabilities("s", proof).await.expect("cap");
    let counts = m.get_trust_level_counts().await;
    assert_eq!(counts.get(&TrustLevel::CapabilityVerified).copied().unwrap_or(0), 1);
}

#[tokio::test]
async fn cleanup_expired_noop_when_nothing_expired() {
    let m = TrustEscalationManager::with_defaults();
    m.establish_anonymous("fresh".to_string()).await.expect("establish");
    assert_eq!(m.cleanup_expired().await, 0);
}

#[tokio::test]
async fn verify_role_accepts_worker_role() {
    let m = TrustEscalationManager::with_defaults();
    m.establish_anonymous("vr".to_string()).await.expect("establish");
    let cap = CapabilityProof {
        capabilities: vec!["orchestration".to_string()],
        proof: "0123456789abcdef0123456789abcdef".to_string(),
        timestamp: SystemTime::now(),
    };
    m.verify_capabilities("vr", cap).await.expect("cap");
    m.verify_role("vr", "Worker".to_string()).await.expect("role");
    assert_eq!(m.get_trust_level("vr").await.expect("level"), TrustLevel::RoleVerified);
}

#[tokio::test]
async fn verify_role_accepts_observer_role() {
    let m = TrustEscalationManager::with_defaults();
    m.establish_anonymous("obs".to_string()).await.expect("establish");
    let cap = CapabilityProof {
        capabilities: vec!["orchestration".to_string()],
        proof: "0123456789abcdef0123456789abcdef".to_string(),
        timestamp: SystemTime::now(),
    };
    m.verify_capabilities("obs", cap).await.expect("cap");
    m.verify_role("obs", "observer".to_string()).await.expect("role");
    assert_eq!(m.get_trust_level("obs").await.expect("level"), TrustLevel::RoleVerified);
}

#[tokio::test]
async fn permission_denied_for_higher_level() {
    let m = TrustEscalationManager::with_defaults();
    m.establish_anonymous("p".to_string()).await.expect("establish");
    assert!(!m.check_permission("p", TrustLevel::CapabilityVerified).await.expect("check"));
}

#[tokio::test]
async fn get_relationship_none_for_unknown_session() {
    let m = TrustEscalationManager::with_defaults();
    assert!(m.get_relationship("nope").await.is_none());
}

#[test]
fn trust_escalation_manager_debug_smoke() {
    let m = TrustEscalationManager::with_defaults();
    let s = format!("{m:?}");
    assert!(s.contains("TrustEscalationManager"));
}

#[tokio::test]
async fn verify_role_rejects_admin_case_insensitive() {
    let m = TrustEscalationManager::with_defaults();
    m.establish_anonymous("adm".to_string()).await.expect("establish");
    let cap = CapabilityProof {
        capabilities: vec!["orchestration".to_string()],
        proof: "0123456789abcdef0123456789abcdef".to_string(),
        timestamp: SystemTime::now(),
    };
    m.verify_capabilities("adm", cap).await.expect("cap");
    assert!(m.verify_role("adm", "ADMIN".to_string()).await.is_err());
}
