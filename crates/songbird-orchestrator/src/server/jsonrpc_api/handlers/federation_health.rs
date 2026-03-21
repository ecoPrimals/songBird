// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use serde_json::Value;
use tracing::debug;

use super::super::JsonRpcState;
use super::super::types::JsonRpcError;

/// songbird.federation.peers
pub async fn handle_federation_peers(state: &JsonRpcState) -> Result<Value, JsonRpcError> {
    let nodes = state.federation_state.active_nodes().await;
    let peers: Vec<Value> = nodes
        .iter()
        .map(|node| {
            serde_json::json!({
                "node_id": node.node_id,
                "name": node.node_name,
                "address": node.node_address,
                "endpoints": node.active_endpoints().iter().map(|ep| &ep.address).collect::<Vec<_>>(),
                "capabilities": node.capabilities,
            })
        })
        .collect();

    Ok(serde_json::json!({
        "peers": peers,
        "count": nodes.len(),
    }))
}

/// songbird.federation.join
pub async fn handle_federation_join(
    state: &JsonRpcState,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    let params = params.ok_or_else(|| JsonRpcError::invalid_params("Missing join parameters"))?;

    let obj = params
        .as_object()
        .ok_or_else(|| JsonRpcError::invalid_params("Parameters must be an object"))?;

    let node_id = obj
        .get("node_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| JsonRpcError::invalid_params("Missing 'node_id'"))?;
    let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or(node_id);

    let now = chrono::Utc::now();
    let registration = songbird_network_federation::state::NodeRegistration {
        node_id: node_id.to_string(),
        node_name: name.to_string(),
        node_address: obj.get("address").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
        endpoints: None,
        cpu_cores: 0,
        memory_gb: 0,
        gpu_model: None,
        storage_gb: None,
        capabilities: obj
            .get("capabilities")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default(),
        status: songbird_network_federation::state::NodeStatus::Active,
        joined_at: now,
        last_heartbeat: now,
    };

    state.federation_state.register_node(registration).await;

    let stats = state.federation_state.get_stats().await;

    Ok(serde_json::json!({
        "status": "joined",
        "node_id": node_id,
        "active_peers": stats.active_nodes,
    }))
}

/// songbird.protocol.capabilities
pub async fn handle_protocol_capabilities() -> Result<Value, JsonRpcError> {
    Ok(serde_json::json!({
        "songbird_version": env!("CARGO_PKG_VERSION"),
        "protocols": {
            "http": {
                "version": "1.1",
                "available": true
            },
            "jsonrpc": {
                "version": "2.0",
                "available": true
            },
            "tarpc": {
                "version": "0.34",
                "available": false,
                "coming_soon": "Week 3-4"
            }
        },
        "preferred_protocol": "jsonrpc",
        "fallback_protocol": "http"
    }))
}

/// songbird.health
pub async fn handle_health(state: &JsonRpcState) -> Result<Value, JsonRpcError> {
    let start_time = state.start_time.read().await;
    let uptime_seconds = start_time.elapsed().as_secs();

    Ok(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_seconds": uptime_seconds
    }))
}

/// songbird.version
pub async fn handle_version() -> Result<Value, JsonRpcError> {
    Ok(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "name": "Songbird Universal Orchestrator",
        "architecture": "100% Rust Core + Universal Compatibility"
    }))
}

/// health - biomeOS-standard health check
pub async fn handle_health_standard(state: &JsonRpcState) -> Result<Value, JsonRpcError> {
    let start_time = state.start_time.read().await;
    let uptime_seconds = start_time.elapsed().as_secs();

    let beardog_connected = {
        let beardog_socket = std::env::var("BEARDOG_SOCKET")
            .or_else(|_| std::env::var("CRYPTO_PROVIDER_SOCKET"))
            .unwrap_or_else(|_| "/tmp/biomeos/beardog.sock".to_string());
        std::path::Path::new(&beardog_socket).exists()
    };

    Ok(serde_json::json!({
        "status": "healthy",
        "uptime_seconds": uptime_seconds,
        "beardog_connected": beardog_connected,
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// identity - Return Songbird's identity and capabilities
pub async fn handle_identity() -> Result<Value, JsonRpcError> {
    let family_id = crate::env_config::family_id();

    let capabilities = vec![
        "network.broadcast",
        "network.listen",
        "network.beacon_exchange",
        "encrypt_discovery",
        "decrypt_discovery",
        "http.post",
        "http.get",
        "http.request",
        "discovery.announce",
        "discovery.query",
        "security.verify",
    ];

    Ok(serde_json::json!({
        "primal": "songbird",
        "version": env!("CARGO_PKG_VERSION"),
        "family_id": family_id,
        "capabilities": capabilities
    }))
}

/// `network.beacon_exchange` - Exchange encrypted beacons with peers
pub async fn handle_beacon_exchange(params: Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params.ok_or_else(|| JsonRpcError::invalid_params("Missing params"))?;

    let beacon = params
        .get("beacon")
        .ok_or_else(|| JsonRpcError::invalid_params("Missing 'beacon' parameter"))?;

    let peer_address = params.get("peer_address").and_then(|v| v.as_str());

    debug!("📡 Beacon exchange request received");
    if let Some(addr) = peer_address {
        debug!("   Peer: {}", addr);
    }

    Ok(serde_json::json!({
        "status": "received",
        "beacon_size": beacon.to_string().len(),
        "peer_address": peer_address,
        "message": "Beacon received. Full peer discovery available via IPC socket."
    }))
}
