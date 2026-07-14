// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Production Security Provider - Unix Socket Implementation
//!
//! Implements all security provider integration traits via Unix socket JSON-RPC.
//! This is the production provider that connects to a real security provider instance.
//!
//! ## Deep Debt Compliance
//!
//! - ✅ Pure Rust (Unix sockets, no HTTP/reqwest)
//! - ✅ Zero unsafe code
//! - ✅ Runtime discovery (socket path from env/discovery)
//! - ✅ Modern async Rust (trait-based, async/await)
//! - ✅ Graceful error handling

use super::{
    AccessLevel, BroadcastKey, EncryptedBirdSong, LineageChain, LineageHint, LineageProof,
    RelaySession,
};
use anyhow::{Context, Result, anyhow};
use serde_json::Value;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, info, warn};

#[cfg(unix)]
use tokio::net::UnixStream as IpcStream;
#[cfg(windows)]
use tokio::net::TcpStream as IpcStream;

/// Production security provider via Unix socket JSON-RPC
///
/// Connects to the security provider's Unix socket to provide:
/// - Lineage management and verification
/// - `BirdSong` encryption/decryption
/// - Relay session management
///
/// ## Usage
///
/// ```rust,no_run
/// use songbird_network_federation::security::production::ProductionSecurityProvider;
///
/// # async fn example() -> anyhow::Result<()> {
/// let provider = ProductionSecurityProvider::new("/tmp/security.sock").await?;
/// # Ok(())
/// # }
/// ```
pub struct ProductionSecurityProvider {
    socket_path: PathBuf,
    family_id: Option<String>,
}

impl ProductionSecurityProvider {
    /// Create new production `security provider` provider
    ///
    /// # Arguments
    ///
    /// * `socket_path` - Path to `security provider` Unix socket (discovered at runtime)
    ///
    /// # Errors
    ///
    /// Returns error if socket connection fails or health check fails
    pub async fn new(socket_path: impl Into<PathBuf>) -> Result<Self> {
        let socket_path = socket_path.into();

        info!("Creating production security provider (IPC)");
        info!("   Socket: {:?}", socket_path);

        let _ = Self::connect_ipc(&socket_path)
            .await
            .context("security provider socket not accessible")?;

        Ok(Self {
            socket_path,
            family_id: None,
        })
    }

    /// Create new production `security provider` provider with explicit `family_id`
    ///
    /// Use this when the `family_id` is known at construction time.
    pub async fn with_family_id(
        socket_path: impl Into<PathBuf>,
        family_id: impl Into<String>,
    ) -> Result<Self> {
        let socket_path = socket_path.into();
        let family_id = family_id.into();

        info!("Creating production security provider with family_id");
        info!("   Socket: {:?}", socket_path);
        info!("   Family: {}", family_id);

        let _ = Self::connect_ipc(&socket_path)
            .await
            .context("security provider socket not accessible")?;

        Ok(Self {
            socket_path,
            family_id: Some(family_id),
        })
    }

    /// Set the `family_id` for `BirdSong` operations
    pub fn set_family_id(&mut self, family_id: impl Into<String>) {
        self.family_id = Some(family_id.into());
    }

    #[cfg(unix)]
    async fn connect_ipc(path: &std::path::Path) -> Result<IpcStream> {
        Ok(IpcStream::connect(path).await?)
    }

    #[cfg(windows)]
    async fn connect_ipc(path: &std::path::Path) -> Result<IpcStream> {
        let port: u16 = tokio::fs::read_to_string(path)
            .await
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(songbird_types::defaults::ports::DEFAULT_HTTP_PORT);
        let addr = format!("127.0.0.1:{port}");
        Ok(IpcStream::connect(&addr).await?)
    }

    /// Call security provider JSON-RPC method via IPC.
    async fn call_security_rpc(&self, method: &str, params: Value) -> Result<Value> {
        debug!("Calling security provider RPC: {}", method);

        let mut stream = Self::connect_ipc(&self.socket_path)
            .await
            .context("Failed to connect to security provider socket")?;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let request_bytes = serde_json::to_vec(&request)?;
        stream.write_all(&request_bytes).await?;
        stream.write_all(b"\n").await.ok();

        let mut response_bytes = Vec::new();
        stream.read_to_end(&mut response_bytes).await?;

        let response: Value = serde_json::from_slice(&response_bytes)
            .context("Invalid JSON response from security provider")?;

        if let Some(error) = response.get("error") {
            return Err(anyhow!("Security provider RPC error: {error}"));
        }

        response
            .get("result")
            .cloned()
            .ok_or_else(|| anyhow!("No result in security provider response"))
    }

    /// Generate lineage for a new node
    pub async fn generate_lineage(&self, node_id: &str, parent_id: &str) -> Result<LineageChain> {
        let params = serde_json::json!({
            "node_id": node_id,
            "parent_id": parent_id
        });

        let result = self.call_security_rpc("genetic.generate_lineage", params).await?;
        serde_json::from_value(result).context("Failed to parse lineage chain")
    }

    /// Verify a lineage proof
    pub async fn verify_lineage(&self, proof: &LineageProof) -> Result<bool> {
        let params = serde_json::json!({
            "proof": proof
        });

        let result = self.call_security_rpc("genetic.verify_lineage", params).await?;
        result
            .get("valid")
            .and_then(serde_json::Value::as_bool)
            .ok_or_else(|| anyhow!("Invalid verify_lineage response"))
    }

    /// Get all descendants of a root
    pub async fn get_descendants(&self, root_id: &str) -> Result<Vec<String>> {
        let params = serde_json::json!({
            "root_id": root_id
        });

        let result = self.call_security_rpc("genetic.get_descendants", params).await?;
        serde_json::from_value(result).context("Failed to parse descendants")
    }

    /// Get lineage depth between two nodes
    pub async fn get_lineage_depth(
        &self,
        ancestor_id: &str,
        descendant_id: &str,
    ) -> Result<Option<usize>> {
        let params = serde_json::json!({
            "ancestor_id": ancestor_id,
            "descendant_id": descendant_id
        });

        let result = self.call_security_rpc("genetic.get_lineage_depth", params).await?;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "lineage depth from JSON u64 fits usize on supported platforms"
        )]
        Ok(result.get("depth").and_then(serde_json::Value::as_u64).map(|d| d as usize))
    }

    /// Encrypt payload for a specific lineage
    pub async fn encrypt_for_lineage(
        &self,
        payload: &[u8],
        lineage_hint: LineageHint,
    ) -> Result<EncryptedBirdSong> {
        use base64::{Engine as _, engine::general_purpose};

        // Get family_id from self, env vars, or default (canonical chain)
        let family_id = self
            .family_id
            .clone()
            .or_else(|| {
                songbird_process_env::var("SONGBIRD_ORCHESTRATOR_FAMILY_ID")
                    .or_else(|_| songbird_process_env::var("BIOMEOS_FAMILY_ID"))
                    .or_else(|_| songbird_process_env::var("SONGBIRD_FAMILY_ID"))
                    .or_else(|_| songbird_process_env::var("FAMILY_ID"))
                    .ok()
            })
            .unwrap_or_else(|| String::from("default"));

        let params = serde_json::json!({
            "plaintext": general_purpose::STANDARD.encode(payload),
            "lineage_hint": format!("{:?}", lineage_hint),
            "family_id": family_id
        });

        let result = self.call_security_rpc("birdsong.encrypt", params).await?;
        serde_json::from_value(result).context("Failed to parse encrypted birdsong")
    }

    /// Decrypt birdSong (if we're in the lineage)
    pub async fn decrypt_birdsong(&self, encrypted: &EncryptedBirdSong) -> Result<Option<Vec<u8>>> {
        // Get family_id from self, env vars, or default (canonical chain)
        let family_id = self
            .family_id
            .clone()
            .or_else(|| {
                songbird_process_env::var("SONGBIRD_ORCHESTRATOR_FAMILY_ID")
                    .or_else(|_| songbird_process_env::var("BIOMEOS_FAMILY_ID"))
                    .or_else(|_| songbird_process_env::var("SONGBIRD_FAMILY_ID"))
                    .or_else(|_| songbird_process_env::var("FAMILY_ID"))
                    .ok()
            })
            .unwrap_or_else(|| String::from("default"));

        let params = serde_json::json!({
            "encrypted": encrypted,
            "family_id": family_id
        });

        let result = self.call_security_rpc("birdsong.decrypt", params).await?;

        let success = result.get("success").and_then(serde_json::Value::as_bool).unwrap_or(false);
        if !success {
            return Ok(None); // Different family (noise)
        }

        use base64::{Engine as _, engine::general_purpose};
        let plaintext_b64 = result
            .get("plaintext")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("No plaintext in decrypt response"))?;

        let plaintext = general_purpose::STANDARD.decode(plaintext_b64)?;
        Ok(Some(plaintext))
    }

    /// Request decryption key for a lineage
    pub async fn request_key(
        &self,
        lineage_hint: &LineageHint,
        proof: LineageProof,
    ) -> Result<BroadcastKey> {
        let params = serde_json::json!({
            "lineage_hint": format!("{:?}", lineage_hint),
            "proof": proof
        });

        let result = self.call_security_rpc("birdsong.request_key", params).await?;
        serde_json::from_value(result).context("Failed to parse broadcast key")
    }

    /// Batch key request (for efficiency)
    pub async fn request_keys_batch(
        &self,
        requests: Vec<(LineageHint, LineageProof)>,
    ) -> Result<Vec<BroadcastKey>> {
        let params = serde_json::json!({
            "requests": requests
        });

        let result = self.call_security_rpc("birdsong.request_keys_batch", params).await?;
        serde_json::from_value(result).context("Failed to parse broadcast keys")
    }

    /// Offer relay service to descendant
    pub async fn offer_relay(
        &self,
        requester: &str,
        target: &str,
        lineage_proof: LineageProof,
    ) -> Result<RelaySession> {
        let params = serde_json::json!({
            "requester": requester,
            "target": target,
            "lineage_proof": lineage_proof
        });

        let result = self.call_security_rpc("relay.offer", params).await?;
        serde_json::from_value(result).context("Failed to parse relay session")
    }

    /// Get visibility level based on lineage depth
    #[must_use]
    pub fn get_visibility_level(&self, lineage_depth: usize) -> AccessLevel {
        AccessLevel::from_lineage_depth(lineage_depth)
    }

    /// Relay packet (with masking enforced)
    pub async fn relay_packet(&self, session: &RelaySession, packet: &[u8]) -> Result<()> {
        use base64::{Engine as _, engine::general_purpose};

        let params = serde_json::json!({
            "session_id": session.session_id,
            "packet": general_purpose::STANDARD.encode(packet)
        });

        self.call_security_rpc("relay.relay_packet", params).await?;
        Ok(())
    }

    /// Revoke relay for a session
    pub async fn revoke_relay(&self, session_id: &str) -> Result<()> {
        let params = serde_json::json!({
            "session_id": session_id
        });

        self.call_security_rpc("relay.revoke", params).await?;
        Ok(())
    }

    /// Check if the provider is available and operational
    pub async fn is_available(&self) -> bool {
        // Try health check
        match self.call_security_rpc("health", serde_json::json!({})).await {
            Ok(result) => {
                result.get("status").and_then(|v| v.as_str()).is_some_and(|s| s == "healthy")
            }
            Err(e) => {
                warn!("security provider health check failed: {}", e);
                false
            }
        }
    }

    /// Provider version for compatibility checking
    #[must_use]
    pub fn version(&self) -> &'static str {
        "production-unix-socket"
    }

    /// Graceful shutdown
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down security provider provider connection");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;
    use chrono::Utc;
    use serde_json::json;
    use std::sync::Arc;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;

    type RpcHandler = Arc<dyn Fn(&str, &Value) -> Value + Send + Sync>;

    async fn spawn_mock_security_server(path: PathBuf, handler: RpcHandler) {
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let listener = UnixListener::bind(&path).unwrap();
            let _ = ready_tx.send(());
            while let Ok((stream, _)) = listener.accept().await {
                let handler = Arc::clone(&handler);
                tokio::spawn(async move {
                    let mut reader = BufReader::new(stream);
                    let mut line = String::new();
                    if reader.read_line(&mut line).await.is_err() {
                        return;
                    }
                    let request: Value =
                        serde_json::from_str(line.trim()).unwrap_or_else(|_| json!({}));
                    let method = request["method"].as_str().unwrap_or("");
                    let id = request["id"].clone();
                    let result = handler(method, request.get("params").unwrap_or(&Value::Null));
                    let response = json!({ "jsonrpc": "2.0", "result": result, "id": id });
                    let mut stream = reader.into_inner();
                    let _ = stream.write_all(&serde_json::to_vec(&response).unwrap()).await;
                });
            }
        });
        ready_rx.await.unwrap();
    }

    fn temp_socket_path(label: &str) -> PathBuf {
        std::env::temp_dir()
            .join(format!("songbird_prod_sec_{label}_{}.sock", uuid::Uuid::new_v4()))
    }

    fn sample_lineage_chain() -> LineageChain {
        LineageChain {
            root_id: "root".into(),
            node_id: "child".into(),
            links: vec![],
            depth: 0,
        }
    }

    fn sample_lineage_proof() -> LineageProof {
        LineageProof {
            chain: sample_lineage_chain(),
            claimer_signature: vec![1, 2, 3],
        }
    }

    #[tokio::test]
    async fn test_production_provider_creation() {
        // Test with non-existent socket (should error gracefully)
        let result =
            ProductionSecurityProvider::new("/tmp/nonexistent_security_provider_test.sock").await;
        assert!(result.is_err(), "Should error when socket doesn't exist");
    }

    #[tokio::test]
    async fn production_connects_to_bound_unix_socket_and_exposes_metadata() {
        let path = std::env::temp_dir()
            .join(format!("songbird_prod_sec_test_{}.sock", uuid::Uuid::new_v4()));
        let _ = std::fs::remove_file(&path);
        let listener = UnixListener::bind(&path).unwrap();
        let accept_task = tokio::spawn(async move {
            listener.accept().await.unwrap();
        });

        let mut provider = ProductionSecurityProvider::new(&path).await.unwrap();
        accept_task.await.unwrap();

        assert_eq!(provider.version(), "production-unix-socket");
        assert_eq!(provider.get_visibility_level(0), AccessLevel::FullLineage);
        assert_eq!(provider.get_visibility_level(2), AccessLevel::SubMasked);
        assert_eq!(provider.get_visibility_level(7), AccessLevel::Masked);

        provider.set_family_id("family-x");
        provider.shutdown().await.unwrap();

        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn with_family_id_stores_family_on_connect() {
        let path = temp_socket_path("family");
        let _ = std::fs::remove_file(&path);
        let listener = UnixListener::bind(&path).unwrap();
        let accept_task = tokio::spawn(async move {
            listener.accept().await.unwrap();
        });

        let provider =
            ProductionSecurityProvider::with_family_id(&path, "family-test").await.unwrap();
        accept_task.await.unwrap();
        assert_eq!(provider.version(), "production-unix-socket");
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn is_available_true_when_health_healthy() {
        let path = temp_socket_path("health_ok");
        let _ = std::fs::remove_file(&path);
        spawn_mock_security_server(
            path.clone(),
            Arc::new(|method, _| {
                if method == "health" {
                    json!({ "status": "healthy" })
                } else {
                    json!({})
                }
            }),
        )
        .await;

        let provider = ProductionSecurityProvider::new(&path).await.unwrap();
        assert!(provider.is_available().await);
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn is_available_false_when_health_degraded() {
        let path = temp_socket_path("health_bad");
        let _ = std::fs::remove_file(&path);
        spawn_mock_security_server(
            path.clone(),
            Arc::new(|method, _| {
                if method == "health" {
                    json!({ "status": "degraded" })
                } else {
                    json!({})
                }
            }),
        )
        .await;

        let provider = ProductionSecurityProvider::new(&path).await.unwrap();
        assert!(!provider.is_available().await);
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn verify_lineage_returns_rpc_valid_flag() {
        let path = temp_socket_path("verify");
        let _ = std::fs::remove_file(&path);
        spawn_mock_security_server(
            path.clone(),
            Arc::new(|method, _| {
                if method == "genetic.verify_lineage" {
                    json!({ "valid": true })
                } else {
                    json!({})
                }
            }),
        )
        .await;

        let provider = ProductionSecurityProvider::new(&path).await.unwrap();
        let proof = sample_lineage_proof();
        assert!(provider.verify_lineage(&proof).await.unwrap());
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn generate_lineage_parses_chain_response() {
        let path = temp_socket_path("gen_lineage");
        let _ = std::fs::remove_file(&path);
        let chain = sample_lineage_chain();
        let chain_json = serde_json::to_value(&chain).unwrap();
        spawn_mock_security_server(
            path.clone(),
            Arc::new(move |method, _| {
                if method == "genetic.generate_lineage" {
                    chain_json.clone()
                } else {
                    json!({})
                }
            }),
        )
        .await;

        let provider = ProductionSecurityProvider::new(&path).await.unwrap();
        let result = provider.generate_lineage("child", "root").await.unwrap();
        assert_eq!(result.root_id, "root");
        assert_eq!(result.node_id, "child");
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn get_descendants_parses_node_list() {
        let path = temp_socket_path("descendants");
        let _ = std::fs::remove_file(&path);
        spawn_mock_security_server(
            path.clone(),
            Arc::new(|method, _| {
                if method == "genetic.get_descendants" {
                    json!(["a", "b", "c"])
                } else {
                    json!({})
                }
            }),
        )
        .await;

        let provider = ProductionSecurityProvider::new(&path).await.unwrap();
        let nodes = provider.get_descendants("root").await.unwrap();
        assert_eq!(nodes, vec!["a", "b", "c"]);
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn get_lineage_depth_parses_optional_depth() {
        let path = temp_socket_path("depth");
        let _ = std::fs::remove_file(&path);
        spawn_mock_security_server(
            path.clone(),
            Arc::new(|method, _| {
                if method == "genetic.get_lineage_depth" {
                    json!({ "depth": 3 })
                } else {
                    json!({})
                }
            }),
        )
        .await;

        let provider = ProductionSecurityProvider::new(&path).await.unwrap();
        assert_eq!(provider.get_lineage_depth("a", "d").await.unwrap(), Some(3));
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn rpc_error_surfaces_as_anyhow() {
        let path = temp_socket_path("rpc_err");
        let _ = std::fs::remove_file(&path);
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        tokio::spawn({
            let path = path.clone();
            async move {
                let listener = UnixListener::bind(&path).unwrap();
                let _ = ready_tx.send(());
                while let Ok((stream, _)) = listener.accept().await {
                    tokio::spawn(async move {
                        let mut reader = BufReader::new(stream);
                        let mut line = String::new();
                        if reader.read_line(&mut line).await.is_err() || line.is_empty() {
                            return;
                        }
                        let request: Value =
                            serde_json::from_str(line.trim()).unwrap_or_else(|_| json!({}));
                        let id = request["id"].clone();
                        let method = request["method"].as_str().unwrap_or("");
                        let response = if method == "genetic.verify_lineage" {
                            json!({
                                "jsonrpc": "2.0",
                                "error": { "code": -1, "message": "denied" },
                                "id": id
                            })
                        } else {
                            json!({ "jsonrpc": "2.0", "result": json!({}), "id": id })
                        };
                        let mut stream = reader.into_inner();
                        let _ = stream.write_all(&serde_json::to_vec(&response).unwrap()).await;
                    });
                }
            }
        });
        ready_rx.await.unwrap();

        let provider = ProductionSecurityProvider::new(&path).await.unwrap();
        let err = provider.verify_lineage(&sample_lineage_proof()).await.unwrap_err();
        assert!(err.to_string().contains("Security provider RPC error"));
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn decrypt_birdsong_returns_none_when_not_in_lineage() {
        let path = temp_socket_path("decrypt_none");
        let _ = std::fs::remove_file(&path);
        spawn_mock_security_server(
            path.clone(),
            Arc::new(|method, _| {
                if method == "birdsong.decrypt" {
                    json!({ "success": false })
                } else {
                    json!({})
                }
            }),
        )
        .await;

        let provider = ProductionSecurityProvider::new(&path).await.unwrap();
        let encrypted = EncryptedBirdSong {
            version: 1,
            ciphertext: vec![9],
            lineage_hint: LineageHint::Universal,
            timestamp: Utc::now(),
            signature: vec![],
            genesis_witness: None,
        };
        assert!(provider.decrypt_birdsong(&encrypted).await.unwrap().is_none());
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn get_visibility_level_maps_lineage_depths() {
        let path = temp_socket_path("visibility");
        let _ = std::fs::remove_file(&path);
        let listener = UnixListener::bind(&path).unwrap();
        let accept_task = tokio::spawn(async move {
            listener.accept().await.unwrap();
        });

        let provider = ProductionSecurityProvider::new(&path).await.unwrap();
        accept_task.await.unwrap();
        assert_eq!(provider.get_visibility_level(0), AccessLevel::FullLineage);
        assert_eq!(provider.get_visibility_level(3), AccessLevel::SubMasked);
        assert_eq!(provider.get_visibility_level(5), AccessLevel::Masked);
        assert_eq!(provider.get_visibility_level(15), AccessLevel::Transport);
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn encrypt_for_lineage_uses_explicit_family_id() {
        let path = temp_socket_path("encrypt");
        let _ = std::fs::remove_file(&path);
        let captured = Arc::new(std::sync::Mutex::new(None::<String>));
        let captured_clone = Arc::clone(&captured);
        spawn_mock_security_server(
            path.clone(),
            Arc::new(move |method, params| {
                if method == "birdsong.encrypt" {
                    *captured_clone.lock().unwrap() =
                        params.get("family_id").and_then(Value::as_str).map(str::to_string);
                    json!({
                        "version": 1,
                        "ciphertext": vec![1, 2],
                        "lineage_hint": "Universal",
                        "timestamp": Utc::now(),
                        "signature": []
                    })
                } else {
                    json!({})
                }
            }),
        )
        .await;

        let provider =
            ProductionSecurityProvider::with_family_id(&path, "my-family").await.unwrap();
        let _ = provider.encrypt_for_lineage(b"hello", LineageHint::Universal).await.unwrap();
        assert_eq!(captured.lock().unwrap().as_deref(), Some("my-family"));
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn revoke_relay_succeeds_on_ok_rpc() {
        let path = temp_socket_path("revoke");
        let _ = std::fs::remove_file(&path);
        spawn_mock_security_server(
            path.clone(),
            Arc::new(|method, _| {
                if method == "relay.revoke" {
                    json!({ "ok": true })
                } else {
                    json!({})
                }
            }),
        )
        .await;

        let provider = ProductionSecurityProvider::new(&path).await.unwrap();
        provider.revoke_relay("sess-123").await.unwrap();
        let _ = std::fs::remove_file(&path);
    }
}
