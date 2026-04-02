// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use super::IpcServiceHandler;
use crate::tower_atomic::JsonRpcHandler;
use serde_json::Value;
use songbird_types::json_rpc_method::{
    BirdsongMethod, CapabilitiesMethod, DiscoveryMethod, FederationMethod, HealthMethod,
    HttpMethod, IgdMethod, IpcMethod, JsonRpcMethod, MeshMethod, OnionMethod, PeerMethod,
    PrimalMethod, PunchMethod, RelayMethod, RendezvousMethod, RpcMethod, StunMethod, TorMethod,
};

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
            JsonRpcMethod::Mesh(MeshMethod::Topology) => {
                self.mesh_handler.handle_topology(params).await
            }
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
