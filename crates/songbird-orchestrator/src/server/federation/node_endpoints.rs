// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Federation Node Management Endpoints
//!
//! Handles node registration, status, heartbeat, and listing operations

use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use serde_json::{Value, json};
use songbird_network_federation::state::{FederationStatus, NodeRegistration, NodeStatus};
use std::sync::Arc;
use tracing::{debug, info, warn};

use super::types::{FederationAppState, FederationHeartbeatResponse, HeartbeatRequest};
use crate::trust::TrustLevel;

/// POST /api/federation/join - Register node with federation
pub async fn federation_join(
    State(state): State<Arc<FederationAppState>>,
    Json(mut registration): Json<NodeRegistration>,
) -> impl IntoResponse {
    info!("🤝 Node '{}' ({}) joining federation", registration.node_name, registration.node_id);

    // Set timestamps
    registration.joined_at = Utc::now();
    registration.last_heartbeat = Utc::now();
    registration.status = NodeStatus::Active;

    // Register node
    state.federation_state.register_node(registration.clone()).await;

    // Return federation status
    let status = get_federation_status(&state).await;

    info!(
        "✅ Node '{}' joined - Federation now has {} active nodes",
        registration.node_name, status.active_nodes
    );

    (StatusCode::OK, Json(status))
}

/// GET /api/federation/status - Get federation status
pub async fn federation_status(State(state): State<Arc<FederationAppState>>) -> impl IntoResponse {
    debug!("📊 Federation status requested");
    let status = get_federation_status(&state).await;
    (StatusCode::OK, Json(status))
}

/// GET /api/federation/nodes - List all nodes
pub async fn federation_nodes(State(state): State<Arc<FederationAppState>>) -> impl IntoResponse {
    debug!("📋 Node list requested");

    let nodes: Vec<NodeRegistration> =
        state.federation_state.nodes.read().await.values().cloned().collect();

    (StatusCode::OK, Json(nodes))
}

/// POST /api/federation/heartbeat - Send heartbeat
pub async fn federation_heartbeat(
    State(state): State<Arc<FederationAppState>>,
    Json(heartbeat): Json<HeartbeatRequest>,
) -> impl IntoResponse {
    debug!("💓 Heartbeat from node: {}", heartbeat.node_id);

    // Update heartbeat
    state.federation_state.update_heartbeat(&heartbeat.node_id).await;

    // Check if node exists
    let nodes = state.federation_state.nodes.read().await;
    let node_exists = nodes.contains_key(&heartbeat.node_id);
    drop(nodes);

    if !node_exists {
        warn!("⚠️  Heartbeat from unknown node: {}", heartbeat.node_id);
        return (
            StatusCode::NOT_FOUND,
            Json(FederationHeartbeatResponse {
                acknowledged: false,
                federation_status: "node_not_registered".to_string(),
            }),
        );
    }

    (
        StatusCode::OK,
        Json(FederationHeartbeatResponse {
            acknowledged: true,
            federation_status: heartbeat.status.unwrap_or_else(|| "active".to_string()),
        }),
    )
}

/// GET /api/federation/nodes (with graduated disclosure) - NEW
pub async fn federation_nodes_graduated(
    State(state): State<Arc<FederationAppState>>,
) -> impl IntoResponse {
    debug!("📋 Node list requested (with graduated disclosure)");

    // For now, default to Anonymous trust level (most restrictive)
    // In production, extract session ID from headers and look up trust level
    let trust_level = TrustLevel::Anonymous;

    let nodes: Vec<NodeRegistration> =
        state.federation_state.nodes.read().await.values().cloned().collect();

    // Apply graduated disclosure to each node
    let filtered_nodes: Vec<Value> =
        nodes.iter().map(|node| filter_node_by_trust(node, &trust_level)).collect();

    (StatusCode::OK, Json(filtered_nodes))
}

/// GET /`api/federation/nodes/:node_id` - Get specific node details (with graduated disclosure)
pub async fn get_node_details(
    State(state): State<Arc<FederationAppState>>,
    Path(node_id): Path<String>,
) -> impl IntoResponse {
    debug!("📋 Node details requested for: {}", node_id);

    // For now, default to Anonymous trust level
    // In production, extract session ID from headers and look up trust level
    let trust_level = TrustLevel::Anonymous;

    let nodes = state.federation_state.nodes.read().await;
    let node = nodes.get(&node_id);

    if let Some(node) = node {
        let filtered_node = filter_node_by_trust(node, &trust_level);
        (StatusCode::OK, Json(filtered_node))
    } else {
        let error = json!({
            "error": "Node not found",
            "node_id": node_id
        });
        (StatusCode::NOT_FOUND, Json(error))
    }
}

// ========================================================================
// HELPER FUNCTIONS
// ========================================================================

/// Helper: Get federation status
#[expect(
    clippy::similar_names,
    reason = "`state` and `stats` are semantically different despite similar names"
)]
async fn get_federation_status(state: &FederationAppState) -> FederationStatus {
    let stats = state.federation_state.get_stats().await;
    let nodes: Vec<NodeRegistration> =
        state.federation_state.nodes.read().await.values().cloned().collect();

    let uptime_seconds = (Utc::now() - state.federation_state.created_at).num_seconds();

    FederationStatus {
        federation_id: state.federation_state.federation_id.to_string(),
        active_nodes: stats.active_nodes,
        nodes,
        total_cpu_cores: stats.total_cpu_cores,
        total_memory_gb: stats.total_memory_gb,
        total_storage_gb: stats.total_storage_gb,
        uptime_seconds,
    }
}

/// Filter node information based on trust level (graduated disclosure)
fn filter_node_by_trust(node: &NodeRegistration, trust_level: &TrustLevel) -> Value {
    match trust_level {
        TrustLevel::Anonymous => {
            // Anonymous: Only basic capabilities
            json!({
                "node_id": node.node_id,
                "capabilities": node.capabilities,
            })
        }
        TrustLevel::CapabilityVerified => {
            // Capability-Verified: + Name and status
            json!({
                "node_id": node.node_id,
                "node_name": node.node_name,
                "capabilities": node.capabilities,
                "status": format!("{:?}", node.status),
            })
        }
        TrustLevel::RoleVerified => {
            // Role-Verified: + Resource info
            json!({
                "node_id": node.node_id,
                "node_name": node.node_name,
                "capabilities": node.capabilities,
                "status": format!("{:?}", node.status),
                "cpu_cores": node.cpu_cores,
                "memory_gb": node.memory_gb,
                "gpu_model": node.gpu_model,
                "joined_at": node.joined_at,
            })
        }
        TrustLevel::IdentityVerified => {
            // Identity-Verified: + Address and heartbeat
            json!({
                "node_id": node.node_id,
                "node_name": node.node_name,
                "node_address": node.node_address,
                "capabilities": node.capabilities,
                "status": format!("{:?}", node.status),
                "cpu_cores": node.cpu_cores,
                "memory_gb": node.memory_gb,
                "gpu_model": node.gpu_model,
                "storage_gb": node.storage_gb,
                "joined_at": node.joined_at,
                "last_heartbeat": node.last_heartbeat,
            })
        }
        TrustLevel::HardwareVerified => {
            // Hardware-Verified: Full access (all fields)
            json!({
                "node_id": node.node_id,
                "node_name": node.node_name,
                "node_address": node.node_address,
                "capabilities": node.capabilities,
                "status": format!("{:?}", node.status),
                "cpu_cores": node.cpu_cores,
                "memory_gb": node.memory_gb,
                "gpu_model": node.gpu_model,
                "storage_gb": node.storage_gb,
                "joined_at": node.joined_at,
                "last_heartbeat": node.last_heartbeat,
            })
        }
    }
}
