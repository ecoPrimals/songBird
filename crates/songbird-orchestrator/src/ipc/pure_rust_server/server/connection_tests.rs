// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for `connection.rs` — BTSP Phase 3 live connection verification.
//!
//! Extracted to a sibling file to keep `connection.rs` under 800L while
//! retaining private-method access via `mod tests` with `#[path]`.

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::*;
use crate::app::connection_manager::ConnectionManager;
use crate::ipc::btsp_phase3::SessionKeys;
use crate::ipc::pure_rust_server::method_gate::CallerContext;
use crate::ipc::registry::ServiceRegistry;
use base64::Engine as _;

fn test_server() -> Arc<UnixSocketServer> {
    let registry = Arc::new(ServiceRegistry::new());
    let conn_mgr = Arc::new(ConnectionManager::new());
    let security = Arc::new(songbird_http_client::SecurityRpcClient::new_direct(
        "/tmp/songbird-phase3-test.sock",
    ));
    Arc::new(UnixSocketServer::new(registry, None, conn_mgr, security))
}

/// Verifies that `handle_encrypted_session` enters the encrypted frame loop
/// on a live async duplex stream and correctly processes JSON-RPC requests
/// through encrypt/decrypt roundtrip.
#[tokio::test]
async fn encrypted_session_loop_on_live_duplex() {
    let server = test_server();
    let (client_stream, server_stream) = tokio::io::duplex(16 * 1024);

    let handshake_key = [0x42u8; 32];
    let client_nonce = b"client_nonce_val";
    let server_nonce = b"server_nonce_val";

    let server_keys =
        SessionKeys::derive(&handshake_key, client_nonce, server_nonce, false).unwrap();
    let client_keys =
        SessionKeys::derive(&handshake_key, client_nonce, server_nonce, true).unwrap();

    let (server_reader, server_writer) = tokio::io::split(server_stream);

    let server_handle = tokio::spawn(async move {
        let caller = CallerContext::from_unix();
        server.handle_encrypted_session(server_reader, server_writer, server_keys, &caller).await
    });

    let (mut client_reader, mut client_writer) = tokio::io::split(client_stream);

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.liveness",
        "id": 1
    });
    let req_bytes = serde_json::to_vec(&request).unwrap();
    let encrypted_req = client_keys.encrypt(&req_bytes).unwrap();
    btsp_phase3::write_encrypted_frame(&mut client_writer, &encrypted_req).await.unwrap();

    let resp_frame = btsp_phase3::read_encrypted_frame(&mut client_reader).await.unwrap();
    let resp_bytes = client_keys.decrypt(&resp_frame).unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&resp_bytes).unwrap();

    assert_eq!(resp["jsonrpc"], "2.0");
    assert_eq!(resp["id"], 1);
    assert!(resp["result"].is_object(), "expected success result");
    assert!(resp["error"].is_null() || resp.get("error").is_none());

    let request2 = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "capabilities.list",
        "id": 2
    });
    let req2_bytes = serde_json::to_vec(&request2).unwrap();
    let encrypted_req2 = client_keys.encrypt(&req2_bytes).unwrap();
    btsp_phase3::write_encrypted_frame(&mut client_writer, &encrypted_req2).await.unwrap();

    let resp2_frame = btsp_phase3::read_encrypted_frame(&mut client_reader).await.unwrap();
    let resp2_bytes = client_keys.decrypt(&resp2_frame).unwrap();
    let resp2: serde_json::Value = serde_json::from_slice(&resp2_bytes).unwrap();

    assert_eq!(resp2["jsonrpc"], "2.0");
    assert_eq!(resp2["id"], 2);
    assert_eq!(resp2["result"]["primal"], "songbird");
    assert!(resp2["result"]["methods"].is_array());

    drop(client_writer);
    drop(client_reader);
    let result = server_handle.await.unwrap();
    assert!(result.is_ok(), "server loop should exit cleanly on client disconnect");
}

/// Verifies that a notification (no `id`) does NOT produce a response frame,
/// and that the subsequent request still gets a response.
#[tokio::test]
async fn encrypted_session_notifications_produce_no_response() {
    let server = test_server();
    let (client_stream, server_stream) = tokio::io::duplex(16 * 1024);

    let hk = [0xAA; 32];
    let cn = b"cn_notif_test!!!";
    let sn = b"sn_notif_test!!!";

    let server_keys = SessionKeys::derive(&hk, cn, sn, false).unwrap();
    let client_keys = SessionKeys::derive(&hk, cn, sn, true).unwrap();

    let (sr, sw) = tokio::io::split(server_stream);
    let server_handle = tokio::spawn(async move {
        let caller = CallerContext::from_unix();
        server.handle_encrypted_session(sr, sw, server_keys, &caller).await
    });

    let (mut cr, mut cw) = tokio::io::split(client_stream);

    let notification = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.liveness"
    });
    let notif_bytes = serde_json::to_vec(&notification).unwrap();
    let enc_notif = client_keys.encrypt(&notif_bytes).unwrap();
    btsp_phase3::write_encrypted_frame(&mut cw, &enc_notif).await.unwrap();

    let follow_up = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "identity.get",
        "id": 99
    });
    let fu_bytes = serde_json::to_vec(&follow_up).unwrap();
    let enc_fu = client_keys.encrypt(&fu_bytes).unwrap();
    btsp_phase3::write_encrypted_frame(&mut cw, &enc_fu).await.unwrap();

    let resp_frame = btsp_phase3::read_encrypted_frame(&mut cr).await.unwrap();
    let resp_bytes = client_keys.decrypt(&resp_frame).unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&resp_bytes).unwrap();

    assert_eq!(resp["id"], 99, "first response should be for id=99 (notification has no response)");
    assert_eq!(resp["result"]["primal"], "songbird");

    drop(cw);
    drop(cr);
    let _ = server_handle.await.unwrap();
}

/// Verifies that `btsp.negotiate` is dispatched on the NDJSON path and
/// gracefully falls back to null cipher when security provider is unavailable.
#[tokio::test]
async fn ndjson_negotiate_dispatch_null_cipher_fallback() {
    let server = test_server();
    let (client_stream, server_stream) = tokio::io::duplex(16 * 1024);

    let (sr, sw) = tokio::io::split(server_stream);
    let reader = BufReader::new(sr);
    let server_handle = tokio::spawn(async move {
        let caller = CallerContext::from_unix();
        server.handle_ndjson_session(reader, sw, &caller).await
    });

    let (mut cr, mut cw) = tokio::io::split(client_stream);

    let negotiate = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "btsp.negotiate",
        "params": {
            "session_id": "test-session-abc",
            "preferred_cipher": "chacha20-poly1305",
            "client_nonce": "AAAAAAAAAAAAAAAAAAAAAA==",
            "bond_type": "Covalent"
        },
        "id": 1
    });
    let mut req_line = serde_json::to_vec(&negotiate).unwrap();
    req_line.push(b'\n');
    cw.write_all(&req_line).await.unwrap();
    cw.flush().await.unwrap();

    let mut resp_buf = vec![0u8; 4096];
    let mut total = 0;
    loop {
        let n = cr.read(&mut resp_buf[total..]).await.unwrap();
        total += n;
        if resp_buf[..total].contains(&b'\n') || n == 0 {
            break;
        }
    }

    let resp: serde_json::Value =
        serde_json::from_slice(&resp_buf[..total].split(|&b| b == b'\n').next().unwrap()).unwrap();

    assert_eq!(resp["jsonrpc"], "2.0");
    assert_eq!(resp["id"], 1);
    assert_eq!(
        resp["result"]["cipher"], "null",
        "without security provider, should fall back to null cipher"
    );

    let follow_up = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.liveness",
        "id": 2
    });
    let mut fu_line = serde_json::to_vec(&follow_up).unwrap();
    fu_line.push(b'\n');
    cw.write_all(&fu_line).await.unwrap();
    cw.flush().await.unwrap();

    total = 0;
    loop {
        let n = cr.read(&mut resp_buf[total..]).await.unwrap();
        total += n;
        if resp_buf[..total].contains(&b'\n') || n == 0 {
            break;
        }
    }
    let resp2: serde_json::Value =
        serde_json::from_slice(&resp_buf[..total].split(|&b| b == b'\n').next().unwrap()).unwrap();

    assert_eq!(resp2["id"], 2);
    assert!(resp2["result"].is_object());

    drop(cw);
    drop(cr);
    let _ = server_handle.await.unwrap();
}

/// Full negotiate → encrypted session transition using a mock security provider.
/// Starts a tiny UDS server that responds to `btsp.server.export_keys` with a
/// known handshake key, then verifies the NDJSON session transitions to
/// encrypted framing after negotiate.
#[cfg(unix)]
#[tokio::test]
async fn ndjson_negotiate_to_encrypted_session_live() {
    use tokio::net::UnixListener;

    let mock_dir = tempfile::tempdir().unwrap();
    let mock_sock = mock_dir.path().join("mock-security.sock");

    let handshake_key = [0x55u8; 32];
    let hk_b64 = base64::prelude::BASE64_STANDARD.encode(handshake_key);

    let listener = UnixListener::bind(&mock_sock).unwrap();
    let hk_b64_clone = hk_b64.clone();
    let mock_handle = tokio::spawn(async move {
        loop {
            let Ok((mut stream, _)) = listener.accept().await else {
                break;
            };
            let hk = hk_b64_clone.clone();
            tokio::spawn(async move {
                let mut buf = vec![0u8; 4096];
                let n = stream.read(&mut buf).await.unwrap_or(0);
                if n == 0 {
                    return;
                }
                let resp = serde_json::json!({
                    "jsonrpc": "2.0",
                    "result": { "handshake_key": hk },
                    "id": 1
                });
                let mut out = serde_json::to_vec(&resp).unwrap();
                out.push(b'\n');
                let _ = stream.write_all(&out).await;
                let _ = stream.flush().await;
            });
        }
    });

    let security =
        Arc::new(songbird_http_client::SecurityRpcClient::new_direct(mock_sock.to_str().unwrap()));
    let registry = Arc::new(ServiceRegistry::new());
    let conn_mgr = Arc::new(ConnectionManager::new());
    let server = Arc::new(UnixSocketServer::new(registry, None, conn_mgr, security));

    let (client_stream, server_stream) = tokio::io::duplex(64 * 1024);
    let (sr, sw) = tokio::io::split(server_stream);
    let reader = BufReader::new(sr);

    let server_handle = tokio::spawn(async move {
        let caller = CallerContext::from_unix();
        server.handle_ndjson_session(reader, sw, &caller).await
    });

    let (mut cr, mut cw) = tokio::io::split(client_stream);

    let negotiate = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "btsp.negotiate",
        "params": {
            "session_id": "live-test-session",
            "preferred_cipher": "chacha20-poly1305",
            "client_nonce": base64::prelude::BASE64_STANDARD.encode(b"client_nonce_16b"),
            "bond_type": "Covalent"
        },
        "id": 1
    });
    let mut req_line = serde_json::to_vec(&negotiate).unwrap();
    req_line.push(b'\n');
    cw.write_all(&req_line).await.unwrap();
    cw.flush().await.unwrap();

    let mut resp_buf = vec![0u8; 4096];
    let mut total = 0;
    loop {
        let n = cr.read(&mut resp_buf[total..]).await.unwrap();
        total += n;
        if resp_buf[..total].contains(&b'\n') || n == 0 {
            break;
        }
    }
    let resp: serde_json::Value =
        serde_json::from_slice(&resp_buf[..total].split(|&b| b == b'\n').next().unwrap()).unwrap();

    assert_eq!(resp["result"]["cipher"], "chacha20-poly1305");
    let server_nonce_b64 = resp["result"]["server_nonce"].as_str().unwrap();
    let server_nonce = base64::prelude::BASE64_STANDARD.decode(server_nonce_b64).unwrap();

    let client_keys =
        SessionKeys::derive(&handshake_key, b"client_nonce_16b", &server_nonce, true).unwrap();

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "capabilities.list",
        "id": 2
    });
    let req_bytes = serde_json::to_vec(&request).unwrap();
    let encrypted = client_keys.encrypt(&req_bytes).unwrap();
    btsp_phase3::write_encrypted_frame(&mut cw, &encrypted).await.unwrap();

    let resp_frame = btsp_phase3::read_encrypted_frame(&mut cr).await.unwrap();
    let resp_bytes = client_keys.decrypt(&resp_frame).unwrap();
    let resp2: serde_json::Value = serde_json::from_slice(&resp_bytes).unwrap();

    assert_eq!(resp2["jsonrpc"], "2.0");
    assert_eq!(resp2["id"], 2);
    assert_eq!(resp2["result"]["primal"], "songbird");
    assert!(resp2["result"]["methods"].is_array());

    drop(cw);
    drop(cr);
    let _ = server_handle.await.unwrap();
    mock_handle.abort();
}
