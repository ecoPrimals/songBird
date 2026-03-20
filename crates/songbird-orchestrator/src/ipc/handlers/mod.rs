// SPDX-License-Identifier: AGPL-3.0-only
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
use songbird_http_client::BearDogClient;

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
}

impl IpcHandlers {
    /// Create new API handlers with individual components
    ///
    /// v3.19.2: Modern Rust - pass only what's needed, not whole orchestrator
    /// v3.20.0: Added `service_registry` parameter
    /// v3.21.0: Added `graph_validator`
    /// v5.27.0: Added `beardog_client` for HTTP handler
    pub fn new(
        service_registry: Arc<ServiceRegistry>,
        discovery_listener: Option<Arc<AnonymousDiscoveryListener>>,
        connection_manager: Arc<ConnectionManager>,
        beardog_client: Arc<BearDogClient>,
    ) -> Self {
        let availability_checker = Arc::new(AvailabilityChecker::new(service_registry.clone()));
        let coordination_validator = Arc::new(CoordinationValidator::new(service_registry.clone()));
        let http_handler = Arc::new(http::HttpHandler::new(beardog_client));

        Self {
            service_registry,
            discovery_listener,
            connection_manager,
            graph_validator: Arc::new(GraphValidator::new()),
            availability_checker,
            coordination_validator,
            http_handler,
        }
    }

    // ========================================================================
    // Service Registry APIs (v3.20.0) - Delegated to service_registry module
    // ========================================================================

    /// Handle `register_service` RPC call (Pure Rust, v3.34.0)
    pub async fn register_service(
        &self,
        params: serde_json::Value,
    ) -> Result<super::types::RegisterServiceResponse, crate::ipc::pure_rust_server::JsonRpcError>
    {
        service_registry::register_service(self, params).await
    }

    /// Handle `discover_by_capability` RPC call (Pure Rust, v3.34.0)
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
    pub async fn get_service_health(
        &self,
        params: serde_json::Value,
    ) -> Result<super::types::GetServiceHealthResponse, crate::ipc::pure_rust_server::JsonRpcError>
    {
        service_registry::get_service_health(self, params).await
    }

    /// Handle `health_check` RPC call (Pure Rust, v3.34.0)
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
    pub async fn discover_by_family(
        &self,
        params: serde_json::Value,
    ) -> Result<super::types::DiscoverByFamilyResponse, crate::ipc::pure_rust_server::JsonRpcError>
    {
        p2p_discovery::discover_by_family(self, params).await
    }

    /// Handle `create_genetic_tunnel` RPC call (Pure Rust, v3.34.0)
    pub async fn create_genetic_tunnel(
        &self,
        params: serde_json::Value,
    ) -> Result<super::types::CreateGeneticTunnelResponse, crate::ipc::pure_rust_server::JsonRpcError>
    {
        p2p_discovery::create_genetic_tunnel(self, params).await
    }

    /// Handle `announce_capabilities` RPC call (Pure Rust, v3.34.0)
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
    pub async fn validate_graph(
        &self,
        params: serde_json::Value,
    ) -> Result<crate::graph::ValidationResult, crate::ipc::pure_rust_server::JsonRpcError> {
        graph_intelligence::validate_graph(self, params).await
    }

    /// Handle `graph.check_availability` RPC call (Pure Rust, v3.34.0)
    pub async fn check_availability(
        &self,
        params: serde_json::Value,
    ) -> Result<crate::graph::AvailabilityReport, crate::ipc::pure_rust_server::JsonRpcError> {
        graph_intelligence::check_availability(self, params).await
    }

    /// Handle `graph.suggest_alternatives` RPC call (Pure Rust, v3.34.0)
    pub async fn suggest_alternatives(
        &self,
        params: serde_json::Value,
    ) -> Result<crate::graph::AlternativeSuggestions, crate::ipc::pure_rust_server::JsonRpcError>
    {
        graph_intelligence::suggest_alternatives(self, params).await
    }

    /// Handle `coordination.validate_pattern` RPC call (Pure Rust, v3.34.0)
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
    pub async fn register_service_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        service_registry::register_service_json(self, params).await
    }

    /// Service Registry: `discover_by_capability` (pure JSON adapter)
    pub async fn discover_by_capability_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        service_registry::discover_by_capability_json(self, params).await
    }

    /// Service Registry: `get_service_health` (pure JSON adapter)
    pub async fn get_service_health_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        service_registry::get_service_health_json(self, params).await
    }

    /// Service Registry: `health_check` (pure JSON adapter)
    pub async fn health_check_json(
        &self,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        service_registry::health_check_json(self).await
    }

    /// P2P Discovery: `discover_by_family` (pure JSON adapter)
    pub async fn discover_by_family_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        p2p_discovery::discover_by_family_json(self, params).await
    }

    /// P2P Discovery: `create_genetic_tunnel` (pure JSON adapter)
    pub async fn create_genetic_tunnel_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        p2p_discovery::create_genetic_tunnel_json(self, params).await
    }

    /// P2P Discovery: `announce_capabilities` (pure JSON adapter)
    pub async fn announce_capabilities_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        p2p_discovery::announce_capabilities_json(self, params).await
    }

    /// Graph Intelligence: `validate_graph` (pure JSON adapter)
    pub async fn validate_graph_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        graph_intelligence::validate_graph_json(self, params).await
    }

    /// Graph Intelligence: `check_availability` (pure JSON adapter)
    pub async fn check_availability_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        graph_intelligence::check_availability_json(self, params).await
    }

    /// Graph Intelligence: `suggest_alternatives` (pure JSON adapter)
    pub async fn suggest_alternatives_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        graph_intelligence::suggest_alternatives_json(self, params).await
    }

    /// Graph Intelligence: `validate_coordination_pattern` (pure JSON adapter)
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
    /// Zero C dependencies - delegates crypto to `BearDog` via RPC.
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
    pub async fn http_request(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        self.http_handler.handle_request(params).await
    }

    /// Handle `http.get` convenience method (v5.27.0)
    pub async fn http_get(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        self.http_handler.handle_get(params).await
    }

    /// Handle `http.post` convenience method (v5.27.0)
    pub async fn http_post(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        self.http_handler.handle_post(params).await
    }

    /// Handle `http.put` convenience method (v5.27.0)
    pub async fn http_put(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        self.http_handler.handle_put(params).await
    }

    /// Handle `http.delete` convenience method (v5.27.0)
    pub async fn http_delete(
        &self,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        self.http_handler.handle_delete(params).await
    }
}
