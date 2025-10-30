//! Federation API Endpoints
//!
//! HTTP/REST API for federation coordination

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use songbird_network_federation::state::{
    FederationState, FederationStatus, NodeRegistration, NodeStatus,
};
use songbird_network_federation::service_registry::{
    FederatedServiceRegistry, ServiceRegistration,
};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Shared application state for federation
#[derive(Debug, Clone)]
pub struct FederationAppState {
    pub federation_state: Arc<FederationState>,
    pub service_registry: Arc<FederatedServiceRegistry>,
}

/// Create federation routes
pub fn federation_routes(
    federation_state: Arc<FederationState>,
    service_registry: Arc<FederatedServiceRegistry>,
) -> Router {
    let app_state = Arc::new(FederationAppState {
        federation_state,
        service_registry,
    });
    
    Router::new()
        // Node management
        .route("/join", post(federation_join))
        .route("/status", get(federation_status))
        .route("/nodes", get(federation_nodes))
        .route("/heartbeat", post(federation_heartbeat))
        // Service management
        .route("/services", get(list_services))
        .route("/services", post(register_service))
        .route("/services/:service_id", get(get_service))
        .route("/services/type/:service_type", get(find_services_by_type))
        .route("/services/stats", get(service_stats))
        .with_state(app_state)
}

/// POST /api/federation/join - Register node with federation
async fn federation_join(
    State(state): State<Arc<FederationAppState>>,
    Json(mut registration): Json<NodeRegistration>,
) -> impl IntoResponse {
    info!(
        "🤝 Node '{}' ({}) joining federation",
        registration.node_name, registration.node_id
    );
    
    // Set timestamps
    registration.joined_at = Utc::now();
    registration.last_heartbeat = Utc::now();
    registration.status = NodeStatus::Active;
    
    // Register node
    state
        .federation_state
        .register_node(registration.clone())
        .await;
    
    // Return federation status
    let status = get_federation_status(&state).await;
    
    info!(
        "✅ Node '{}' joined - Federation now has {} active nodes",
        registration.node_name, status.active_nodes
    );
    
    (StatusCode::OK, Json(status))
}

/// GET /api/federation/status - Get federation status
async fn federation_status(
    State(state): State<Arc<FederationAppState>>,
) -> impl IntoResponse {
    debug!("📊 Federation status requested");
    let status = get_federation_status(&state).await;
    (StatusCode::OK, Json(status))
}

/// GET /api/federation/nodes - List all nodes
async fn federation_nodes(
    State(state): State<Arc<FederationAppState>>,
) -> impl IntoResponse {
    debug!("📋 Node list requested");
    
    let nodes: Vec<NodeRegistration> = state
        .federation_state
        .nodes
        .read()
        .await
        .values()
        .cloned()
        .collect();
    
    (StatusCode::OK, Json(nodes))
}

/// POST /api/federation/heartbeat - Send heartbeat
#[derive(Debug, Deserialize)]
struct HeartbeatRequest {
    node_id: String,
    #[allow(dead_code)]
    timestamp: String,
    status: Option<String>,
    #[allow(dead_code)]
    metrics: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct HeartbeatResponse {
    acknowledged: bool,
    federation_status: String,
}

async fn federation_heartbeat(
    State(state): State<Arc<FederationAppState>>,
    Json(heartbeat): Json<HeartbeatRequest>,
) -> impl IntoResponse {
    debug!("💓 Heartbeat from node: {}", heartbeat.node_id);
    
    // Update heartbeat
    state
        .federation_state
        .update_heartbeat(&heartbeat.node_id)
        .await;
    
    // Check if node exists
    let nodes = state.federation_state.nodes.read().await;
    let node_exists = nodes.contains_key(&heartbeat.node_id);
    drop(nodes);
    
    if !node_exists {
        warn!("⚠️  Heartbeat from unknown node: {}", heartbeat.node_id);
        return (
            StatusCode::NOT_FOUND,
            Json(HeartbeatResponse {
                acknowledged: false,
                federation_status: "node_not_registered".to_string(),
            }),
        );
    }
    
    (
        StatusCode::OK,
        Json(HeartbeatResponse {
            acknowledged: true,
            federation_status: heartbeat.status.unwrap_or_else(|| "active".to_string()),
        }),
    )
}

/// Helper: Get federation status
async fn get_federation_status(state: &FederationAppState) -> FederationStatus {
    let stats = state.federation_state.get_stats().await;
    let nodes: Vec<NodeRegistration> = state
        .federation_state
        .nodes
        .read()
        .await
        .values()
        .cloned()
        .collect();
    
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

//
// ═══════════════════════════════════════════════════════════════
// SERVICE FEDERATION ENDPOINTS
// ═══════════════════════════════════════════════════════════════
//

/// GET /api/federation/services - List all services
async fn list_services(
    State(state): State<Arc<FederationAppState>>,
) -> impl IntoResponse {
    debug!("📋 Service list requested");
    
    let services = state.service_registry.get_all_services().await;
    (StatusCode::OK, Json(services))
}

/// POST /api/federation/services - Register a service
async fn register_service(
    State(state): State<Arc<FederationAppState>>,
    Json(service): Json<ServiceRegistration>,
) -> impl IntoResponse {
    info!(
        "📝 Service registration request: {} ({})",
        service.service_name, service.service_type
    );
    
    // Determine if this is a local or remote service
    // (For now, treat all as remote since they come via API)
    state.service_registry.register_remote(service).await;
    
    (StatusCode::CREATED, Json(serde_json::json!({
        "status": "registered",
        "message": "Service registered successfully"
    })))
}

/// GET /api/federation/services/:service_id - Get specific service
async fn get_service(
    State(state): State<Arc<FederationAppState>>,
    axum::extract::Path(service_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    debug!("🔍 Service lookup: {}", service_id);
    
    match state.service_registry.find_by_id(&service_id).await {
        Some(service) => (StatusCode::OK, Json(service)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Service not found",
                "service_id": service_id
            })),
        )
            .into_response(),
    }
}

/// GET /api/federation/services/type/:service_type - Find services by type
async fn find_services_by_type(
    State(state): State<Arc<FederationAppState>>,
    axum::extract::Path(service_type): axum::extract::Path<String>,
) -> impl IntoResponse {
    debug!("🔍 Services by type: {}", service_type);
    
    let services = state
        .service_registry
        .find_by_type(&service_type)
        .await;
    
    (StatusCode::OK, Json(services))
}

/// GET /api/federation/services/stats - Get service registry statistics
async fn service_stats(
    State(state): State<Arc<FederationAppState>>,
) -> impl IntoResponse {
    debug!("📊 Service stats requested");
    
    let stats = state.service_registry.get_stats().await;
    (StatusCode::OK, Json(stats))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;
    
    fn create_test_state() -> Arc<FederationAppState> {
        Arc::new(FederationAppState {
            federation_state: Arc::new(FederationState::new()),
            service_registry: Arc::new(FederatedServiceRegistry::new()),
        })
    }
    
    #[tokio::test]
    async fn test_federation_status_endpoint() {
        let state = create_test_state();
        let app = Router::new()
            .route("/status", get(federation_status))
            .with_state(state);
        
        let response = app
            .oneshot(Request::builder().uri("/status").body(Body::empty()).unwrap())
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::OK);
    }
}

