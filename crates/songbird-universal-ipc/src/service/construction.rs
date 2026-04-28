// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::{HandlerBundle, IpcServiceHandler};
use crate::handlers::birdsong_handler::BirdSongHandler;
use crate::handlers::discovery_bridge::DiscoveryListenerBridge;
use crate::handlers::discovery_handler::DiscoveryHandler;
use crate::handlers::http_handler::HttpHandler;
use crate::handlers::http_rendezvous_client::HttpRendezvousClient;
use crate::handlers::igd_handler::IgdHandler;
use crate::handlers::mesh_handler::MeshHandler;
use crate::handlers::onion_handler::OnionHandler;
use crate::handlers::peer_handler::PeerHandler;
use crate::handlers::punch_handler::PunchHandler;
use crate::handlers::rendezvous_handler::{RendezvousClient, RendezvousHandler};
use crate::handlers::stun_handler::StunHandler;
use crate::handlers::tor_handler::TorHandler;
use crate::handlers::udp_peer_connector::{PeerConnector, UdpPeerConnector};
use crate::registry::ServiceRegistry;
use songbird_lineage_relay::relay::RelayAuthority;
use songbird_lineage_relay::relay_handler::RelayHandler;
use songbird_lineage_relay::security::SecurityRelayAuthority;
use songbird_network_federation::state::FederationState;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

impl IpcServiceHandler {
    /// Build all production-ready handler instances.
    ///
    /// Shared by all constructors to eliminate duplication.
    /// All handlers use real implementations (zero mocks in production).
    fn build_handlers() -> HandlerBundle {
        let stun_handler = Arc::new(StunHandler::new());
        let rendezvous_handler = Arc::new(RendezvousHandler::new(Arc::new(
            RendezvousClient::Http(HttpRendezvousClient::new()),
        )));
        let peer_handler =
            Arc::new(PeerHandler::new(Arc::new(PeerConnector::Udp(UdpPeerConnector::new()))));
        let birdsong_handler = Arc::new(BirdSongHandler::new());
        let relay_handler = Arc::new(RelayHandler::new(Arc::new(RelayAuthority::from(
            SecurityRelayAuthority::new(),
        ))));
        let mesh_handler = Arc::new(MeshHandler::new());
        let onion_handler = Arc::new(OnionHandler::new());

        // Create a real HolePunchCoordinator so punch.request works
        let node_id = songbird_process_env::var("SONGBIRD_NODE_ID")
            .or_else(|_| songbird_process_env::var("NODE_ID"))
            .unwrap_or_else(|_| "songbird-default".to_string());
        let punch_config = songbird_onion_relay::coordinator::HolePunchConfig::default();
        let (coordinator, _signal_tx, _signal_rx) =
            songbird_onion_relay::HolePunchCoordinator::new(node_id, punch_config);
        let punch_handler = Arc::new(PunchHandler::with_coordinator(Arc::new(coordinator)));

        let tor_handler = Arc::new(TorHandler::new());
        let igd_handler = Arc::new(IgdHandler::new());

        (
            stun_handler,
            rendezvous_handler,
            peer_handler,
            birdsong_handler,
            relay_handler,
            mesh_handler,
            onion_handler,
            punch_handler,
            tor_handler,
            igd_handler,
        )
    }

    /// Build a crypto provider for signing IPC registrations.
    ///
    /// Returns `Some` when `FAMILY_ID` is set (NUCLEUS composition mode),
    /// indicating `BearDog` is expected to be available for Ed25519 signing.
    /// Returns `None` in standalone mode — registrations proceed unsigned.
    fn build_crypto_provider() -> Option<Arc<songbird_crypto_provider::CryptoProvider>> {
        if songbird_process_env::var("FAMILY_ID").is_ok() {
            let provider = songbird_crypto_provider::CryptoProvider::from_env();
            tracing::info!("IPC registration signing enabled (FAMILY_ID set)");
            Some(Arc::new(provider))
        } else {
            tracing::debug!("IPC registration signing disabled (no FAMILY_ID)");
            None
        }
    }

    /// Assemble a handler from pre-built components.
    fn assemble(
        registry: Arc<RwLock<ServiceRegistry>>,
        http_handler: Arc<HttpHandler>,
        discovery_handler: Arc<DiscoveryHandler>,
        federation_state: Option<Arc<FederationState>>,
    ) -> Self {
        let (
            stun_handler,
            rendezvous_handler,
            peer_handler,
            birdsong_handler,
            relay_handler,
            mesh_handler,
            onion_handler,
            punch_handler,
            tor_handler,
            igd_handler,
        ) = Self::build_handlers();

        Self {
            registry,
            family_id_overrides: None,
            crypto_provider: Self::build_crypto_provider(),
            http_handler,
            stun_handler,
            discovery_handler,
            rendezvous_handler,
            peer_handler,
            birdsong_handler,
            relay_handler,
            mesh_handler,
            onion_handler,
            punch_handler,
            tor_handler,
            igd_handler,
            start_time: Arc::new(RwLock::new(std::time::Instant::now())),
            federation_state,
        }
    }

    /// Create a new IPC service handler with production defaults.
    ///
    /// Uses real implementations for all handlers (zero mocks in production).
    pub fn new(registry: Arc<RwLock<ServiceRegistry>>) -> Self {
        Self::assemble(
            registry,
            Arc::new(HttpHandler::with_default_discovery()),
            Arc::new(DiscoveryHandler::new()),
            None,
        )
    }

    /// Same as [`new`](Self::new) but attaches federation state for `federation.*` JSON-RPC methods.
    #[must_use]
    pub fn with_federation_state(
        registry: Arc<RwLock<ServiceRegistry>>,
        federation_state: Arc<FederationState>,
    ) -> Self {
        Self::assemble(
            registry,
            Arc::new(HttpHandler::with_default_discovery()),
            Arc::new(DiscoveryHandler::new()),
            Some(federation_state),
        )
    }

    /// Create with a peer registry for orchestrator-level discovery.
    pub fn with_discovery_registry(
        registry: Arc<RwLock<ServiceRegistry>>,
        peer_registry: Arc<DiscoveryListenerBridge>,
    ) -> Self {
        Self::assemble(
            registry,
            Arc::new(HttpHandler::with_default_discovery()),
            Arc::new(DiscoveryHandler::with_bridge(peer_registry)),
            None,
        )
    }

    /// Create with a custom HTTP handler (for dependency injection).
    pub fn with_http_handler(
        registry: Arc<RwLock<ServiceRegistry>>,
        http_handler: Arc<HttpHandler>,
    ) -> Self {
        Self::assemble(registry, http_handler, Arc::new(DiscoveryHandler::new()), None)
    }

    /// Same as [`new`](Self::new) but resolves `family_id` for `identity` using this map instead of
    /// [`songbird_process_env::var`] (for tests and injected configuration).
    /// Crypto provider is `None` (test mode — registrations are unsigned).
    #[must_use]
    pub fn with_family_id_overrides(
        registry: Arc<RwLock<ServiceRegistry>>,
        overrides: HashMap<String, String>,
    ) -> Self {
        let mut h = Self::new(registry);
        h.family_id_overrides = Some(Arc::new(overrides));
        h.crypto_provider = None;
        h
    }

    /// Access the underlying service registry (for startup auto-discovery seeding).
    #[must_use]
    pub fn registry(&self) -> &Arc<RwLock<ServiceRegistry>> {
        &self.registry
    }
}
