// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Mesh Trust Exchange — automatic `auth.exchange_trust` after mesh seeding.
//!
//! After BTSP-secured mesh connections are established, this module exchanges
//! Ed25519 trust keys bidirectionally with each peer, eliminating operator
//! intervention for `auth.trust_issuer` registration.
//!
//! ## Flow
//!
//! 1. Get local gate's Ed25519 public key from local security provider
//! 2. For each reachable peer: POST `auth.exchange_trust` via Songbird HTTP JSON-RPC
//! 3. Register returned remote key on local security provider
//!
//! All steps are async and best-effort — mesh connectivity is not gated on trust
//! exchange success. Failures are logged and retried on the next health cycle.

use serde_json::{Value, json};
use std::net::SocketAddr;
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::env_config::security_crypto_ipc_socket_from_env;

const TRUST_EXCHANGE_TIMEOUT: Duration = Duration::from_secs(10);

/// Spawn trust exchange attempts for all known mesh peers.
///
/// Called after `spawn_mesh_seed` completes. Non-blocking — spawns a tokio task
/// that runs with a short delay to allow TCP connections to establish.
pub fn spawn_trust_exchange(peers: Vec<(String, SocketAddr)>) {
    if peers.is_empty() {
        return;
    }

    let node_id = resolve_node_id();
    let family_id = songbird_process_env::var("FAMILY_ID").unwrap_or_default();

    info!(
        node_id = %node_id,
        peer_count = peers.len(),
        "Scheduling mesh trust exchange with peers"
    );

    tokio::spawn(async move {
        // Brief delay to allow mesh TCP connections to stabilize
        tokio::time::sleep(Duration::from_secs(2)).await;

        let local_key = match get_local_public_key().await {
            Ok(key) => key,
            Err(e) => {
                warn!(error = %e, "Cannot perform trust exchange: local security provider key unavailable");
                return;
            }
        };

        for (peer_id, peer_addr) in &peers {
            match exchange_trust_with_peer(peer_id, *peer_addr, &local_key, &node_id, &family_id)
                .await
            {
                Ok(remote_key) => {
                    if let Err(e) = register_remote_key(&remote_key).await {
                        warn!(
                            peer = %peer_id,
                            error = %e,
                            "Trust exchange succeeded but local registration failed"
                        );
                    } else {
                        info!(
                            peer = %peer_id,
                            remote_did = %remote_key.did,
                            "Bidirectional trust established"
                        );
                    }
                }
                Err(e) => {
                    debug!(
                        peer = %peer_id,
                        error = %e,
                        "Trust exchange deferred (peer may not be ready)"
                    );
                }
            }
        }
    });
}

/// Remote key material returned by `auth.exchange_trust`.
struct RemoteKeyMaterial {
    public_key: String,
    did: String,
    gate_id: String,
}

/// Get our local Ed25519 public key from the security provider.
async fn get_local_public_key() -> Result<String, String> {
    let socket_path = discover_security_provider_socket()?;

    let request = json!({
        "jsonrpc": "2.0",
        "method": "auth.exchange_trust",
        "params": {
            "public_key": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
            "gate_id": "__self_probe__",
        },
        "id": 1
    });

    let response = call_uds_jsonrpc(&socket_path, &request).await?;

    response
        .get("local_public_key")
        .and_then(Value::as_str)
        .map(String::from)
        .ok_or_else(|| String::from("security provider did not return local_public_key"))
}

/// Call `auth.exchange_trust` on a remote peer's Songbird HTTP endpoint.
async fn exchange_trust_with_peer(
    peer_id: &str,
    peer_addr: SocketAddr,
    local_public_key: &str,
    local_node_id: &str,
    local_family_id: &str,
) -> Result<RemoteKeyMaterial, String> {
    let endpoint = songbird_types::constants::endpoint_url(
        &peer_addr.ip().to_string(),
        peer_addr.port(),
        "/jsonrpc",
    );

    let request = json!({
        "jsonrpc": "2.0",
        "method": "capability.call",
        "params": {
            "capability": "auth",
            "operation": "exchange_trust",
            "params": {
                "public_key": local_public_key,
                "gate_id": local_node_id,
                "family_id": local_family_id,
            },
            "routing": "local"
        },
        "id": 1
    });

    let response = http_post_jsonrpc(&endpoint, &request)
        .await
        .map_err(|e| format!("Failed to reach peer {peer_id} at {endpoint}: {e}"))?;

    if let Some(error) = response.get("error") {
        let msg = error.get("message").and_then(Value::as_str).unwrap_or("unknown error");
        return Err(format!("Remote {peer_id} rejected trust exchange: {msg}"));
    }

    let result = response.get("result").ok_or("No result in response")?;

    // Handle nested capability.call result structure
    let inner = result.get("result").unwrap_or(result);

    let public_key = inner
        .get("local_public_key")
        .and_then(Value::as_str)
        .ok_or("Missing local_public_key in remote response")?
        .to_string();

    let did = inner.get("local_did").and_then(Value::as_str).unwrap_or("").to_string();

    let gate_id = inner.get("local_gate_id").and_then(Value::as_str).unwrap_or(peer_id).to_string();

    Ok(RemoteKeyMaterial {
        public_key,
        did,
        gate_id,
    })
}

/// Register a remote gate's key on our local security provider.
async fn register_remote_key(remote: &RemoteKeyMaterial) -> Result<(), String> {
    let socket_path = discover_security_provider_socket()?;

    let request = json!({
        "jsonrpc": "2.0",
        "method": "auth.exchange_trust",
        "params": {
            "public_key": remote.public_key,
            "did": remote.did,
            "gate_id": remote.gate_id,
        },
        "id": 1
    });

    let response = call_uds_jsonrpc(&socket_path, &request).await?;

    if let Some(error) = response.get("error") {
        let msg = error
            .get("message")
            .and_then(Value::as_str)
            .or_else(|| response.get("error").and_then(Value::as_str))
            .unwrap_or("unknown");
        return Err(format!("Local security provider registration failed: {msg}"));
    }

    let registered = response.get("registered").and_then(Value::as_bool).unwrap_or(false);
    if registered {
        info!(
            remote_gate = %remote.gate_id,
            remote_did = %remote.did,
            "Registered remote trust issuer on local security provider"
        );
    } else {
        debug!(
            remote_gate = %remote.gate_id,
            "Remote key already registered (idempotent)"
        );
    }

    Ok(())
}

/// Discover the security provider's UDS socket via capability-based discovery.
///
/// Resolution: env vars → XDG runtime (`$XDG_RUNTIME_DIR/{BIOMEOS_RUNTIME_SUBDIR}/*.sock`)
fn discover_security_provider_socket() -> Result<String, String> {
    let path = security_crypto_ipc_socket_from_env(|| {
        if let Ok(xdg) = songbird_process_env::var("XDG_RUNTIME_DIR") {
            let biomeos_dir = std::path::PathBuf::from(&xdg)
                .join(songbird_types::defaults::paths::BIOMEOS_RUNTIME_SUBDIR);
            for name in songbird_types::defaults::paths::CRYPTO_PROVIDER_SOCKET_FILENAMES_XDG {
                let p = biomeos_dir.join(name);
                if p.exists() {
                    return p.to_string_lossy().to_string();
                }
            }
        }
        String::new()
    });

    if path.is_empty() {
        return Err(String::from(
            "No security provider socket discovered (set SECURITY_PROVIDER_SOCKET)",
        ));
    }
    Ok(path)
}

/// Make a JSON-RPC call over IPC and return the result.
async fn call_uds_jsonrpc(socket_path: &str, request: &Value) -> Result<Value, String> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut stream = tokio::time::timeout(
        TRUST_EXCHANGE_TIMEOUT,
        songbird_types::IpcStream::connect(socket_path),
    )
    .await
    .map_err(|_| format!("Timeout connecting to {socket_path}"))?
    .map_err(|e| format!("Cannot connect to {socket_path}: {e}"))?;

    let mut bytes = serde_json::to_vec(request).map_err(|e| format!("Serialize error: {e}"))?;
    bytes.push(b'\n');

    stream.write_all(&bytes).await.map_err(|e| format!("Write error: {e}"))?;
    stream.shutdown().await.map_err(|e| format!("Shutdown error: {e}"))?;

    let mut response_buf = Vec::new();
    stream.read_to_end(&mut response_buf).await.map_err(|e| format!("Read error: {e}"))?;

    let response: Value =
        serde_json::from_slice(&response_buf).map_err(|e| format!("Parse error: {e}"))?;

    if let Some(error) = response.get("error").filter(|e| e.is_object()) {
        let msg = error.get("message").and_then(Value::as_str).unwrap_or("RPC error");
        return Err(msg.to_string());
    }

    response.get("result").cloned().ok_or_else(|| String::from("No result in JSON-RPC response"))
}

/// Send a JSON-RPC request via HTTP POST and return the full response.
async fn http_post_jsonrpc(endpoint: &str, request: &Value) -> Result<Value, String> {
    use http_body_util::{BodyExt, Full};
    use hyper::Request;
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let body_bytes = serde_json::to_vec(request).map_err(|e| format!("Serialize: {e}"))?;

    let uri: hyper::Uri = endpoint.parse().map_err(|e| format!("Invalid URI '{endpoint}': {e}"))?;

    let http_request = Request::builder()
        .method(hyper::Method::POST)
        .uri(&uri)
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(body_bytes)))
        .map_err(|e| format!("Build request: {e}"))?;

    let client = Client::builder(TokioExecutor::new()).build_http::<Full<Bytes>>();

    let response = tokio::time::timeout(TRUST_EXCHANGE_TIMEOUT, client.request(http_request))
        .await
        .map_err(|_| format!("Timeout: {endpoint}"))?
        .map_err(|e| format!("HTTP POST: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    let body =
        response.into_body().collect().await.map_err(|e| format!("Read body: {e}"))?.to_bytes();

    serde_json::from_slice(&body).map_err(|e| format!("Parse response: {e}"))
}

fn resolve_node_id() -> String {
    songbird_process_env::var("SONGBIRD_NODE_ID")
        .or_else(|_| songbird_process_env::var("NODE_ID"))
        .or_else(|_| songbird_process_env::var("HOSTNAME"))
        .unwrap_or_else(|_| gethostname::gethostname().to_string_lossy().to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::test_sync_env::env_lock;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn discover_security_provider_socket_returns_err_when_no_env() {
        let _guard = env_lock();
        songbird_process_env::remove_var("SECURITY_PROVIDER_SOCKET");
        songbird_process_env::remove_var("CRYPTO_PROVIDER_SOCKET");
        songbird_process_env::remove_var("SECURITY_SOCKET");
        songbird_process_env::remove_var("BEARDOG_SOCKET");
        songbird_process_env::remove_var("XDG_RUNTIME_DIR");

        let result = discover_security_provider_socket();
        assert!(result.is_err());
    }

    #[test]
    fn discover_security_provider_socket_uses_env() {
        let _guard = env_lock();
        let path = format!("/tmp/songbird-test-trust-exchange-{}.sock", std::process::id());
        std::fs::File::create(&path).ok();
        songbird_process_env::set_var("SECURITY_PROVIDER_SOCKET", &path);

        let result = discover_security_provider_socket();
        assert_eq!(result.unwrap(), path);

        songbird_process_env::remove_var("SECURITY_PROVIDER_SOCKET");
        std::fs::remove_file(&path).ok();
    }

    #[tokio::test]
    async fn spawn_trust_exchange_empty_peers_returns_immediately() {
        spawn_trust_exchange(vec![]);
    }

    #[test]
    fn resolve_node_id_falls_back_to_hostname() {
        let _guard = env_lock();
        songbird_process_env::remove_var("SONGBIRD_NODE_ID");
        songbird_process_env::remove_var("NODE_ID");
        songbird_process_env::remove_var("HOSTNAME");

        let id = resolve_node_id();
        assert!(!id.is_empty(), "should fallback to gethostname");
    }

    #[test]
    fn resolve_node_id_prefers_songbird_node_id() {
        let _guard = env_lock();
        songbird_process_env::set_var("SONGBIRD_NODE_ID", "test-gate-trust");
        let id = resolve_node_id();
        assert_eq!(id, "test-gate-trust");
        songbird_process_env::remove_var("SONGBIRD_NODE_ID");
    }

    #[tokio::test]
    async fn call_uds_jsonrpc_fails_on_nonexistent_socket() {
        let result = call_uds_jsonrpc("/tmp/nonexistent-beardog-trust-test.sock", &json!({})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn spawn_trust_exchange_with_peers_does_not_panic() {
        let peers = vec![(
            String::from("test-peer"),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 7700),
        )];
        spawn_trust_exchange(peers);
        tokio::task::yield_now().await;
    }
}
