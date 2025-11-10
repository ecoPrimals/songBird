//! Federation API Endpoints
//!
//! HTTP/REST API for federation coordination and capability registration

use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use songbird_network_federation::service_registry::{
    FederatedServiceRegistry, ServiceRegistration,
};
use songbird_network_federation::state::{
    FederationState, FederationStatus, NodeRegistration, NodeStatus,
};
use std::sync::Arc;
use tracing::{debug, info, warn};

// Import capability registry types
use crate::core::registry::{CapabilityRegistry, HeartbeatConfig};
use crate::core::registry::types::{
    CapabilityRegistrationRequest, CapabilityRegistrationResponse,
    HeartbeatRequest as CapabilityHeartbeatRequest, HeartbeatResponse,
    ProviderListResponse, RegistrationData, HeartbeatData,
    ProviderListData, ProviderSummary,
};

/// Shared application state for federation
#[derive(Debug, Clone)]
pub struct FederationAppState {
    pub federation_state: Arc<FederationState>,
    pub service_registry: Arc<FederatedServiceRegistry>,
    pub capability_registry: Option<Arc<CapabilityRegistry>>,
}

/// Create federation routes
pub fn federation_routes(
    federation_state: Arc<FederationState>,
    service_registry: Arc<FederatedServiceRegistry>,
) -> Router {
    let app_state = Arc::new(FederationAppState {
        federation_state,
        service_registry,
        capability_registry: None,
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

/// Create federation routes with capability registry
pub fn federation_routes_with_capabilities(
    federation_state: Arc<FederationState>,
    service_registry: Arc<FederatedServiceRegistry>,
    capability_registry: Arc<CapabilityRegistry>,
) -> Router {
    let app_state = Arc::new(FederationAppState {
        federation_state,
        service_registry,
        capability_registry: Some(capability_registry.clone()),
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
        // Capability registration (NEW)
        .route("/register", post(register_capability_provider))
        .route("/capability/heartbeat", post(capability_provider_heartbeat))
        .route("/register/:provider_id", delete(unregister_capability_provider))
        .route("/providers", get(list_capability_providers))
        .with_state(app_state)
}

/// POST /api/federation/join - Register node with federation
async fn federation_join(
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
async fn federation_status(State(state): State<Arc<FederationAppState>>) -> impl IntoResponse {
    debug!("📊 Federation status requested");
    let status = get_federation_status(&state).await;
    (StatusCode::OK, Json(status))
}

/// GET /api/federation/nodes - List all nodes
async fn federation_nodes(State(state): State<Arc<FederationAppState>>) -> impl IntoResponse {
    debug!("📋 Node list requested");

    let nodes: Vec<NodeRegistration> =
        state.federation_state.nodes.read().await.values().cloned().collect();

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
struct FederationHeartbeatResponse {
    acknowledged: bool,
    federation_status: String,
}

async fn federation_heartbeat(
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

// ========================================================================
// CAPABILITY REGISTRATION ENDPOINTS (NEW)
// ========================================================================

/// POST /api/v1/federation/register - Register a capability provider
async fn register_capability_provider(
    State(state): State<Arc<FederationAppState>>,
    Json(request): Json<CapabilityRegistrationRequest>,
) -> impl IntoResponse {
    info!(
        "🔌 Capability provider '{}' ({}) registering with {} capabilities",
        request.provider_name,
        request.provider_id,
        request.capabilities.len()
    );

    let capability_registry = match &state.capability_registry {
        Some(registry) => registry,
        None => {
            warn!("Capability registry not initialized");
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(CapabilityRegistrationResponse {
                    success: false,
                    data: None,
                    error: Some("Capability registry not available".to_string()),
                    timestamp: Utc::now(),
                }),
            );
        }
    };

    match capability_registry.register(request.clone()).await {
        Ok(registration_id) => {
            info!(
                "✅ Provider '{}' registered successfully with ID: {}",
                request.provider_id, registration_id
            );

            (
                StatusCode::OK,
                Json(CapabilityRegistrationResponse {
                    success: true,
                    data: Some(RegistrationData {
                        provider_id: request.provider_id.clone(),
                        registration_id,
                        status: "registered".to_string(),
                        heartbeat_interval_ms: capability_registry.config().interval_ms,
                        heartbeat_endpoint: "/api/v1/federation/capability/heartbeat".to_string(),
                    }),
                    error: None,
                    timestamp: Utc::now(),
                }),
            )
        }
        Err(e) => {
            warn!(
                "❌ Failed to register provider '{}': {}",
                request.provider_id, e
            );

            (
                StatusCode::BAD_REQUEST,
                Json(CapabilityRegistrationResponse {
                    success: false,
                    data: None,
                    error: Some(format!("{}", e)),
                    timestamp: Utc::now(),
                }),
            )
        }
    }
}

/// POST /api/v1/federation/capability/heartbeat - Update provider heartbeat
async fn capability_provider_heartbeat(
    State(state): State<Arc<FederationAppState>>,
    Json(request): Json<CapabilityHeartbeatRequest>,
) -> impl IntoResponse {
    debug!(
        "💓 Heartbeat from provider '{}'",
        request.provider_id
    );

    let capability_registry = match &state.capability_registry {
        Some(registry) => registry,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(crate::core::registry::types::HeartbeatResponse {
                    success: false,
                    data: None,
                    error: Some("Capability registry not available".to_string()),
                    timestamp: Utc::now(),
                }),
            );
        }
    };

    // Convert heartbeat status to ProviderHealth if provided
    let health = request.health_status.map(|status| {
        use crate::core::registry::types::{ProviderHealth, HealthStatus, ResourceUsage};
        
        ProviderHealth {
            status: match status.status.as_str() {
                "healthy" => HealthStatus::Healthy,
                "degraded" => HealthStatus::Degraded,
                "unhealthy" => HealthStatus::Unhealthy,
                _ => HealthStatus::Healthy,
            },
            available_capacity: status.available_capacity,
            resource_usage: ResourceUsage {
                cpu_percent: status.resource_usage.cpu_percent,
                memory_percent: status.resource_usage.memory_percent,
                gpu_utilization: status.resource_usage.gpu_utilization,
            },
        }
    });

    match capability_registry
        .update_heartbeat(&request.provider_id, &request.registration_id, health)
        .await
    {
        Ok(()) => {
            debug!(
                "✅ Heartbeat acknowledged from '{}'",
                request.provider_id
            );

            (
                StatusCode::OK,
                Json(crate::core::registry::types::HeartbeatResponse {
                    success: true,
                    data: Some(HeartbeatData {
                        acknowledged: true,
                        next_heartbeat_ms: capability_registry.config().interval_ms,
                    }),
                    error: None,
                    timestamp: Utc::now(),
                }),
            )
        }
        Err(e) => {
            warn!(
                "❌ Heartbeat failed from '{}': {}",
                request.provider_id, e
            );

            (
                StatusCode::BAD_REQUEST,
                Json(crate::core::registry::types::HeartbeatResponse {
                    success: false,
                    data: None,
                    error: Some(format!("{}", e)),
                    timestamp: Utc::now(),
                }),
            )
        }
    }
}

/// DELETE /api/v1/federation/register/:provider_id - Unregister a provider
async fn unregister_capability_provider(
    State(state): State<Arc<FederationAppState>>,
    Path(provider_id): Path<String>,
) -> impl IntoResponse {
    info!("🔌 Provider '{}' unregistering", provider_id);

    let capability_registry = match &state.capability_registry {
        Some(registry) => registry,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(CapabilityRegistrationResponse {
                    success: false,
                    data: None,
                    error: Some("Capability registry not available".to_string()),
                    timestamp: Utc::now(),
                }),
            );
        }
    };

    match capability_registry.unregister(&provider_id).await {
        Ok(()) => {
            info!("✅ Provider '{}' unregistered successfully", provider_id);

            (
                StatusCode::OK,
                Json(CapabilityRegistrationResponse {
                    success: true,
                    data: Some(RegistrationData {
                        provider_id: provider_id.clone(),
                        registration_id: String::new(),
                        status: "unregistered".to_string(),
                        heartbeat_interval_ms: 0,
                        heartbeat_endpoint: String::new(),
                    }),
                    error: None,
                    timestamp: Utc::now(),
                }),
            )
        }
        Err(e) => {
            warn!("❌ Failed to unregister provider '{}': {}", provider_id, e);

            (
                StatusCode::NOT_FOUND,
                Json(CapabilityRegistrationResponse {
                    success: false,
                    data: None,
                    error: Some(format!("{}", e)),
                    timestamp: Utc::now(),
                }),
            )
        }
    }
}

/// GET /api/v1/federation/providers - List all registered capability providers
async fn list_capability_providers(
    State(state): State<Arc<FederationAppState>>,
) -> impl IntoResponse {
    debug!("📋 Listing all capability providers");

    let capability_registry = match &state.capability_registry {
        Some(registry) => registry,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ProviderListResponse {
                    success: false,
                    data: None,
                    error: Some("Capability registry not available".to_string()),
                    timestamp: Utc::now(),
                }),
            );
        }
    };

    let providers = capability_registry.list_providers().await;
    let summaries: Vec<ProviderSummary> = providers.iter().map(ProviderSummary::from).collect();

    (
        StatusCode::OK,
        Json(ProviderListResponse {
            success: true,
            data: Some(ProviderListData {
                total_count: summaries.len(),
                providers: summaries,
            }),
            error: None,
            timestamp: Utc::now(),
        }),
    )
}

// ========================================================================
// HELPER FUNCTIONS
// ========================================================================

/// Helper: Get federation status
#[allow(clippy::similar_names)] // `state` and `stats` are semantically different despite similar names
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

//
// ═══════════════════════════════════════════════════════════════
// SERVICE FEDERATION ENDPOINTS
// ═══════════════════════════════════════════════════════════════
//

/// GET /api/federation/services - List all services
async fn list_services(State(state): State<Arc<FederationAppState>>) -> impl IntoResponse {
    debug!("📋 Service list requested");

    let services = state.service_registry.get_all_services().await;
    (StatusCode::OK, Json(services))
}

/// POST /api/federation/services - Register a service
async fn register_service(
    State(state): State<Arc<FederationAppState>>,
    Json(service): Json<ServiceRegistration>,
) -> impl IntoResponse {
    info!("📝 Service registration request: {} ({})", service.service_name, service.service_type);

    // Determine if this is a local or remote service
    // (For now, treat all as remote since they come via API)
    state.service_registry.register_remote(service).await;

    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "status": "registered",
            "message": "Service registered successfully"
        })),
    )
}

/// GET /`api/federation/services/:service_id` - Get specific service
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

/// GET /`api/federation/services/type/:service_type` - Find services by type
async fn find_services_by_type(
    State(state): State<Arc<FederationAppState>>,
    axum::extract::Path(service_type): axum::extract::Path<String>,
) -> impl IntoResponse {
    debug!("🔍 Services by type: {}", service_type);

    let services = state.service_registry.find_by_type(&service_type).await;

    (StatusCode::OK, Json(services))
}

/// GET /api/federation/services/stats - Get service registry statistics
#[allow(clippy::similar_names)] // `state` and `stats` are semantically different despite similar names
async fn service_stats(State(state): State<Arc<FederationAppState>>) -> impl IntoResponse {
    debug!("📊 Service stats requested");

    let stats = state.service_registry.get_stats().await;
    (StatusCode::OK, Json(stats))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use songbird_types::SongbirdError;

    use tower::ServiceExt;

    fn create_test_state() -> Arc<FederationAppState> {
        Arc::new(FederationAppState {
            federation_state: Arc::new(FederationState::new()),
            service_registry: Arc::new(FederatedServiceRegistry::new()),
            capability_registry: None,
        })
    }

    #[tokio::test]
    async fn test_federation_status_endpoint() -> Result<(), Box<dyn std::error::Error>> {
        let state = create_test_state();
        let app = Router::new().route("/status", get(federation_status)).with_state(state);

        let response = app
            .oneshot(Request::builder().uri("/status").body(Body::empty()).map_err(|e| {
                SongbirdError::configuration(format!("Failed to build request: {}", e))
            })?)
            .await
            .map_err(|e| SongbirdError::configuration(format!("Failed to send request: {}", e)))?;

        assert_eq!(response.status(), StatusCode::OK);
        Ok(())
    }

    #[tokio::test]
    #[ignore] // TODO: Re-enable once FederationState API is stable
    async fn test_federation_app_state_creation() {
        let _state = create_test_state();
        // Temporarily disabled - API under refactoring
        // Just verify it creates without panicking
    }

    #[tokio::test]
    #[ignore] // TODO: Re-enable once FederationState API is stable
    async fn test_federation_app_state_clone() {
        let state = create_test_state();
        let _cloned = state.clone();
        // Temporarily disabled - API under refactoring
        // Just verify clone works without panicking
    }

    #[tokio::test]
    async fn test_federation_nodes_endpoint_empty() -> Result<(), Box<dyn std::error::Error>> {
        let state = create_test_state();
        let app = Router::new().route("/nodes", get(federation_nodes)).with_state(state);

        let response = app
            .oneshot(Request::builder().uri("/nodes").body(Body::empty()).map_err(|e| {
                SongbirdError::configuration(format!("Failed to build request: {}", e))
            })?)
            .await
            .map_err(|e| SongbirdError::configuration(format!("Failed to send request: {}", e)))?;

        assert_eq!(response.status(), StatusCode::OK);
        Ok(())
    }

    #[tokio::test]
    async fn test_heartbeat_request_deserialization() -> Result<(), Box<dyn std::error::Error>> {
        let json = r#"{
            "node_id": "test-node",
            "timestamp": "2025-11-03T20:00:00Z",
            "status": "active"
        }"#;

        let request: Result<HeartbeatRequest, _> = serde_json::from_str(json);
        assert!(request.is_ok());

        let req = request
            .map_err(|e| SongbirdError::configuration(format!("Failed to parse request: {}", e)))?;
        assert_eq!(req.node_id, "test-node");
        assert_eq!(req.status, Some("active".to_string()));
        Ok(())
    }

    #[tokio::test]
    async fn test_heartbeat_request_optional_fields() -> Result<(), Box<dyn std::error::Error>> {
        let json = r#"{
            "node_id": "test-node",
            "timestamp": "2025-11-03T20:00:00Z"
        }"#;

        let request: Result<HeartbeatRequest, _> = serde_json::from_str(json);
        assert!(request.is_ok());

        let req = request
            .map_err(|e| SongbirdError::configuration(format!("Failed to parse request: {}", e)))?;
        assert_eq!(req.status, None);
        assert_eq!(req.metrics, None);
        Ok(())
    }

    #[tokio::test]
    async fn test_heartbeat_response_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let response = FederationHeartbeatResponse {
            acknowledged: true,
            federation_status: "active".to_string(),
        };

        let json = serde_json::to_string(&response);
        assert!(json.is_ok());
        assert!(json
            .map_err(|e| {
                SongbirdError::configuration(format!("Failed to serialize response: {}", e))
            })?
            .contains("acknowledged"));
        Ok(())
    }

    #[tokio::test]
    async fn test_get_federation_status_empty() {
        let state = create_test_state();
        let status = get_federation_status(&state).await;

        assert_eq!(status.active_nodes, 0);
        assert_eq!(status.nodes.len(), 0);
        assert_eq!(status.total_cpu_cores, 0);
        assert_eq!(status.total_memory_gb, 0);
        assert!(status.uptime_seconds >= 0);
    }

    #[tokio::test]
    async fn test_get_federation_status_with_nodes() {
        let state = create_test_state();

        // Add a node
        let registration = NodeRegistration {
            node_id: "test-node".to_string(),
            node_name: "Test Node".to_string(),
            node_address: "192.168.1.100:8080".to_string(),
            cpu_cores: 8,
            memory_gb: 16,
            gpu_model: None,
            storage_gb: Some(500),
            capabilities: vec!["compute".to_string()],
            status: NodeStatus::Active,
            joined_at: Utc::now(),
            last_heartbeat: Utc::now(),
        };

        state.federation_state.register_node(registration).await;

        let status = get_federation_status(&state).await;

        assert_eq!(status.active_nodes, 1);
        assert_eq!(status.nodes.len(), 1);
        assert_eq!(status.total_cpu_cores, 8);
        assert_eq!(status.total_memory_gb, 16);
        assert_eq!(status.total_storage_gb, 500);
    }

    #[tokio::test]
    async fn test_federation_routes_construction() {
        let federation_state = Arc::new(FederationState::new());
        let service_registry = Arc::new(FederatedServiceRegistry::new());

        let router = federation_routes(federation_state, service_registry);

        // Router should be created without panicking
        assert!(true);
    }

    #[test]
    fn test_heartbeat_request_debug() {
        let request = HeartbeatRequest {
            node_id: "test".to_string(),
            timestamp: "2025-11-03T20:00:00Z".to_string(),
            status: Some("active".to_string()),
            metrics: None,
        };

        let debug_str = format!("{:?}", request);
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_heartbeat_response_debug() {
        let response = FederationHeartbeatResponse {
            acknowledged: true,
            federation_status: "active".to_string(),
        };

        let debug_str = format!("{:?}", response);
        assert!(debug_str.contains("true"));
    }

    #[test]
    fn test_federation_app_state_debug() {
        let state = Arc::new(FederationAppState {
            federation_state: Arc::new(FederationState::new()),
            service_registry: Arc::new(FederatedServiceRegistry::new()),
            capability_registry: None,
        });

        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("FederationAppState"));
    }

    #[tokio::test]
    async fn test_heartbeat_with_metrics() -> Result<(), Box<dyn std::error::Error>> {
        let json = r#"{
            "node_id": "test-node",
            "timestamp": "2025-11-03T20:00:00Z",
            "status": "active",
            "metrics": {
                "cpu_usage": 45.5,
                "memory_usage": 60.2
            }
        }"#;

        let request: Result<HeartbeatRequest, _> = serde_json::from_str(json);
        assert!(request.is_ok());

        let req = request
            .map_err(|e| SongbirdError::configuration(format!("Failed to parse request: {}", e)))?;
        assert!(req.metrics.is_some());
        Ok(())
    }

    #[tokio::test]
    async fn test_multiple_nodes_listing() {
        let state = create_test_state();

        // Add multiple nodes
        for i in 0..3 {
            let registration = NodeRegistration {
                node_id: format!("node-{}", i),
                node_name: format!("Node {}", i),
                node_address: format!("192.168.1.{}:8080", 100 + i),
                cpu_cores: 4,
                memory_gb: 8,
                gpu_model: None,
                storage_gb: None,
                capabilities: vec![],
                status: NodeStatus::Active,
                joined_at: Utc::now(),
                last_heartbeat: Utc::now(),
            };
            state.federation_state.register_node(registration).await;
        }

        let status = get_federation_status(&state).await;
        assert_eq!(status.nodes.len(), 3);
        assert_eq!(status.active_nodes, 3);
    }

    #[tokio::test]
    async fn test_federation_status_uptime() {
        let state = create_test_state();

        // Small delay to ensure uptime > 0
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let status = get_federation_status(&state).await;
        assert!(status.uptime_seconds >= 0);
    }

    #[tokio::test]
    async fn test_service_registry_operations() {
        let state = create_test_state();

        // Initially empty
        let services = state.service_registry.get_all_services().await;
        assert_eq!(services.len(), 0);

        // Get stats
        let stats = state.service_registry.get_stats().await;
        assert_eq!(stats.total_services, 0);
    }
}
