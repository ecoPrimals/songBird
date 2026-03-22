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
use serde::{Deserialize, Serialize};
use serde_json::Value;
use songbird_lineage_relay::beardog::BearDogRelayAuthority; // Production relay auth (Feb 8, 2026)
use songbird_lineage_relay::relay_handler::RelayHandler; // Relay Server (Feb 5, 2026)
use std::env::VarError;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Injectable environment reader (for concurrent-safe tests without global mutation).
type EnvReader = dyn Fn(&str) -> Result<String, VarError> + Send + Sync;

/// IPC service request parameters for registration
#[derive(Debug, Clone, Deserialize)]
pub struct RegisterParams {
    pub primal_id: String,
    pub capabilities: Vec<String>,
    pub endpoint: String,
}

/// IPC service request parameters for resolution
#[derive(Debug, Clone, Deserialize)]
pub struct ResolveParams {
    pub primal_id: String,
}

/// IPC service request parameters for discovery
#[derive(Debug, Clone, Deserialize)]
pub struct DiscoverParams {
    pub capability: String,
}

/// IPC service response for registration
#[derive(Debug, Clone, Serialize)]
pub struct RegisterResult {
    pub virtual_endpoint: String,
    pub registered_at: String,
}

/// IPC service response for resolution
#[derive(Debug, Clone, Serialize)]
pub struct ResolveResult {
    pub virtual_endpoint: String,
    pub native_endpoint: String,
    pub capabilities: Vec<String>,
}

/// IPC service response for discovery
#[derive(Debug, Clone, Serialize)]
pub struct DiscoverResult {
    pub providers: Vec<ProviderInfo>,
}

/// Provider information
#[derive(Debug, Clone, Serialize)]
pub struct ProviderInfo {
    pub primal_id: String,
    pub virtual_endpoint: String,
    pub native_endpoint: String,
    pub capabilities: Vec<String>,
}

/// IPC service response for listing
#[derive(Debug, Clone, Serialize)]
pub struct ListResult {
    pub services: Vec<ServiceInfo>,
}

/// Service information
#[derive(Debug, Clone, Serialize)]
pub struct ServiceInfo {
    pub primal_id: String,
    pub virtual_endpoint: String,
    pub capabilities: Vec<String>,
}

/// Songbird IPC Service Handler
///
/// This handler provides IPC brokering as a JSON-RPC service,
/// allowing other primals to discover and connect to services
/// without embedding Songbird code.
///
/// **TRUE PRIMAL**: Zero code embedding, pure service protocol!
pub struct IpcServiceHandler {
    registry: Arc<RwLock<ServiceRegistry>>,
    /// When set, used instead of [`std::env::var`] for identity `family_id` resolution (tests).
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
        let node_id = std::env::var("SONGBIRD_NODE_ID")
            .or_else(|_| std::env::var("NODE_ID"))
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
        )
    }

    /// Create with a custom HTTP handler (for dependency injection).
    pub fn with_http_handler(
        registry: Arc<RwLock<ServiceRegistry>>,
        http_handler: Arc<HttpHandler>,
    ) -> Self {
        Self::assemble(registry, http_handler, Arc::new(DiscoveryHandler::new()))
    }

    /// Same as [`new`](Self::new) but resolves `family_id` for `identity` via `env` instead of
    /// [`std::env::var`] (for tests and injected configuration).
    #[must_use]
    pub fn with_family_id_env<F>(registry: Arc<RwLock<ServiceRegistry>>, env: F) -> Self
    where
        F: Fn(&str) -> Result<String, VarError> + Send + Sync + 'static,
    {
        let mut h = Self::new(registry);
        h.family_id_env = Some(Arc::new(env));
        h
    }

    /// Handle `ipc.register` method
    async fn handle_register(&self, params: Value) -> Result<Value, String> {
        let params: RegisterParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        info!("Registering primal: {} at {}", params.primal_id, params.endpoint);

        // Parse native endpoint
        let native_endpoint = if params.endpoint.starts_with('/') {
            // Unix socket path
            NativeEndpoint::UnixSocket(params.endpoint.into())
        } else if params.endpoint.starts_with("127.0.0.1:") || params.endpoint.contains(':') {
            // TCP localhost
            let port: u16 = params
                .endpoint
                .split(':')
                .nth(1)
                .and_then(|p| p.parse().ok())
                .ok_or_else(|| "Invalid TCP port".to_string())?;
            NativeEndpoint::TcpLocal(port)
        } else {
            return Err(format!("Invalid endpoint format: {}", params.endpoint));
        };

        // Register in registry
        let registry = self.registry.write().await;
        let virtual_endpoint = registry
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
        let registry = self.registry.read().await;
        let entry = registry
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
        let registry = self.registry.read().await;
        let services = registry.list_services().await;
        Ok(crate::introspection::health(uptime_secs, services.len()))
    }

    /// Handle `identity` method — canonical family-id lookup
    async fn handle_identity(&self) -> Result<Value, String> {
        let family_id = match &self.family_id_env {
            Some(f) => crate::introspection::canonical_family_id(|k| (f)(k)),
            None => crate::introspection::canonical_family_id(|k| std::env::var(k)),
        };
        Ok(crate::introspection::identity(&family_id))
    }
}

#[async_trait]
impl JsonRpcHandler for IpcServiceHandler {
    async fn handle(&self, method: &str, params: Value) -> Result<Value, String> {
        match method {
            // ── Introspection ────────────────────────────────────────
            "primal.info" => Ok(crate::introspection::primal_info()),
            "primal.capabilities" => Ok(crate::introspection::primal_capabilities()),
            "rpc.methods" => Ok(crate::introspection::rpc_methods()),
            "rpc.discover" => Ok(crate::introspection::rpc_discover_standard()),
            "discover_capabilities" => Ok(crate::introspection::discover_capabilities()),

            // ── biomeOS standard ─────────────────────────────────────
            "health.liveness" => Ok(crate::introspection::health_liveness()),
            "health" | "health.readiness" | "health.check" => self.handle_health().await,
            "capabilities.list" => Ok(crate::introspection::capabilities_list()),
            "identity" => self.handle_identity().await,

            // ── IPC registry ─────────────────────────────────────────
            "ipc.register" => self.handle_register(params).await,
            "ipc.resolve" => self.handle_resolve(params).await,
            "ipc.discover" => self.handle_discover(params).await,
            "ipc.list" => self.handle_list(params).await,

            // ── HTTP/HTTPS ───────────────────────────────────────────
            "http.request" => self.handle_http_request(params).await,
            "http.get" => self.handle_http_get(params).await,
            "http.post" => self.handle_http_post(params).await,

            // ── STUN / NAT traversal ─────────────────────────────────
            "stun.serve" => self.stun_handler.handle_serve(params).await,
            "stun.stop" => self.stun_handler.handle_stop(params).await,
            "stun.status" => self.stun_handler.handle_status(params).await,
            "stun.get_public_address" => self.stun_handler.handle_get_public_address(params).await,
            "stun.bind" => self.stun_handler.handle_bind(params).await,
            "stun.probe_port_pattern" => self.stun_handler.handle_probe_port_pattern(params).await,
            "stun.detect_nat_type" => self.stun_handler.handle_detect_nat_type(params).await,

            // ── IGD router auto-configuration ────────────────────────
            "igd.discover" => Ok(self.igd_handler.handle_discover(params).await),
            "igd.map_port" => Ok(self.igd_handler.handle_map_port(params).await),
            "igd.unmap_port" => Ok(self.igd_handler.handle_unmap_port(params).await),
            "igd.status" => Ok(self.igd_handler.handle_status(params).await),
            "igd.external_ip" => Ok(self.igd_handler.handle_external_ip(params).await),
            "igd.auto_configure" => Ok(self.igd_handler.handle_auto_configure(params).await),

            // ── Relay server ─────────────────────────────────────────
            "relay.serve" => self.relay_handler.handle_serve(params).await,
            "relay.stop" => self.relay_handler.handle_stop(params).await,
            "relay.status" => self.relay_handler.handle_status(params).await,
            "relay.allocate" => self.relay_handler.handle_allocate(params).await,

            // ── Discovery / rendezvous / peers ───────────────────────
            "discovery.peers" | "discovery.find_primals" | "find_primals" => Self::wrap_result(
                self.discovery_handler.handle_list_peers(params).await,
                "Discovery peers failed",
            ),
            "discovery.announce" | "announce_presence" => Self::wrap_result(
                self.discovery_handler.handle_announce(params).await,
                "Discovery announce failed",
            ),
            "rendezvous.register" => Self::wrap_result(
                self.rendezvous_handler.handle_register(params).await,
                "Rendezvous register failed",
            ),
            "rendezvous.lookup" => Self::wrap_result(
                self.rendezvous_handler.handle_lookup(params).await,
                "Rendezvous lookup failed",
            ),
            "peer.connect" => Self::wrap_result(
                self.peer_handler.handle_connect(params).await,
                "Peer connect failed",
            ),

            // ── BirdSong encrypted discovery ─────────────────────────
            "birdsong.generate_encrypted_beacon" => {
                self.birdsong_handler.handle_generate_encrypted_beacon(params).await
            }
            "birdsong.decrypt_beacon" => self.birdsong_handler.handle_decrypt_beacon(params).await,
            "birdsong.verify_lineage" => self.birdsong_handler.handle_verify_lineage(params).await,
            "birdsong.get_lineage" => self.birdsong_handler.handle_get_lineage(params).await,
            "birdsong.advertise" => self.handle_birdsong_advertise(params).await,

            // ── Mesh networking ──────────────────────────────────────
            "mesh.init" => self.mesh_handler.handle_init(params).await,
            "mesh.status" => self.mesh_handler.handle_status(params).await,
            "mesh.find_path" => self.mesh_handler.handle_find_path(params).await,
            "mesh.announce" => self.mesh_handler.handle_announce(params).await,
            "mesh.peers" => self.mesh_handler.handle_peers(params).await,
            "mesh.health_check" => self.mesh_handler.handle_health_check(params).await,
            "mesh.auto_discover" => self.mesh_handler.handle_auto_discover(params).await,

            // ── Hole punching ────────────────────────────────────────
            "punch.request" => self.punch_handler.handle_request(params).await,
            "punch.coordinate" => self.punch_handler.handle_coordinate(params).await,
            "punch.status" => self.punch_handler.handle_status(params).await,

            // ── Sovereign onion ──────────────────────────────────────
            "onion.start" => self.onion_handler.handle_start(params).await,
            "onion.stop" => self.onion_handler.handle_stop(params).await,
            "onion.status" => self.onion_handler.handle_status(params).await,
            "onion.connect" => self.onion_handler.handle_connect(params).await,
            "onion.address" => self.onion_handler.handle_address(params).await,

            // ── Federation ─────────────────────────────────────────────
            "songbird.federation.peers" | "federation.peers" => Ok(serde_json::json!({
                "peers": [],
                "total_count": 0,
                "federation_enabled": false,
                "comment": "Federation subsystem available, no active peers"
            })),
            "songbird.federation.status" | "federation.status" => Ok(serde_json::json!({
                "enabled": false,
                "active_connections": 0,
                "comment": "Federation subsystem initialized, awaiting peer connections"
            })),

            // ── Pure Rust Tor ────────────────────────────────────────
            "tor.status" => self.tor_handler.handle_status(params).await,
            "tor.connect" => self.tor_handler.handle_connect(params).await,
            "tor.service.start" => self.tor_handler.handle_service_start(params).await,
            "tor.service.stop" => self.tor_handler.handle_service_stop(params).await,
            "tor.consensus.fetch" => self.tor_handler.handle_consensus_fetch(params).await,
            "tor.circuit.build" => self.tor_handler.handle_circuit_build(params).await,
            "tor.circuit.close" => self.tor_handler.handle_circuit_close(params).await,

            _ => Err(format!("Unknown method: {method}")),
        }
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions")]
#[expect(clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_ipc_service_register() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let handler = IpcServiceHandler::new(registry.clone());

        let params = json!({
            "primal_id": "beardog",
            "capabilities": ["crypto", "btsp"],
            "endpoint": "/tmp/primal-beardog.sock"
        });

        let result = handler.handle("ipc.register", params).await;
        assert!(result.is_ok());

        let result_value = result.unwrap();
        assert_eq!(result_value["virtual_endpoint"], "/primal/beardog");
    }

    #[tokio::test]
    async fn test_ipc_service_resolve() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let handler = IpcServiceHandler::new(registry.clone());

        // Register first
        let register_params = json!({
            "primal_id": "beardog",
            "capabilities": ["crypto"],
            "endpoint": "/tmp/primal-beardog.sock"
        });
        handler.handle("ipc.register", register_params).await.unwrap();

        // Then resolve
        let resolve_params = json!({
            "primal_id": "beardog"
        });

        let result = handler.handle("ipc.resolve", resolve_params).await;
        assert!(result.is_ok());

        let result_value = result.unwrap();
        assert_eq!(result_value["virtual_endpoint"], "/primal/beardog");
        assert!(result_value["native_endpoint"].as_str().unwrap().contains("beardog"));
    }

    #[tokio::test]
    async fn test_ipc_service_discover() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let handler = IpcServiceHandler::new(registry.clone());

        // Register service with capability
        let register_params = json!({
            "primal_id": "beardog",
            "capabilities": ["crypto", "btsp"],
            "endpoint": "/tmp/primal-beardog.sock"
        });
        handler.handle("ipc.register", register_params).await.unwrap();

        // Discover by capability
        let discover_params = json!({
            "capability": "crypto"
        });

        let result = handler.handle("ipc.discover", discover_params).await;
        assert!(result.is_ok());

        let result_value = result.unwrap();
        let providers = result_value["providers"].as_array().unwrap();
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0]["primal_id"], "beardog");
    }

    #[tokio::test]
    async fn test_primal_info_introspection() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let handler = IpcServiceHandler::new(registry.clone());

        let result = handler.handle("primal.info", json!({})).await;
        assert!(result.is_ok());

        let info = result.unwrap();
        assert_eq!(info["name"], "songbird");
        assert!(info["version"].is_string());
        assert!(info["capabilities"].is_array());
        assert!(info["capabilities"].as_array().unwrap().contains(&json!("discovery")));
        assert!(info["capabilities"].as_array().unwrap().contains(&json!("stun")));
    }

    #[tokio::test]
    async fn test_primal_capabilities_introspection() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let handler = IpcServiceHandler::new(registry.clone());

        let result = handler.handle("primal.capabilities", json!({})).await;
        assert!(result.is_ok());

        let caps = result.unwrap();
        assert!(caps["capabilities"].is_array());

        let caps_array = caps["capabilities"].as_array().unwrap();
        assert!(!caps_array.is_empty());

        // Verify discovery capability exists with operations
        let discovery_cap = caps_array
            .iter()
            .find(|c| c["name"] == "discovery")
            .expect("discovery capability should exist");

        assert!(discovery_cap["operations"].is_array());
        assert!(discovery_cap["description"].is_string());
    }

    #[tokio::test]
    async fn test_rpc_methods_introspection() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let handler = IpcServiceHandler::new(registry.clone());

        let result = handler.handle("rpc.methods", json!({})).await;
        assert!(result.is_ok());

        let methods = result.unwrap();
        assert!(methods["methods"].is_array());

        let methods_array = methods["methods"].as_array().unwrap();
        assert!(!methods_array.is_empty());

        // Verify introspection methods are listed
        let method_names: Vec<String> =
            methods_array.iter().filter_map(|m| m["name"].as_str()).map(String::from).collect();

        assert!(method_names.contains(&"primal.info".to_string()));
        assert!(method_names.contains(&"primal.capabilities".to_string()));
        assert!(method_names.contains(&"rpc.methods".to_string()));
        assert!(method_names.contains(&"ipc.register".to_string()));
    }

    #[tokio::test]
    async fn test_ipc_service_list() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let handler = IpcServiceHandler::new(registry.clone());

        // Register multiple services
        for (id, caps) in &[("beardog", vec!["crypto"]), ("squirrel", vec!["ai"])] {
            let params = json!({
                "primal_id": id,
                "capabilities": caps,
                "endpoint": format!("/tmp/primal-{id}.sock")
            });
            handler.handle("ipc.register", params).await.unwrap();
        }

        // List all
        let result = handler.handle("ipc.list", json!({})).await;
        assert!(result.is_ok());

        let result_value = result.unwrap();
        let services = result_value["services"].as_array().unwrap();
        assert_eq!(services.len(), 2);
    }

    #[tokio::test]
    async fn test_discover_capabilities() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let handler = IpcServiceHandler::new(registry.clone());

        let result = handler.handle("discover_capabilities", json!({})).await;
        assert!(result.is_ok());

        let caps = result.unwrap();
        assert_eq!(caps["primal"], "songbird");

        let capabilities = caps["capabilities"].as_array().unwrap();
        assert!(!capabilities.is_empty());

        // Verify key capabilities that other primals scan for
        let cap_strs: Vec<&str> = capabilities.iter().filter_map(|c| c.as_str()).collect();
        assert!(cap_strs.contains(&"http.request"), "must advertise http.request");
        assert!(cap_strs.contains(&"secure_http"), "must advertise secure_http");
        assert!(cap_strs.contains(&"discovery.peers"), "must advertise discovery.peers");
        assert!(cap_strs.contains(&"stun.detect"), "must advertise stun capability");
        assert!(cap_strs.contains(&"mesh.status"), "must advertise mesh capability");
        assert!(cap_strs.contains(&"punch.request"), "must advertise punch capability");
    }

    #[test]
    fn ipc_register_params_deserialize_roundtrip() {
        let v = json!({
            "primal_id": "beardog",
            "capabilities": ["crypto"],
            "endpoint": "/tmp/x.sock"
        });
        let p: RegisterParams = serde_json::from_value(v).expect("RegisterParams");
        assert_eq!(p.primal_id, "beardog");
        assert_eq!(p.capabilities, vec!["crypto".to_string()]);
        assert_eq!(p.endpoint, "/tmp/x.sock");
    }

    #[test]
    fn ipc_resolve_and_discover_params_deserialize() {
        let r: ResolveParams = serde_json::from_value(json!({"primal_id": "a"})).expect("resolve");
        assert_eq!(r.primal_id, "a");

        let d: DiscoverParams =
            serde_json::from_value(json!({"capability": "stun"})).expect("disc");
        assert_eq!(d.capability, "stun");
    }

    #[test]
    fn ipc_list_and_provider_serialization_shapes() {
        let list = ListResult {
            services: vec![ServiceInfo {
                primal_id: "p".into(),
                virtual_endpoint: "/primal/p".into(),
                capabilities: vec!["c".into()],
            }],
        };
        let v = serde_json::to_value(&list).expect("list json");
        assert_eq!(v["services"][0]["primal_id"], "p");

        let dr = DiscoverResult {
            providers: vec![ProviderInfo {
                primal_id: "q".into(),
                virtual_endpoint: "/primal/q".into(),
                native_endpoint: "unix:///run/q".into(),
                capabilities: vec![],
            }],
        };
        let v2 = serde_json::to_value(&dr).expect("discover result");
        assert_eq!(v2["providers"][0]["native_endpoint"], "unix:///run/q");
    }

    #[test]
    fn register_and_resolve_result_serialization() {
        let reg = RegisterResult {
            virtual_endpoint: "/primal/x".into(),
            registered_at: "t0".into(),
        };
        let v = serde_json::to_value(&reg).expect("RegisterResult json");
        assert_eq!(v["virtual_endpoint"], "/primal/x");

        let res = ResolveResult {
            virtual_endpoint: "/primal/x".into(),
            native_endpoint: "native".into(),
            capabilities: vec!["c".into()],
        };
        let v2 = serde_json::to_value(&res).expect("ResolveResult json");
        assert_eq!(v2["capabilities"][0], "c");
    }

    #[tokio::test]
    async fn ipc_resolve_errors_when_primal_not_registered() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let handler = IpcServiceHandler::new(registry.clone());
        let err = handler
            .handle("ipc.resolve", json!({ "primal_id": "ghost" }))
            .await
            .expect_err("not registered");
        assert!(err.contains("not found") || err.contains("Not found") || err.contains("found"));
    }

    #[tokio::test]
    async fn unknown_rpc_method_returns_error() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let handler = IpcServiceHandler::new(registry.clone());
        let err = handler.handle("no.such.method", json!({})).await.expect_err("unknown method");
        assert!(err.contains("Unknown method"));
    }

    #[tokio::test]
    async fn health_liveness_returns_healthy_status_only() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let handler = IpcServiceHandler::new(registry.clone());
        let v = handler.handle("health.liveness", json!({})).await.expect("liveness");
        assert_eq!(v, json!({ "status": "healthy" }));
    }

    #[tokio::test]
    async fn capabilities_list_returns_expected_tokens() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let handler = IpcServiceHandler::new(registry.clone());
        let v = handler.handle("capabilities.list", json!({})).await.expect("caps");
        let arr = v.as_array().expect("capabilities.list must return a JSON array");
        let strings: Vec<&str> = arr.iter().filter_map(|x| x.as_str()).collect();
        for expected in crate::introspection::SONGBIRD_CAPABILITY_STRINGS {
            assert!(strings.contains(expected), "missing capability token {expected}");
        }
        assert_eq!(strings.len(), crate::introspection::SONGBIRD_CAPABILITY_STRINGS.len());
    }
}
