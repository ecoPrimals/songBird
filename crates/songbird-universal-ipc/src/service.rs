// SPDX-License-Identifier: AGPL-3.0-only
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
//! Other Primals (BearDog, Squirrel, etc.):
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

use crate::endpoint::NativeEndpoint;
use crate::handlers::birdsong_handler::BirdSongHandler; // BirdSong (Feb 2, 2026)
use crate::handlers::discovery_handler::{DiscoveryHandler, PeerRegistry};
use crate::handlers::http_handler::{HttpHandler, HttpRequestParams};
use crate::handlers::http_rendezvous_client::HttpRendezvousClient;
use crate::handlers::igd_handler::IgdHandler; // IGD router config (Feb 8, 2026)
use crate::handlers::mesh_handler::MeshHandler; // Mesh networking (Feb 4, 2026)
use crate::handlers::onion_handler::OnionHandler; // Sovereign onion (Feb 4, 2026)
use crate::handlers::peer_handler::PeerHandler;
use crate::handlers::punch_handler::PunchHandler; // Hole punch (Feb 4, 2026)
use crate::handlers::rendezvous_handler::RendezvousHandler;
use crate::handlers::stun_handler::StunHandler;
use crate::handlers::tor_handler::TorHandler; // Pure Rust Tor (Feb 7, 2026)
use crate::handlers::udp_peer_connector::UdpPeerConnector;
use crate::registry::ServiceRegistry;
use crate::tower_atomic::JsonRpcHandler;
use async_trait::async_trait;
use serde::Serialize;
use serde_json::Value;
use songbird_lineage_relay::beardog::BearDogRelayAuthority; // Production relay auth (Feb 8, 2026)
use songbird_lineage_relay::relay_handler::RelayHandler; // Relay Server (Feb 5, 2026)
use songbird_network_federation::state::FederationState;
use songbird_types::json_rpc_method::{
    BirdsongMethod, CapabilitiesMethod, DiscoveryMethod, FederationMethod, HealthMethod,
    HttpMethod, IgdMethod, IpcMethod, JsonRpcMethod, MeshMethod, OnionMethod, PeerMethod,
    PrimalMethod, PunchMethod, RelayMethod, RendezvousMethod, RpcMethod, StunMethod, TorMethod,
};
use std::env::VarError;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Injectable environment reader (for concurrent-safe tests without global mutation).
type EnvReader = dyn Fn(&str) -> Result<String, VarError> + Send + Sync;

pub use crate::service_types::{
    DiscoverParams, DiscoverResult, FederationPeersResponse, FederationStatusResponse, ListResult,
    ProviderInfo, RegisterParams, RegisterResult, ResolveParams, ResolveResult, ServiceInfo,
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
    family_id_env: Option<Arc<EnvReader>>,
    http_handler: Arc<HttpHandler>,
    stun_handler: Arc<StunHandler>,
    discovery_handler: Arc<DiscoveryHandler>,
    rendezvous_handler: Arc<RendezvousHandler>,
    peer_handler: Arc<PeerHandler>,
    birdsong_handler: Arc<BirdSongHandler>, // BirdSong (Feb 2, 2026)
    relay_handler: Arc<RelayHandler>,       // Relay Server (Feb 5, 2026)
    mesh_handler: Arc<MeshHandler>,         // Mesh networking (Feb 4, 2026)
    onion_handler: Arc<OnionHandler>,       // Sovereign onion (Feb 4, 2026)
    punch_handler: Arc<PunchHandler>,       // Hole punch (Feb 4, 2026)
    tor_handler: Arc<TorHandler>,           // Pure Rust Tor (Feb 7, 2026)
    igd_handler: Arc<IgdHandler>,           // IGD router config (Feb 8, 2026)
    start_time: Arc<RwLock<std::time::Instant>>, // Track uptime (Feb 5, 2026)
    /// When set, `federation.*` methods reflect live [`FederationState`].
    federation_state: Option<Arc<FederationState>>,
}

/// All handler instances built by `build_handlers()`.
type HandlerBundle = (
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

impl IpcServiceHandler {
    /// Build all production-ready handler instances.
    ///
    /// Shared by all constructors to eliminate duplication.
    /// All handlers use real implementations (zero mocks in production).
    fn build_handlers() -> HandlerBundle {
        let stun_handler = Arc::new(StunHandler::new());
        let rendezvous_handler =
            Arc::new(RendezvousHandler::new(Arc::new(HttpRendezvousClient::new())));
        let peer_handler = Arc::new(PeerHandler::new(Arc::new(UdpPeerConnector::new())));
        let birdsong_handler = Arc::new(BirdSongHandler::new());
        let relay_handler = Arc::new(RelayHandler::new(Arc::new(BearDogRelayAuthority::new())));
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
            family_id_env: None,
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
        peer_registry: Arc<dyn PeerRegistry>,
    ) -> Self {
        Self::assemble(
            registry,
            Arc::new(HttpHandler::with_default_discovery()),
            Arc::new(DiscoveryHandler::with_registry(peer_registry)),
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

    /// Same as [`new`](Self::new) but resolves `family_id` for `identity` via `env` instead of
    /// [`songbird_process_env::var`] (for tests and injected configuration).
    #[must_use]
    pub fn with_family_id_env<F>(registry: Arc<RwLock<ServiceRegistry>>, env: F) -> Self
    where
        F: Fn(&str) -> Result<String, VarError> + Send + Sync + 'static,
    {
        let mut h = Self::new(registry);
        h.family_id_env = Some(Arc::new(env));
        h
    }

    fn parse_tcp_port(addr: &str) -> Result<u16, String> {
        addr.split(':')
            .next_back()
            .and_then(|p| p.parse().ok())
            .ok_or_else(|| format!("Invalid TCP address: {addr}"))
    }

    fn parse_local_tcp_endpoint(endpoint: &str) -> Option<u16> {
        let (host, port_str) = endpoint.rsplit_once(':')?;
        let port: u16 = port_str.parse().ok()?;
        let is_local = matches!(host, "127.0.0.1" | "0.0.0.0" | "localhost" | "::1" | "[::1]");
        is_local.then_some(port)
    }

    /// Handle `ipc.register` method
    async fn handle_register(&self, params: Value) -> Result<Value, String> {
        let params: RegisterParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        info!("Registering primal: {} at {}", params.primal_id, params.endpoint);

        // Parse native endpoint
        let native_endpoint = if params.endpoint.starts_with('/') {
            NativeEndpoint::UnixSocket(params.endpoint.into())
        } else if let Some(stripped) = params.endpoint.strip_prefix("unix://") {
            NativeEndpoint::UnixSocket(stripped.into())
        } else if let Some(stripped) = params.endpoint.strip_prefix("tcp://") {
            let port = Self::parse_tcp_port(stripped)?;
            NativeEndpoint::TcpLocal(port)
        } else if let Some(port) = Self::parse_local_tcp_endpoint(&params.endpoint) {
            NativeEndpoint::TcpLocal(port)
        } else {
            return Err(format!(
                "Invalid endpoint format: '{}'. Expected unix socket path, unix://<path>, tcp://<host>:<port>, or <localhost>:<port>",
                params.endpoint
            ));
        };

        // Register in registry (`register` takes `&self` and uses its own inner lock)
        let virtual_endpoint = self
            .registry
            .read()
            .await
            .register(&params.primal_id, native_endpoint, params.capabilities)
            .await
            .map_err(|e| format!("Registration failed: {e}"))?;

        let result = RegisterResult {
            virtual_endpoint: virtual_endpoint.path,
            registered_at: chrono::Utc::now().to_rfc3339(),
        };

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `ipc.resolve` method
    async fn handle_resolve(&self, params: Value) -> Result<Value, String> {
        let params: ResolveParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        debug!("Resolving primal: {}", params.primal_id);

        // Get service entry from registry
        let entry = self
            .registry
            .read()
            .await
            .get_service(&params.primal_id)
            .await
            .ok_or_else(|| format!("Primal not found: {}", params.primal_id))?;

        let result = ResolveResult {
            virtual_endpoint: entry.virtual_endpoint.path,
            native_endpoint: entry.native_endpoint.display(),
            capabilities: entry.capabilities,
        };

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `ipc.discover` method
    async fn handle_discover(&self, params: Value) -> Result<Value, String> {
        let params: DiscoverParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        debug!("Discovering capability: {}", params.capability);

        // Discover from registry (returns virtual paths)
        let registry = self.registry.read().await;
        let virtual_paths = registry.find_by_capability(&params.capability).await;

        // Get full service entries for each path
        let mut provider_infos = Vec::new();
        for virtual_path in virtual_paths {
            // Extract service name from virtual path
            if let Some(name) = virtual_path.strip_prefix("/primal/")
                && let Some(entry) = registry.get_service(name).await
            {
                provider_infos.push(ProviderInfo {
                    primal_id: name.to_string(),
                    virtual_endpoint: entry.virtual_endpoint.path,
                    native_endpoint: entry.native_endpoint.display(),
                    capabilities: entry.capabilities,
                });
            }
        }

        let result = DiscoverResult {
            providers: provider_infos,
        };

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `ipc.list` method
    async fn handle_list(&self, _params: Value) -> Result<Value, String> {
        debug!("Listing all services");

        // List all from registry (returns service names)
        let registry = self.registry.read().await;
        let service_names = registry.list_services().await;

        // Get full service entries for each name
        let mut service_infos = Vec::new();
        for name in service_names {
            if let Some(entry) = registry.get_service(&name).await {
                service_infos.push(ServiceInfo {
                    primal_id: name,
                    virtual_endpoint: entry.virtual_endpoint.path,
                    capabilities: entry.capabilities,
                });
            }
        }

        let result = ListResult {
            services: service_infos,
        };

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `http.request` method - Full HTTP/HTTPS request
    async fn handle_http_request(&self, params: Value) -> Result<Value, String> {
        let params: HttpRequestParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        info!("HTTP request via IPC: {} {}", params.method, params.url);

        let result = self
            .http_handler
            .handle_request(params)
            .await
            .map_err(|e| format!("HTTP request failed: {e}"))?;

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `http.get` method - GET request shorthand
    async fn handle_http_get(&self, params: Value) -> Result<Value, String> {
        let url = params
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'url' parameter".to_string())?;

        info!("HTTP GET via IPC: {}", url);

        let result =
            self.http_handler.handle_get(url).await.map_err(|e| format!("HTTP GET failed: {e}"))?;

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `http.post` method - POST request shorthand
    async fn handle_http_post(&self, params: Value) -> Result<Value, String> {
        let url = params
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'url' parameter".to_string())?;

        let body = params
            .get("body")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'body' parameter".to_string())?;

        let content_type = params.get("content_type").and_then(|v| v.as_str());

        // FIX: Extract headers from params (Issue #1 - Jan 28, 2026)
        let headers: std::collections::HashMap<String, String> = params
            .get("headers")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        info!("HTTP POST via IPC: {}", url);

        let result = self
            .http_handler
            .handle_post(url, body, content_type, headers)
            .await
            .map_err(|e| format!("HTTP POST failed: {e}"))?;

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Serialize a handler result into a JSON-RPC response `Value`.
    fn wrap_result<T: Serialize>(
        result: std::result::Result<T, impl std::fmt::Display>,
        context: &str,
    ) -> Result<Value, String> {
        let val = result.map_err(|e| format!("{context}: {e}"))?;
        serde_json::to_value(val).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `birdsong.advertise` method
    ///
    /// Generates an encrypted beacon with the onion endpoint (if running).
    /// This is the complete Dark Forest beacon - only family can see the .onion address.
    ///
    /// NEW (Feb 6, 2026) - Combines onion service and birdsong beacon
    async fn handle_birdsong_advertise(&self, params: Value) -> Result<Value, String> {
        // Get node_id and capabilities from params
        let node_id = params
            .get("node_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing node_id parameter")?
            .to_string();

        let capabilities: Vec<String> = params
            .get("capabilities")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        // Get onion address if service is running
        let onion_status = self.onion_handler.handle_status(serde_json::json!({})).await?;
        let onion_endpoint = if onion_status.get("running") == Some(&serde_json::json!(true)) {
            let addr = onion_status.get("onion_address").and_then(|v| v.as_str());
            let port = onion_status.get("port").and_then(serde_json::Value::as_u64).unwrap_or(3492);
            addr.map(|a| format!("{a}:{port}"))
        } else {
            None
        };

        // Also include any direct endpoint hints
        let endpoint_hints = params.get("endpoint_hints").cloned();

        // Generate the encrypted beacon with all endpoints
        let beacon_params = serde_json::json!({
            "node_id": node_id,
            "capabilities": capabilities,
            "onion_endpoint": onion_endpoint,
            "endpoint_hints": endpoint_hints,
        });

        let beacon_result =
            self.birdsong_handler.handle_generate_encrypted_beacon(beacon_params).await?;

        // Return combined result
        Ok(serde_json::json!({
            "beacon": beacon_result,
            "onion_endpoint": onion_endpoint,
            "onion_running": onion_status.get("running"),
        }))
    }

    /// Handle `health` method — requires uptime + registry info
    async fn handle_health(&self) -> Result<Value, String> {
        let uptime_secs = self.start_time.read().await.elapsed().as_secs();
        let service_count = self.registry.read().await.list_services().await.len();
        Ok(crate::introspection::health(uptime_secs, service_count))
    }

    /// Handle `identity` method — canonical family-id lookup
    async fn handle_identity(&self) -> Result<Value, String> {
        let family_id = self.family_id_env.as_ref().map_or_else(
            || crate::introspection::canonical_family_id(|k| songbird_process_env::var(k)),
            |f| crate::introspection::canonical_family_id(|k| (f)(k)),
        );
        Ok(crate::introspection::identity(&family_id))
    }

    /// `songbird.federation.peers` / `federation.peers`
    async fn handle_federation_peers_rpc(&self) -> Result<Value, String> {
        let Some(ref state) = self.federation_state else {
            return serde_json::to_value(FederationPeersResponse {
                peers: vec![],
                total_count: 0,
                federation_enabled: false,
            })
            .map_err(|e| format!("Serialization error: {e}"));
        };

        let mut peers: Vec<String> =
            state.active_nodes().await.into_iter().map(|n| n.node_id).collect();
        peers.sort();
        let federation_stats = state.get_stats().await;
        let total_count = peers.len();
        let federation_enabled = federation_stats.total_nodes > 0;

        serde_json::to_value(FederationPeersResponse {
            peers,
            total_count,
            federation_enabled,
        })
        .map_err(|e| format!("Serialization error: {e}"))
    }

    /// `songbird.federation.status` / `federation.status`
    async fn handle_federation_status_rpc(&self) -> Result<Value, String> {
        let Some(ref state) = self.federation_state else {
            return serde_json::to_value(FederationStatusResponse {
                enabled: false,
                active_connections: 0,
            })
            .map_err(|e| format!("Serialization error: {e}"));
        };

        let fed_stats = state.get_stats().await;
        serde_json::to_value(FederationStatusResponse {
            enabled: fed_stats.total_nodes > 0,
            active_connections: fed_stats.active_nodes,
        })
        .map_err(|e| format!("Serialization error: {e}"))
    }
}

#[async_trait]
impl JsonRpcHandler for IpcServiceHandler {
    #[expect(
        clippy::too_many_lines,
        reason = "JSON-RPC dispatch table — single match over all methods"
    )]
    async fn handle(&self, method: &str, params: Value) -> Result<Value, String> {
        let method = match JsonRpcMethod::parse_ipc(method) {
            Ok(m) => m,
            Err(e) => return Err(e.to_string()),
        };
        match method {
            // ── Introspection ────────────────────────────────────────
            JsonRpcMethod::Primal(PrimalMethod::Info) => Ok(crate::introspection::primal_info()),
            JsonRpcMethod::Primal(PrimalMethod::Capabilities) => {
                Ok(crate::introspection::primal_capabilities())
            }
            JsonRpcMethod::Rpc(RpcMethod::Methods) => Ok(crate::introspection::rpc_methods()),
            JsonRpcMethod::Rpc(RpcMethod::Discover) => {
                Ok(crate::introspection::rpc_discover_standard())
            }
            JsonRpcMethod::DiscoverCapabilities => {
                Ok(crate::introspection::discover_capabilities())
            }

            // ── biomeOS / ecosystem standard ─────────────────────────
            JsonRpcMethod::Health(HealthMethod::Liveness) => {
                Ok(crate::introspection::health_liveness())
            }
            JsonRpcMethod::Health(HealthMethod::Readiness) => {
                Ok(crate::introspection::health_readiness())
            }
            JsonRpcMethod::Health(HealthMethod::Check) => self.handle_health().await,
            JsonRpcMethod::Capabilities(CapabilitiesMethod::List) => {
                Ok(crate::introspection::capabilities_list())
            }
            JsonRpcMethod::Identity => self.handle_identity().await,

            // ── IPC registry ─────────────────────────────────────────
            JsonRpcMethod::Ipc(IpcMethod::Register) => self.handle_register(params).await,
            JsonRpcMethod::Ipc(IpcMethod::Resolve) => self.handle_resolve(params).await,
            JsonRpcMethod::Ipc(IpcMethod::Discover) => self.handle_discover(params).await,
            JsonRpcMethod::Ipc(IpcMethod::List) => self.handle_list(params).await,

            // ── HTTP/HTTPS ───────────────────────────────────────────
            JsonRpcMethod::Http(HttpMethod::Request) => self.handle_http_request(params).await,
            JsonRpcMethod::Http(HttpMethod::Get) => self.handle_http_get(params).await,
            JsonRpcMethod::Http(HttpMethod::Post) => self.handle_http_post(params).await,

            // ── STUN / NAT traversal ─────────────────────────────────
            JsonRpcMethod::Stun(StunMethod::Serve) => self.stun_handler.handle_serve(params).await,
            JsonRpcMethod::Stun(StunMethod::Stop) => self.stun_handler.handle_stop(params).await,
            JsonRpcMethod::Stun(StunMethod::Status) => {
                self.stun_handler.handle_status(params).await
            }
            JsonRpcMethod::Stun(StunMethod::GetPublicAddress) => {
                self.stun_handler.handle_get_public_address(params).await
            }
            JsonRpcMethod::Stun(StunMethod::Bind) => self.stun_handler.handle_bind(params).await,
            JsonRpcMethod::Stun(StunMethod::ProbePortPattern) => {
                self.stun_handler.handle_probe_port_pattern(params).await
            }
            JsonRpcMethod::Stun(StunMethod::DetectNatType) => {
                self.stun_handler.handle_detect_nat_type(params).await
            }

            // ── IGD router auto-configuration ────────────────────────
            JsonRpcMethod::Igd(IgdMethod::Discover) => {
                Ok(self.igd_handler.handle_discover(params).await)
            }
            JsonRpcMethod::Igd(IgdMethod::MapPort) => {
                Ok(self.igd_handler.handle_map_port(params).await)
            }
            JsonRpcMethod::Igd(IgdMethod::UnmapPort) => {
                Ok(self.igd_handler.handle_unmap_port(params).await)
            }
            JsonRpcMethod::Igd(IgdMethod::Status) => {
                Ok(self.igd_handler.handle_status(params).await)
            }
            JsonRpcMethod::Igd(IgdMethod::ExternalIp) => {
                Ok(self.igd_handler.handle_external_ip(params).await)
            }
            JsonRpcMethod::Igd(IgdMethod::AutoConfigure) => {
                Ok(self.igd_handler.handle_auto_configure(params).await)
            }

            // ── Relay server ─────────────────────────────────────────
            JsonRpcMethod::Relay(RelayMethod::Serve) => {
                self.relay_handler.handle_serve(params).await
            }
            JsonRpcMethod::Relay(RelayMethod::Stop) => self.relay_handler.handle_stop(params).await,
            JsonRpcMethod::Relay(RelayMethod::Status) => {
                self.relay_handler.handle_status(params).await
            }
            JsonRpcMethod::Relay(RelayMethod::Allocate) => {
                self.relay_handler.handle_allocate(params).await
            }

            // ── Discovery / rendezvous / peers ───────────────────────
            JsonRpcMethod::Discovery(DiscoveryMethod::Peers) => Self::wrap_result(
                self.discovery_handler.handle_list_peers(params).await,
                "Discovery peers failed",
            ),
            JsonRpcMethod::Discovery(DiscoveryMethod::Announce) => Self::wrap_result(
                self.discovery_handler.handle_announce(params).await,
                "Discovery announce failed",
            ),
            JsonRpcMethod::Rendezvous(RendezvousMethod::Register) => Self::wrap_result(
                self.rendezvous_handler.handle_register(params).await,
                "Rendezvous register failed",
            ),
            JsonRpcMethod::Rendezvous(RendezvousMethod::Lookup) => Self::wrap_result(
                self.rendezvous_handler.handle_lookup(params).await,
                "Rendezvous lookup failed",
            ),
            JsonRpcMethod::Peer(PeerMethod::Connect) => Self::wrap_result(
                self.peer_handler.handle_connect(params).await,
                "Peer connect failed",
            ),

            // ── BirdSong encrypted discovery ─────────────────────────
            JsonRpcMethod::Birdsong(BirdsongMethod::GenerateEncryptedBeacon) => {
                self.birdsong_handler.handle_generate_encrypted_beacon(params).await
            }
            JsonRpcMethod::Birdsong(BirdsongMethod::DecryptBeacon) => {
                self.birdsong_handler.handle_decrypt_beacon(params).await
            }
            JsonRpcMethod::Birdsong(BirdsongMethod::VerifyLineage) => {
                self.birdsong_handler.handle_verify_lineage(params).await
            }
            JsonRpcMethod::Birdsong(BirdsongMethod::GetLineage) => {
                self.birdsong_handler.handle_get_lineage(params).await
            }
            JsonRpcMethod::Birdsong(BirdsongMethod::Advertise) => {
                self.handle_birdsong_advertise(params).await
            }
            JsonRpcMethod::Birdsong(BirdsongMethod::Schema) => {
                self.birdsong_handler.handle_schema(params).await
            }

            // ── Mesh networking ──────────────────────────────────────
            JsonRpcMethod::Mesh(MeshMethod::Init) => self.mesh_handler.handle_init(params).await,
            JsonRpcMethod::Mesh(MeshMethod::Status) => {
                self.mesh_handler.handle_status(params).await
            }
            JsonRpcMethod::Mesh(MeshMethod::FindPath) => {
                self.mesh_handler.handle_find_path(params).await
            }
            JsonRpcMethod::Mesh(MeshMethod::Announce) => {
                self.mesh_handler.handle_announce(params).await
            }
            JsonRpcMethod::Mesh(MeshMethod::Peers) => self.mesh_handler.handle_peers(params).await,
            JsonRpcMethod::Mesh(MeshMethod::HealthCheck) => {
                self.mesh_handler.handle_health_check(params).await
            }
            JsonRpcMethod::Mesh(MeshMethod::AutoDiscover) => {
                self.mesh_handler.handle_auto_discover(params).await
            }

            // ── Hole punching ────────────────────────────────────────
            JsonRpcMethod::Punch(PunchMethod::Request) => {
                self.punch_handler.handle_request(params).await
            }
            JsonRpcMethod::Punch(PunchMethod::Coordinate) => {
                self.punch_handler.handle_coordinate(params).await
            }
            JsonRpcMethod::Punch(PunchMethod::Status) => {
                self.punch_handler.handle_status(params).await
            }

            // ── Sovereign onion ──────────────────────────────────────
            JsonRpcMethod::Onion(OnionMethod::Start) => {
                self.onion_handler.handle_start(params).await
            }
            JsonRpcMethod::Onion(OnionMethod::Stop) => self.onion_handler.handle_stop(params).await,
            JsonRpcMethod::Onion(OnionMethod::Status) => {
                self.onion_handler.handle_status(params).await
            }
            JsonRpcMethod::Onion(OnionMethod::Connect) => {
                self.onion_handler.handle_connect(params).await
            }
            JsonRpcMethod::Onion(OnionMethod::Address) => {
                self.onion_handler.handle_address(params).await
            }

            // ── Federation ─────────────────────────────────────────────
            JsonRpcMethod::Federation(FederationMethod::Peers) => {
                self.handle_federation_peers_rpc().await
            }
            JsonRpcMethod::Federation(FederationMethod::Status) => {
                self.handle_federation_status_rpc().await
            }

            // ── Pure Rust Tor ────────────────────────────────────────
            JsonRpcMethod::Tor(TorMethod::Status) => self.tor_handler.handle_status(params).await,
            JsonRpcMethod::Tor(TorMethod::Connect) => self.tor_handler.handle_connect(params).await,
            JsonRpcMethod::Tor(TorMethod::ServiceStart) => {
                self.tor_handler.handle_service_start(params).await
            }
            JsonRpcMethod::Tor(TorMethod::ServiceStop) => {
                self.tor_handler.handle_service_stop(params).await
            }
            JsonRpcMethod::Tor(TorMethod::ConsensusFetch) => {
                self.tor_handler.handle_consensus_fetch(params).await
            }
            JsonRpcMethod::Tor(TorMethod::CircuitBuild) => {
                self.tor_handler.handle_circuit_build(params).await
            }
            JsonRpcMethod::Tor(TorMethod::CircuitClose) => {
                self.tor_handler.handle_circuit_close(params).await
            }

            _ => Err(format!("Unknown method: {method}")),
        }
    }
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
