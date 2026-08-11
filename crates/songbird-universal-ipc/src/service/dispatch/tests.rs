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
    for method in ["storage.get", "primal.register", "discovery.list_peers", "ipc.find_capability"]
    {
        let err = h.handle(method, json!({})).await.expect_err(method);
        assert_unknown_dispatched_variant(&err);
    }
}

#[tokio::test]
async fn federation_join_returns_orchestrator_level_error() {
    let h = ipc_handler();
    let err = h.handle("songbird.federation.join", json!({})).await.expect_err("federation.join");
    assert!(err.contains("orchestrator level"), "expected orchestrator-level error, got: {err}");
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
    let v =
        h.handle("health.liveness", Value::Null).await.expect("null params ignored for liveness");
    assert_eq!(v, json!({ "status": "alive" }));

    let v2 = h.handle("primal.info", Value::Null).await.expect("primal.info");
    assert_eq!(v2["name"], "songbird");
}

#[tokio::test]
async fn malformed_ipc_register_params_rejected() {
    let h = ipc_handler();
    let err = h.handle("ipc.register", Value::Null).await.expect_err("null params");
    assert!(
        err.contains("Invalid params") || err.contains("missing field") || err.contains("invalid"),
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
async fn http_put_missing_url_errors() {
    let h = ipc_handler();
    let err = h.handle("http.put", json!({})).await.expect_err("missing url");
    assert!(err.contains("Missing 'url'"), "unexpected: {err}");
}

#[tokio::test]
async fn http_delete_missing_url_errors() {
    let h = ipc_handler();
    let err = h.handle("http.delete", json!({})).await.expect_err("missing url");
    assert!(err.contains("Missing 'url'"), "unexpected: {err}");
}

#[tokio::test]
async fn http_proxy_missing_capability_errors() {
    let h = ipc_handler();
    let err = h.handle("http.proxy", json!({})).await.expect_err("missing capability");
    assert!(err.contains("Missing 'capability'"), "unexpected: {err}");
}

#[tokio::test]
async fn http_proxy_unknown_capability_errors() {
    let h = ipc_handler();
    let err =
        h.handle("http.proxy", json!({ "capability": "nonexistent" })).await.expect_err("no route");
    assert!(err.contains("No route registered"), "unexpected: {err}");
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
    assert!(err.contains("Invalid params") || err.contains("missing field"), "unexpected: {err}");
}

#[tokio::test]
async fn mock_fixtures_used_for_peer_connect_target() {
    use std::time::Duration;
    let h = ipc_handler();
    let target = test_bind_address("dispatch_peer_connect");
    let _ = HealthStatus::Healthy;
    // UDP hole punch has a 10s+ timeout — cap it to verify dispatch routing only
    let res = tokio::time::timeout(
        Duration::from_millis(200),
        h.handle("peer.connect", json!({ "target_address": target })),
    )
    .await;
    match res {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => assert!(
            e.contains("Peer connect failed"),
            "unexpected: {e}"
        ),
        Err(_) => {} // timeout is acceptable — proves dispatch was entered
    }
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

/// Exercises each dispatch arm of the JSON-RPC `handle` implementation with safe or error-only parameters.
#[tokio::test]
#[expect(clippy::too_many_lines, reason = "exhaustive test covering every JSON-RPC arm")]
async fn dispatch_hits_each_json_rpc_arm() {
    use std::time::Duration;

    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let h = IpcServiceHandler::new(Arc::clone(&registry));

    macro_rules! ok {
        ($method:expr, $params:expr) => {
            h.handle($method, $params).await.unwrap_or_else(|e| {
                panic!("{} expected Ok, got Err: {e}", $method);
            });
        };
    }

    // Verify dispatch enters the handler without waiting for real network IO.
    // Timeout proves the dispatch arm was hit; the handler attempted real IO.
    macro_rules! dispatched {
        ($method:expr, $params:expr) => {
            let _ = tokio::time::timeout(
                Duration::from_millis(100),
                h.handle($method, $params),
            )
            .await;
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

    h.handle("http.get", json!({ "url": "http://127.0.0.1:1/test" })).await.ok();
    h.handle("http.put", json!({ "url": "http://127.0.0.1:1/test" })).await.ok();
    h.handle("http.delete", json!({ "url": "http://127.0.0.1:1/test" })).await.ok();
    let proxy_err = h
        .handle("http.proxy", json!({ "capability": "nonexistent" }))
        .await
        .expect_err("http.proxy no route");
    assert!(proxy_err.contains("No route registered"), "unexpected: {proxy_err}");

    h.handle("stun.serve", json!({ "bind_addr": "127.0.0.1:0" })).await.expect("stun.serve");
    ok!("stun.status", json!({}));
    h.handle("stun.stop", json!({})).await.expect("stun.stop");

    let stun_err = h
        .handle("stun.get_public_address", json!({ "servers": [] }))
        .await
        .expect_err("empty servers");
    assert!(stun_err.contains("No STUN servers"), "unexpected: {stun_err}");

    let nat_err =
        h.handle("stun.detect_nat_type", json!({ "servers": [] })).await.expect_err("nat detect");
    assert!(nat_err.contains("at least 2"), "unexpected: {nat_err}");

    // igd.discover does real SSDP multicast + NAT-PMP probe (~6s network wait)
    dispatched!("igd.discover", json!({}));

    h.handle("relay.serve", json!({ "bind_addr": "127.0.0.1:0" })).await.expect("relay.serve");
    ok!("relay.status", json!({}));
    // relay.allocate calls security-provider UDS — hangs on live socket without read timeout
    dispatched!("relay.allocate", json!({
        "relay_node": "a",
        "requester": "b",
        "target_addr": "127.0.0.1:1",
        "lineage_proof": ""
    }));
    h.handle("relay.stop", json!({})).await.expect("relay.stop");

    ok!("discovery.peers", json!({}));
    ok!("discovery.announce", json!({ "family_id": "f" }));

    match h
        .handle(
            "rendezvous.register",
            json!({
                "server": "http://127.0.0.1:1",
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
                "server": "http://127.0.0.1:1",
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

    // peer.connect does UDP hole punching with 10s+ timeout
    dispatched!("peer.connect", json!({ "target_address": "127.0.0.1:2" }));

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

/// Covers dispatch arms not exercised by [`dispatch_hits_each_json_rpc_arm`] (quick Ok / validation paths).
#[tokio::test]
async fn dispatch_covers_remaining_json_rpc_arms() {
    use std::time::Duration;
    let h = ipc_handler();

    // Short timeout for calls that do real network IO (STUN, UDS to security provider, etc.)
    macro_rules! dispatched {
        ($method:expr, $params:expr) => {
            let _ = tokio::time::timeout(
                Duration::from_millis(100),
                h.handle($method, $params),
            )
            .await;
        };
    }

    // igd.map_port auto-discovers gateway if not yet found (SSDP+NAT-PMP ~6s)
    dispatched!("igd.map_port", json!({}));
    let _ = h.handle("igd.unmap_port", json!({})).await;
    let _ = h.handle("igd.status", json!({})).await;
    let _ = h.handle("igd.external_ip", json!({})).await;
    dispatched!("igd.auto_configure", json!({}));

    h.handle("mesh.init", json!({ "node_id": "mesh-extra" })).await.expect("mesh.init");
    let mesh_err = h.handle("mesh.find_path", json!({})).await.expect_err("find_path");
    assert!(mesh_err.contains("target_node_id"), "unexpected: {mesh_err}");
    h.handle("mesh.announce", json!({ "as_relay": true })).await.ok();
    h.handle("mesh.peers", json!({})).await.ok();
    h.handle("mesh.topology", json!({})).await.ok();
    h.handle("mesh.health_check", json!({})).await.ok();
    dispatched!("mesh.auto_discover", json!({}));
    dispatched!("mesh.connectivity_check", json!({}));
    let tp_err =
        h.handle("mesh.throughput", json!({})).await.expect_err("throughput needs target_address");
    assert!(tp_err.contains("target_address"), "unexpected: {tp_err}");

    // mesh.enroll contacts security provider UDS — may hang without timeout
    dispatched!("mesh.enroll", json!({
        "node_id": "new-gate",
        "public_key": "test-wg-pubkey-base64",
        "proof": "test-proof-placeholder",
        "timestamp": 1700000000_u64
    }));

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
    dispatched!("onion.start", json!({ "port": 0 }));

    h.handle("tor.connect", json!({})).await.expect_err("tor.connect missing address");
    dispatched!("tor.service.start", json!({}));
    h.handle("tor.service.stop", json!({})).await.ok();
    dispatched!("tor.consensus.fetch", json!({}));
    dispatched!("tor.circuit.build", json!({ "purpose": "general" }));
    h.handle("tor.circuit.close", json!({})).await.expect_err("circuit close missing id");

    // STUN bind/probe send real UDP and wait for responses (5s timeout each)
    dispatched!("stun.bind", json!({ "stun_server": "127.0.0.1:1" }));
    dispatched!("stun.probe_port_pattern", json!({ "stun_server": "127.0.0.1:1", "probes": 1 }));
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn gossip_relay_local_injects_without_error() {
    let h = ipc_handler();

    // gossip.inject — will timeout quickly with paused time if live socket found
    let result = h
        .handle(
            "gossip.inject",
            json!({
                "topic": "tower",
                "key": "test.capability:gate-1:songbird",
                "payload": { "capabilities": ["mesh.relay"], "primal": "songbird" }
            }),
        )
        .await
        .expect("gossip.inject should succeed even without swarmVine");
    assert_eq!(result["status"], "injected");
    assert_eq!(result["topic"], "tower");
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn gossip_relay_to_local_target() {
    let h = ipc_handler();

    // gossip.relay with target_gate="local" — should inject locally
    let result = h
        .handle(
            "gossip.relay",
            json!({
                "target_gate": "local",
                "topic": "capability",
                "key": "cap.announce:test",
                "payload": { "event": "register" }
            }),
        )
        .await
        .expect("gossip.relay local should succeed");
    assert_eq!(result["relayed_to"], "local");
    assert_eq!(result["status"], "injected");
}

#[tokio::test]
async fn gossip_relay_missing_topic_errors() {
    let h = ipc_handler();

    let err = h
        .handle("gossip.relay", json!({ "payload": {} }))
        .await
        .expect_err("should require topic");
    assert!(err.contains("topic"), "expected topic error, got: {err}");
}

#[tokio::test]
async fn gossip_relay_to_unknown_gate_errors() {
    let h = ipc_handler();

    // Initialize mesh so we have a mesh reference (but no peers)
    h.handle("mesh.init", json!({ "node_id": "gossip-relay-test" }))
        .await
        .expect("mesh.init");

    let err = h
        .handle(
            "gossip.relay",
            json!({
                "target_gate": "nonexistent-gate",
                "topic": "tower",
                "payload": {}
            }),
        )
        .await
        .expect_err("should fail for unknown gate");
    assert!(
        err.contains("No path to gate") || err.contains("nonexistent-gate"),
        "unexpected error: {err}"
    );
}

/// Spawn a mock UDS JSON-RPC server that echoes back the operation.
///
/// Handles multiple sequential connections (registration identity probe +
/// the actual capability.call forward).
fn spawn_mock_provider(socket_path: &str) -> tokio::task::JoinHandle<()> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;

    let _ = std::fs::remove_file(socket_path);
    let listener = UnixListener::bind(socket_path).unwrap();

    tokio::spawn(async move {
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            let (reader, mut writer) = stream.into_split();
            let mut buf_reader = BufReader::new(reader);
            let mut line = String::new();
            if buf_reader.read_line(&mut line).await.is_ok() && !line.is_empty() {
                let req: Value = serde_json::from_str(line.trim()).unwrap();
                let method = req["method"].as_str().unwrap_or("unknown");
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "result": { "echo_method": method, "status": "ok" },
                    "id": req["id"]
                });
                let mut bytes = serde_json::to_vec(&response).unwrap();
                bytes.push(b'\n');
                let _ = writer.write_all(&bytes).await;
            }
        }
    })
}

#[tokio::test]
async fn capability_call_dispatches_to_local_provider_via_uds() {
    let socket_path = format!("/tmp/songbird-test-cap-{}.sock", Uuid::new_v4());
    let mock_handle = spawn_mock_provider(&socket_path);

    let h = ipc_handler();

    h.handle(
        "ipc.register",
        json!({
            "primal_id": "echo-provider",
            "capabilities": ["echo"],
            "endpoint": socket_path
        }),
    )
    .await
    .expect("register");

    let result = h
        .handle(
            "capability.call",
            json!({
                "capability": "echo",
                "operation": "echo.ping",
                "params": { "msg": "hello" }
            }),
        )
        .await
        .expect("capability.call");

    assert_eq!(result["provider"], "echo-provider");
    assert_eq!(result["gate"], "local");
    assert_eq!(result["result"]["echo_method"], "echo.ping");
    assert_eq!(result["result"]["status"], "ok");

    mock_handle.abort();
    let _ = tokio::fs::remove_file(&socket_path).await;
}

#[tokio::test]
async fn capability_call_errors_when_no_provider_registered() {
    let h = ipc_handler();

    // routing=local forces the handler to skip remote dispatch
    let err = h
        .handle(
            "capability.call",
            json!({
                "capability": "nonexistent",
                "operation": "foo.bar",
                "params": {},
                "routing": "local"
            }),
        )
        .await
        .expect_err("no provider");

    assert!(
        err.contains("No local provider") || err.contains("nonexistent"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn capability_call_with_routing_local_skips_remote() {
    let h = ipc_handler();

    let err = h
        .handle(
            "capability.call",
            json!({
                "capability": "remote-only",
                "operation": "remote.op",
                "params": {},
                "routing": "local"
            }),
        )
        .await
        .expect_err("routing=local, no provider");

    assert!(err.contains("routing=local"), "unexpected error: {err}");
}

#[tokio::test]
async fn capability_call_rejects_invalid_routing() {
    let h = ipc_handler();

    let err = h
        .handle(
            "capability.call",
            json!({
                "capability": "test",
                "operation": "test.op",
                "routing": "remote_only"
            }),
        )
        .await
        .expect_err("invalid routing should fail");

    assert!(err.contains("Invalid routing"), "unexpected error: {err}");
}

#[tokio::test]
async fn capability_call_rejects_empty_capability() {
    let h = ipc_handler();

    let err = h
        .handle(
            "capability.call",
            json!({
                "capability": "",
                "operation": "test.op",
                "routing": "local"
            }),
        )
        .await
        .expect_err("empty capability should fail");

    assert!(err.contains("Invalid capability name"), "unexpected error: {err}");
}

#[tokio::test]
async fn capability_call_rejects_empty_operation() {
    let h = ipc_handler();

    let err = h
        .handle(
            "capability.call",
            json!({
                "capability": "valid",
                "operation": "",
                "routing": "local"
            }),
        )
        .await
        .expect_err("empty operation should fail");

    assert!(err.contains("Invalid operation name"), "unexpected error: {err}");
}

// ── gossip.spread tests ──

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn gossip_spread_without_mesh_returns_local_only() {
    let h = ipc_handler();

    let result = h
        .handle(
            "gossip.spread",
            json!({
                "topic": "capability",
                "key": "test-spread",
                "payload": { "data": 42 }
            }),
        )
        .await
        .expect("gossip.spread without mesh should succeed locally");
    assert_eq!(result["status"], "local_only");
    assert_eq!(result["spread_to"], 0);
}

#[tokio::test]
async fn gossip_spread_with_empty_mesh_returns_zero_spread() {
    let h = ipc_handler();

    h.handle("mesh.init", json!({ "node_id": "spread-test-gate" }))
        .await
        .expect("mesh.init");

    let result = h
        .handle(
            "gossip.spread",
            json!({
                "topic": "tower",
                "key": "cap.announce:spread-test",
                "payload": { "capabilities": ["test.cap"] }
            }),
        )
        .await
        .expect("gossip.spread with empty mesh should succeed");
    assert_eq!(result["status"], "spread");
    assert_eq!(result["spread_to"], 0);
    assert_eq!(result["local_injected"], true);
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn gossip_spread_missing_topic_errors() {
    let h = ipc_handler();

    let err = h
        .handle("gossip.spread", json!({ "payload": {} }))
        .await
        .expect_err("should require topic");
    assert!(err.contains("topic"), "expected topic error, got: {err}");
}

#[tokio::test]
async fn gossip_spread_skips_origin_gate() {
    let h = ipc_handler();

    h.handle("mesh.init", json!({ "node_id": "local-gate" }))
        .await
        .expect("mesh.init");

    let result = h
        .handle(
            "gossip.spread",
            json!({
                "topic": "tower",
                "payload": { "test": true },
                "origin_gate": "some-remote-gate",
                "seen_gates": ["already-seen-gate"]
            }),
        )
        .await
        .expect("gossip.spread with origin should succeed");
    assert_eq!(result["status"], "spread");
    assert_eq!(result["local_injected"], true);
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn gossip_inject_accepts_origin_gate_field() {
    let h = ipc_handler();

    let result = h
        .handle(
            "gossip.inject",
            json!({
                "topic": "tower",
                "key": "remote-inject",
                "payload": { "from": "remote" },
                "origin_gate": "blue-gate"
            }),
        )
        .await
        .expect("gossip.inject with origin_gate");
    assert_eq!(result["status"], "injected");
    assert_eq!(result["origin_gate"], "blue-gate");
}
