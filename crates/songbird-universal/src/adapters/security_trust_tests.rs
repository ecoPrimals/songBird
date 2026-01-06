//! Tests for SecurityAdapter trust evaluation methods

use super::*;

#[tokio::test]
async fn test_trust_evaluation_request_creation() {
    let request = crate::trust_types::TrustEvaluationRequest::new(
        "tower2",
        vec!["crypto:family:a3f2".to_string()],
    );
    
    assert_eq!(request.peer_id, "tower2");
    assert_eq!(request.peer_tags.len(), 1);
}

#[tokio::test]
async fn test_trust_evaluation_response_helpers() {
    let response = crate::trust_types::TrustEvaluationResponse {
        decision: "auto_accept".to_string(),
        trust_level: songbird_types::TrustLevel::Highest,
        reason: "Same family".to_string(),
        suggested_action: None,
        metadata: None,
    };
    
    assert!(response.is_auto_accept());
    assert!(!response.is_reject());
}

#[tokio::test]
async fn test_identity_response_structure() {
    let identity = crate::trust_types::IdentityResponse {
        encryption_tag: "crypto:family:test".to_string(),
        capabilities: vec!["identity".to_string(), "trust-evaluation".to_string()],
    };
    
    assert!(identity.encryption_tag.contains("crypto:family"));
    assert_eq!(identity.capabilities.len(), 2);
}

// Integration tests would go here when we have a mock security provider
// For now, these structural tests validate the types work correctly

