// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for `connection.rs` — BTSP Phase 3 live connection verification.
//!
//! Extracted to a sibling file to keep `connection.rs` under 800L while
//! retaining private-method access via `mod tests` with `#[path]`.

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::*;
use crate::app::connection_manager::ConnectionManager;
use crate::ipc::btsp_phase3;
use crate::ipc::btsp_phase3::SessionKeys;
use crate::ipc::pure_rust_server::method_gate::CallerContext;
use crate::ipc::registry::ServiceRegistry;
use base64::Engine as _;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
        serde_json::from_slice(resp_buf[..total].split(|&b| b == b'\n').next().unwrap()).unwrap();

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
        serde_json::from_slice(resp_buf[..total].split(|&b| b == b'\n').next().unwrap()).unwrap();

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
        serde_json::from_slice(resp_buf[..total].split(|&b| b == b'\n').next().unwrap()).unwrap();

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

/// Multi-frame stress test: sends 100 encrypted JSON-RPC requests in rapid succession
/// and verifies all responses arrive correctly ordered.
#[tokio::test]
async fn encrypted_session_multi_frame_stress_100_requests() {
    let server = test_server();
    let (client_stream, server_stream) = tokio::io::duplex(64 * 1024);

    let hk = [0x77u8; 32];
    let cn = b"stress_client_16";
    let sn = b"stress_server_16";

    let server_keys = SessionKeys::derive(&hk, cn, sn, false).unwrap();
    let client_keys = SessionKeys::derive(&hk, cn, sn, true).unwrap();

    let (sr, sw) = tokio::io::split(server_stream);
    let server_handle = tokio::spawn(async move {
        let caller = CallerContext::from_unix();
        server.handle_encrypted_session(sr, sw, server_keys, &caller).await
    });

    let (mut cr, mut cw) = tokio::io::split(client_stream);
    let client_keys_read = SessionKeys::derive(&hk, cn, sn, true).unwrap();

    const NUM_REQUESTS: u64 = 100;

    for i in 1..=NUM_REQUESTS {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "health.liveness",
            "id": i
        });
        let req_bytes = serde_json::to_vec(&request).unwrap();
        let encrypted = client_keys.encrypt(&req_bytes).unwrap();
        btsp_phase3::write_encrypted_frame(&mut cw, &encrypted).await.unwrap();
    }

    for i in 1..=NUM_REQUESTS {
        let frame = btsp_phase3::read_encrypted_frame(&mut cr).await.unwrap();
        let decrypted = client_keys_read.decrypt(&frame).unwrap();
        let resp: serde_json::Value = serde_json::from_slice(&decrypted).unwrap();
        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], i, "response ordering must match request ordering");
        assert!(resp["result"].is_object(), "response {i} should have result");
    }

    drop(cw);
    drop(cr);
    let result = server_handle.await.unwrap();
    assert!(result.is_ok(), "server should exit cleanly after stress test");
}

/// Sustained load: sends requests with varying payload sizes (1B to 4KB JSON params)
/// to exercise buffer management under sustained load conditions.
#[tokio::test]
async fn encrypted_session_sustained_load_varying_payload() {
    let server = test_server();
    let (client_stream, server_stream) = tokio::io::duplex(128 * 1024);

    let hk = [0x88u8; 32];
    let cn = b"payload_client16";
    let sn = b"payload_server16";

    let server_keys = SessionKeys::derive(&hk, cn, sn, false).unwrap();
    let client_keys = SessionKeys::derive(&hk, cn, sn, true).unwrap();

    let (sr, sw) = tokio::io::split(server_stream);
    let server_handle = tokio::spawn(async move {
        let caller = CallerContext::from_unix();
        server.handle_encrypted_session(sr, sw, server_keys, &caller).await
    });

    let (mut cr, mut cw) = tokio::io::split(client_stream);
    let client_keys_read = SessionKeys::derive(&hk, cn, sn, true).unwrap();

    const NUM_REQUESTS: u64 = 50;

    for i in 1..=NUM_REQUESTS {
        let payload_size = ((i as usize) * 80).min(4000);
        let payload = "x".repeat(payload_size);
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "health.liveness",
            "params": { "data": payload },
            "id": i
        });
        let req_bytes = serde_json::to_vec(&request).unwrap();
        let encrypted = client_keys.encrypt(&req_bytes).unwrap();
        btsp_phase3::write_encrypted_frame(&mut cw, &encrypted).await.unwrap();
    }

    for i in 1..=NUM_REQUESTS {
        let frame = btsp_phase3::read_encrypted_frame(&mut cr).await.unwrap();
        let decrypted = client_keys_read.decrypt(&frame).unwrap();
        let resp: serde_json::Value = serde_json::from_slice(&decrypted).unwrap();
        assert_eq!(resp["id"], i, "response {i} out of order");
        assert_eq!(resp["jsonrpc"], "2.0");
    }

    drop(cw);
    drop(cr);
    let result = server_handle.await.unwrap();
    assert!(result.is_ok());
}

/// Concurrent multi-client stress: 10 simultaneous encrypted sessions each
/// sending 20 requests, verifying session isolation under load.
#[tokio::test]
async fn encrypted_session_concurrent_multi_client() {
    let server = test_server();
    let mut handles = Vec::new();

    for client_idx in 0u8..10 {
        let server_clone = Arc::clone(&server);
        let handle = tokio::spawn(async move {
            let (client_stream, server_stream) = tokio::io::duplex(32 * 1024);

            let hk = [client_idx.wrapping_add(0x10); 32];
            let cn_bytes: [u8; 16] = {
                let mut buf = [0u8; 16];
                let s = format!("cn{client_idx:02}");
                buf[..s.len()].copy_from_slice(s.as_bytes());
                buf
            };
            let sn_bytes: [u8; 16] = {
                let mut buf = [0u8; 16];
                let s = format!("sn{client_idx:02}");
                buf[..s.len()].copy_from_slice(s.as_bytes());
                buf
            };

            let server_keys = SessionKeys::derive(&hk, &cn_bytes, &sn_bytes, false).unwrap();
            let client_keys = SessionKeys::derive(&hk, &cn_bytes, &sn_bytes, true).unwrap();
            let client_keys_read = SessionKeys::derive(&hk, &cn_bytes, &sn_bytes, true).unwrap();

            let (sr, sw) = tokio::io::split(server_stream);
            let server_handle = tokio::spawn(async move {
                let caller = CallerContext::from_unix();
                server_clone.handle_encrypted_session(sr, sw, server_keys, &caller).await
            });

            let (mut cr, mut cw) = tokio::io::split(client_stream);

            for i in 1..=20u64 {
                let request = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "health.liveness",
                    "id": i
                });
                let req_bytes = serde_json::to_vec(&request).unwrap();
                let encrypted = client_keys.encrypt(&req_bytes).unwrap();
                btsp_phase3::write_encrypted_frame(&mut cw, &encrypted).await.unwrap();
            }

            for i in 1..=20u64 {
                let frame = btsp_phase3::read_encrypted_frame(&mut cr).await.unwrap();
                let decrypted = client_keys_read.decrypt(&frame).unwrap();
                let resp: serde_json::Value = serde_json::from_slice(&decrypted).unwrap();
                assert_eq!(resp["id"], i);
            }

            drop(cw);
            drop(cr);
            let result = server_handle.await.unwrap();
            assert!(result.is_ok(), "client {client_idx} server session failed");
        });
        handles.push(handle);
    }

    for h in handles {
        h.await.unwrap();
    }
}

// ============================================================================
// riboCipher transport signal detection tests
// ============================================================================

/// Verifies that a clear-tier riboCipher signal (0xEC 0x01) followed by
/// JSON-RPC is routed correctly through `handle_connection_with_peek`.
#[cfg(unix)]
#[tokio::test]
async fn ribocipher_clear_signal_routes_to_ndjson() {
    use songbird_types::constants::ribocipher;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let server = test_server();
    let (mut client, server_stream) = tokio::net::UnixStream::pair().unwrap();

    let caller = super::super::super::method_gate::CallerContext::from_unix();
    let server_handle =
        tokio::spawn(
            async move { server.handle_connection_with_peek(server_stream, &caller).await },
        );

    // Send riboCipher clear prefix + JSON-RPC request
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.liveness",
        "id": 1
    });
    let mut payload = ribocipher::CLEAR_PREFIX.to_vec();
    let req_bytes = serde_json::to_vec(&request).unwrap();
    payload.extend_from_slice(&req_bytes);
    payload.push(b'\n');
    client.write_all(&payload).await.unwrap();

    // Read response
    let mut buf = vec![0u8; 4096];
    let n = client.read(&mut buf).await.unwrap();
    assert!(n > 0, "expected response from server");

    let resp: serde_json::Value = serde_json::from_slice(&buf[..n]).unwrap();
    assert_eq!(resp["id"], 1);
    assert!(resp.get("result").is_some() || resp.get("error").is_some());

    drop(client);
    let _ = server_handle.await;
}

/// Verifies that mito-tier riboCipher signal (0xED 0x01) is accepted and
/// routes to federation-tier NDJSON processing.
#[cfg(unix)]
#[tokio::test]
async fn ribocipher_mito_signal_accepted() {
    use songbird_types::constants::ribocipher;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let server = test_server();
    let (mut client, server_stream) = tokio::net::UnixStream::pair().unwrap();

    let caller = super::super::super::method_gate::CallerContext::from_unix();
    let server_handle =
        tokio::spawn(
            async move { server.handle_connection_with_peek(server_stream, &caller).await },
        );

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.liveness",
        "id": 42
    });
    let mut payload = ribocipher::MITO_PREFIX.to_vec();
    let req_bytes = serde_json::to_vec(&request).unwrap();
    payload.extend_from_slice(&req_bytes);
    payload.push(b'\n');
    client.write_all(&payload).await.unwrap();

    let mut buf = vec![0u8; 4096];
    let n = client.read(&mut buf).await.unwrap();
    assert!(n > 0, "expected mito-tier response");

    let resp: serde_json::Value = serde_json::from_slice(&buf[..n]).unwrap();
    assert_eq!(resp["id"], 42);

    drop(client);
    let _ = server_handle.await;
}

/// Verifies that an unsupported riboCipher version byte causes a clean drop
/// without panic or hang.
#[cfg(unix)]
#[tokio::test]
async fn ribocipher_unsupported_version_drops_cleanly() {
    use tokio::io::AsyncWriteExt;

    let server = test_server();
    let (mut client, server_stream) = tokio::net::UnixStream::pair().unwrap();

    let caller = super::super::super::method_gate::CallerContext::from_unix();
    let server_handle =
        tokio::spawn(
            async move { server.handle_connection_with_peek(server_stream, &caller).await },
        );

    // Send signal byte 0xEC with bad version 0xFF
    client.write_all(&[0xEC, 0xFF]).await.unwrap();
    drop(client);

    let result = server_handle.await.unwrap();
    assert!(result.is_ok(), "unsupported version should not error");
}

/// Verifies that a riboCipher signal byte with no following version byte
/// (disconnect after signal) is handled gracefully.
#[cfg(unix)]
#[tokio::test]
async fn ribocipher_signal_only_no_version_drops_cleanly() {
    use tokio::io::AsyncWriteExt;

    let server = test_server();
    let (mut client, server_stream) = tokio::net::UnixStream::pair().unwrap();

    let caller = super::super::super::method_gate::CallerContext::from_unix();
    let server_handle =
        tokio::spawn(
            async move { server.handle_connection_with_peek(server_stream, &caller).await },
        );

    // Send only the signal byte, then disconnect
    client.write_all(&[0xED]).await.unwrap();
    drop(client);

    let result = server_handle.await.unwrap();
    assert!(result.is_ok(), "missing version byte should not error");
}
