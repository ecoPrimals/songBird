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

// Domain-focused handler modules
pub mod graph_intelligence;
pub mod p2p_discovery;
pub mod service_registry;

/// API handlers for Unix socket JSON-RPC methods
///
/// v3.19.2: Refactored to take individual components instead of whole orchestrator
/// v3.20.0: Added service registry for capability-based discovery
/// v3.21.0: Added graph validator for Collaborative Intelligence
/// v3.22.1: Refactored into domain modules for <1000 lines per file
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
}

impl IpcHandlers {
    /// Create new API handlers with individual components
    ///
    /// v3.19.2: Modern Rust - pass only what's needed, not whole orchestrator
    /// v3.20.0: Added service_registry parameter
    /// v3.21.0: Added graph_validator
    pub fn new(
        service_registry: Arc<ServiceRegistry>,
        discovery_listener: Option<Arc<AnonymousDiscoveryListener>>,
        connection_manager: Arc<ConnectionManager>,
    ) -> Self {
        let availability_checker = Arc::new(AvailabilityChecker::new(service_registry.clone()));
        let coordination_validator = Arc::new(CoordinationValidator::new(service_registry.clone()));
        Self {
            service_registry,
            discovery_listener,
            connection_manager,
            graph_validator: Arc::new(GraphValidator::new()),
            availability_checker,
            coordination_validator,
        }
    }

    // ========================================================================
    // Service Registry APIs (v3.20.0) - Delegated to service_registry module
    // ========================================================================

    /// Handle `register_service` RPC call (jsonrpsee)
    pub async fn register_service(
        &self,
        params: jsonrpsee::types::Params<'_>,
    ) -> Result<super::types::RegisterServiceResponse, jsonrpsee::types::ErrorObject<'static>> {
        service_registry::register_service(self, params).await
    }

    /// Handle `discover_by_capability` RPC call (jsonrpsee)
    pub async fn discover_by_capability(
        &self,
        params: jsonrpsee::types::Params<'_>,
    ) -> Result<super::types::DiscoverByCapabilityResponse, jsonrpsee::types::ErrorObject<'static>>
    {
        service_registry::discover_by_capability(self, params).await
    }

    /// Handle `get_service_health` RPC call (jsonrpsee)
    pub async fn get_service_health(
        &self,
        params: jsonrpsee::types::Params<'_>,
    ) -> Result<super::types::GetServiceHealthResponse, jsonrpsee::types::ErrorObject<'static>>
    {
        service_registry::get_service_health(self, params).await
    }

    /// Handle `health_check` RPC call (jsonrpsee)
    pub async fn health_check(
        &self,
        params: jsonrpsee::types::Params<'_>,
    ) -> Result<super::types::HealthCheckResponse, jsonrpsee::types::ErrorObject<'static>> {
        service_registry::health_check(self, params).await
    }

    // ========================================================================
    // P2P Discovery APIs (v3.19.1) - Delegated to p2p_discovery module
    // ========================================================================

    /// Handle `discover_by_family` RPC call (jsonrpsee)
    pub async fn discover_by_family(
        &self,
        params: jsonrpsee::types::Params<'_>,
    ) -> Result<super::types::DiscoverByFamilyResponse, jsonrpsee::types::ErrorObject<'static>>
    {
        p2p_discovery::discover_by_family(self, params).await
    }

    /// Handle `create_genetic_tunnel` RPC call (jsonrpsee)
    pub async fn create_genetic_tunnel(
        &self,
        params: jsonrpsee::types::Params<'_>,
    ) -> Result<super::types::CreateGeneticTunnelResponse, jsonrpsee::types::ErrorObject<'static>>
    {
        p2p_discovery::create_genetic_tunnel(self, params).await
    }

    /// Handle `announce_capabilities` RPC call (jsonrpsee)
    pub async fn announce_capabilities(
        &self,
        params: jsonrpsee::types::Params<'_>,
    ) -> Result<super::types::AnnounceCapabilitiesResponse, jsonrpsee::types::ErrorObject<'static>>
    {
        p2p_discovery::announce_capabilities(self, params).await
    }

    // ========================================================================
    // Graph Intelligence APIs (v3.21.0) - Delegated to graph_intelligence module
    // ========================================================================

    /// Handle `graph.validate` RPC call (jsonrpsee)
    pub async fn validate_graph(
        &self,
        params: jsonrpsee::types::Params<'_>,
    ) -> Result<crate::graph::ValidationResult, jsonrpsee::types::ErrorObject<'static>> {
        graph_intelligence::validate_graph(self, params).await
    }

    /// Handle `graph.check_availability` RPC call (jsonrpsee)
    pub async fn check_availability(
        &self,
        params: jsonrpsee::types::Params<'_>,
    ) -> Result<crate::graph::AvailabilityReport, jsonrpsee::types::ErrorObject<'static>> {
        graph_intelligence::check_availability(self, params).await
    }

    /// Handle `graph.suggest_alternatives` RPC call (jsonrpsee)
    pub async fn suggest_alternatives(
        &self,
        params: jsonrpsee::types::Params<'_>,
    ) -> Result<crate::graph::AlternativeSuggestions, jsonrpsee::types::ErrorObject<'static>> {
        graph_intelligence::suggest_alternatives(self, params).await
    }

    /// Handle `coordination.validate_pattern` RPC call (jsonrpsee)
    pub async fn validate_coordination_pattern(
        &self,
        params: jsonrpsee::types::Params<'_>,
    ) -> Result<crate::graph::CoordinationValidationResult, jsonrpsee::types::ErrorObject<'static>>
    {
        graph_intelligence::validate_coordination_pattern(self, params).await
    }

    // ========================================================================
    // Pure JSON Adapters (v3.22.0) - Delegated to domain modules
    // ========================================================================

    /// Service Registry: register_service (pure JSON adapter)
    pub async fn register_service_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::server_pure_rust::JsonRpcError> {
        service_registry::register_service_json(self, params).await
    }

    /// Service Registry: discover_by_capability (pure JSON adapter)
    pub async fn discover_by_capability_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::server_pure_rust::JsonRpcError> {
        service_registry::discover_by_capability_json(self, params).await
    }

    /// Service Registry: get_service_health (pure JSON adapter)
    pub async fn get_service_health_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::server_pure_rust::JsonRpcError> {
        service_registry::get_service_health_json(self, params).await
    }

    /// Service Registry: health_check (pure JSON adapter)
    pub async fn health_check_json(
        &self,
    ) -> Result<serde_json::Value, crate::ipc::server_pure_rust::JsonRpcError> {
        service_registry::health_check_json(self).await
    }

    /// P2P Discovery: discover_by_family (pure JSON adapter)
    pub async fn discover_by_family_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::server_pure_rust::JsonRpcError> {
        p2p_discovery::discover_by_family_json(self, params).await
    }

    /// P2P Discovery: create_genetic_tunnel (pure JSON adapter)
    pub async fn create_genetic_tunnel_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::server_pure_rust::JsonRpcError> {
        p2p_discovery::create_genetic_tunnel_json(self, params).await
    }

    /// P2P Discovery: announce_capabilities (pure JSON adapter)
    pub async fn announce_capabilities_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::server_pure_rust::JsonRpcError> {
        p2p_discovery::announce_capabilities_json(self, params).await
    }

    /// Graph Intelligence: validate_graph (pure JSON adapter)
    pub async fn validate_graph_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::server_pure_rust::JsonRpcError> {
        graph_intelligence::validate_graph_json(self, params).await
    }

    /// Graph Intelligence: check_availability (pure JSON adapter)
    pub async fn check_availability_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::server_pure_rust::JsonRpcError> {
        graph_intelligence::check_availability_json(self, params).await
    }

    /// Graph Intelligence: suggest_alternatives (pure JSON adapter)
    pub async fn suggest_alternatives_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::server_pure_rust::JsonRpcError> {
        graph_intelligence::suggest_alternatives_json(self, params).await
    }

    /// Graph Intelligence: validate_coordination_pattern (pure JSON adapter)
    pub async fn validate_coordination_pattern_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::server_pure_rust::JsonRpcError> {
        graph_intelligence::validate_coordination_pattern_json(self, params).await
    }
}
