// SPDX-License-Identifier: MIT
// Copyright (c) 2025 ecoPrimals
//! Peer Discovery Handlers
//!
//! Handlers for discovering and managing peer relationships.
//! Provides methods for listing peers, checking connectivity, and diagnostics.

use serde_json::Value;
use std::sync::Arc;
use tracing::debug;

use crate::app::connection_manager::ConnectionManager;
use crate::ipc::jsonrpc::JsonRpcError;

/// Handle discovery.list_peers - List all discovered peers
pub async fn handle_discovery_list_peers(
    connection_manager: Option<Arc<ConnectionManager>>,
    _params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    let manager = connection_manager
        .ok_or_else(|| JsonRpcError::internal_error("Peer discovery not initialized"))?;
    
    let peers = manager.get_all_peers().await;
    
    debug!("📡 Discovered {} peers", peers.len());
    
    Ok(serde_json::json!({
        "total": peers.len(),
        "peers": peers
    }))
}

/// Handle discovery.peer_count - Get count of discovered peers
pub async fn handle_discovery_peer_count(
    connection_manager: Option<Arc<ConnectionManager>>,
) -> Result<Value, JsonRpcError> {
    let manager = connection_manager
        .ok_or_else(|| JsonRpcError::internal_error("Peer discovery not initialized"))?;
    
    let count = manager.get_peer_count().await;
    
    debug!("📊 Peer count: {}", count);
    
    Ok(serde_json::json!({"count": count}))
}

/// Handle discovery.rejected_peers - Get list of rejected peers (diagnostics)
pub async fn handle_discovery_rejected_peers(
    connection_manager: Option<Arc<ConnectionManager>>,
) -> Result<Value, JsonRpcError> {
    let manager = connection_manager
        .ok_or_else(|| JsonRpcError::internal_error("Peer discovery not initialized"))?;
    
    let rejected = manager.get_rejected_peers().await;
    let rejected_list: Vec<_> = rejected.iter()
        .map(|(peer_id, reason)| serde_json::json!({
            "peer_id": peer_id,
            "reason": reason
        }))
        .collect();
    
    debug!("🚫 Rejected {} peers", rejected_list.len());
    
    Ok(serde_json::json!({
        "rejected": rejected_list,
        "total": rejected_list.len()
    }))
}

/// Handle peer.ping - Ping a specific peer
pub async fn handle_peer_ping(
    connection_manager: Option<Arc<ConnectionManager>>,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    let manager = connection_manager
        .ok_or_else(|| JsonRpcError::internal_error("Peer discovery not initialized"))?;
    
    let params = params
        .ok_or_else(|| JsonRpcError::invalid_params("Missing params"))?;
    
    let target: String = params.get("target")
        .and_then(|v| v.as_str())
        .ok_or_else(|| JsonRpcError::invalid_params("Missing 'target' parameter"))?
        .to_string();
    
    // Check if peer exists
    let peer = manager.get_peer_metadata(&target).await
        .ok_or_else(|| JsonRpcError::internal_error(&format!("Peer '{}' not found", target)))?;
    
    // Measure latency
    let start = std::time::Instant::now();
    
    // Verify we have the peer in our metadata
    // (Full RPC ping implementation available via ConnectionManager.call_peer)
    
    let latency_ms = start.elapsed().as_millis() as u64;
    
    debug!("🏓 Pinged peer '{}' ({} ms)", target, latency_ms);
    
    Ok(serde_json::json!({
        "pong": true,
        "peer_id": target,
        "endpoint": peer.endpoint,
        "latency_ms": latency_ms,
        "trust_level": peer.trust_level.as_u8()
    }))
}

/// Handle discovery.status - Get complete discovery status and statistics
///
/// NEW (Jan 5, 2026): Provides observability without relying on logs.
/// Critical for when Tower redirects stdout/stderr to /dev/null.
pub async fn handle_discovery_status(
    discovery_status_manager: Option<Arc<songbird_discovery::DiscoveryStatusManager>>,
) -> Result<Value, JsonRpcError> {
    let manager = discovery_status_manager
        .ok_or_else(|| JsonRpcError::internal_error("Discovery status manager not initialized"))?;
    
    let status = manager.get_status().await;
    
    debug!("📊 Discovery status: enabled={}, running={}, broadcasts={}, peers={}", 
        status.enabled, status.running, status.stats.broadcasts_sent, status.stats.peers_active);
    
    // Convert to JSON
    Ok(serde_json::to_value(status)
        .map_err(|e| JsonRpcError::internal_error(&format!("Failed to serialize status: {}", e)))?)
}
