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

mod discovery_handlers;
mod http_methods;
mod json_adapters;
mod mesh_dispatch;
mod typed_delegates;

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
        let availability_checker =
            Arc::new(AvailabilityChecker::new(Arc::clone(&service_registry)));
        let coordination_validator =
            Arc::new(CoordinationValidator::new(Arc::clone(&service_registry)));
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
}

#[cfg(test)]
#[path = "ipc_handlers_tests.rs"]
mod ipc_handlers_tests;
