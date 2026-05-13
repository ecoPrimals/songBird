// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! JSON-RPC API handlers for Unix socket IPC (v3.22.1)
//!
//! Refactored into domain-focused modules for maintainability.
//! v3.19.1: Modern idiomatic Rust handlers for biomeOS integration
//! v3.20.0: Service registry handlers for primal discovery
//! v3.22.1: Smart refactoring into 4 focused modules (Jan 12, 2026)

use std::sync::Arc;

use super::registry::ServiceRegistry;
use crate::app::connection_manager::ConnectionManager;
use crate::graph::{AvailabilityChecker, CoordinationValidator, GraphValidator};
use songbird_discovery::anonymous::AnonymousDiscoveryListener;
use songbird_http_client::SecurityRpcClient;
use songbird_universal_ipc::handlers::MeshHandler;

// Domain-focused handler modules
pub mod graph_intelligence;
pub mod http;
pub mod p2p_discovery;
pub mod service_registry;

/// API handlers for Unix socket JSON-RPC methods
///
/// v3.19.2: Refactored to take individual components instead of whole orchestrator
/// v3.20.0: Added service registry for capability-based discovery
/// v3.21.0: Added graph validator for Collaborative Intelligence
/// v3.22.1: Refactored into domain modules for <1000 lines per file
/// v5.27.0: Added HTTP handler for Tower Atomic (Jan 25, 2026)
pub struct IpcHandlers {
    /// Service registry (v3.20.0)
    pub(crate) service_registry: Arc<ServiceRegistry>,

    /// Discovery listener for getting discovered peers (v3.19.1)
    pub(crate) discovery_listener: Option<Arc<AnonymousDiscoveryListener>>,

    /// Connection manager for establishing connections (v3.19.1)
    pub(crate) connection_manager: Arc<ConnectionManager>,

    /// Graph validator for Collaborative Intelligence (v3.21.0)
    pub(crate) graph_validator: Arc<GraphValidator>,

    /// Availability checker for Collaborative Intelligence (v3.21.0)
    pub(crate) availability_checker: Arc<AvailabilityChecker>,

    /// Coordination validator for Collaborative Intelligence (v3.21.0 Week 3)
    pub(crate) coordination_validator: Arc<CoordinationValidator>,

    /// HTTP handler for Tower Atomic HTTPS requests (v5.27.0)
    pub(crate) http_handler: Arc<http::HttpHandler>,

    /// Mesh handler for beacon mesh networking (GAP-16: Tower atomic validation)
    pub(crate) mesh_handler: Arc<MeshHandler>,
}

impl IpcHandlers {
    /// Create new API handlers with individual components
    ///
    /// v3.19.2: Modern Rust - pass only what's needed, not whole orchestrator
    /// v3.20.0: Added `service_registry` parameter
    /// v3.21.0: Added `graph_validator`
    /// v5.27.0: Added `security_client` for HTTP handler
    pub fn new(
        service_registry: Arc<ServiceRegistry>,
        discovery_listener: Option<Arc<AnonymousDiscoveryListener>>,
        connection_manager: Arc<ConnectionManager>,
        security_client: Arc<SecurityRpcClient>,
    ) -> Self {
        let availability_checker = Arc::new(AvailabilityChecker::new(service_registry.clone()));
        let coordination_validator = Arc::new(CoordinationValidator::new(service_registry.clone()));
        let http_handler = Arc::new(http::HttpHandler::new(security_client));
        let mesh_handler = Arc::new(MeshHandler::new());

        Self {
            service_registry,
            discovery_listener,
            connection_manager,
            graph_validator: Arc::new(GraphValidator::new()),
            availability_checker,
            coordination_validator,
            http_handler,
            mesh_handler,
        }
    }

    // ========================================================================
    // Service Registry APIs (v3.20.0) - Delegated to service_registry module
    // ========================================================================

    /// Handle `register_service` RPC call (Pure Rust, v3.34.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn register_service(
        &self,
        params: serde_json::Value,
    ) -> Result<super::types::RegisterServiceResponse, crate::ipc::pure_rust_server::JsonRpcError>
    {
        service_registry::register_service(self, params).await
    }

    /// Handle `discover_by_capability` RPC call (Pure Rust, v3.34.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn discover_by_capability(
        &self,
        params: serde_json::Value,
    ) -> Result<
        super::types::DiscoverByCapabilityResponse,
        crate::ipc::pure_rust_server::JsonRpcError,
    > {
        service_registry::discover_by_capability(self, params).await
    }

    /// Handle `get_service_health` RPC call (Pure Rust, v3.34.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn get_service_health(
        &self,
        params: serde_json::Value,
    ) -> Result<super::types::GetServiceHealthResponse, crate::ipc::pure_rust_server::JsonRpcError>
    {
        service_registry::get_service_health(self, params).await
    }

    /// Handle `health_check` RPC call (Pure Rust, v3.34.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn health_check(
        &self,
        params: serde_json::Value,
    ) -> Result<super::types::HealthCheckResponse, crate::ipc::pure_rust_server::JsonRpcError> {
        service_registry::health_check(self, params).await
    }

    // ========================================================================
    // P2P Discovery APIs (v3.19.1) - Delegated to p2p_discovery module
    // ========================================================================

    /// Handle `discover_by_family` RPC call (Pure Rust, v3.34.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn discover_by_family(
        &self,
        params: serde_json::Value,
    ) -> Result<super::types::DiscoverByFamilyResponse, crate::ipc::pure_rust_server::JsonRpcError>
    {
        p2p_discovery::discover_by_family(self, params).await
    }

    /// Handle `create_genetic_tunnel` RPC call (Pure Rust, v3.34.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn create_genetic_tunnel(
        &self,
        params: serde_json::Value,
    ) -> Result<super::types::CreateGeneticTunnelResponse, crate::ipc::pure_rust_server::JsonRpcError>
    {
        p2p_discovery::create_genetic_tunnel(self, params).await
    }

    /// Handle `announce_capabilities` RPC call (Pure Rust, v3.34.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn announce_capabilities(
        &self,
        params: serde_json::Value,
    ) -> Result<
        super::types::AnnounceCapabilitiesResponse,
        crate::ipc::pure_rust_server::JsonRpcError,
    > {
        p2p_discovery::announce_capabilities(self, params).await
    }

    // ========================================================================
    // Graph Intelligence APIs (v3.21.0) - Delegated to graph_intelligence module
    // ========================================================================

    /// Handle `graph.validate` RPC call (Pure Rust, v3.34.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn validate_graph(
        &self,
        params: serde_json::Value,
    ) -> Result<crate::graph::ValidationResult, crate::ipc::pure_rust_server::JsonRpcError> {
        graph_intelligence::validate_graph(self, params).await
    }

    /// Handle `graph.check_availability` RPC call (Pure Rust, v3.34.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn check_availability(
        &self,
        params: serde_json::Value,
    ) -> Result<crate::graph::AvailabilityReport, crate::ipc::pure_rust_server::JsonRpcError> {
        graph_intelligence::check_availability(self, params).await
    }

    /// Handle `graph.suggest_alternatives` RPC call (Pure Rust, v3.34.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn suggest_alternatives(
        &self,
        params: serde_json::Value,
    ) -> Result<crate::graph::AlternativeSuggestions, crate::ipc::pure_rust_server::JsonRpcError>
    {
        graph_intelligence::suggest_alternatives(self, params).await
    }

    /// Handle `coordination.validate_pattern` RPC call (Pure Rust, v3.34.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn validate_coordination_pattern(
        &self,
        params: serde_json::Value,
    ) -> Result<
        crate::graph::CoordinationValidationResult,
        crate::ipc::pure_rust_server::JsonRpcError,
    > {
        graph_intelligence::validate_coordination_pattern(self, params).await
    }

    // ========================================================================
    // Pure JSON Adapters (v3.22.0) - Delegated to domain modules
    // ========================================================================

    /// Service Registry: `register_service` (pure JSON adapter)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn register_service_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        service_registry::register_service_json(self, params).await
    }

    /// `capability.resolve` — single-step routing by capability (pure JSON adapter)
    /// # Errors
    ///
    /// Returns an error if no provider is registered for the capability.
    pub async fn capability_resolve_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        service_registry::capability_resolve_json(self, params).await
    }

    /// Service Registry: `discover_by_capability` (pure JSON adapter)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn discover_by_capability_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        service_registry::discover_by_capability_json(self, params).await
    }

    /// Service Registry: `get_service_health` (pure JSON adapter)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn get_service_health_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        service_registry::get_service_health_json(self, params).await
    }

    /// Service Registry: `health_check` (pure JSON adapter)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn health_check_json(
        &self,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        service_registry::health_check_json(self).await
    }

    /// P2P Discovery: `discover_by_family` (pure JSON adapter)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn discover_by_family_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        p2p_discovery::discover_by_family_json(self, params).await
    }

    /// P2P Discovery: `create_genetic_tunnel` (pure JSON adapter)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn create_genetic_tunnel_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        p2p_discovery::create_genetic_tunnel_json(self, params).await
    }

    /// P2P Discovery: `announce_capabilities` (pure JSON adapter)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn announce_capabilities_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        p2p_discovery::announce_capabilities_json(self, params).await
    }

    /// Graph Intelligence: `validate_graph` (pure JSON adapter)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn validate_graph_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        graph_intelligence::validate_graph_json(self, params).await
    }

    /// Graph Intelligence: `check_availability` (pure JSON adapter)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn check_availability_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        graph_intelligence::check_availability_json(self, params).await
    }

    /// Graph Intelligence: `suggest_alternatives` (pure JSON adapter)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn suggest_alternatives_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        graph_intelligence::suggest_alternatives_json(self, params).await
    }

    /// Graph Intelligence: `validate_coordination_pattern` (pure JSON adapter)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn validate_coordination_pattern_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        graph_intelligence::validate_coordination_pattern_json(self, params).await
    }

    // ========================================================================
    // HTTP/HTTPS APIs (v5.27.0) - Tower Atomic Pure Rust TLS 1.3
    // ========================================================================

    /// Handle `http.request` RPC call (Pure Rust TLS 1.3, v5.27.0)
    ///
    /// Makes HTTP/HTTPS requests using Pure Rust Tower Atomic pattern.
    /// Zero C dependencies - delegates crypto to `security provider` via RPC.
    ///
    /// # Method
    ///
    /// `http.request`
    ///
    /// # Parameters
    ///
    /// - `method`: HTTP method (GET, POST, PUT, DELETE, HEAD, PATCH)
    /// - `url`: Target URL (http:// or https://)
    /// - `headers`: Optional headers object
    /// - `body`: Optional body (base64-encoded)
    ///
    /// # Returns
    ///
    /// Response with:
    /// - `status`: HTTP status code
    /// - `headers`: Response headers
    /// - `body`: Response body (base64-encoded)
    ///
    /// # Example
    ///
    /// ```json
    /// {
    ///     "jsonrpc": "2.0",
    ///     "method": "http.request",
    ///     "params": {
    ///         "method": "GET",
    ///         "url": "https://api.github.com/",
    ///         "headers": {},
    ///         "body": null
    ///     },
    ///     "id": 1
    /// }
    /// ```
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn http_request(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        self.http_handler.handle_request(params).await
    }

    /// Handle `http.get` convenience method (v5.27.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn http_get(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        self.http_handler.handle_get(params).await
    }

    /// Handle `http.post` convenience method (v5.27.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn http_post(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        self.http_handler.handle_post(params).await
    }

    /// Handle `http.put` convenience method (v5.27.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn http_put(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        self.http_handler.handle_put(params).await
    }

    /// Handle `http.delete` convenience method (v5.27.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn http_delete(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        self.http_handler.handle_delete(params).await
    }

    // ========================================================================
    // Mesh Networking APIs (GAP-16: Tower Atomic Validation)
    // ========================================================================

    /// Dispatch a `mesh.*` JSON-RPC method to the `MeshHandler`.
    ///
    /// Converts the `Result<Value, String>` return type from `MeshHandler` to
    /// the `Result<Value, JsonRpcError>` expected by the UDS routing layer.
    pub async fn mesh_dispatch(
        &self,
        method: songbird_types::json_rpc_method::MeshMethod,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        use songbird_types::json_rpc_method::MeshMethod;

        let params = params.unwrap_or(serde_json::Value::Null);
        let result: Result<serde_json::Value, String> = match method {
            MeshMethod::Init => self.mesh_handler.handle_init(params).await,
            MeshMethod::Status => self.mesh_handler.handle_status(params).await,
            MeshMethod::FindPath => self.mesh_handler.handle_find_path(params).await,
            MeshMethod::Announce => self.mesh_handler.handle_announce(params).await,
            MeshMethod::Peers => self.mesh_handler.handle_peers(params).await,
            MeshMethod::Topology => self.mesh_handler.handle_topology(params).await,
            MeshMethod::HealthCheck => self.mesh_handler.handle_health_check(params).await,
            MeshMethod::AutoDiscover => self.mesh_handler.handle_auto_discover(params).await,
        };
        result.map_err(|e| crate::ipc::pure_rust_server::JsonRpcError::internal_error(e))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod ipc_handlers_tests {
    use super::IpcHandlers;
    use crate::app::connection_manager::ConnectionManager;
    use crate::ipc::registry::ServiceRegistry;
    use songbird_http_client::SecurityRpcClient;
    use songbird_types::json_rpc_method::MeshMethod;
    use std::sync::Arc;

    fn test_handlers() -> IpcHandlers {
        IpcHandlers::new(
            Arc::new(ServiceRegistry::new()),
            None,
            Arc::new(ConnectionManager::new()),
            Arc::new(SecurityRpcClient::new("/tmp/songbird-orchestrator-ipc-handlers-test.sock")),
        )
    }

    #[test]
    fn new_preserves_registry_and_connection_manager_arcs() {
        let registry = Arc::new(ServiceRegistry::new());
        let connections = Arc::new(ConnectionManager::new());
        let security =
            Arc::new(SecurityRpcClient::new("/tmp/songbird-orchestrator-ipc-handlers-test.sock"));
        let registry_ptr = Arc::as_ptr(&registry);
        let connections_ptr = Arc::as_ptr(&connections);

        let handlers = IpcHandlers::new(registry, None, connections, security);

        assert!(handlers.discovery_listener.is_none());
        assert_eq!(Arc::as_ptr(&handlers.service_registry), registry_ptr);
        assert_eq!(Arc::as_ptr(&handlers.connection_manager), connections_ptr);
    }

    #[tokio::test]
    async fn mesh_status_requires_init() {
        let handlers = test_handlers();
        let result = handlers.mesh_dispatch(MeshMethod::Status, None).await;
        assert!(result.is_err(), "mesh.status should fail before mesh.init");
    }

    #[tokio::test]
    async fn mesh_status_works_after_init() {
        let handlers = test_handlers();
        let init_params = serde_json::json!({
            "node_id": "tower-status-test",
            "bootstrap_onions": []
        });
        handlers.mesh_dispatch(MeshMethod::Init, Some(init_params)).await.unwrap();
        let result = handlers.mesh_dispatch(MeshMethod::Status, None).await;
        assert!(result.is_ok());
        let val = result.unwrap();
        assert!(val.is_object(), "mesh.status should return a JSON object");
    }

    #[tokio::test]
    async fn mesh_init_succeeds_with_valid_params() {
        let handlers = test_handlers();
        let params = serde_json::json!({
            "node_id": "tower-test-node-12345678",
            "bootstrap_onions": []
        });
        let result = handlers.mesh_dispatch(MeshMethod::Init, Some(params)).await;
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["initialized"], true);
    }

    #[tokio::test]
    async fn mesh_init_fails_without_node_id() {
        let handlers = test_handlers();
        let result = handlers.mesh_dispatch(MeshMethod::Init, Some(serde_json::json!({}))).await;
        assert!(result.is_err());
    }
}
