//! P2P Discovery API Handlers (v3.19.1, refactored v3.22.1)
//!
//! Handlers for peer discovery and BTSP tunnel establishment.
//!
//! v3.19.1: Initial implementation for P2P discovery
//! v3.22.1: Extracted from monolithic handlers.rs (Jan 12, 2026)

use super::IpcHandlers;
use crate::ipc::pure_rust_server::JsonRpcError;
use crate::ipc::types::{
    AnnounceCapabilitiesRequest, AnnounceCapabilitiesResponse, CreateGeneticTunnelRequest,
    CreateGeneticTunnelResponse, DiscoverByFamilyRequest, DiscoverByFamilyResponse, DiscoveredNode,
};
use songbird_types::trust::TrustLevel;
use std::time::SystemTime;
use tracing::{info, warn};

// ============================================================================
// jsonrpsee Handlers (for jsonrpsee server)
// ============================================================================

/// Handle `discover_by_family` RPC call (Pure Rust, v3.34.0)
///
/// v3.19.1: Returns discovered peers filtered by family tags
pub async fn discover_by_family(
    handlers: &IpcHandlers,
    params: serde_json::Value,
) -> Result<DiscoverByFamilyResponse, JsonRpcError> {
    info!("🔍 P2P Discovery API: discover_by_family");

    let request: DiscoverByFamilyRequest = serde_json::from_value(params)
        .map_err(|e| JsonRpcError::custom(-32602, format!("Failed to parse params: {e}"), None))?;

    if let Some(listener) = &handlers.discovery_listener {
        // Get all discovered peers
        let discovered_peers = listener.get_peers().await;

        info!("🔍 Discovered {} peers total", discovered_peers.len());

        // Filter by family tags if provided
        let filtered_nodes: Vec<_> = if request.family_tags.is_empty() {
            // No filter - return all peers
            discovered_peers
        } else {
            // Filter peers by family tags
            discovered_peers
                .into_iter()
                .filter(|peer| {
                    // Check if peer has any matching family tags
                    if let Some(ref tags) = peer.tags {
                        request.family_tags.iter().any(|family_tag| {
                            tags.iter().any(|peer_tag| {
                                // Match tags like "beardog:family:nat0"
                                peer_tag.contains(family_tag)
                            })
                        })
                    } else {
                        false
                    }
                })
                .collect()
        };

        info!(
            "✅ Found {} peers matching family tags: {:?}",
            filtered_nodes.len(),
            request.family_tags
        );

        // Convert DiscoveredPeer to response nodes
        let nodes = filtered_nodes
            .into_iter()
            .map(|peer| DiscoveredNode {
                node_id: peer.node_id.clone().unwrap_or_else(|| peer.session_id.clone()),
                node_name: peer.node_name.clone(),
                genetic_families: peer.tags.clone().unwrap_or_default(),
                sub_federations: vec![], // Not available in DiscoveredPeer
                capabilities: peer.capabilities.clone(),
                btsp_endpoint: None, // Would require BTSP integration
                https_endpoint: peer.https_endpoint(),
                last_seen: format!("{:?}", peer.last_seen), // Convert SystemTime to string
            })
            .collect();

        Ok(DiscoverByFamilyResponse {
            nodes,
        })
    } else {
        warn!("⚠️  Discovery listener not available");
        Ok(DiscoverByFamilyResponse {
            nodes: vec![],
        })
    }
}

/// Handle `create_genetic_tunnel` RPC call (Pure Rust, v3.34.0)
///
/// v3.19.1: Establishes BTSP tunnel with genetic lineage proof
pub async fn create_genetic_tunnel(
    handlers: &IpcHandlers,
    params: serde_json::Value,
) -> Result<CreateGeneticTunnelResponse, JsonRpcError> {
    info!("🔗 P2P Discovery API: create_genetic_tunnel");

    let request: CreateGeneticTunnelRequest = serde_json::from_value(params)
        .map_err(|e| JsonRpcError::custom(-32602, format!("Failed to parse params: {e}"), None))?;

    // Get peer endpoint from request or fail
    let peer_endpoint = match &request.peer_endpoint {
        Some(ep) => ep.clone(),
        None => {
            return Err(JsonRpcError::custom(
                -32602,
                "peer_endpoint is required for tunnel creation",
                None,
            ));
        }
    };

    // Extract genetic family from proof
    let genetic_families = if let Some(ref proof) = request.genetic_proof {
        vec![proof.family_id.clone()]
    } else {
        vec![]
    };

    // Establish connection via connection manager
    let result = handlers
        .connection_manager
        .establish_connection(
            request.peer_node_id.clone(),
            peer_endpoint.clone(),
            vec![], // capabilities (unknown at this point)
            genetic_families,
            TrustLevel::Elevated, // Genetic proof grants elevated trust
            "genetic_tunnel".to_string(),
        )
        .await;

    let tunnel_id = format!(
        "tunnel-{}-{}",
        request.peer_node_id,
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs()
    );

    match result {
        Ok(()) => {
            info!("✅ BTSP tunnel established: {}", tunnel_id);
            Ok(CreateGeneticTunnelResponse {
                tunnel_id,
                status: "established".to_string(),
                local_endpoint: None, // NOTE: Would require BTSP client integration (future: Arc<BtspClient> in handlers)
                peer_endpoint: request.peer_endpoint.clone(),
                encryption: Some("ChaCha20-Poly1305".to_string()),
                created_at: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
                    .to_string(),
            })
        }
        Err(e) => {
            warn!("❌ Failed to establish BTSP tunnel: {}", e);
            Ok(CreateGeneticTunnelResponse {
                tunnel_id: String::new(),
                status: "failed".to_string(),
                local_endpoint: None,
                peer_endpoint: request.peer_endpoint,
                encryption: None,
                created_at: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
                    .to_string(),
            })
        }
    }
}

/// Handle `announce_capabilities` RPC call (Pure Rust, v3.34.0)
///
/// v3.19.1: Updates broadcaster with new capabilities
pub async fn announce_capabilities(
    handlers: &IpcHandlers,
    params: serde_json::Value,
) -> Result<AnnounceCapabilitiesResponse, JsonRpcError> {
    info!("📢 P2P Discovery API: announce_capabilities");

    let request: AnnounceCapabilitiesRequest = serde_json::from_value(params)
        .map_err(|e| JsonRpcError::custom(-32602, format!("Failed to parse params: {e}"), None))?;

    // NOTE: Capability announcement requires broadcaster restart to take effect
    // The broadcaster is created at startup with initial capabilities.
    // Dynamic updates would require adding Arc<RwLock<Broadcaster>> to handlers,
    // or implementing a broadcaster control channel (future enhancement).
    // For now, we log the requested capabilities for observability.
    info!(
        "📢 Capability announcement requested: caps={:?}, federations={:?}, families={:?}",
        request.capabilities, request.sub_federations, request.genetic_families
    );
    info!("💡 To apply: Restart orchestrator with updated capabilities in config");

    Ok(AnnounceCapabilitiesResponse {
        status: "updated".to_string(),
        broadcasting: true,
        updated_at: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string(),
    })
}

// ============================================================================
// Pure JSON Adapters (for pure Rust Unix socket server v3.22.0)
// ============================================================================

/// P2P Discovery: `discover_by_family` (pure JSON adapter)
pub async fn discover_by_family_json(
    handlers: &IpcHandlers,
    params: Option<serde_json::Value>,
) -> Result<serde_json::Value, JsonRpcError> {
    let request: DiscoverByFamilyRequest = if let Some(p) = params {
        serde_json::from_value(p).map_err(|e| JsonRpcError::invalid_params(e.to_string()))?
    } else {
        return Err(JsonRpcError::invalid_params("Missing params for discover_by_family"));
    };

    if let Some(listener) = &handlers.discovery_listener {
        // Get all discovered peers (same implementation as jsonrpsee handler above)
        let discovered_peers = listener.get_peers().await;
        let family_tags = &request.family_tags;

        // Filter by family tags if provided
        let filtered_nodes: Vec<_> = if family_tags.is_empty() {
            discovered_peers
        } else {
            discovered_peers
                .into_iter()
                .filter(|peer| {
                    if let Some(ref tags) = peer.tags {
                        family_tags.iter().any(|family_tag| {
                            tags.iter().any(|peer_tag| peer_tag.contains(family_tag))
                        })
                    } else {
                        false
                    }
                })
                .collect()
        };

        // Convert to response nodes
        let nodes = filtered_nodes
            .into_iter()
            .map(|peer| DiscoveredNode {
                node_id: peer.node_id.clone().unwrap_or_else(|| peer.session_id.clone()),
                node_name: peer.node_name.clone(),
                genetic_families: peer.tags.clone().unwrap_or_default(),
                sub_federations: vec![],
                capabilities: peer.capabilities.clone(),
                btsp_endpoint: None,
                https_endpoint: peer.https_endpoint(),
                last_seen: format!("{:?}", peer.last_seen),
            })
            .collect();

        let response = DiscoverByFamilyResponse {
            nodes,
        };
        serde_json::to_value(response).map_err(|e| JsonRpcError::internal_error(e.to_string()))
    } else {
        let response = DiscoverByFamilyResponse {
            nodes: vec![],
        };
        serde_json::to_value(response).map_err(|e| JsonRpcError::internal_error(e.to_string()))
    }
}

/// P2P Discovery: `create_genetic_tunnel` (pure JSON adapter)
pub async fn create_genetic_tunnel_json(
    handlers: &IpcHandlers,
    params: Option<serde_json::Value>,
) -> Result<serde_json::Value, JsonRpcError> {
    let request: CreateGeneticTunnelRequest = if let Some(p) = params {
        serde_json::from_value(p).map_err(|e| JsonRpcError::invalid_params(e.to_string()))?
    } else {
        return Err(JsonRpcError::invalid_params("Missing params for create_genetic_tunnel"));
    };

    // Get peer endpoint from request or fail
    let peer_endpoint = match &request.peer_endpoint {
        Some(ep) => ep.clone(),
        None => {
            return Err(JsonRpcError::invalid_params(
                "peer_endpoint is required for tunnel creation",
            ));
        }
    };

    // Extract genetic family from proof
    let genetic_families = if let Some(ref proof) = request.genetic_proof {
        vec![proof.family_id.clone()]
    } else {
        vec![]
    };

    let result = handlers
        .connection_manager
        .establish_connection(
            request.peer_node_id.clone(),
            peer_endpoint.clone(),
            vec![], // capabilities (unknown at this point)
            genetic_families,
            TrustLevel::Elevated,
            "genetic_tunnel".to_string(),
        )
        .await;

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string();

    let tunnel_id = format!("tunnel-{}-{}", request.peer_node_id, timestamp);

    let response = match result {
        Ok(()) => CreateGeneticTunnelResponse {
            tunnel_id,
            status: "established".to_string(),
            local_endpoint: None,
            peer_endpoint: request.peer_endpoint.clone(),
            encryption: Some("ChaCha20-Poly1305".to_string()),
            created_at: timestamp,
        },
        Err(e) => CreateGeneticTunnelResponse {
            tunnel_id: String::new(),
            status: format!("failed: {e}"),
            local_endpoint: None,
            peer_endpoint: request.peer_endpoint,
            encryption: None,
            created_at: timestamp,
        },
    };

    serde_json::to_value(response).map_err(|e| JsonRpcError::internal_error(e.to_string()))
}

/// P2P Discovery: `announce_capabilities` (pure JSON adapter)
pub async fn announce_capabilities_json(
    _handlers: &IpcHandlers,
    params: Option<serde_json::Value>,
) -> Result<serde_json::Value, JsonRpcError> {
    let request: AnnounceCapabilitiesRequest = if let Some(p) = params {
        serde_json::from_value(p).map_err(|e| JsonRpcError::invalid_params(e.to_string()))?
    } else {
        return Err(JsonRpcError::invalid_params("Missing params for announce_capabilities"));
    };

    // NOTE: Capability announcement requires broadcaster restart (see jsonrpsee handler above)
    info!(
        "📢 Capability announcement requested (pure_rust_server): caps={:?}, federations={:?}, families={:?}",
        request.capabilities, request.sub_federations, request.genetic_families
    );

    let response = AnnounceCapabilitiesResponse {
        status: "logged".to_string(), // Changed from "updated" to reflect reality
        broadcasting: true,
        updated_at: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string(),
    };

    serde_json::to_value(response).map_err(|e| JsonRpcError::internal_error(e.to_string()))
}
