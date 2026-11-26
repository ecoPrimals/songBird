// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Trust evaluation pipeline: tags → security provider → [`PeerTrustDecision`](super::types::PeerTrustDecision).

use anyhow::Result;
use songbird_http_client::lineage::{self, GateMode};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock, PoisonError};
use std::time::{Duration, Instant};
use tracing::{info, warn};

use crate::security_capability_client::{
    ConnectionInfo, SecurityCapabilityClient, TrustEvaluationRequest, TrustEvaluationResponse,
};

use super::types::{DiscoveredPeer, PeerTrustDecision};

/// Evaluate whether to trust a discovered peer
///
/// Consults security provider (security provider) for trust decision.
/// security provider checks genetic lineage and returns `auto_accept/prompt_user/reject`.
///
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn evaluate_peer_trust(
    peer: &DiscoveredPeer,
    security_client: &SecurityCapabilityClient,
) -> Result<PeerTrustDecision> {
    info!("🔍 Evaluating trust for peer: {}", peer.node_id);

    // A family tag is a claim. With the lineage gate on, the claim counts only once the peer
    // has proved it; until then the peer is evaluated as one that announced no family.
    let proven_tags = proven_tags(peer).await;
    let peer = &DiscoveredPeer {
        tags: proven_tags,
        ..peer.clone()
    };

    let peer_family = extract_family_from_tags(&peer.tags);

    if let Some(ref family) = peer_family {
        info!("🏷️  Peer {} family extracted from tags: {}", peer.node_id, family);
    } else {
        warn!("⚠️  Peer {} has no family tag - security provider will reject", peer.node_id);
    }

    let request = TrustEvaluationRequest {
        peer_id: peer.node_id.clone(),
        peer_family,
        peer_tags: peer.tags.clone(),
        connection_info: Some(ConnectionInfo {
            endpoint: peer.endpoint.clone(),
            protocol: String::from("tarpc"),
        }),
        context: Some({
            let mut ctx = std::collections::HashMap::new();
            ctx.insert(String::from("discovery_method"), peer.discovery_method.clone());
            ctx.insert(String::from("first_seen_at"), peer.first_seen_at.to_string());
            ctx
        }),
    };

    match security_client.evaluate_trust(&request).await {
        Ok(response) => handle_trust_response(&peer.node_id, response),
        Err(e) => {
            warn!("⚠️ security provider unavailable for peer {}: {}", peer.node_id, e);
            warn!("   Defaulting to prompt user (safe default)");

            Ok(PeerTrustDecision::PromptUser {
                reason: String::from("security_provider_unavailable"),
                peer_id: peer.node_id.clone(),
                recommendation: String::from("neutral"),
            })
        }
    }
}

/// How long a peer's proof (or its failure) stands before it is asked again.
const PROOF_STANDS: Duration = Duration::from_secs(300);
const PROOF_FAILURE_STANDS: Duration = Duration::from_secs(30);

/// Whether a discovery tag names a family at all.
fn is_family_claim(tag: &str) -> bool {
    tag.contains(":family:") || tag.contains("family_")
}

/// Whether any of the tags claims the family `ours`.
pub fn claims_family(tags: &[String], ours: &str) -> bool {
    !ours.is_empty()
        && tags.iter().any(|tag| {
            tag.split_once(":family:").is_some_and(|(_, rest)| {
                rest == ours || rest.strip_prefix(ours).is_some_and(|r| r.starts_with(':'))
            }) || tag.contains(&format!("family_{ours}"))
        })
}

/// The tags without any family claim.
pub fn without_family_claims(tags: &[String]) -> Vec<String> {
    tags.iter().filter(|tag| !is_family_claim(tag)).cloned().collect()
}

/// The peer's tags as far as they are proven.
///
/// Gate off, or the peer does not claim our family: the tags as announced. Otherwise the
/// peer is challenged (`lineage.prove`); if it cannot prove the claim, every family claim is
/// dropped (enforce) or kept with a warning (observe).
async fn proven_tags(peer: &DiscoveredPeer) -> Vec<String> {
    let gate = lineage::gate();
    if gate.mode() == GateMode::Off || !claims_family(&peer.tags, &gate.config().family_id) {
        return peer.tags.clone();
    }

    match cached_challenge(&peer.endpoint).await {
        Ok(device) => {
            info!(
                "🧬 Peer {} proved family '{}' as device '{}'",
                peer.node_id,
                gate.config().family_id,
                device
            );
            peer.tags.clone()
        }
        Err(reason) if gate.mode() == GateMode::Observe => {
            warn!(
                "🧬 Peer {} claims family '{}' without proof ({}) — observe mode, claim kept",
                peer.node_id,
                gate.config().family_id,
                reason
            );
            peer.tags.clone()
        }
        Err(reason) => {
            warn!(
                "🧬 Peer {} claims family '{}' without proof ({}) — claim dropped",
                peer.node_id,
                gate.config().family_id,
                reason
            );
            without_family_claims(&peer.tags)
        }
    }
}

/// [`challenge_peer`], remembered per endpoint so a discovery tick does not re-ask every peer.
async fn cached_challenge(endpoint: &str) -> Result<String, String> {
    /// When an endpoint was last challenged, and what came of it.
    type Asked = HashMap<String, (Instant, Result<String, String>)>;
    static ASKED: OnceLock<Mutex<Asked>> = OnceLock::new();
    let asked = ASKED.get_or_init(|| Mutex::new(HashMap::new()));

    let remembered = asked.lock().unwrap_or_else(PoisonError::into_inner).get(endpoint).cloned();
    if let Some((at, outcome)) = remembered {
        let stands = if outcome.is_ok() {
            PROOF_STANDS
        } else {
            PROOF_FAILURE_STANDS
        };
        if at.elapsed() < stands {
            return outcome;
        }
    }

    let outcome = challenge_peer(endpoint).await;
    let mut asked = asked.lock().unwrap_or_else(PoisonError::into_inner);
    asked.retain(|_, (at, _)| at.elapsed() < PROOF_STANDS);
    asked.insert(endpoint.to_string(), (Instant::now(), outcome.clone()));
    outcome
}

/// Ask the peer at `endpoint` to prove it belongs to our family. Returns its device name.
async fn challenge_peer(endpoint: &str) -> Result<String, String> {
    let uri: hyper::Uri =
        endpoint.parse().map_err(|e| format!("peer endpoint '{endpoint}': {e}"))?;
    let host = uri.host().ok_or_else(|| format!("peer endpoint '{endpoint}' has no host"))?;
    let port = uri.port_u16().ok_or_else(|| format!("peer endpoint '{endpoint}' has no port"))?;
    let addr = tokio::net::lookup_host((host.trim_matches(['[', ']']), port))
        .await
        .map_err(|e| format!("resolve {host}: {e}"))?
        .next()
        .ok_or_else(|| format!("{host} resolves to nothing"))?;

    challenge_addr(addr).await
}

/// Ask the songBird listening at `addr` to prove it belongs to our family: it must sign our
/// fresh nonce, for the address we dialled, with a key our family's anchor vouches for.
/// Returns its device name.
///
/// # Errors
///
/// Returns why the peer is not proven (unreachable, no such method, proof does not hold).
pub async fn challenge_addr(addr: std::net::SocketAddr) -> Result<String, String> {
    let nonce = lineage::new_nonce();
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": lineage::METHOD_PROVE,
        "params": { "nonce": nonce },
        "id": 1
    });
    let url = songbird_types::constants::endpoint_url(
        &addr.ip().to_string(),
        addr.port(),
        songbird_types::constants::JSONRPC_PATH,
    );
    let response = crate::mesh_trust_exchange::http_post_jsonrpc(&url, &request).await?;
    if let Some(error) = response.get("error").filter(|e| !e.is_null()) {
        let msg = error.get("message").and_then(serde_json::Value::as_str).unwrap_or("error");
        return Err(format!("peer refused to prove: {msg}"));
    }
    let answer = response.get("result").ok_or("peer gave no answer")?;
    lineage::gate().check_prove_answer(answer, addr, &nonce).await.map(|peer| peer.device_id)
}

/// Handle trust response from security provider
pub fn handle_trust_response(
    peer_id: &str,
    response: TrustEvaluationResponse,
) -> Result<PeerTrustDecision> {
    match response.decision.as_str() {
        "auto_accept" => {
            info!("✅ security provider says AUTO-ACCEPT peer {} ({})", peer_id, response.reason);
            info!(
                "   Trust level: {} | Confidence: {:.2}",
                response.trust_level, response.confidence
            );

            Ok(PeerTrustDecision::AutoAccept {
                reason: response.reason,
                confidence: response.confidence,
                encryption_tag: response.encryption_tag,
            })
        }

        "prompt_user" => {
            info!(
                "⚠️ security provider says PROMPT USER for peer {} ({})",
                peer_id, response.reason
            );
            info!(
                "   Trust level: {} | Confidence: {:.2}",
                response.trust_level, response.confidence
            );

            Ok(PeerTrustDecision::PromptUser {
                reason: response.reason.clone(),
                peer_id: peer_id.to_string(),
                recommendation: if response.confidence > 0.5 {
                    String::from("accept")
                } else {
                    String::from("neutral")
                },
            })
        }

        "reject" => {
            warn!("❌ security provider says REJECT peer {} ({})", peer_id, response.reason);
            warn!(
                "   Trust level: {} | Confidence: {:.2}",
                response.trust_level, response.confidence
            );

            Ok(PeerTrustDecision::Reject {
                reason: response.reason,
                trust_level: response.trust_level,
            })
        }

        unknown => {
            warn!("❓ Unknown decision from security provider: {}", unknown);
            warn!("   Defaulting to prompt user (safe default)");

            Ok(PeerTrustDecision::PromptUser {
                reason: format!("unknown_decision: {unknown}"),
                peer_id: peer_id.to_string(),
                recommendation: String::from("reject"),
            })
        }
    }
}

/// Extract family ID from peer tags (v3.14.1)
pub fn extract_family_from_tags(tags: &[String]) -> Option<String> {
    const FAMILY_TAG_PREFIXES: &[&str] = &["crypto:family:", "security:family:", "beardog:family:"];

    for tag in tags {
        for prefix in FAMILY_TAG_PREFIXES {
            if let Some(family_id) = tag.strip_prefix(prefix)
                && !family_id.is_empty()
            {
                return Some(family_id.to_string());
            }
        }
    }

    None
}
