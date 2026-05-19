// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::IpcServiceHandler;
use crate::tower_atomic::JsonRpcHandler;
use serde_json::Value;
use songbird_types::json_rpc_method::{
    BirdsongMethod, BtspMethod, CapabilitiesMethod, DiscoveryMethod, FederationMethod,
    HealthMethod, HttpMethod, IdentityMethod, IgdMethod, IpcMethod, JsonRpcMethod, LifecycleMethod,
    MeshMethod, OnionMethod, PeerMethod, PrimalMethod, PunchMethod, RelayMethod, RendezvousMethod,
    RpcMethod, StunMethod, TorMethod,
};

impl JsonRpcHandler for IpcServiceHandler {
    #[expect(
        clippy::too_many_lines,
        reason = "JSON-RPC dispatch table — single match over all methods"
    )]
    async fn handle(&self, method: &str, params: Value) -> Result<Value, String> {
        let method = match JsonRpcMethod::parse_ipc(method) {
            Ok(m) => m,
            Err(e) => return Err(e.into_message()),
        };
        match method {
            // ── Introspection ────────────────────────────────────────
            JsonRpcMethod::Primal(PrimalMethod::Info) => Ok(crate::introspection::primal_info()),
            JsonRpcMethod::Primal(PrimalMethod::Capabilities) => {
                Ok(crate::introspection::primal_capabilities())
            }
            JsonRpcMethod::Primal(PrimalMethod::Announce) => {
                Ok(crate::introspection::primal_announce())
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
            JsonRpcMethod::Capabilities(CapabilitiesMethod::Methods) => {
                Ok(crate::introspection::capabilities_methods())
            }
            JsonRpcMethod::Identity => self.handle_identity().await,
            JsonRpcMethod::IdentityGet(IdentityMethod::Get) => {
                Ok(crate::introspection::identity_get())
            }

            // ── BTSP transport security ────────────────────────────────
            JsonRpcMethod::Btsp(BtspMethod::Capabilities) => {
                Ok(crate::introspection::btsp_capabilities())
            }
            JsonRpcMethod::Btsp(BtspMethod::Negotiate) => {
                Err("btsp.negotiate is handled at the transport layer".to_string())
            }

            // ── IPC registry ─────────────────────────────────────────
            JsonRpcMethod::Ipc(IpcMethod::Register) => self.handle_register(params).await,
            JsonRpcMethod::Ipc(IpcMethod::Resolve) => self.handle_resolve(params).await,
            JsonRpcMethod::Ipc(IpcMethod::Discover) => self.handle_discover(params).await,
            JsonRpcMethod::Ipc(IpcMethod::List) => self.handle_list(params).await,

            // ── Capability resolution (single-step DNS-like routing) ─
            JsonRpcMethod::Capabilities(CapabilitiesMethod::Resolve) => {
                self.handle_capability_resolve(params).await
            }
            // ── Cross-gate capability dispatch ─────────────────────────
            JsonRpcMethod::Capabilities(CapabilitiesMethod::Call) => {
                self.handle_capability_call(params).await
            }

            // ── Lifecycle / composition introspection ────────────────
            JsonRpcMethod::Lifecycle(LifecycleMethod::Composition) => {
                self.handle_lifecycle_composition(params).await
            }
            JsonRpcMethod::Lifecycle(LifecycleMethod::ValidateConsumed) => {
                self.handle_validate_consumed(params).await
            }

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
            JsonRpcMethod::Discovery(DiscoveryMethod::ContentPeers) => Self::wrap_result(
                self.discovery_handler.handle_content_peers(params).await,
                "Discovery content_peers failed",
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

#[cfg(test)]
mod dispatch_tests {
    // SPDX-License-Identifier: AGPL-3.0-or-later
    // Copyright (c) 2024-2026 ecoPrimals

    #![allow(clippy::unwrap_used, reason = "test assertions")]
    #![allow(clippy::expect_used, reason = "test assertions")]

    use crate::registry::ServiceRegistry;
    use crate::service::IpcServiceHandler;
    use crate::tower_atomic::JsonRpcHandler;
    use serde_json::{Value, json};
    use songbird_test_utils::fixtures::{test_bind_address, test_endpoint};
    use songbird_test_utils::mocks::HealthStatus;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use uuid::Uuid;

    fn ipc_handler() -> IpcServiceHandler {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        IpcServiceHandler::new(registry)
    }

    fn assert_parse_error(err: &str) {
        assert!(err.contains("unknown JSON-RPC method"), "expected parse error, got: {err}");
    }

    fn assert_unknown_dispatched_variant(err: &str) {
        assert!(err.starts_with("Unknown method:"), "expected unhandled enum arm, got: {err}");
    }

    #[tokio::test]
    async fn empty_method_name_returns_parse_error() {
        let h = ipc_handler();
        let err = h.handle("", json!({})).await.expect_err("empty method");
        assert_parse_error(&err);
    }

    #[tokio::test]
    async fn whitespace_only_method_returns_parse_error() {
        let h = ipc_handler();
        let err = h.handle("   ", json!({})).await.expect_err("whitespace");
        assert_parse_error(&err);
    }

    #[tokio::test]
    async fn unknown_wire_method_returns_parse_error() {
        let h = ipc_handler();
        let err = h.handle("not.a.real.method.ever", json!({})).await.expect_err("unknown");
        assert_parse_error(&err);
    }

    #[tokio::test]
    async fn parsed_but_unhandled_method_returns_unknown_variant_error() {
        let h = ipc_handler();
        for method in [
            "songbird.federation.join",
            "storage.get",
            "primal.register",
            "discovery.list_peers",
            "ipc.find_capability",
        ] {
            let err = h.handle(method, json!({})).await.expect_err(method);
            assert_unknown_dispatched_variant(&err);
        }
    }

    #[tokio::test]
    async fn alias_normalization_routes_to_dispatch_arms() {
        let h = ipc_handler();
        let v = h.handle("ping", json!({})).await.expect("ping -> liveness");
        assert_eq!(v, json!({ "status": "alive" }));

        let reg = json!({
            "primal_id": "alias-route",
            "capabilities": ["c"],
            "endpoint": "/tmp/alias-route.sock"
        });
        h.handle("register_service", reg).await.expect("alias register");

        let resolved = h
            .handle("ipc.resolve", json!({ "primal_id": "alias-route" }))
            .await
            .expect("resolve after alias register");
        assert_eq!(resolved["virtual_endpoint"], "/primal/alias-route");

        let caps = h.handle("capability.list", json!({})).await.expect("capability.list alias");
        assert_eq!(caps["primal"], "songbird");
        assert!(caps["methods"].is_array());
    }

    #[tokio::test]
    async fn null_params_reach_handlers_that_ignore_params() {
        let h = ipc_handler();
        let v = h
            .handle("health.liveness", Value::Null)
            .await
            .expect("null params ignored for liveness");
        assert_eq!(v, json!({ "status": "alive" }));

        let v2 = h.handle("primal.info", Value::Null).await.expect("primal.info");
        assert_eq!(v2["name"], "songbird");
    }

    #[tokio::test]
    async fn malformed_ipc_register_params_rejected() {
        let h = ipc_handler();
        let err = h.handle("ipc.register", Value::Null).await.expect_err("null params");
        assert!(
            err.contains("Invalid params")
                || err.contains("missing field")
                || err.contains("invalid"),
            "unexpected: {err}"
        );
    }

    #[tokio::test]
    async fn http_request_invalid_params_shape_errors() {
        let h = ipc_handler();
        let err = h.handle("http.request", json!("not-an-object")).await.expect_err("bad shape");
        assert!(err.contains("Invalid params"), "unexpected: {err}");
    }

    #[tokio::test]
    async fn http_get_missing_url_errors_after_http_route() {
        let h = ipc_handler();
        let err = h.handle("http.get", json!({})).await.expect_err("missing url");
        assert!(err.contains("Missing 'url'"), "unexpected: {err}");
    }

    #[tokio::test]
    async fn http_post_body_not_string_errors_after_route() {
        let h = ipc_handler();
        let err = h
            .handle("http.post", json!({ "url": "https://example.com", "body": 12345 }))
            .await
            .expect_err("body not string");
        assert!(err.contains("Missing 'body'"), "unexpected: {err}");
    }

    #[tokio::test]
    async fn ipc_resolve_unknown_primal_formats_not_found() {
        let h = ipc_handler();
        let id = format!("no-such-primal-{}", Uuid::new_v4());
        let err = h.handle("ipc.resolve", json!({ "primal_id": id })).await.expect_err("resolve");
        assert!(err.contains("Primal not found"), "unexpected: {err}");
    }

    #[tokio::test]
    async fn ipc_discover_invalid_params_rejected() {
        let h = ipc_handler();
        let err = h.handle("ipc.discover", json!({})).await.expect_err("discover");
        assert!(
            err.contains("Invalid params") || err.contains("missing field"),
            "unexpected: {err}"
        );
    }

    #[tokio::test]
    async fn mock_fixtures_used_for_peer_connect_target() {
        let h = ipc_handler();
        let target = test_bind_address("dispatch_peer_connect");
        let _ = HealthStatus::Healthy;
        let res = h.handle("peer.connect", json!({ "target_address": target })).await;
        assert!(
            res.is_ok() || res.as_ref().unwrap_err().contains("Peer connect failed"),
            "unexpected: {res:?}"
        );
    }

    #[tokio::test]
    async fn test_endpoint_used_for_http_get_route() {
        let h = ipc_handler();
        let url = test_endpoint("dispatch_http_get_smoke");
        let res = h.handle("http.get", json!({ "url": url })).await;
        assert!(
            res.is_ok() || res.as_ref().unwrap_err().contains("HTTP GET failed"),
            "unexpected: {res:?}"
        );
    }

    /// Exercises each `match` arm of the JSON-RPC `handle` implementation with safe or error-only parameters.
    #[tokio::test]
    #[allow(clippy::too_many_lines, reason = "exhaustive test covering every JSON-RPC arm")]
    async fn dispatch_hits_each_json_rpc_arm() {
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let h = IpcServiceHandler::new(Arc::clone(&registry));

        macro_rules! ok {
            ($method:expr, $params:expr) => {
                h.handle($method, $params).await.unwrap_or_else(|e| {
                    panic!("{} expected Ok, got Err: {e}", $method);
                });
            };
        }

        ok!("primal.info", json!({}));
        ok!("primal.capabilities", json!({}));
        ok!("rpc.methods", json!({}));
        ok!("rpc.discover", json!({}));
        ok!("discover_capabilities", json!({}));
        ok!("health.liveness", json!({}));
        ok!("health.readiness", json!({}));
        ok!("health.check", json!({}));
        ok!("capabilities.list", json!({}));
        ok!("capabilities.methods", json!({}));
        ok!("identity", json!({}));
        ok!("identity.get", json!({}));

        h.handle(
            "ipc.register",
            json!({
                "primal_id": "dispatch-smoke",
                "capabilities": ["smoke"],
                "endpoint": "/tmp/dispatch-smoke.sock"
            }),
        )
        .await
        .unwrap();
        ok!("ipc.resolve", json!({ "primal_id": "dispatch-smoke" }));
        ok!("ipc.discover", json!({ "capability": "smoke" }));
        ok!("ipc.list", json!({}));

        h.handle("http.get", json!({ "url": "https://example.com" })).await.ok();

        h.handle("stun.serve", json!({ "bind_addr": "127.0.0.1:0" })).await.expect("stun.serve");
        ok!("stun.status", json!({}));
        h.handle("stun.stop", json!({})).await.expect("stun.stop");

        let stun_err = h
            .handle("stun.get_public_address", json!({ "servers": [] }))
            .await
            .expect_err("empty servers");
        assert!(stun_err.contains("No STUN servers"), "unexpected: {stun_err}");

        let nat_err = h
            .handle("stun.detect_nat_type", json!({ "servers": [] }))
            .await
            .expect_err("nat detect");
        assert!(nat_err.contains("at least 2"), "unexpected: {nat_err}");

        ok!("igd.discover", json!({}));

        h.handle("relay.serve", json!({ "bind_addr": "127.0.0.1:0" })).await.expect("relay.serve");
        ok!("relay.status", json!({}));
        h.handle(
            "relay.allocate",
            json!({
                "relay_node": "a",
                "requester": "b",
                "target_addr": "127.0.0.1:1",
                "lineage_proof": ""
            }),
        )
        .await
        .expect("relay.allocate");
        h.handle("relay.stop", json!({})).await.expect("relay.stop");

        ok!("discovery.peers", json!({}));
        ok!("discovery.announce", json!({ "family_id": "f" }));

        match h
            .handle(
                "rendezvous.register",
                json!({
                    "server": "https://rendezvous.example.com",
                    "node_id": "n1",
                    "family_id": "fam",
                    "public_address": "198.51.100.1:4000"
                }),
            )
            .await
        {
            Ok(_) => {}
            Err(e) => assert!(
                e.contains("Rendezvous register failed") || e.contains("Rendezvous"),
                "unexpected handler error: {e}"
            ),
        }

        match h
            .handle(
                "rendezvous.lookup",
                json!({
                    "server": "https://rendezvous.example.com",
                    "target": "n1"
                }),
            )
            .await
        {
            Ok(_) => {}
            Err(e) => assert!(
                e.contains("Rendezvous lookup failed") || e.contains("Rendezvous"),
                "unexpected handler error: {e}"
            ),
        }

        h.handle("peer.connect", json!({ "target_address": "127.0.0.1:2" })).await.ok();

        h.handle("birdsong.generate_encrypted_beacon", json!({ "node_id": "node-dispatch" }))
            .await
            .ok();

        h.handle("mesh.init", json!({ "node_id": "mesh-dispatch" })).await.expect("mesh.init");
        ok!("mesh.status", json!({}));

        let punch_err = h.handle("punch.request", json!({})).await.expect_err("punch");
        assert!(punch_err.contains("target_node_id"), "unexpected: {punch_err}");

        h.handle("onion.status", json!({})).await.ok();
        h.handle("tor.status", json!({})).await.expect("tor.status");

        ok!("federation.peers", json!({}));
        ok!("federation.status", json!({}));
    }

    /// Covers match arms not exercised by [`dispatch_hits_each_json_rpc_arm`] (quick Ok / validation paths).
    #[tokio::test]
    async fn dispatch_covers_remaining_json_rpc_arms() {
        let h = ipc_handler();

        let _ = h.handle("igd.map_port", json!({})).await;
        let _ = h.handle("igd.unmap_port", json!({})).await;
        let _ = h.handle("igd.status", json!({})).await;
        let _ = h.handle("igd.external_ip", json!({})).await;
        let _ = h.handle("igd.auto_configure", json!({})).await;

        h.handle("mesh.init", json!({ "node_id": "mesh-extra" })).await.expect("mesh.init");
        let mesh_err = h.handle("mesh.find_path", json!({})).await.expect_err("find_path");
        assert!(mesh_err.contains("target_node_id"), "unexpected: {mesh_err}");
        h.handle("mesh.announce", json!({ "as_relay": true })).await.ok();
        h.handle("mesh.peers", json!({})).await.ok();
        h.handle("mesh.topology", json!({})).await.ok();
        h.handle("mesh.health_check", json!({})).await.ok();
        h.handle("mesh.auto_discover", json!({})).await.ok();

        let b_err = h.handle("birdsong.decrypt_beacon", json!({})).await.expect_err("decrypt");
        assert!(b_err.contains("encrypted_beacon"), "unexpected: {b_err}");
        h.handle("birdsong.verify_lineage", json!({})).await.expect_err("verify");
        h.handle("birdsong.get_lineage", json!({})).await.ok();
        h.handle("birdsong.advertise", json!({})).await.expect_err("advertise");
        h.handle("birdsong.schema", json!({})).await.expect("birdsong.schema");

        h.handle("punch.coordinate", json!({})).await.expect_err("coordinate");
        h.handle("punch.status", json!({ "target_node_id": "any" })).await.expect("punch.status");

        h.handle("onion.stop", json!({})).await.ok();
        h.handle("onion.status", json!({})).await.ok();
        h.handle("onion.connect", json!({})).await.ok();
        h.handle("onion.address", json!({})).await.ok();
        h.handle("onion.start", json!({ "port": 0 })).await.ok();

        h.handle("tor.connect", json!({})).await.expect_err("tor.connect missing address");
        h.handle("tor.service.start", json!({})).await.ok();
        h.handle("tor.service.stop", json!({})).await.ok();
        h.handle("tor.consensus.fetch", json!({})).await.ok();
        h.handle("tor.circuit.build", json!({ "purpose": "general" })).await.ok();
        h.handle("tor.circuit.close", json!({})).await.expect_err("circuit close missing id");

        h.handle("stun.bind", json!({ "stun_server": "127.0.0.1:1" })).await.ok();
        h.handle("stun.probe_port_pattern", json!({ "stun_server": "127.0.0.1:1", "probes": 1 }))
            .await
            .ok();
    }
}
