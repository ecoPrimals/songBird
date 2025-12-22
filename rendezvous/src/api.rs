//! HTTP API endpoints for rendezvous server

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

use crate::messages::*;
use crate::registry::SessionRegistry;

/// Register presence
pub async fn register_presence(
    State(registry): State<Arc<SessionRegistry>>,
    Json(msg): Json<RegisterPresenceMessage>,
) -> Result<Json<RegisterPresenceResponse>, StatusCode> {
    debug!("📥 Register presence request from node: {}", &msg.node_identity.node_id[..8]);

    // Generate ephemeral session ID (if not provided or invalid)
    let session_id = if msg.node_identity.ephemeral_session_id.is_empty() {
        Uuid::new_v4().to_string()
    } else {
        msg.node_identity.ephemeral_session_id.clone()
    };

    // Convert to registry types (just use message types directly)
    let session =
        registry.register_session(session_id.clone(), msg.node_identity, msg.network_context).await;

    info!("✅ Registered session: {}", &session_id[..8]);

    // Build response
    let response = RegisterPresenceResponse {
        status: "registered".to_string(),
        session_id: session_id.clone(),
        expires_at: session.expires_at,
        rendezvous_endpoint: Some(format!("wss://rendezvous/ws/{}", session_id)),
    };

    Ok(Json(response))
}

/// Heartbeat
pub async fn heartbeat(
    State(registry): State<Arc<SessionRegistry>>,
    Json(msg): Json<HeartbeatMessage>,
) -> Result<StatusCode, StatusCode> {
    debug!("💓 Heartbeat from: {}", &msg.session_id[..8]);

    if registry.heartbeat(&msg.session_id).await.is_some() {
        Ok(StatusCode::OK)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Query peers
pub async fn query_peers(
    State(registry): State<Arc<SessionRegistry>>,
    Json(msg): Json<QueryPeersMessage>,
) -> Result<Json<QueryPeersResponse>, StatusCode> {
    debug!("🔍 Query peers from: {}", &msg.requester.session_id[..8]);
    debug!("   Required caps: {:?}", msg.query.capabilities_required);

    // Find matching sessions
    let sessions = registry
        .query_peers(
            &msg.query.capabilities_required,
            &msg.query.exclude_node_ids,
            msg.query.max_results,
        )
        .await;

    // Convert to peer info (NO IP ADDRESSES!)
    let peers: Vec<PeerInfo> = sessions
        .into_iter()
        .map(|session| PeerInfo {
            ephemeral_session_id: session.session_id,
            public_key_fingerprint: session.public_key_fingerprint,
            capabilities: session.identity.capabilities,
            protocols: session.identity.protocols,
            network_context: NetworkContext {
                nat_type: session.network_context.nat_type,
                reachability: session.network_context.reachability,
                connection_quality: session.network_context.connection_quality,
            },
            last_heartbeat: session.last_heartbeat,
        })
        .collect();

    let count = peers.len();
    info!("🔍 Query returned {} peers", count);

    let response = QueryPeersResponse {
        peers: peers.clone(),
        total_matches: count,
        returned: peers.len(),
    };

    Ok(Json(response))
}

/// Get peer info by session ID
pub async fn get_peer_info(
    State(registry): State<Arc<SessionRegistry>>,
    Path(session_id): Path<String>,
) -> Result<Json<PeerInfo>, StatusCode> {
    debug!("ℹ️  Get peer info: {}", &session_id[..8]);

    if let Some(session) = registry.get_session(&session_id).await {
        let peer_info = PeerInfo {
            ephemeral_session_id: session.session_id,
            public_key_fingerprint: session.public_key_fingerprint,
            capabilities: session.identity.capabilities,
            protocols: session.identity.protocols,
            network_context: NetworkContext {
                nat_type: session.network_context.nat_type,
                reachability: session.network_context.reachability,
                connection_quality: session.network_context.connection_quality,
            },
            last_heartbeat: session.last_heartbeat,
        };

        Ok(Json(peer_info))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Request connection
pub async fn request_connection(
    State(_registry): State<Arc<SessionRegistry>>,
    Json(msg): Json<RequestConnectionMessage>,
) -> Result<Json<RequestConnectionResponse>, StatusCode> {
    info!(
        "🔗 Connection request: {} -> {}",
        &msg.requester.session_id[..8],
        &msg.target.session_id[..8]
    );

    // TODO: Forward to target via WebSocket
    // TODO: Create coordination token
    // TODO: Setup relay endpoint if needed

    let coordination_token = Uuid::new_v4().to_string();

    let response = RequestConnectionResponse {
        status: "forwarded".to_string(),
        coordination_token,
        relay_endpoint: None, // TODO: Implement relay
    };

    Ok(Json(response))
}

/// Respond to connection request
pub async fn respond_connection(
    State(_registry): State<Arc<SessionRegistry>>,
    Json(msg): Json<ResponseConnectionMessage>,
) -> Result<StatusCode, StatusCode> {
    info!(
        "📨 Connection response: token={}, decision={}",
        &msg.coordination_token[..8],
        msg.decision
    );

    // TODO: Forward response to original requester
    // TODO: Setup relay if decision is accept

    Ok(StatusCode::OK)
}
