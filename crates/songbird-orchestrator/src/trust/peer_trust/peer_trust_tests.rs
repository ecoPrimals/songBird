// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

use super::evaluation::{extract_family_from_tags, handle_trust_response};
use super::types::PeerTrustDecision;
use crate::security_capability_client::TrustEvaluationResponse;

#[test]
fn test_extract_family_from_tags_found() {
    let tags = vec![
        "some:other:tag".to_string(),
        "beardog:family:nat0".to_string(),
        "another:tag".to_string(),
    ];

    let family = extract_family_from_tags(&tags);
    assert_eq!(family, Some("nat0".to_string()));
}

#[test]
fn test_extract_family_from_tags_not_found() {
    let tags = vec!["some:other:tag".to_string(), "another:tag".to_string()];

    let family = extract_family_from_tags(&tags);
    assert_eq!(family, None);
}

#[test]
fn test_extract_family_from_tags_empty_family() {
    let tags = vec!["beardog:family:".to_string()];

    let family = extract_family_from_tags(&tags);
    assert_eq!(family, None);
}

#[test]
fn test_extract_family_from_tags_multiple_families() {
    let tags = vec!["beardog:family:nat0".to_string(), "beardog:family:acmecorp".to_string()];

    let family = extract_family_from_tags(&tags);
    assert_eq!(family, Some("nat0".to_string()));
}

#[test]
fn test_extract_family_from_tags_complex_family_id() {
    let tags = vec!["beardog:family:acmecorp-engineering-prod".to_string()];

    let family = extract_family_from_tags(&tags);
    assert_eq!(family, Some("acmecorp-engineering-prod".to_string()));
}

#[test]
fn test_peer_trust_decision_types() {
    let _auto = PeerTrustDecision::AutoAccept {
        reason: "test".to_string(),
        confidence: 1.0,
        encryption_tag: None,
    };

    let _prompt = PeerTrustDecision::PromptUser {
        reason: "test".to_string(),
        peer_id: "peer1".to_string(),
        recommendation: "neutral".to_string(),
    };

    let _reject = PeerTrustDecision::Reject {
        reason: "test".to_string(),
        trust_level: "none".to_string(),
    };
}

#[test]
fn test_handle_auto_accept_response() {
    use std::collections::HashMap;

    let response = TrustEvaluationResponse {
        decision: "auto_accept".to_string(),
        trust_level: "high".to_string(),
        confidence: 1.0,
        reason: "same_genetic_family".to_string(),
        encryption_tag: Some("beardog:family:a3f2".to_string()),
        metadata: HashMap::new(),
    };

    let decision = handle_trust_response("peer1", response).expect("handle");

    match decision {
        PeerTrustDecision::AutoAccept {
            reason,
            confidence,
            ..
        } => {
            assert_eq!(reason, "same_genetic_family");
            assert_eq!(confidence, 1.0);
        }
        _ => panic!("Expected AutoAccept"),
    }
}

#[test]
fn test_handle_prompt_user_response() {
    use std::collections::HashMap;

    let response = TrustEvaluationResponse {
        decision: "prompt_user".to_string(),
        trust_level: "low".to_string(),
        confidence: 0.5,
        reason: "different_genetic_family".to_string(),
        encryption_tag: None,
        metadata: HashMap::new(),
    };

    let decision = handle_trust_response("peer2", response).expect("handle");

    match decision {
        PeerTrustDecision::PromptUser {
            reason,
            ..
        } => {
            assert_eq!(reason, "different_genetic_family");
        }
        _ => panic!("Expected PromptUser"),
    }
}

#[test]
fn test_handle_reject_response() {
    use std::collections::HashMap;

    let response = TrustEvaluationResponse {
        decision: "reject".to_string(),
        trust_level: "none".to_string(),
        confidence: 0.0,
        reason: "no_genetic_lineage".to_string(),
        encryption_tag: None,
        metadata: HashMap::new(),
    };

    let decision = handle_trust_response("peer3", response).expect("handle");

    match decision {
        PeerTrustDecision::Reject {
            reason,
            ..
        } => {
            assert_eq!(reason, "no_genetic_lineage");
        }
        _ => panic!("Expected Reject"),
    }
}

#[test]
fn test_handle_unknown_response() {
    use std::collections::HashMap;

    let response = TrustEvaluationResponse {
        decision: "unknown_decision".to_string(),
        trust_level: "unknown".to_string(),
        confidence: 0.0,
        reason: "test".to_string(),
        encryption_tag: None,
        metadata: HashMap::new(),
    };

    let decision = handle_trust_response("peer4", response).expect("handle");

    match decision {
        PeerTrustDecision::PromptUser {
            ..
        } => {}
        _ => panic!("Expected PromptUser for unknown decision"),
    }
}

#[test]
fn handle_prompt_user_high_confidence_recommends_accept() {
    use std::collections::HashMap;
    let response = TrustEvaluationResponse {
        decision: "prompt_user".to_string(),
        trust_level: "medium".to_string(),
        confidence: 0.9,
        reason: "r".to_string(),
        encryption_tag: None,
        metadata: HashMap::new(),
    };
    let decision = handle_trust_response("p", response).expect("handle");
    match decision {
        PeerTrustDecision::PromptUser {
            recommendation,
            ..
        } => assert_eq!(recommendation, "accept"),
        _ => panic!("expected prompt"),
    }
}

#[test]
fn handle_prompt_user_low_confidence_recommends_neutral() {
    use std::collections::HashMap;
    let response = TrustEvaluationResponse {
        decision: "prompt_user".to_string(),
        trust_level: "low".to_string(),
        confidence: 0.4,
        reason: "r".to_string(),
        encryption_tag: None,
        metadata: HashMap::new(),
    };
    let decision = handle_trust_response("p", response).expect("handle");
    match decision {
        PeerTrustDecision::PromptUser {
            recommendation,
            ..
        } => assert_eq!(recommendation, "neutral"),
        _ => panic!("expected prompt"),
    }
}

#[test]
fn extract_family_ignores_wrong_prefix() {
    let tags = vec!["Beardog:family:nat0".to_string()];
    assert_eq!(extract_family_from_tags(&tags), None);
}

#[test]
fn extract_family_from_tags_empty_slice() {
    let tags: Vec<String> = vec![];
    assert_eq!(extract_family_from_tags(&tags), None);
}

#[test]
fn handle_prompt_user_confidence_boundary_half_recommends_neutral() {
    use std::collections::HashMap;
    let response = TrustEvaluationResponse {
        decision: "prompt_user".to_string(),
        trust_level: "low".to_string(),
        confidence: 0.5,
        reason: "boundary".to_string(),
        encryption_tag: None,
        metadata: HashMap::new(),
    };
    let decision = handle_trust_response("peer-b", response).expect("handle");
    match decision {
        PeerTrustDecision::PromptUser {
            recommendation,
            ..
        } => assert_eq!(recommendation, "neutral"),
        _ => panic!("expected prompt"),
    }
}

#[test]
fn handle_prompt_user_confidence_above_half_recommends_accept() {
    use std::collections::HashMap;
    let response = TrustEvaluationResponse {
        decision: "prompt_user".to_string(),
        trust_level: "low".to_string(),
        confidence: 0.5000001,
        reason: "boundary".to_string(),
        encryption_tag: None,
        metadata: HashMap::new(),
    };
    let decision = handle_trust_response("peer-b2", response).expect("handle");
    match decision {
        PeerTrustDecision::PromptUser {
            recommendation,
            ..
        } => assert_eq!(recommendation, "accept"),
        _ => panic!("expected prompt"),
    }
}

#[test]
fn handle_prompt_user_confidence_just_below_half_neutral() {
    use std::collections::HashMap;
    let response = TrustEvaluationResponse {
        decision: "prompt_user".to_string(),
        trust_level: "low".to_string(),
        confidence: 0.49,
        reason: "boundary".to_string(),
        encryption_tag: None,
        metadata: HashMap::new(),
    };
    let decision = handle_trust_response("peer-c", response).expect("handle");
    match decision {
        PeerTrustDecision::PromptUser {
            recommendation,
            ..
        } => assert_eq!(recommendation, "neutral"),
        _ => panic!("expected prompt"),
    }
}

#[test]
fn peer_trust_decision_serde_roundtrip_auto_accept() {
    let d = PeerTrustDecision::AutoAccept {
        reason: "r".to_string(),
        confidence: 0.75,
        encryption_tag: Some("t".to_string()),
    };
    let json = serde_json::to_string(&d).expect("serialize");
    let back: PeerTrustDecision = serde_json::from_str(&json).expect("deserialize");
    match back {
        PeerTrustDecision::AutoAccept {
            reason,
            confidence,
            ..
        } => {
            assert_eq!(reason, "r");
            assert!((confidence - 0.75).abs() < f64::EPSILON);
        }
        _ => panic!("Expected AutoAccept"),
    }
}

#[test]
fn unknown_decision_uses_reject_recommendation() {
    use std::collections::HashMap;
    let response = TrustEvaluationResponse {
        decision: "weird".to_string(),
        trust_level: "low".to_string(),
        confidence: 0.1,
        reason: "x".to_string(),
        encryption_tag: None,
        metadata: HashMap::new(),
    };
    let decision = handle_trust_response("peer-x", response).expect("handle");
    match decision {
        PeerTrustDecision::PromptUser {
            recommendation,
            reason,
            ..
        } => {
            assert_eq!(recommendation, "reject");
            assert!(reason.contains("unknown_decision"));
        }
        _ => panic!("expected PromptUser"),
    }
}

#[test]
fn reject_response_preserves_trust_level_string() {
    use std::collections::HashMap;
    let response = TrustEvaluationResponse {
        decision: "reject".to_string(),
        trust_level: "blocked".to_string(),
        confidence: 0.0,
        reason: "bad".to_string(),
        encryption_tag: None,
        metadata: HashMap::new(),
    };
    let decision = handle_trust_response("peer-r", response).expect("handle");
    match decision {
        PeerTrustDecision::Reject {
            trust_level,
            reason,
        } => {
            assert_eq!(trust_level, "blocked");
            assert_eq!(reason, "bad");
        }
        _ => panic!("expected Reject"),
    }
}

#[test]
fn auto_accept_passes_through_encryption_tag() {
    use std::collections::HashMap;
    let response = TrustEvaluationResponse {
        decision: "auto_accept".to_string(),
        trust_level: "high".to_string(),
        confidence: 1.0,
        reason: "ok".to_string(),
        encryption_tag: Some("tag-123".to_string()),
        metadata: HashMap::new(),
    };
    let decision = handle_trust_response("peer-a", response).expect("handle");
    match decision {
        PeerTrustDecision::AutoAccept {
            encryption_tag,
            ..
        } => assert_eq!(encryption_tag.as_deref(), Some("tag-123")),
        _ => panic!("expected AutoAccept"),
    }
}

#[test]
fn peer_trust_decision_serde_roundtrip_prompt_and_reject() {
    let p = PeerTrustDecision::PromptUser {
        reason: "ask".to_string(),
        peer_id: "p1".to_string(),
        recommendation: "neutral".to_string(),
    };
    let pj = serde_json::to_string(&p).expect("serialize");
    let pr: PeerTrustDecision = serde_json::from_str(&pj).expect("deserialize");
    assert!(matches!(pr, PeerTrustDecision::PromptUser { .. }));

    let r = PeerTrustDecision::Reject {
        reason: "no".to_string(),
        trust_level: "none".to_string(),
    };
    let rj = serde_json::to_string(&r).expect("serialize");
    let rr: PeerTrustDecision = serde_json::from_str(&rj).expect("deserialize");
    assert!(matches!(rr, PeerTrustDecision::Reject { .. }));
}

#[test]
fn prompt_user_peer_id_matches_input() {
    use std::collections::HashMap;
    let response = TrustEvaluationResponse {
        decision: "prompt_user".to_string(),
        trust_level: "low".to_string(),
        confidence: 0.2,
        reason: "review".to_string(),
        encryption_tag: None,
        metadata: HashMap::new(),
    };
    let decision = handle_trust_response("my-peer-id", response).expect("handle");
    match decision {
        PeerTrustDecision::PromptUser {
            peer_id,
            ..
        } => assert_eq!(peer_id, "my-peer-id"),
        _ => panic!("expected prompt"),
    }
}
