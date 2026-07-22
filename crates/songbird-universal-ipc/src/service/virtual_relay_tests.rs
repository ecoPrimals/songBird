// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used)]

use super::*;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

#[test]
fn default_base_dir_under_xdg() {
    let dir = VirtualRelayManager::default_base_dir();
    let path_str = dir.to_string_lossy();
    assert!(path_str.contains("biomeos/songbird/virtual"));
}

#[test]
fn relay_socket_path_format() {
    let mgr = VirtualRelayManager::new(PathBuf::from("/run/user/1000/biomeos/songbird/virtual"));
    let path = mgr.relay_socket_path("beardog");
    assert_eq!(path, PathBuf::from("/run/user/1000/biomeos/songbird/virtual/beardog.sock"));
}

#[tokio::test]
async fn relay_manager_lifecycle() {
    let dir = tempfile::tempdir().unwrap();
    let mgr = VirtualRelayManager::new(dir.path().to_path_buf());

    assert!(!mgr.has_relay("test-primal").await);
    assert!(mgr.list_relays().await.is_empty());

    // Can't actually start a relay without a real native target, but we can test
    // the path computation
    let expected = dir.path().join("test-primal.sock");
    assert_eq!(mgr.relay_socket_path("test-primal"), expected);
}

#[tokio::test]
async fn start_and_stop_relay_with_mock_target() {
    let dir = tempfile::tempdir().unwrap();
    let native_dir = tempfile::tempdir().unwrap();

    // Create a mock native listener
    let native_path = native_dir.path().join("mock.sock");
    let native_listener = UnixListener::bind(&native_path).unwrap();

    // Spawn a mock responder
    let mock_handle = tokio::spawn(async move {
        if let Ok((stream, _)) = native_listener.accept().await {
            let (reader, mut writer) = stream.into_split();
            let mut buf = BufReader::new(reader);
            let mut line = String::new();
            if buf.read_line(&mut line).await.is_ok() {
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "result": {"ok": true},
                    "id": 1
                });
                let mut bytes = serde_json::to_vec(&response).unwrap();
                bytes.push(b'\n');
                let _ = writer.write_all(&bytes).await;
            }
        }
    });

    let mgr = VirtualRelayManager::new(dir.path().to_path_buf());
    let relay_path = mgr.start_relay("mock-primal", native_path.to_str().unwrap()).await.unwrap();

    assert!(mgr.has_relay("mock-primal").await);
    assert_eq!(mgr.list_relays().await.len(), 1);

    // Connect to the virtual relay (retry until socket is ready)
    let stream = tokio::time::timeout(std::time::Duration::from_secs(2), async {
        loop {
            match UnixStream::connect(&relay_path).await {
                Ok(s) => return s,
                Err(_) => tokio::task::yield_now().await,
            }
        }
    })
    .await
    .expect("relay socket should be connectable within 2s");
    let (reader, mut writer) = stream.into_split();

    let request = serde_json::json!({"jsonrpc":"2.0","method":"test.ping","params":{},"id":1});
    let mut req_bytes = serde_json::to_vec(&request).unwrap();
    req_bytes.push(b'\n');
    writer.write_all(&req_bytes).await.unwrap();

    let mut buf = BufReader::new(reader);
    let mut response_line = String::new();
    buf.read_line(&mut response_line).await.unwrap();

    let response: serde_json::Value = serde_json::from_str(response_line.trim()).unwrap();
    assert_eq!(response["result"]["ok"], true);

    // Stop relay
    mgr.stop_relay("mock-primal").await;
    assert!(!mgr.has_relay("mock-primal").await);

    mock_handle.abort();
}

#[test]
fn relay_metrics_avg_overhead() {
    let metrics = RelayMetrics::new();
    assert_eq!(metrics.avg_overhead_us(), 0);

    metrics.requests.store(4, Ordering::Relaxed);
    metrics.overhead_us.store(1000, Ordering::Relaxed);
    assert_eq!(metrics.avg_overhead_us(), 250);
}

#[tokio::test]
async fn relay_rejects_tampered_btsp_signature() {
    use base64::Engine as _;

    let dir = tempfile::tempdir().unwrap();
    let native_dir = tempfile::tempdir().unwrap();

    let native_path = native_dir.path().join("mock.sock");
    let native_listener = UnixListener::bind(&native_path).unwrap();

    // Mock native: should NEVER receive a request (relay rejects before forwarding)
    let mock_handle = tokio::spawn(async move {
        if let Ok((stream, _)) = native_listener.accept().await {
            let (reader, mut writer) = stream.into_split();
            let mut buf = BufReader::new(reader);
            let mut line = String::new();
            if buf.read_line(&mut line).await.is_ok() {
                let resp = serde_json::json!({"jsonrpc":"2.0","result":{"leaked":true},"id":1});
                let mut bytes = serde_json::to_vec(&resp).unwrap();
                bytes.push(b'\n');
                let _ = writer.write_all(&bytes).await;
            }
        }
    });

    // Create relay with a rejecting verifier
    struct RejectAllVerifier;
    impl BtspSignatureVerifier for RejectAllVerifier {
        fn verify(
            &self,
            _: &str,
            _: &[u8],
            _: &[u8],
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send + '_>>
        {
            Box::pin(async { Ok(false) })
        }
    }

    let mut mgr = VirtualRelayManager::new(dir.path().to_path_buf());
    mgr.set_signature_verifier(Arc::new(RejectAllVerifier));
    let relay_path = mgr.start_relay("tamper-test", native_path.to_str().unwrap()).await.unwrap();

    let stream = tokio::time::timeout(std::time::Duration::from_secs(2), async {
        loop {
            match UnixStream::connect(&relay_path).await {
                Ok(s) => return s,
                Err(_) => tokio::task::yield_now().await,
            }
        }
    })
    .await
    .expect("relay socket should be connectable within 2s");
    let (reader, mut writer) = stream.into_split();

    // Build a signed BTSP token (will be rejected by RejectAllVerifier)
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let payload = serde_json::json!({"node_id": "attacker", "ts": now});
    let payload_b64 =
        base64::engine::general_purpose::STANDARD.encode(payload.to_string().as_bytes());
    let sig_b64 = base64::engine::general_purpose::STANDARD.encode(b"tampered-sig");
    let token = format!("{payload_b64}.{sig_b64}");

    let request = format!(
        r#"{{"jsonrpc":"2.0","method":"secrets.steal","params":{{}},"id":99,"_btsp_session":"{token}"}}"#
    );
    let mut req_bytes = request.into_bytes();
    req_bytes.push(b'\n');
    writer.write_all(&req_bytes).await.unwrap();

    let mut buf = BufReader::new(reader);
    let mut response_line = String::new();
    buf.read_line(&mut response_line).await.unwrap();

    let response: serde_json::Value = serde_json::from_str(response_line.trim()).unwrap();
    assert_eq!(response["error"]["code"], -32603);
    assert!(
        response["error"]["message"].as_str().unwrap().contains("signature verification failed")
    );
    assert_eq!(response["id"], 99);
    // The native mock should NOT have received the request
    assert!(response.get("result").is_none());

    mgr.stop_relay("tamper-test").await;
    mock_handle.abort();
}
