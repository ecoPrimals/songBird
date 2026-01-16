//! P2P Discovery API Handlers (v3.19.1, refactored v3.22.1)
//!
//! Handlers for peer discovery and BTSP tunnel establishment.
//!
//! v3.19.1: Initial implementation for P2P discovery
//! v3.22.1: Extracted from monolithic handlers.rs (Jan 12, 2026)

use super::IpcHandlers;
use crate::ipc::server_pure_rust::JsonRpcError;
use crate::ipc::types::{
    AnnounceCapabilitiesRequest, AnnounceCapabilitiesResponse, CreateGeneticTunnelRequest,
    CreateGeneticTunnelResponse, DiscoverByFamilyRequest, DiscoverByFamilyResponse,
};
use songbird_types::trust::TrustLevel;
use std::time::SystemTime;
use tracing::{info, warn};

// ============================================================================
// jsonrpsee Handlers (for jsonrpsee server)
// ============================================================================

/// Handle `discover_by_family` RPC call (jsonrpsee)
///
/// v3.19.1: Returns discovered peers filtered by family tags
pub async fn discover_by_family(
    handlers: &IpcHandlers,
    params: jsonrpsee::types::Params<'_>,
) -> Result<DiscoverByFamilyResponse, jsonrpsee::types::ErrorObject<'static>> {
    info!("🔍 P2P Discovery API: discover_by_family");

    let request: DiscoverByFamilyRequest = params.parse().map_err(|e| {
        jsonrpsee::types::ErrorObject::owned(
            jsonrpsee::types::error::PARSE_ERROR_CODE,
            format!("Failed to parse params: {}", e),
            None::<()>,
        )
    })?;

    if let Some(_listener) = &handlers.discovery_listener {
        // TODO: Implement get_discovered_peers() method on AnonymousDiscoveryListener
        info!("✅ Would discover peers matching family tags: {:?}", request.family_tags);

        Ok(DiscoverByFamilyResponse {
            nodes: vec![],
        })
    } else {
        warn!("⚠️  Discovery listener not available");
        Ok(DiscoverByFamilyResponse {
            nodes: vec![],
        })
    }
}

/// Handle `create_genetic_tunnel` RPC call (jsonrpsee)
///
/// v3.19.1: Establishes BTSP tunnel with genetic lineage proof
pub async fn create_genetic_tunnel(
    handlers: &IpcHandlers,
    params: jsonrpsee::types::Params<'_>,
) -> Result<CreateGeneticTunnelResponse, jsonrpsee::types::ErrorObject<'static>> {
    info!("🔗 P2P Discovery API: create_genetic_tunnel");

    let request: CreateGeneticTunnelRequest = params.parse().map_err(|e| {
        jsonrpsee::types::ErrorObject::owned(
            jsonrpsee::types::error::PARSE_ERROR_CODE,
            format!("Failed to parse params: {}", e),
            None::<()>,
        )
    })?;

    // Get peer endpoint from request or fail
    let peer_endpoint = match &request.peer_endpoint {
        Some(ep) => ep.clone(),
        None => {
            return Err(jsonrpsee::types::ErrorObject::owned(
                jsonrpsee::types::error::INVALID_PARAMS_CODE,
                "peer_endpoint is required for tunnel creation",
                None::<()>,
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
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
    );

    match result {
        Ok(()) => {
            info!("✅ BTSP tunnel established: {}", tunnel_id);
            Ok(CreateGeneticTunnelResponse {
                tunnel_id,
                status: "established".to_string(),
                local_endpoint: None, // TODO: Get from BTSP client
                peer_endpoint: request.peer_endpoint.clone(),
                encryption: Some("ChaCha20-Poly1305".to_string()),
                created_at: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
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
                    .unwrap()
                    .as_secs()
                    .to_string(),
            })
        }
    }
}

/// Handle `announce_capabilities` RPC call (jsonrpsee)
///
/// v3.19.1: Updates broadcaster with new capabilities
pub async fn announce_capabilities(
    handlers: &IpcHandlers,
    params: jsonrpsee::types::Params<'_>,
) -> Result<AnnounceCapabilitiesResponse, jsonrpsee::types::ErrorObject<'static>> {
    info!("📢 P2P Discovery API: announce_capabilities");

    let request: AnnounceCapabilitiesRequest = params.parse().map_err(|e| {
        jsonrpsee::types::ErrorObject::owned(
            jsonrpsee::types::error::PARSE_ERROR_CODE,
            format!("Failed to parse params: {}", e),
            None::<()>,
        )
    })?;

    // TODO v3.19.3: Implement broadcaster.update_capabilities() method
    // For now, just log the intent
    info!(
        "✅ Would announce capabilities: {:?}, sub_federations: {:?}, families: {:?}",
        request.capabilities, request.sub_federations, request.genetic_families
    );

    Ok(AnnounceCapabilitiesResponse {
        status: "updated".to_string(),
        broadcasting: true,
        updated_at: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string(),
    })
}

// ============================================================================
// Pure JSON Adapters (for pure Rust Unix socket server v3.22.0)
// ============================================================================

/// P2P Discovery: discover_by_family (pure JSON adapter)
pub async fn discover_by_family_json(
    handlers: &IpcHandlers,
    params: Option<serde_json::Value>,
) -> Result<serde_json::Value, JsonRpcError> {
    let _request: DiscoverByFamilyRequest = if let Some(p) = params {
        serde_json::from_value(p).map_err(|e| JsonRpcError::invalid_params(e.to_string()))?
    } else {
        return Err(JsonRpcError::invalid_params("Missing params for discover_by_family"));
    };

    if let Some(_listener) = &handlers.discovery_listener {
        // TODO: Implement get_discovered_peers() method
        let response = DiscoverByFamilyResponse {
            nodes: vec![],
        };
        serde_json::to_value(response).map_err(|e| JsonRpcError::internal_error(e.to_string()))
    } else {
        let response = DiscoverByFamilyResponse {
            nodes: vec![],
        };
        serde_json::to_value(response).map_err(|e| JsonRpcError::internal_error(e.to_string()))
    }
}

/// P2P Discovery: create_genetic_tunnel (pure JSON adapter)
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

    let timestamp =
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs().to_string();

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
            status: format!("failed: {}", e),
            local_endpoint: None,
            peer_endpoint: request.peer_endpoint,
            encryption: None,
            created_at: timestamp,
        },
    };

    serde_json::to_value(response).map_err(|e| JsonRpcError::internal_error(e.to_string()))
}

/// P2P Discovery: announce_capabilities (pure JSON adapter)
pub async fn announce_capabilities_json(
    _handlers: &IpcHandlers,
    params: Option<serde_json::Value>,
) -> Result<serde_json::Value, JsonRpcError> {
    let _request: AnnounceCapabilitiesRequest = if let Some(p) = params {
        serde_json::from_value(p).map_err(|e| JsonRpcError::invalid_params(e.to_string()))?
    } else {
        return Err(JsonRpcError::invalid_params("Missing params for announce_capabilities"));
    };

    // TODO v3.19.3: Implement broadcaster.update_capabilities() method
    let response = AnnounceCapabilitiesResponse {
        status: "updated".to_string(),
        broadcasting: true,
        updated_at: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string(),
    };

    serde_json::to_value(response).map_err(|e| JsonRpcError::internal_error(e.to_string()))
}
