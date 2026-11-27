// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! The lineage gate at the mesh port.
//!
//! Every way into the mesh port — riboCipher lines, HTTPS and plain HTTP —
//! passes here before anything is dispatched. A caller on this host is served
//! as before. A caller from the network gets the public methods (health,
//! capabilities, identity, `lineage.prove`, `lineage.status`) and nothing else
//! unless the call carries a lineage proof: a certificate signed by the
//! family's key plus a fresh signature by the caller's own device key.
//!
//! See [`songbird_http_client::lineage`] for the proof itself and its settings.

use axum::body::Body;
use axum::extract::Request;
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};
use songbird_http_client::lineage::{
    self, Decision, GateMode, METHOD_ENROLL, METHOD_PROVE, METHOD_STATUS, PROOF_HEADER, PROOF_PARAM,
};
use std::net::SocketAddr;
use tracing::{debug, info, warn};

/// JSON-RPC error code for a refused call (`METHOD_GATE_STANDARD`: permission denied).
const PERMISSION_DENIED: i32 = -32_001;
/// Largest JSON-RPC body read from a caller on the network before the gate has decided.
const NETWORK_JSONRPC_BODY_LIMIT: usize = 8 * 1024 * 1024;
/// Largest body read otherwise (the router's own limit).
const BODY_LIMIT: usize = 100 * 1024 * 1024;

/// Both ends of an accepted connection. Attached to every HTTP request served on the mesh port.
#[derive(Debug, Clone, Copy)]
pub(super) struct PeerAddrs {
    /// Where the caller is.
    pub remote: SocketAddr,
    /// Which of our addresses the caller reached.
    pub local: SocketAddr,
}

/// Say once, at startup, what the gate is set to and whether this node can prove itself.
pub(super) async fn announce() {
    let gate = lineage::gate();
    match gate.mode() {
        GateMode::Off => info!(
            "lineage gate: off — the mesh port serves every caller on the network (set SONGBIRD_LINEAGE_ANCHOR to close it)"
        ),
        GateMode::Observe => info!(
            "lineage gate: observe — every caller is served as before; calls that enforce would refuse are logged (family '{}')",
            gate.config().family_id
        ),
        GateMode::Enforce => info!(
            "lineage gate: enforce — callers from the network get public methods only unless they prove family '{}'",
            gate.config().family_id
        ),
    }
    if gate.mode() != GateMode::Off && !gate.can_verify() {
        warn!(
            "lineage gate: no family anchor or family name — no caller from the network can pass"
        );
    }
    let Some(certificate) = gate.config().certificate.as_ref() else {
        if gate.mode() != GateMode::Off {
            warn!(
                "lineage gate: this node has no lineage certificate — family peers that ask for proof will refuse it"
            );
        }
        return;
    };
    let device = certificate.get("child_device_id").and_then(Value::as_str).unwrap_or("?");
    let certified_key = certificate.get("child_public_key").and_then(Value::as_str);
    match gate.crypto().public_key().await {
        Ok(key) if certified_key == Some(key.as_str()) => {
            info!("lineage gate: this node proves itself as '{device}'");
        }
        Ok(_) => warn!(
            "lineage gate: the certificate was issued for a different key than the security provider holds — this node cannot prove itself"
        ),
        Err(e) => {
            warn!("lineage gate: could not ask the security provider for this node's key: {e}");
        }
    }
    let status = gate.status();
    if let Some(generations) = status["enroll"].as_u64().filter(|g| *g > 0) {
        let until = status["enroll_expires_at"]
            .as_u64()
            .map_or_else(|| String::from("with no end date"), |t| format!("until {t} (Unix time)"));
        info!(
            "lineage gate: this node may enroll other devices ({generations} generation(s)), {until}; only callers on the device itself can ask it to"
        );
    }
}

/// Whether the gate refuses callers (so other listeners can follow it). Observe changes nothing.
pub fn is_enforcing() -> bool {
    lineage::gate().mode() == GateMode::Enforce
}

fn rpc_error(id: &Value, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "error": {"code": PERMISSION_DENIED, "message": message}, "id": id})
}

/// Pass one JSON-RPC call through the gate.
///
/// Returns `Some(answer)` when the gate answers the call itself (a refusal,
/// `lineage.status`, `lineage.prove`), `None` when the call goes on to dispatch.
/// Any proof is removed from `params` either way.
pub(super) async fn rpc_gate(
    addrs: PeerAddrs,
    method: &str,
    params: &mut Value,
    id: &Value,
) -> Option<Value> {
    let gate = lineage::gate();

    if method == METHOD_STATUS {
        let mut status = gate.status();
        // The device's public key is what an enrolling device signs. It is public, and a
        // device that has no certificate yet can show it this way.
        if let Ok(key) = gate.crypto().public_key().await {
            status["device_public_key"] = Value::String(key);
        }
        return Some(json!({"jsonrpc": "2.0", "result": status, "id": id}));
    }
    if method == METHOD_ENROLL {
        // Signing a certificate is an act of the device's holder: never served to the
        // network, with or without a proof.
        if !lineage::is_local(addrs.remote) {
            warn!(remote = %addrs.remote, method, "lineage gate: refused — enrolling is done on the device itself");
            return Some(rpc_error(
                id,
                "permission denied: lineage.enroll is served to callers on this device only",
            ));
        }
        return Some(match gate.enroll(params).await {
            Ok(result) => {
                let child = result["certificate"]["child_device_id"].as_str().unwrap_or("?");
                info!(child, "lineage gate: signed a certificate for another device");
                json!({"jsonrpc": "2.0", "result": result, "id": id})
            }
            Err(reason) => {
                json!({"jsonrpc": "2.0", "error": {"code": -32_602, "message": reason}, "id": id})
            }
        });
    }
    if method == METHOD_PROVE {
        return Some(match gate.answer_prove(params, addrs.local).await {
            Ok(result) => json!({"jsonrpc": "2.0", "result": result, "id": id}),
            Err(reason) => {
                json!({"jsonrpc": "2.0", "error": {"code": -32_602, "message": reason}, "id": id})
            }
        });
    }

    match gate.authorize_rpc(addrs.remote, addrs.local, method, params).await {
        Decision::Allow => None,
        Decision::AllowFamily(peer) => {
            debug!(remote = %addrs.remote, method, device = %peer.device_id, "lineage gate: family call");
            None
        }
        Decision::Observed(reason) => {
            warn!(remote = %addrs.remote, method, %reason, "lineage gate (observe): would refuse");
            None
        }
        Decision::Deny(reason) => {
            warn!(remote = %addrs.remote, method, %reason, "lineage gate: refused");
            Some(rpc_error(id, &format!("permission denied: {reason}")))
        }
    }
}

fn json_response(status: StatusCode, body: &Value) -> Response {
    (status, [(header::CONTENT_TYPE, "application/json")], body.to_string()).into_response()
}

fn with_body(mut parts: axum::http::request::Parts, bytes: Vec<u8>) -> Request {
    parts.headers.insert(header::CONTENT_LENGTH, HeaderValue::from(bytes.len()));
    Request::from_parts(parts, Body::from(bytes))
}

/// Axum layer: the gate for everything served over HTTP(S) on the mesh port.
pub(super) async fn http_gate(request: Request, next: Next) -> Response {
    let gate = lineage::gate();
    let Some(addrs) = request.extensions().get::<PeerAddrs>().copied() else {
        // Served without connection addresses: only safe while the gate is off.
        if gate.mode() == GateMode::Off {
            return next.run(request).await;
        }
        return json_response(
            StatusCode::FORBIDDEN,
            &json!({"error": "lineage gate: connection addresses unknown"}),
        );
    };

    let path = request.uri().path().to_string();
    let is_jsonrpc = request.method() == Method::POST
        && path.trim_end_matches('/') == songbird_types::constants::JSONRPC_PATH;

    if is_jsonrpc {
        return jsonrpc_gate(addrs, request, next).await;
    }

    if gate.mode() == GateMode::Off
        || lineage::is_local(addrs.remote)
        || (request.method() == Method::GET && path == "/health")
    {
        return next.run(request).await;
    }

    let method = request.method().as_str().to_string();
    let target = request.uri().path_and_query().map_or_else(|| path.clone(), ToString::to_string);
    let proof = request.headers().get(PROOF_HEADER).and_then(|v| v.to_str().ok()).map(String::from);

    let deny = |reason: String| {
        warn!(remote = %addrs.remote, %method, %path, %reason, "lineage gate: refused");
        json_response(
            StatusCode::UNAUTHORIZED,
            &json!({"error": format!("permission denied: {reason}")}),
        )
    };

    // Without a proof there is nothing to check the body against: decide before reading it.
    let Some(proof) = proof else {
        return match gate
            .authorize_http(addrs.remote, addrs.local, None, &method, &target, b"")
            .await
        {
            Decision::Deny(reason) => deny(reason),
            decision => {
                if let Decision::Observed(reason) = decision {
                    warn!(remote = %addrs.remote, %method, %path, %reason, "lineage gate (observe): would refuse");
                }
                next.run(request).await
            }
        };
    };

    let (parts, body) = request.into_parts();
    let Ok(bytes) = axum::body::to_bytes(body, BODY_LIMIT).await else {
        return StatusCode::PAYLOAD_TOO_LARGE.into_response();
    };
    match gate
        .authorize_http(addrs.remote, addrs.local, Some(&proof), &method, &target, &bytes)
        .await
    {
        Decision::Deny(reason) => deny(reason),
        decision => {
            if let Decision::Observed(reason) = decision {
                warn!(remote = %addrs.remote, %method, %path, %reason, "lineage gate (observe): would refuse");
            }
            next.run(with_body(parts, bytes.to_vec())).await
        }
    }
}

/// The gate for `POST /jsonrpc`: decide per method, and take the proof out of the params.
async fn jsonrpc_gate(addrs: PeerAddrs, request: Request, next: Next) -> Response {
    let (parts, body) = request.into_parts();
    let limit = if lineage::is_local(addrs.remote) {
        BODY_LIMIT
    } else {
        NETWORK_JSONRPC_BODY_LIMIT
    };
    let Ok(bytes) = axum::body::to_bytes(body, limit).await else {
        return StatusCode::PAYLOAD_TOO_LARGE.into_response();
    };

    // Not JSON: no method can be called with it. The handler answers with its own parse error.
    let Ok(mut value) = serde_json::from_slice::<Value>(&bytes) else {
        return next.run(with_body(parts, bytes.to_vec())).await;
    };

    let mut changed = false;
    let calls: Vec<&mut Value> = match &mut value {
        Value::Array(items) => items.iter_mut().collect(),
        single => vec![single],
    };
    let batch = calls.len() > 1;

    for call in calls {
        let Some(object) = call.as_object_mut() else {
            continue;
        };
        let method = object.get("method").and_then(Value::as_str).unwrap_or_default().to_string();
        let id = object.get("id").cloned().unwrap_or(Value::Null);
        let mut absent = Value::Null;
        let params = object.get_mut("params").unwrap_or(&mut absent);
        changed |= params.get(PROOF_PARAM).is_some();

        if let Some(answer) = rpc_gate(addrs, &method, params, &id).await {
            let refused = answer.get("error").and_then(|e| e.get("code")).and_then(Value::as_i64)
                == Some(i64::from(PERMISSION_DENIED));
            let status = if refused {
                StatusCode::FORBIDDEN
            } else {
                StatusCode::OK
            };
            // One refused call refuses the whole batch; the gate's own methods are not served in batches.
            let body = if batch && !refused {
                rpc_error(&id, "lineage.prove and lineage.status are not served in a batch")
            } else {
                answer
            };
            return json_response(
                if batch && !refused {
                    StatusCode::BAD_REQUEST
                } else {
                    status
                },
                &body,
            );
        }
    }

    let forward = if changed {
        serde_json::to_vec(&value).unwrap_or_default()
    } else {
        bytes.to_vec()
    };
    next.run(with_body(parts, forward)).await
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use axum::Router;
    use axum::routing::{get, post};
    use tower::ServiceExt;

    fn app() -> Router {
        Router::new()
            .route("/jsonrpc", post(|body: String| async move { format!("handled:{body}") }))
            .route("/health", get(|| async { "OK" }))
            .route("/api/federation/nodes", get(|| async { "nodes" }))
            .layer(axum::middleware::from_fn(http_gate))
    }

    fn request(method: &str, path: &str, body: &str, remote: &str) -> Request {
        let mut req =
            Request::builder().method(method).uri(path).body(Body::from(body.to_string())).unwrap();
        req.extensions_mut().insert(PeerAddrs {
            remote: remote.parse().unwrap(),
            local: "192.168.4.244:7700".parse().unwrap(),
        });
        req
    }

    async fn read(response: Response) -> (StatusCode, String) {
        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), 1 << 20).await.unwrap();
        (status, String::from_utf8_lossy(&bytes).to_string())
    }

    // The process-wide gate is off in the test environment (no anchor set), so these check
    // the parts of the door that do not depend on the mode: its own two methods, proof
    // stripping, and that nothing is lost on the way through.

    #[tokio::test]
    async fn lineage_status_is_answered_at_the_door() {
        let body = r#"{"jsonrpc":"2.0","method":"lineage.status","id":7}"#;
        let (status, text) = read(
            app().oneshot(request("POST", "/jsonrpc", body, "192.168.4.50:4000")).await.unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let v: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["id"], 7);
        assert_eq!(v["result"]["gate"], "off");
        assert_eq!(v["result"]["can_prove"], false);
    }

    #[tokio::test]
    async fn lineage_prove_without_a_certificate_is_an_error_not_a_crash() {
        let body = r#"{"jsonrpc":"2.0","method":"lineage.prove","params":{"nonce":"0123456789abcdef0123"},"id":1}"#;
        let (status, text) = read(
            app().oneshot(request("POST", "/jsonrpc", body, "192.168.4.50:4000")).await.unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let v: Value = serde_json::from_str(&text).unwrap();
        assert!(v["error"]["message"].as_str().unwrap().contains("certificate"));
    }

    #[tokio::test]
    async fn a_call_without_a_proof_reaches_the_handler_byte_for_byte() {
        let body = r#"{"jsonrpc":"2.0",  "method":"mesh.status","params":{"b":1,"a":2},"id":1}"#;
        let (status, text) =
            read(app().oneshot(request("POST", "/jsonrpc", body, "127.0.0.1:4000")).await.unwrap())
                .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(text, format!("handled:{body}"));
    }

    #[tokio::test]
    async fn a_proof_is_taken_out_before_the_handler_sees_the_call() {
        let body = r#"{"jsonrpc":"2.0","method":"mesh.status","params":{"a":2,"_lineage_proof":{"v":1}},"id":1}"#;
        let (_, text) =
            read(app().oneshot(request("POST", "/jsonrpc", body, "127.0.0.1:4000")).await.unwrap())
                .await;
        assert!(text.starts_with("handled:"));
        assert!(!text.contains("_lineage_proof"), "{text}");
        let v: Value = serde_json::from_str(text.trim_start_matches("handled:")).unwrap();
        assert_eq!(v["params"], json!({"a": 2}));
    }

    #[tokio::test]
    async fn other_paths_and_bodies_that_are_not_json_pass_while_the_gate_is_off() {
        let (status, text) =
            read(app().oneshot(request("GET", "/health", "", "192.168.4.50:4000")).await.unwrap())
                .await;
        assert_eq!((status, text.as_str()), (StatusCode::OK, "OK"));
        let (status, text) = read(
            app()
                .oneshot(request("GET", "/api/federation/nodes", "", "192.168.4.50:4000"))
                .await
                .unwrap(),
        )
        .await;
        assert_eq!((status, text.as_str()), (StatusCode::OK, "nodes"));
        let (status, text) = read(
            app()
                .oneshot(request("POST", "/jsonrpc", "not json", "192.168.4.50:4000"))
                .await
                .unwrap(),
        )
        .await;
        assert_eq!((status, text.as_str()), (StatusCode::OK, "handled:not json"));
    }

    #[tokio::test]
    async fn rpc_gate_answers_its_own_methods_and_lets_the_rest_through() {
        let addrs = PeerAddrs {
            remote: "192.168.4.50:4000".parse().unwrap(),
            local: "192.168.4.244:7700".parse().unwrap(),
        };
        let mut params = json!({});
        let status = rpc_gate(addrs, METHOD_STATUS, &mut params, &json!(1)).await.unwrap();
        assert_eq!(status["result"]["gate"], "off");
        let mut params = json!({"x": 1, PROOF_PARAM: {"v": 1}});
        assert!(rpc_gate(addrs, "mesh.status", &mut params, &json!(1)).await.is_none());
        assert_eq!(params, json!({"x": 1}));
    }

    #[tokio::test]
    async fn enrolling_is_never_served_to_the_network() {
        let request = json!({"child_device_id": "laptop", "child_public_key": "AAAA"});

        // From the network: refused as a matter of permission, before anything is looked at.
        let network = PeerAddrs {
            remote: "192.168.4.50:4000".parse().unwrap(),
            local: "192.168.4.244:7700".parse().unwrap(),
        };
        let answer =
            rpc_gate(network, METHOD_ENROLL, &mut request.clone(), &json!(1)).await.unwrap();
        assert_eq!(answer["error"]["code"], PERMISSION_DENIED);

        // From the device itself the request is looked at (and fails here for lack of a certificate).
        for remote in ["127.0.0.1:4000", "[::ffff:127.0.0.1]:4000", "[::1]:4000"] {
            let local = PeerAddrs {
                remote: remote.parse().unwrap(),
                local: "127.0.0.1:7700".parse().unwrap(),
            };
            let answer =
                rpc_gate(local, METHOD_ENROLL, &mut request.clone(), &json!(1)).await.unwrap();
            assert_eq!(answer["error"]["code"], -32_602, "{remote}");
        }
    }
}
