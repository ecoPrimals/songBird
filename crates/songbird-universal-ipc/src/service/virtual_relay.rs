// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Virtual Endpoint Relay — Phase 2 (Default Mode)
//!
//! Creates per-primal relay UDS sockets under `$XDG_RUNTIME_DIR/biomeos/songbird/virtual/`.
//! Each relay socket accepts JSON-RPC connections and transparently forwards them to the
//! primal's native endpoint. Phase 2: virtual endpoints are the default in `ipc.resolve`
//! (opt-out via `native: true`).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use songbird_types::defaults::timeouts::DEFAULT_SOCKET_IO_TIMEOUT;

/// Manages virtual relay listeners for registered primals.
///
/// Each registered primal gets a relay socket that proxies JSON-RPC to its native endpoint.
pub struct VirtualRelayManager {
    relays: RwLock<HashMap<String, RelayEntry>>,
    base_dir: PathBuf,
    /// Shared relay metrics (request count, total overhead in microseconds).
    metrics: Arc<RelayMetrics>,
    /// Signature verifier for Phase 3.5 (defaults to noop — accepts all).
    signature_verifier: Arc<dyn BtspSignatureVerifier>,
}

/// Relay performance metrics (shared atomically across all relay tasks).
pub struct RelayMetrics {
    /// Total number of requests relayed.
    pub requests: AtomicU64,
    /// Cumulative relay overhead in microseconds (time spent in relay, excluding native processing).
    pub overhead_us: AtomicU64,
}

impl RelayMetrics {
    fn new() -> Self {
        Self {
            requests: AtomicU64::new(0),
            overhead_us: AtomicU64::new(0),
        }
    }

    /// Average overhead per request in microseconds.
    pub fn avg_overhead_us(&self) -> u64 {
        let count = self.requests.load(Ordering::Relaxed);
        if count == 0 {
            return 0;
        }
        self.overhead_us.load(Ordering::Relaxed) / count
    }
}

struct RelayEntry {
    socket_path: PathBuf,
    #[allow(dead_code)]
    native_target: String,
    task: JoinHandle<()>,
}

impl VirtualRelayManager {
    /// Create a new relay manager with the given base directory for virtual sockets.
    ///
    /// Base directory is typically `$XDG_RUNTIME_DIR/biomeos/songbird/virtual/`.
    #[must_use]
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            relays: RwLock::new(HashMap::new()),
            base_dir,
            metrics: Arc::new(RelayMetrics::new()),
            signature_verifier: Arc::new(UnavailableVerifier),
        }
    }

    /// Create with Phase 3.5 signature verification via bearDog crypto socket.
    ///
    /// If `crypto_socket` is provided, relay requests with Ed25519 signed BTSP
    /// tokens will be cryptographically verified. If unreachable at request time,
    /// degrades to trust-on-accept.
    #[must_use]
    pub fn with_crypto_verifier(base_dir: PathBuf, crypto_socket: Option<String>) -> Self {
        let signature_verifier: Arc<dyn BtspSignatureVerifier> = if let Some(path) = crypto_socket {
            tracing::info!(
                socket = %path,
                "Phase 3.5: Relay signature verification enabled via CryptoProvider"
            );
            Arc::new(super::relay_security::CryptoProviderVerifier::new(path))
        } else {
            tracing::warn!(
                "Phase 3.5: No crypto socket — signed relay requests will be rejected until provider available"
            );
            Arc::new(UnavailableVerifier)
        };

        Self {
            relays: RwLock::new(HashMap::new()),
            base_dir,
            metrics: Arc::new(RelayMetrics::new()),
            signature_verifier,
        }
    }

    /// Replace the signature verifier (Phase 3.5: inject bearDog-backed verifier).
    pub fn set_signature_verifier(&mut self, verifier: Arc<dyn BtspSignatureVerifier>) {
        self.signature_verifier = verifier;
    }

    /// Determine the base directory for virtual relay sockets.
    ///
    /// Priority: `$XDG_RUNTIME_DIR/biomeos/songbird/virtual/` → `{temp_dir}/biomeos/songbird/virtual/`
    #[must_use]
    pub fn default_base_dir() -> PathBuf {
        let base = if let Ok(xdg) = songbird_process_env::var("XDG_RUNTIME_DIR") {
            PathBuf::from(xdg)
        } else {
            std::env::temp_dir()
        };
        base.join("biomeos").join("songbird").join("virtual")
    }

    /// Get the relay socket path for a given primal name.
    #[must_use]
    pub fn relay_socket_path(&self, primal_name: &str) -> PathBuf {
        self.base_dir.join(format!("{primal_name}.sock"))
    }

    /// Start a relay for the given primal.
    ///
    /// Creates a UDS listener at `<base_dir>/<primal_name>.sock` that forwards all
    /// JSON-RPC requests to `native_socket_path`.
    pub async fn start_relay(
        &self,
        primal_name: &str,
        native_socket_path: &str,
    ) -> anyhow::Result<PathBuf> {
        let socket_path = self.relay_socket_path(primal_name);

        // Ensure parent directory exists
        if let Some(parent) = socket_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Remove stale socket
        let _ = tokio::fs::remove_file(&socket_path).await;

        let listener = UnixListener::bind(&socket_path)?;
        let native_target = native_socket_path.to_string();
        let relay_path = socket_path.clone();

        tracing::info!(
            primal = primal_name,
            virtual_socket = %relay_path.display(),
            native_target = %native_target,
            "Virtual relay listener started (Phase 1 shadow mode)"
        );

        let task = tokio::spawn(relay_accept_loop(
            listener,
            native_target.clone(),
            Arc::clone(&self.metrics),
            Arc::clone(&self.signature_verifier),
        ));

        let mut relays = self.relays.write().await;
        relays.insert(
            primal_name.to_string(),
            RelayEntry {
                socket_path: relay_path.clone(),
                native_target,
                task,
            },
        );

        Ok(relay_path)
    }

    /// Stop and remove a relay for the given primal.
    pub async fn stop_relay(&self, primal_name: &str) {
        let mut relays = self.relays.write().await;
        if let Some(entry) = relays.remove(primal_name) {
            entry.task.abort();
            let _ = tokio::fs::remove_file(&entry.socket_path).await;
            tracing::info!(primal = primal_name, "Virtual relay stopped");
        }
    }

    /// Check if a relay is active for the given primal.
    pub async fn has_relay(&self, primal_name: &str) -> bool {
        self.relays.read().await.contains_key(primal_name)
    }

    /// Get the relay socket path if active, otherwise None.
    pub async fn get_relay_path(&self, primal_name: &str) -> Option<PathBuf> {
        self.relays.read().await.get(primal_name).map(|e| e.socket_path.clone())
    }

    /// List all active relays.
    pub async fn list_relays(&self) -> Vec<(String, PathBuf)> {
        self.relays
            .read()
            .await
            .iter()
            .map(|(name, entry)| (name.clone(), entry.socket_path.clone()))
            .collect()
    }

    /// Access relay performance metrics.
    #[must_use]
    pub fn metrics(&self) -> &Arc<RelayMetrics> {
        &self.metrics
    }

    /// Cleanup all relay sockets on shutdown.
    pub async fn shutdown(&self) {
        let mut relays = self.relays.write().await;
        for (name, entry) in relays.drain() {
            entry.task.abort();
            let _ = tokio::fs::remove_file(&entry.socket_path).await;
            tracing::debug!(primal = %name, "Virtual relay cleaned up on shutdown");
        }
    }
}

impl Drop for VirtualRelayManager {
    fn drop(&mut self) {
        let relays = self.relays.get_mut();
        for (_, entry) in relays.drain() {
            entry.task.abort();
            let _ = std::fs::remove_file(&entry.socket_path);
        }
    }
}

/// Accept loop for a virtual relay listener.
///
/// Each accepted connection is spawned as an independent relay task.
async fn relay_accept_loop(
    listener: UnixListener,
    native_target: String,
    metrics: Arc<RelayMetrics>,
    verifier: Arc<dyn BtspSignatureVerifier>,
) {
    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let target = native_target.clone();
                let m = Arc::clone(&metrics);
                let v = Arc::clone(&verifier);
                tokio::spawn(async move {
                    if let Err(e) = relay_connection(stream, &target, &m, v.as_ref()).await {
                        tracing::debug!(error = %e, "Virtual relay connection ended");
                    }
                });
            }
            Err(e) => {
                tracing::warn!(error = %e, "Virtual relay accept error");
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
        }
    }
}

// ─── Phase 3.5: Cryptographic Verification Trait ────────────────────────────
//
// When bearDog delivers the CryptoProvider integration design, implement this
// trait against bearDog IPC (`crypto.verify_signature`). Until then, the relay
// uses structural + temporal validation only (Phase 3).

/// Trait for verifying Ed25519 signatures on BTSP relay tokens.
///
/// Implementations may call bearDog via IPC, use an in-process Ed25519 library,
/// or delegate to any signing authority. The relay calls `verify` only when a
/// structured token (payload.signature) is present.
///
/// Object-safe: uses boxed futures for dynamic dispatch.
pub trait BtspSignatureVerifier: Send + Sync + 'static {
    /// Verify that `signature_bytes` is a valid Ed25519 signature over
    /// `payload_bytes` from the node identified by `node_id`.
    ///
    /// Returns `Ok(true)` if signature is valid, `Ok(false)` if invalid,
    /// or `Err` if the verifier is unavailable (e.g., bearDog offline).
    fn verify(
        &self,
        node_id: &str,
        payload_bytes: &[u8],
        signature_bytes: &[u8],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send + '_>>;
}

/// Strict verifier used when no crypto provider is available at startup.
///
/// Returns `Err` for any verification attempt, causing the relay to reject
/// signed requests until a crypto provider becomes available. This is the
/// secure default — unsigned Phase 2 tokens still pass through (no signature
/// to verify), but signed Phase 3.5 tokens cannot be validated without a provider.
pub(crate) struct UnavailableVerifier;

impl BtspSignatureVerifier for UnavailableVerifier {
    fn verify(
        &self,
        _node_id: &str,
        _payload_bytes: &[u8],
        _signature_bytes: &[u8],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send + '_>>
    {
        Box::pin(async {
            Err("No crypto provider available — signature verification unavailable".to_string())
        })
    }
}

/// Test-only verifier that accepts all signatures unconditionally.
#[cfg(test)]
pub(crate) struct NoopSignatureVerifier;

#[cfg(test)]
impl BtspSignatureVerifier for NoopSignatureVerifier {
    fn verify(
        &self,
        _node_id: &str,
        _payload_bytes: &[u8],
        _signature_bytes: &[u8],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send + '_>>
    {
        Box::pin(async { Ok(true) })
    }
}

use super::btsp_validation::{BtspValidation, validate_btsp_session};

/// Relay a single client connection: maintains a persistent native connection for the
/// session lifetime. Requests stream from client → native, responses stream back.
///
/// Connection pooling: one native UDS connection is held open for the entire client
/// session (NDJSON streaming). Reconnects automatically on native connection failure.
/// Measures relay overhead (time spent in relay logic, excluding native I/O wait).
async fn relay_connection(
    client_stream: UnixStream,
    native_target: &str,
    metrics: &RelayMetrics,
    verifier: &dyn BtspSignatureVerifier,
) -> anyhow::Result<()> {
    let (client_reader, mut client_writer) = client_stream.into_split();
    let mut client_buf = BufReader::new(client_reader);

    // Establish persistent connection to native endpoint
    let mut native_conn = connect_native(native_target).await.ok();

    let mut line = String::new();
    loop {
        line.clear();
        let bytes_read = tokio::time::timeout(
            std::time::Duration::from_secs(300),
            client_buf.read_line(&mut line),
        )
        .await??;

        if bytes_read == 0 {
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // BTSP session validation (Phase 3: structural + timestamp verification)
        match validate_btsp_session(trimmed) {
            Err(reject) => {
                let mut reject_bytes = serde_json::to_vec(&reject)?;
                reject_bytes.push(b'\n');
                client_writer.write_all(&reject_bytes).await?;
                tracing::warn!(target: "relay_audit", "BTSP validation rejected request");
                continue;
            }
            Ok(BtspValidation::Valid {
                ref node_id,
                ref payload_bytes,
                ref signature_bytes,
            }) => {
                if let Some(rejection) = super::relay_security::verify_relay_signature(
                    verifier,
                    node_id.as_deref(),
                    payload_bytes,
                    signature_bytes,
                    trimmed,
                    native_target,
                )
                .await
                {
                    let mut reject_bytes = serde_json::to_vec(&rejection)?;
                    reject_bytes.push(b'\n');
                    client_writer.write_all(&reject_bytes).await?;
                    continue;
                }
            }
            Ok(BtspValidation::NoToken) => {}
        }

        let relay_start = std::time::Instant::now();

        // Try persistent connection first, reconnect on failure
        let response = if let Some(conn) = &mut native_conn {
            if let Ok(resp) = forward_on_persistent(trimmed, conn).await {
                resp
            } else {
                // Reconnect and retry once
                native_conn = connect_native(native_target).await.ok();
                forward_or_fallback(trimmed, &mut native_conn, native_target).await
            }
        } else {
            native_conn = connect_native(native_target).await.ok();
            forward_or_fallback(trimmed, &mut native_conn, native_target).await
        };

        let mut response_bytes = serde_json::to_vec(&response)?;
        response_bytes.push(b'\n');
        client_writer.write_all(&response_bytes).await?;

        // Record metrics (overhead includes serialization + write, not client read wait)
        let elapsed_us = u64::try_from(relay_start.elapsed().as_micros()).unwrap_or(u64::MAX);
        metrics.requests.fetch_add(1, Ordering::Relaxed);
        metrics.overhead_us.fetch_add(elapsed_us, Ordering::Relaxed);
    }

    Ok(())
}

/// Forward on existing connection or fall back to a fresh one-shot connection.
async fn forward_or_fallback(
    request_line: &str,
    native_conn: &mut Option<NativeConn>,
    native_target: &str,
) -> serde_json::Value {
    if let Some(conn) = native_conn {
        forward_on_persistent(request_line, conn)
            .await
            .unwrap_or_else(|e| make_error_response(request_line, &e))
    } else {
        forward_fresh(request_line, native_target).await
    }
}

/// Persistent native connection state (writer + buffered reader).
struct NativeConn {
    writer: tokio::net::unix::OwnedWriteHalf,
    reader: BufReader<tokio::net::unix::OwnedReadHalf>,
}

/// Establish a connection to the native endpoint.
async fn connect_native(native_target: &str) -> anyhow::Result<NativeConn> {
    let stream =
        tokio::time::timeout(DEFAULT_SOCKET_IO_TIMEOUT, UnixStream::connect(native_target))
            .await
            .map_err(|_| anyhow::anyhow!("Timeout connecting to native endpoint"))?
            .map_err(|e| anyhow::anyhow!("Cannot connect to native endpoint: {e}"))?;

    let (reader, writer) = stream.into_split();
    Ok(NativeConn {
        writer,
        reader: BufReader::new(reader),
    })
}

/// Forward a request on a persistent native connection.
async fn forward_on_persistent(
    request_line: &str,
    conn: &mut NativeConn,
) -> anyhow::Result<serde_json::Value> {
    let mut request_bytes = request_line.as_bytes().to_vec();
    if !request_bytes.ends_with(b"\n") {
        request_bytes.push(b'\n');
    }

    tokio::time::timeout(DEFAULT_SOCKET_IO_TIMEOUT, conn.writer.write_all(&request_bytes))
        .await
        .map_err(|_| anyhow::anyhow!("Timeout writing to native endpoint"))?
        .map_err(|e| anyhow::anyhow!("Write error: {e}"))?;

    let mut response_line = String::new();
    tokio::time::timeout(DEFAULT_SOCKET_IO_TIMEOUT, conn.reader.read_line(&mut response_line))
        .await
        .map_err(|_| anyhow::anyhow!("Timeout reading from native endpoint"))?
        .map_err(|e| anyhow::anyhow!("Read error: {e}"))?;

    if response_line.is_empty() {
        return Err(anyhow::anyhow!("Native endpoint closed connection"));
    }

    serde_json::from_str(response_line.trim())
        .map_err(|e| anyhow::anyhow!("Invalid JSON from native provider: {e}"))
}

/// Fallback: open a fresh connection for a single request (no pooling).
async fn forward_fresh(request_line: &str, native_target: &str) -> serde_json::Value {
    match forward_fresh_inner(request_line, native_target).await {
        Ok(response) => response,
        Err(e) => make_error_response(request_line, &e),
    }
}

async fn forward_fresh_inner(
    request_line: &str,
    native_target: &str,
) -> anyhow::Result<serde_json::Value> {
    let mut conn = connect_native(native_target).await?;
    forward_on_persistent(request_line, &mut conn).await
}

fn make_error_response(request_line: &str, error: &anyhow::Error) -> serde_json::Value {
    let id = serde_json::from_str::<serde_json::Value>(request_line)
        .ok()
        .and_then(|v| v.get("id").cloned())
        .unwrap_or(serde_json::Value::Null);
    serde_json::json!({
        "jsonrpc": "2.0",
        "error": {
            "code": -32603,
            "message": format!("Relay error: {error}")
        },
        "id": id
    })
}

/// Resolve the relay socket path for a primal given the base directory.
///
/// Utility for callers who don't have a `VirtualRelayManager` reference.
#[must_use]
pub fn virtual_socket_path(base_dir: &Path, primal_name: &str) -> PathBuf {
    base_dir.join(format!("{primal_name}.sock"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_base_dir_under_xdg() {
        let dir = VirtualRelayManager::default_base_dir();
        let path_str = dir.to_string_lossy();
        assert!(path_str.contains("biomeos/songbird/virtual"));
    }

    #[test]
    fn relay_socket_path_format() {
        let mgr =
            VirtualRelayManager::new(PathBuf::from("/run/user/1000/biomeos/songbird/virtual"));
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
        let relay_path =
            mgr.start_relay("mock-primal", native_path.to_str().unwrap()).await.unwrap();

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
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<bool, String>> + Send + '_>,
            > {
                Box::pin(async { Ok(false) })
            }
        }

        let mut mgr = VirtualRelayManager::new(dir.path().to_path_buf());
        mgr.set_signature_verifier(Arc::new(RejectAllVerifier));
        let relay_path =
            mgr.start_relay("tamper-test", native_path.to_str().unwrap()).await.unwrap();

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
        let now =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
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
            response["error"]["message"]
                .as_str()
                .unwrap()
                .contains("signature verification failed")
        );
        assert_eq!(response["id"], 99);
        // The native mock should NOT have received the request
        assert!(response.get("result").is_none());

        mgr.stop_relay("tamper-test").await;
        mock_handle.abort();
    }
}
