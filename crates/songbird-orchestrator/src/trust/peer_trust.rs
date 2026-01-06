//! Peer Trust Evaluation
//!
//! Evaluates whether to trust discovered peers by consulting the security provider (BearDog).
//! This is part of the USB seed integration - BearDog makes the trust decision based on
//! genetic lineage derived from the USB family seed.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::security_capability_client::{
    SecurityCapabilityClient, TrustEvaluationRequest, TrustEvaluationResponse,
    ConnectionInfo,
};

/// Result of peer trust evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PeerTrustDecision {
    /// Automatically accept this peer (same family, high trust)
    AutoAccept {
        /// Reason for auto-acceptance
        reason: String,
        /// Confidence score (0.0-1.0)
        confidence: f64,
        /// Encryption tag if available
        encryption_tag: Option<String>,
    },
    
    /// Prompt user for consent (different family or unknown)
    PromptUser {
        /// Reason for prompting
        reason: String,
        /// Peer information for user review
        peer_id: String,
        /// Recommended action
        recommendation: String,
    },
    
    /// Reject this peer (no lineage, untrusted, security concern)
    Reject {
        /// Reason for rejection
        reason: String,
        /// Trust level
        trust_level: String,
    },
}

/// Discovered peer information
#[derive(Debug, Clone)]
pub struct DiscoveredPeer {
    /// Peer node ID
    pub node_id: String,
    /// Peer tags (including security provider encryption tags) - legacy
    pub tags: Vec<String>,
    /// Identity attestations (generic, structured) - NEW
    pub identity_attestations: Vec<crate::trust::UniversalIdentityAttestation>,
    /// Peer endpoint
    pub endpoint: String,
    /// Peer capabilities
    pub capabilities: Vec<String>,
    /// Discovery method
    pub discovery_method: String,
    /// When first seen
    pub first_seen_at: u64,
}

/// Evaluate whether to trust a discovered peer
///
/// Consults BearDog (security provider) for trust decision.
/// BearDog checks genetic lineage and returns auto_accept/prompt_user/reject.
///
/// # Architecture
///
/// - **Songbird**: Discovers peers, asks "should I trust?"
/// - **BearDog**: Knows lineage, answers yes/no/prompt
/// - **Clean API**: Songbird doesn't need crypto knowledge
///
/// # Arguments
///
/// * `peer` - Discovered peer information
/// * `beardog_client` - Client for querying BearDog
///
/// # Returns
///
/// Decision on whether to trust the peer
pub async fn evaluate_peer_trust(
    peer: &DiscoveredPeer,
    beardog_client: &SecurityCapabilityClient,
) -> Result<PeerTrustDecision> {
    info!("🔍 Evaluating trust for peer: {}", peer.node_id);
    
    // Build trust evaluation request
    let request = TrustEvaluationRequest {
        peer_id: peer.node_id.clone(),
        peer_tags: peer.tags.clone(),
        connection_info: Some(ConnectionInfo {
            endpoint: peer.endpoint.clone(),
            protocol: "tarpc".to_string(), // or "http" based on config
        }),
        context: Some({
            let mut ctx = std::collections::HashMap::new();
            ctx.insert("discovery_method".to_string(), peer.discovery_method.clone());
            ctx.insert("first_seen_at".to_string(), peer.first_seen_at.to_string());
            ctx
        }),
    };
    
    // Ask BearDog: "Should I trust this peer?"
    match beardog_client.evaluate_trust(&request).await {
        Ok(response) => {
            // Handle BearDog's decision
            handle_trust_response(&peer.node_id, response)
        }
        Err(e) => {
            // BearDog unavailable - default to prompting user
            warn!("⚠️ BearDog unavailable for peer {}: {}", peer.node_id, e);
            warn!("   Defaulting to prompt user (safe default)");
            
            Ok(PeerTrustDecision::PromptUser {
                reason: "security_provider_unavailable".to_string(),
                peer_id: peer.node_id.clone(),
                recommendation: "neutral".to_string(),
            })
        }
    }
}

/// Handle trust response from BearDog
fn handle_trust_response(
    peer_id: &str,
    response: TrustEvaluationResponse,
) -> Result<PeerTrustDecision> {
    match response.decision.as_str() {
        "auto_accept" => {
            info!("✅ BearDog says AUTO-ACCEPT peer {} ({})", peer_id, response.reason);
            info!("   Trust level: {} | Confidence: {:.2}", 
                  response.trust_level, response.confidence);
            
            Ok(PeerTrustDecision::AutoAccept {
                reason: response.reason,
                confidence: response.confidence,
                encryption_tag: response.encryption_tag,
            })
        }
        
        "prompt_user" => {
            info!("⚠️ BearDog says PROMPT USER for peer {} ({})", peer_id, response.reason);
            info!("   Trust level: {} | Confidence: {:.2}", 
                  response.trust_level, response.confidence);
            
            Ok(PeerTrustDecision::PromptUser {
                reason: response.reason.clone(),
                peer_id: peer_id.to_string(),
                recommendation: if response.confidence > 0.5 {
                    "accept".to_string()
                } else {
                    "neutral".to_string()
                },
            })
        }
        
        "reject" => {
            warn!("❌ BearDog says REJECT peer {} ({})", peer_id, response.reason);
            warn!("   Trust level: {} | Confidence: {:.2}", 
                  response.trust_level, response.confidence);
            
            Ok(PeerTrustDecision::Reject {
                reason: response.reason,
                trust_level: response.trust_level,
            })
        }
        
        unknown => {
            warn!("❓ Unknown decision from BearDog: {}", unknown);
            warn!("   Defaulting to prompt user (safe default)");
            
            Ok(PeerTrustDecision::PromptUser {
                reason: format!("unknown_decision: {}", unknown),
                peer_id: peer_id.to_string(),
                recommendation: "reject".to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_peer_trust_decision_types() {
        // Test decision types exist and can be created
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
        
        let decision = handle_trust_response("peer1", response).unwrap();
        
        match decision {
            PeerTrustDecision::AutoAccept { reason, confidence, .. } => {
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
        
        let decision = handle_trust_response("peer2", response).unwrap();
        
        match decision {
            PeerTrustDecision::PromptUser { reason, .. } => {
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
        
        let decision = handle_trust_response("peer3", response).unwrap();
        
        match decision {
            PeerTrustDecision::Reject { reason, .. } => {
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
        
        let decision = handle_trust_response("peer4", response).unwrap();
        
        // Unknown decisions should default to PromptUser (safe default)
        match decision {
            PeerTrustDecision::PromptUser { .. } => {},
            _ => panic!("Expected PromptUser for unknown decision"),
        }
    }
}

