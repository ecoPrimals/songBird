// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! IPC Service - Songbird's JSON-RPC IPC Broker
//!
//! This module provides Songbird's IPC brokering service. Instead of other
//! primals importing `songbird-universal-ipc` as a library (which would violate
//! primal autonomy), they connect to Songbird's IPC service via JSON-RPC.
//!
//! ## TRUE PRIMAL Architecture
//!
//! **Problem**: Library embedding violates primal autonomy
//! **Solution**: Service-based architecture via JSON-RPC
//!
//! ```text
//! Other primals (security provider, AI capability provider, etc.):
//!   - Use tokio::net::UnixStream (standard library!)
//!   - Connect to /primal/songbird
//!   - Call JSON-RPC methods for discovery
//!   - Connect directly to discovered services
//!   - ZERO Songbird code embedded!
//!
//! Songbird IPC Service (this module):
//!   - Maintains service registry
//!   - Provides discovery via JSON-RPC
//!   - Manages platform abstraction internally
//!   - Pure service - no code embedding!
//! ```

use crate::handlers::birdsong_handler::BirdSongHandler;
use crate::handlers::discovery_handler::DiscoveryHandler;
use crate::handlers::http_handler::HttpHandler;
use crate::handlers::igd_handler::IgdHandler;
use crate::handlers::mesh_handler::MeshHandler;
use crate::handlers::onion_handler::OnionHandler;
use crate::handlers::peer_handler::PeerHandler;
use crate::handlers::punch_handler::PunchHandler;
use crate::handlers::rendezvous_handler::RendezvousHandler;
use crate::handlers::stun_handler::StunHandler;
use crate::handlers::tor_handler::TorHandler;
use crate::registry::ServiceRegistry;
use songbird_lineage_relay::relay_handler::RelayHandler;
use songbird_network_federation::state::FederationState;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

mod construction;
mod dispatch;
mod http;
mod ipc_registry;
mod meta;
pub mod relay_security;
mod remote_dispatch;
mod util;
pub mod virtual_relay;

pub use crate::service_types::{
    CapabilityCallParams, CapabilityCallResult, CapabilityResolveParams, CapabilityResolveResult,
    CompositionPrimalInfo, CompositionState, DiscoverParams, DiscoverResult,
    FederationPeersResponse, FederationStatusResponse, ListResult, ProviderInfo, RegisterParams,
    RegisterResult, ResolveParams, ResolveResult, ServiceInfo, ValidateConsumedResult,
};

/// Songbird IPC Service Handler
///
/// This handler provides IPC brokering as a JSON-RPC service,
/// allowing other primals to discover and connect to services
/// without embedding Songbird code.
///
/// **TRUE PRIMAL**: Zero code embedding, pure service protocol!
pub struct IpcServiceHandler {
    registry: Arc<RwLock<ServiceRegistry>>,
    /// When set, used instead of [`songbird_process_env::var`] for identity `family_id` resolution (tests).
    family_id_overrides: Option<Arc<HashMap<String, String>>>,
    /// Crypto provider for signing IPC registrations via `BearDog` Ed25519 delegation.
    /// `None` in standalone mode (no `FAMILY_ID`) — registrations proceed unsigned.
    crypto_provider: Option<Arc<songbird_crypto_provider::CryptoProvider>>,
    http_handler: Arc<HttpHandler>,
    stun_handler: Arc<StunHandler>,
    discovery_handler: Arc<DiscoveryHandler>,
    rendezvous_handler: Arc<RendezvousHandler>,
    peer_handler: Arc<PeerHandler>,
    birdsong_handler: Arc<BirdSongHandler>,
    relay_handler: Arc<RelayHandler>,
    mesh_handler: Arc<MeshHandler>,
    onion_handler: Arc<OnionHandler>,
    punch_handler: Arc<PunchHandler>,
    tor_handler: Arc<TorHandler>,
    igd_handler: Arc<IgdHandler>,
    start_time: Arc<RwLock<std::time::Instant>>,
    /// When set, `federation.*` methods reflect live [`FederationState`].
    federation_state: Option<Arc<FederationState>>,
    /// Virtual endpoint relay manager (Phase 1: shadow mode).
    virtual_relay: Arc<virtual_relay::VirtualRelayManager>,
}

impl IpcServiceHandler {
    /// Access the mesh handler (for startup auto-seeding from `SONGBIRD_PEERS`).
    #[must_use]
    pub fn mesh_handler(&self) -> &Arc<MeshHandler> {
        &self.mesh_handler
    }
}

/// All handler instances built by `build_handlers()`.
pub(super) type HandlerBundle = (
    Arc<StunHandler>,
    Arc<RendezvousHandler>,
    Arc<PeerHandler>,
    Arc<BirdSongHandler>,
    Arc<RelayHandler>,
    Arc<MeshHandler>,
    Arc<OnionHandler>,
    Arc<PunchHandler>,
    Arc<TorHandler>,
    Arc<IgdHandler>,
);

#[cfg(test)]
mod service_tests;
