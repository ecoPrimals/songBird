// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Universal IPC Broker Service
//!
//! This module integrates Songbird's Universal IPC service into the orchestrator.
//! It provides JSON-RPC brokering for inter-primal communication, allowing other
//! primals to discover and connect to services without embedding Songbird code.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │ Songbird Orchestrator                                    │
//! │ ┌─────────────────────────────────────────────────────┐ │
//! │ │ Universal IPC Broker (this module)                  │ │
//! │ │ Listens on: /primal/songbird                        │ │
//! │ │                                                      │ │
//! │ │ JSON-RPC Methods:                                   │ │
//! │ │  • ipc.*          - Service registration/discovery │ │
//! │ │  • http.*         - HTTP/HTTPS requests            │ │
//! │ │  • stun.*         - NAT traversal (Dark Forest)    │ │
//! │ │  • discovery.*    - Peer discovery (Dark Forest)   │ │
//! │ │  • rendezvous.*   - Relay server (Dark Forest)     │ │
//! │ │  • peer.*         - Hole punching (Dark Forest)    │ │
//! │ └─────────────────────────────────────────────────────┘ │
//! │                                                           │
//! │ Uses internally:                                          │
//! │  - songbird-universal-ipc crate (service layer)           │
//! │  - Platform abstraction (Unix/Windows/etc)                │
//! │  - Service registry                                       │
//! └─────────────────────────────────────────────────────────┘
//!
//! Other Primals:
//!  - Use tokio::net::UnixStream (standard!)
//!  - Connect to /primal/songbird
//!  - Make JSON-RPC calls
//!  - ZERO Songbird code embedded! ✅
//! ```
//!
//! ## TRUE PRIMAL Architecture
//!
//! This design preserves primal autonomy:
//! - ✅ No code embedding (other primals use standard tokio)
//! - ✅ Service-based (protocol, not library)
//! - ✅ Platform-agnostic (Songbird handles abstraction)
//! - ✅ Runtime discovery (capability-based)

use anyhow::{Context, Result};
use songbird_discovery::anonymous::AnonymousDiscoveryListener;
use songbird_types::primal_names;
use songbird_universal_ipc::endpoint::VirtualEndpoint;
use songbird_universal_ipc::handlers::DiscoveryListenerBridge;
use songbird_universal_ipc::ipc;
use songbird_universal_ipc::service::IpcServiceHandler;
use songbird_universal_ipc::tower_atomic::TowerAtomicServer;
use std::sync::Arc;
use tracing::{error, info, warn};

/// Universal IPC Broker
///
/// Provides JSON-RPC service for inter-primal IPC discovery and registration.
pub struct UniversalIpcBroker {
    endpoint: VirtualEndpoint,
    server: TowerAtomicServer<IpcServiceHandler>,
    registry: Arc<tokio::sync::RwLock<songbird_universal_ipc::registry::ServiceRegistry>>,
    mesh_handler: Arc<songbird_universal_ipc::handlers::MeshHandler>,
}

impl UniversalIpcBroker {
    /// Create a new Universal IPC Broker
    ///
    /// This initializes the service handler, registers the songbird endpoint,
    /// and creates a Tower Atomic server for handling JSON-RPC requests.
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn new() -> Result<Self> {
        Self::with_discovery_listener(None).await
    }

    /// Create a new Universal IPC Broker with discovery listener
    ///
    /// When a discovery listener is provided, the broker will expose real peer
    /// discovery data via the `discovery.peers` JSON-RPC method.
    ///
    /// ## Runtime Discovery (Zero Hardcoding)
    ///
    /// This follows the TRUE PRIMAL pattern:
    /// - No hardcoding: discovers peers at runtime
    /// - Capability-based: uses trait-based dependency injection
    /// - Smart refactoring: bridge pattern, not tight coupling
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn with_discovery_listener(
        discovery_listener: Option<Arc<AnonymousDiscoveryListener>>,
    ) -> Result<Self> {
        use songbird_universal_ipc::registry::ServiceRegistry;
        use tokio::sync::RwLock;

        info!("🌍 Initializing Universal IPC Broker");

        // Initialize Universal IPC system
        ipc::init().context("Failed to initialize Universal IPC")?;

        // Register Songbird as an IPC service provider
        // Note: If already registered, this will return an error which we handle gracefully
        let endpoint = match ipc::register(
            primal_names::SELF_NAME,
            vec![
                String::from("ipc"),
                String::from("discovery"),
                String::from("registry"),
                String::from("stun"), // NEW: STUN methods
            ],
        )
        .await
        {
            Ok(endpoint) => {
                info!("✅ Songbird registered at endpoint: {}", endpoint.path);
                endpoint
            }
            Err(e) if e.to_string().contains("already registered") => {
                warn!("⚠️  Songbird already registered, using existing registration");
                VirtualEndpoint {
                    path: String::from("/primal/songbird"),
                }
            }
            Err(e) => {
                return Err(e).context("Failed to register Songbird IPC endpoint");
            }
        };

        // Create service registry (shared — returned via `registry()` for startup seeding)
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));

        // Create service handler with discovery bridge if listener provided
        let handler = if let Some(listener) = discovery_listener {
            info!("🌉 Wiring up discovery listener bridge for runtime peer discovery");

            let bridge: Arc<DiscoveryListenerBridge> =
                Arc::new(DiscoveryListenerBridge::new(listener));

            // Create handler with discovery registry
            IpcServiceHandler::with_discovery_registry(Arc::clone(&registry), bridge)
        } else {
            info!("⚠️  No discovery listener provided, discovery.peers will return empty");

            // Create handler without discovery (testing mode)
            IpcServiceHandler::new(Arc::clone(&registry))
        };

        // Capture mesh handler before handler moves into server
        let mesh_handler = Arc::clone(handler.mesh_handler());

        // Create Tower Atomic server
        let server = TowerAtomicServer::new(handler);

        info!("✅ Universal IPC Broker initialized");
        info!("   Endpoint: {}", endpoint.path);
        info!("   Protocol: JSON-RPC 2.0");
        info!("   Methods: ipc.*, http.*, stun.*, discovery.*, rendezvous.*, peer.*");

        Ok(Self {
            endpoint,
            server,
            registry,
            mesh_handler,
        })
    }

    /// Access the service registry backing this broker's `ipc.resolve` / `capability.resolve`.
    ///
    /// Used by startup auto-discovery (LD-08) to seed the registry with primals
    /// found in the biomeos socket directory.
    #[must_use]
    pub fn registry(
        &self,
    ) -> &Arc<tokio::sync::RwLock<songbird_universal_ipc::registry::ServiceRegistry>> {
        &self.registry
    }

    /// Access the mesh handler for auto-seeding from `SONGBIRD_PEERS` on boot.
    #[must_use]
    pub fn mesh_handler(&self) -> &Arc<songbird_universal_ipc::handlers::MeshHandler> {
        &self.mesh_handler
    }

    /// Start the Universal IPC Broker (runs indefinitely).
    ///
    /// # Errors
    ///
    /// Returns an error if the server fails to bind or encounters a fatal error.
    pub async fn start(self) -> Result<()> {
        info!("🚀 Starting Universal IPC Broker");
        info!("   Listening on: {}", self.endpoint.path);
        info!("   Waiting for primal connections...");

        self.server.serve(self.endpoint).await.context("Universal IPC Broker server error")?;
        Ok(())
    }

    /// Start with readiness notification — signals when the socket is bound.
    ///
    /// # Errors
    ///
    /// Returns an error if the server fails to bind or encounters a fatal error.
    pub async fn start_with_ready(self, ready: tokio::sync::oneshot::Sender<()>) -> Result<()> {
        info!("🚀 Starting Universal IPC Broker");
        info!("   Listening on: {}", self.endpoint.path);

        self.server
            .serve_with_ready(self.endpoint, ready)
            .await
            .context("Universal IPC Broker server error")?;
        Ok(())
    }

    /// Create broker using a pre-built shared handler (state unification).
    ///
    /// Both the HTTP server and UDS broker share the same `IpcServiceHandler`,
    /// ensuring `ipc.register` and `mesh.init` state is visible on both transports.
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn with_shared_handler(
        handler: Arc<IpcServiceHandler>,
        registry: Arc<tokio::sync::RwLock<songbird_universal_ipc::registry::ServiceRegistry>>,
    ) -> Result<Self> {
        use songbird_universal_ipc::ipc;

        info!("🌍 Initializing Universal IPC Broker (shared handler mode)");

        ipc::init().context("Failed to initialize Universal IPC")?;

        let endpoint = match ipc::register(
            primal_names::SELF_NAME,
            vec![
                "ipc".to_string(),
                "discovery".to_string(),
                "registry".to_string(),
                "stun".to_string(),
            ],
        )
        .await
        {
            Ok(endpoint) => {
                info!("✅ Songbird registered at endpoint: {}", endpoint.path);
                endpoint
            }
            Err(e) if e.to_string().contains("already registered") => {
                warn!("⚠️  Songbird already registered, using existing registration");
                VirtualEndpoint {
                    path: "/primal/songbird".to_string(),
                }
            }
            Err(e) => {
                return Err(e).context("Failed to register Songbird IPC endpoint");
            }
        };

        let mesh_handler = Arc::clone(handler.mesh_handler());
        let server = TowerAtomicServer::from_shared(handler);

        info!("✅ Universal IPC Broker initialized (shared state)");

        Ok(Self {
            endpoint,
            server,
            registry,
            mesh_handler,
        })
    }
}

/// Shared handle to the broker's service registry (returned by [`start_broker_with_discovery`]).
pub type SharedServiceRegistry =
    Arc<tokio::sync::RwLock<songbird_universal_ipc::registry::ServiceRegistry>>;

/// Start the Universal IPC Broker as a background task
///
/// This is the main entry point for integrating the Universal IPC Broker
/// into the Songbird orchestrator startup sequence.
///
/// ## Runtime Discovery
///
/// Pass a discovery listener to enable real-time peer discovery via JSON-RPC.
/// Without it, `discovery.peers` returns empty (testing mode).
///
/// ## Usage
///
/// ```no_run
/// use songbird_orchestrator::ipc::universal_broker;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Start the Universal IPC Broker with discovery
///     let _registry = universal_broker::start_broker_with_discovery(None).await?;
///     
///     // Broker now runs in background, handling JSON-RPC requests
///     Ok(())
/// }
/// ```
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn start_broker() -> Result<BrokerHandle> {
    start_broker_with_discovery(None).await
}

/// Handle returned by [`start_broker_with_discovery`] containing the registry
/// and mesh handler for startup wiring.
pub struct BrokerHandle {
    /// Shared service registry for auto-discovery seeding.
    pub registry: SharedServiceRegistry,
    /// Mesh handler for auto-seeding peers from `SONGBIRD_PEERS`.
    pub mesh_handler: Arc<songbird_universal_ipc::handlers::MeshHandler>,
}

/// Start the Universal IPC Broker with discovery listener.
///
/// Returns a [`BrokerHandle`] containing the shared `ServiceRegistry` and
/// mesh handler so the startup sequence can seed registrations and bootstrap
/// mesh peers.
///
/// Enables real-time peer discovery when a listener is provided.
/// This is the recommended way to start the broker in production.
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn start_broker_with_discovery(
    discovery_listener: Option<Arc<AnonymousDiscoveryListener>>,
) -> Result<BrokerHandle> {
    info!("🌍 Starting Universal IPC Broker (service-based architecture)");

    if discovery_listener.is_some() {
        info!("   Runtime discovery: ✅ ENABLED (real peer data)");
    } else {
        info!("   Runtime discovery: ⚠️  DISABLED (testing mode)");
    }

    // Create broker with discovery listener
    let broker = UniversalIpcBroker::with_discovery_listener(discovery_listener)
        .await
        .context("Failed to create Universal IPC Broker")?;

    info!("✅ Universal IPC Broker created successfully");

    let registry = Arc::clone(broker.registry());
    let mesh_handler = Arc::clone(broker.mesh_handler());
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        if let Err(e) = broker.start_with_ready(ready_tx).await {
            error!("❌ Universal IPC Broker error: {}", e);
        }
    });

    ready_rx
        .await
        .map_err(|_| anyhow::anyhow!("Universal IPC Broker failed to bind (task dropped)"))?;

    info!("✅ Universal IPC Broker started in background");
    info!("   Other primals can now connect to /primal/songbird");
    info!("   Methods: ipc.*, http.*, stun.*, discovery.*, rendezvous.*, peer.*");
    info!("   NOTE: Service layer handles platform abstraction internally");

    Ok(BrokerHandle {
        registry,
        mesh_handler,
    })
}

/// Start the Universal IPC Broker with a pre-built shared handler.
///
/// This achieves HTTP/UDS state unification: the same `IpcServiceHandler`
/// backs both the HTTP `/jsonrpc` endpoint and the UDS `/primal/songbird` socket.
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn start_broker_with_shared_handler(
    handler: Arc<songbird_universal_ipc::service::IpcServiceHandler>,
    registry: SharedServiceRegistry,
) -> Result<BrokerHandle> {
    info!("🌍 Starting Universal IPC Broker (shared handler — HTTP/UDS unified)");

    let mesh_handler = Arc::clone(handler.mesh_handler());
    let broker = UniversalIpcBroker::with_shared_handler(handler, Arc::clone(&registry))
        .await
        .context("Failed to create Universal IPC Broker (shared)")?;

    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        if let Err(e) = broker.start_with_ready(ready_tx).await {
            error!("❌ Universal IPC Broker error: {}", e);
        }
    });

    ready_rx.await.map_err(|_| anyhow::anyhow!("Universal IPC Broker (shared) failed to bind"))?;

    info!("✅ Universal IPC Broker started (shared state with HTTP)");

    Ok(BrokerHandle {
        registry,
        mesh_handler,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_broker_creation() {
        // Note: This test may fail if run concurrently with other tests
        // that also initialize the global IPC system, due to the singleton
        // nature of the IPC registry. This is expected behavior.
        //
        // In production, only one broker instance is created at startup.
        let broker = UniversalIpcBroker::new().await;

        // Either succeeds or fails with "already registered" (both OK)
        match broker {
            Ok(_) => {
                // Success case
            }
            Err(e) => {
                let err_msg = e.to_string();
                assert!(
                    err_msg.contains("already registered")
                        || err_msg.contains("Service already registered"),
                    "Unexpected error: {err_msg}"
                );
            }
        }
    }
}
