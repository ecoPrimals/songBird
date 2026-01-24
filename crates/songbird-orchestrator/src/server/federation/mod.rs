//! Federation API Module
//!
//! Domain-driven architecture for federation coordination and capability registration.
//!
//! ## Structure
//!
//! - `types`: Shared types and state
//! - `node_endpoints`: Node management operations (join, status, heartbeat, listing)
//! - `capability_endpoints`: Capability provider operations (register, heartbeat, discovery)
//! - `service_endpoints`: Service registry operations (register, list, stats)
//!
//! ## Evolution
//!
//! **Refactored**: January 21, 2026 - Split from 971-line monolithic file into 4 cohesive modules  
//! **Strategy**: Domain-driven (not arbitrary line-count splitting)  
//! **Result**: Clear separation of concerns, easier testing, better maintainability

use axum::{
    routing::{delete, get, post},
    Router,
};
use songbird_network_federation::service_registry::FederatedServiceRegistry;
use songbird_network_federation::state::FederationState;
use std::sync::Arc;

// Module declarations
pub mod capability_endpoints;
pub mod node_endpoints;
pub mod service_endpoints;
pub mod types;

// Re-exports for backward compatibility
pub use types::FederationAppState;

use crate::core::registry::CapabilityRegistry;
use crate::trust::TrustEscalationManager;

/// Create federation routes
pub fn federation_routes(
    federation_state: Arc<FederationState>,
    service_registry: Arc<FederatedServiceRegistry>,
) -> Router {
    let app_state = Arc::new(FederationAppState {
        federation_state,
        service_registry,
        capability_registry: None,
        trust_manager: None,
    });

    Router::new()
        // Node management
        .route("/join", post(node_endpoints::federation_join))
        .route("/status", get(node_endpoints::federation_status))
        .route("/nodes", get(node_endpoints::federation_nodes))
        .route("/nodes/:node_id", get(node_endpoints::get_node_details))
        .route("/heartbeat", post(node_endpoints::federation_heartbeat))
        // Service management
        .route("/services", get(service_endpoints::list_services))
        .route("/services", post(service_endpoints::register_service))
        .route("/services/:service_id", get(service_endpoints::get_service))
        .route("/services/type/:service_type", get(service_endpoints::find_services_by_type))
        .route("/services/stats", get(service_endpoints::service_stats))
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
        capability_registry: Some(capability_registry),
        trust_manager: None,
    });

    Router::new()
        // Node management
        .route("/join", post(node_endpoints::federation_join))
        .route("/status", get(node_endpoints::federation_status))
        .route("/nodes", get(node_endpoints::federation_nodes))
        .route("/nodes/:node_id", get(node_endpoints::get_node_details))
        .route("/heartbeat", post(node_endpoints::federation_heartbeat))
        // Service management
        .route("/services", get(service_endpoints::list_services))
        .route("/services", post(service_endpoints::register_service))
        .route("/services/:service_id", get(service_endpoints::get_service))
        .route("/services/type/:service_type", get(service_endpoints::find_services_by_type))
        .route("/services/stats", get(service_endpoints::service_stats))
        // Capability registration (NEW)
        .route("/register", post(capability_endpoints::register_capability_provider))
        .route("/capability/heartbeat", post(capability_endpoints::capability_provider_heartbeat))
        .route(
            "/register/:provider_id",
            delete(capability_endpoints::unregister_capability_provider),
        )
        .route("/providers", get(capability_endpoints::list_capability_providers))
        .with_state(app_state)
}

/// Create federation routes with trust manager (NEW - secure by default)
pub fn federation_routes_with_trust(
    federation_state: Arc<FederationState>,
    service_registry: Arc<FederatedServiceRegistry>,
    capability_registry: Option<Arc<CapabilityRegistry>>,
    trust_manager: Arc<TrustEscalationManager>,
) -> Router {
    let app_state = Arc::new(FederationAppState {
        federation_state,
        service_registry,
        capability_registry,
        trust_manager: Some(trust_manager),
    });

    Router::new()
        // Node management with graduated disclosure
        .route("/join", post(node_endpoints::federation_join))
        .route("/status", get(node_endpoints::federation_status))
        .route("/nodes", get(node_endpoints::federation_nodes_graduated))
        .route("/nodes/:node_id", get(node_endpoints::get_node_details))
        .route("/heartbeat", post(node_endpoints::federation_heartbeat))
        // Service management
        .route("/services", get(service_endpoints::list_services))
        .route("/services", post(service_endpoints::register_service))
        .route("/services/:service_id", get(service_endpoints::get_service))
        .route("/services/type/:service_type", get(service_endpoints::find_services_by_type))
        .route("/services/stats", get(service_endpoints::service_stats))
        // Capability registration (if available)
        .route("/register", post(capability_endpoints::register_capability_provider))
        .route("/capability/heartbeat", post(capability_endpoints::capability_provider_heartbeat))
        .route(
            "/register/:provider_id",
            delete(capability_endpoints::unregister_capability_provider),
        )
        .route("/providers", get(capability_endpoints::list_capability_providers))
        .with_state(app_state)
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
            federation_state: Arc::new(FederationState::new("default".to_string())),
            service_registry: Arc::new(FederatedServiceRegistry::new()),
            capability_registry: None,
            trust_manager: Some(Arc::new(TrustEscalationManager::with_defaults())),
        })
    }

    #[tokio::test]
    async fn test_federation_status_endpoint() -> Result<(), Box<dyn std::error::Error>> {
        let state = create_test_state();
        let app = Router::new()
            .route("/status", get(node_endpoints::federation_status))
            .with_state(state);

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
    async fn test_federation_nodes_endpoint_empty() -> Result<(), Box<dyn std::error::Error>> {
        let state = create_test_state();
        let app =
            Router::new().route("/nodes", get(node_endpoints::federation_nodes)).with_state(state);

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
    async fn test_federation_routes_construction() {
        let federation_state = Arc::new(FederationState::new("default".to_string()));
        let service_registry = Arc::new(FederatedServiceRegistry::new());

        let _router = federation_routes(federation_state, service_registry);

        // Router should be created without panicking
        assert!(true);
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
