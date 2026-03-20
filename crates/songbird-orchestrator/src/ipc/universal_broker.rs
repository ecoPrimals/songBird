// SPDX-License-Identifier: AGPL-3.0-only
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
use songbird_universal_ipc::endpoint::VirtualEndpoint;
use songbird_universal_ipc::handlers::{DiscoveryListenerBridge, PeerRegistry};
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
}

impl UniversalIpcBroker {
    /// Create a new Universal IPC Broker
    ///
    /// This initializes the service handler, registers the songbird endpoint,
    /// and creates a Tower Atomic server for handling JSON-RPC requests.
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
            "songbird",
            vec![
                "ipc".to_string(),
                "discovery".to_string(),
                "registry".to_string(),
                "stun".to_string(), // NEW: STUN methods
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

        // Create service registry
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));

        // Create service handler with discovery bridge if listener provided
        let handler = if let Some(listener) = discovery_listener {
            info!("🌉 Wiring up discovery listener bridge for runtime peer discovery");

            // Create bridge from listener to PeerRegistry trait
            let bridge: Arc<dyn PeerRegistry> = Arc::new(DiscoveryListenerBridge::new(listener));

            // Create handler with discovery registry
            IpcServiceHandler::with_discovery_registry(registry, bridge)
        } else {
            info!("⚠️  No discovery listener provided, discovery.peers will return empty");

            // Create handler without discovery (testing mode)
            IpcServiceHandler::new(registry)
        };

        // Create Tower Atomic server
        let server = TowerAtomicServer::new(handler);

        info!("✅ Universal IPC Broker initialized");
        info!("   Endpoint: {}", endpoint.path);
        info!("   Protocol: JSON-RPC 2.0");
        info!("   Methods: ipc.*, http.*, stun.*, discovery.*, rendezvous.*, peer.*");

        Ok(Self {
            endpoint,
            server,
        })
    }

    /// Start the Universal IPC Broker
    ///
    /// This starts the Tower Atomic server and begins listening for
    /// JSON-RPC requests from other primals.
    ///
    /// Runs indefinitely until the server is stopped or an error occurs.
    pub async fn start(self) -> Result<()> {
        info!("🚀 Starting Universal IPC Broker");
        info!("   Listening on: {}", self.endpoint.path);
        info!("   Waiting for primal connections...");

        // Start Tower Atomic server (runs forever)
        self.server.serve(self.endpoint).await.context("Universal IPC Broker server error")?;

        Ok(())
    }
}

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
///     universal_broker::start_broker_with_discovery(None).await?;
///     
///     // Broker now runs in background, handling JSON-RPC requests
///     Ok(())
/// }
/// ```
pub async fn start_broker() -> Result<()> {
    start_broker_with_discovery(None).await
}

/// Start the Universal IPC Broker with discovery listener
///
/// Enables real-time peer discovery when a listener is provided.
/// This is the recommended way to start the broker in production.
pub async fn start_broker_with_discovery(
    discovery_listener: Option<Arc<AnonymousDiscoveryListener>>,
) -> Result<()> {
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

    // Start broker in background task (runs indefinitely)
    tokio::spawn(async move {
        if let Err(e) = broker.start().await {
            error!("❌ Universal IPC Broker error: {}", e);
        }
    });

    // Give server time to start listening
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    info!("✅ Universal IPC Broker started in background");
    info!("   Other primals can now connect to /primal/songbird");
    info!("   Methods: ipc.*, http.*, stun.*, discovery.*, rendezvous.*, peer.*");
    info!("   NOTE: Service layer handles platform abstraction internally");

    Ok(())
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
                    "Unexpected error: {}",
                    err_msg
                );
            }
        }
    }
}
