// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Rendezvous Client
//!
//! Client for connecting to Songbird Rendezvous servers for internet-wide discovery

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use songbird_universal::UnixRpcClient;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::state::NodeRegistration;

/// Rendezvous client for internet discovery
/// **Pure Rust**: Uses Unix socket RPC instead of HTTP
#[derive(Debug)]
pub struct RendezvousClient {
    /// Rendezvous server socket path
    socket_path: PathBuf,

    /// RPC client for JSON-RPC communication
    rpc_client: UnixRpcClient,

    /// Current session ID (if registered)
    session_id: Arc<RwLock<Option<String>>>,

    /// Our node information
    node_info: Option<NodeRegistration>,
}

impl RendezvousClient {
    /// Create a new rendezvous client (Pure Rust Unix socket)
    pub fn new(_server_url: String) -> Result<Self> {
        // Convert server_url to socket path or use env var
        let socket_path = songbird_process_env::var("RENDEZVOUS_SOCKET_PATH")
            .map_or_else(|_| std::env::temp_dir().join("rendezvous.sock"), PathBuf::from);

        let rpc_client = UnixRpcClient::new(&socket_path)?;

        Ok(Self {
            socket_path,
            rpc_client,
            session_id: Arc::new(RwLock::new(None)),
            node_info: None,
        })
    }

    /// Set node information
    pub fn set_node_info(&mut self, node_info: NodeRegistration) {
        self.node_info = Some(node_info);
    }

    /// Register presence with rendezvous server
    pub async fn register_presence(&self) -> Result<String> {
        let node_info =
            self.node_info.as_ref().ok_or_else(|| anyhow::anyhow!("Node info not set"))?;

        info!("📡 Registering with rendezvous via RPC at {:?}", self.socket_path);

        // Get public key fingerprint (may involve security-provider RPC)
        let public_key_fingerprint = self.get_public_key_fingerprint().await?;

        // Get signature (may involve security-provider RPC)
        let signature = self.sign_message_for_registration().await;

        let msg = RegisterPresenceMessage {
            message_type: String::from("register_presence"),
            version: String::from("1.0"),
            timestamp: Utc::now(),
            node_identity: NodeIdentity {
                node_id: node_info.node_id.clone(),
                ephemeral_session_id: String::new(), // Server will generate
                public_key_fingerprint,
                capabilities: node_info.capabilities.clone(),
                protocols: vec![String::from("https"), String::from("btsp")],
            },
            network_context: NetworkContext {
                nat_type: String::from("unknown"),
                reachability: String::from("unknown"),
                connection_quality: String::from("unknown"),
            },
            security: SecurityInfo {
                signature,
            },
        };

        let reg_response: RegisterPresenceResponse =
            self.rpc_client.call("rendezvous.register", &msg).await?;

        let session_id = reg_response.session_id.clone();
        *self.session_id.write().await = Some(session_id.clone());

        info!("✅ Registered with rendezvous: {}", &session_id[..8]);

        Ok(session_id)
    }

    /// Send heartbeat to maintain session
    pub async fn heartbeat(&self) -> Result<()> {
        let session_id = self
            .session_id
            .read()
            .await
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Not registered"))?;

        debug!("💓 Sending heartbeat to rendezvous");

        let msg = HeartbeatMessage {
            session_id: session_id.clone(),
            timestamp: Utc::now(),
            signature: None,
        };

        let _hb_response: serde_json::Value =
            self.rpc_client.call("rendezvous.heartbeat", &msg).await?;

        debug!("💓 Heartbeat acknowledged");
        Ok(())
    }

    /// Query for peers with specific capabilities
    pub async fn query_peers(&self, capabilities: Vec<String>) -> Result<Vec<PeerInfo>> {
        let session_id = self
            .session_id
            .read()
            .await
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Not registered"))?;

        let node_info =
            self.node_info.as_ref().ok_or_else(|| anyhow::anyhow!("Node info not set"))?;

        debug!("🔍 Querying rendezvous for capabilities: {:?}", capabilities);

        let msg = QueryPeersMessage {
            message_type: String::from("query_peers"),
            version: String::from("1.0"),
            timestamp: Utc::now(),
            requester: RequesterInfo {
                session_id: session_id.clone(),
                signature: None,
            },
            query: PeerQuery {
                capabilities_required: capabilities,
                capabilities_optional: vec![],
                exclude_node_ids: vec![node_info.node_id.clone()],
                max_results: 10,
            },
            filters: None,
        };

        let query_response: QueryPeersResponse =
            self.rpc_client.call("rendezvous.query", &msg).await?;

        info!("🔍 Found {} peers via rendezvous", query_response.peers.len());

        Ok(query_response.peers)
    }

    /// Start heartbeat loop
    pub async fn start_heartbeat_loop(self: Arc<Self>) {
        info!("💓 Starting rendezvous heartbeat loop (every 30s)");

        let mut interval = tokio::time::interval(Duration::from_secs(30));

        loop {
            interval.tick().await;

            if let Err(e) = self.heartbeat().await {
                warn!("⚠️  Heartbeat error: {}", e);

                // Try to re-register
                if let Err(e) = self.register_presence().await {
                    warn!("⚠️  Re-registration failed: {}", e);
                }
            }
        }
    }

    /// Get public key fingerprint from the security provider, or a deterministic HMAC-based surrogate.
    ///
    /// When `SECURITY_PROVIDER_SOCKET`, `BEARDOG_SOCKET`, or legacy `BEARDOG_SOCKET_PATH` is set and `crypto.get_public_key` succeeds, returns
    /// `sha256:` + hex(SHA-256(pubkey)). Otherwise derives
    /// `hmac-sha256:` + hex(HMAC-SHA256(key, `node_id`)) using a fixed domain key so the
    /// value is stable per node without silently using a global placeholder string.
    ///
    /// # Errors
    ///
    /// Returns an error when the security provider is unavailable or fails and there is no `node_id`
    /// to derive a surrogate fingerprint from (`CryptoUnavailable`).
    async fn get_public_key_fingerprint(&self) -> Result<String> {
        let crypto = songbird_crypto_provider::CryptoProvider::from_env();
        match crypto.call("crypto.get_public_key", serde_json::json!({})).await {
            Ok(result) => {
                use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
                let key_b64 = result
                    .as_str()
                    .or_else(|| result.get("key").and_then(serde_json::Value::as_str))
                    .unwrap_or("");
                if let Ok(key_data) = BASE64.decode(key_b64) {
                    let hash = crate::crypto_helpers::sha256_hash(Some(&crypto), &key_data).await;
                    return Ok(format!("sha256:{}", hex::encode(hash)));
                }
            }
            Err(e) => {
                debug!(
                    "CryptoProvider failed to fetch public key: {e}; falling back to HMAC surrogate"
                );
            }
        }

        let mut legacy_socket_path = None;
        for k in ["SECURITY_PROVIDER_SOCKET", "BEARDOG_SOCKET", "BEARDOG_SOCKET_PATH"] {
            if let Ok(p) = songbird_process_env::var(k) {
                if matches!(k, "BEARDOG_SOCKET" | "BEARDOG_SOCKET_PATH") {
                    tracing::warn!(
                        "{k} is deprecated — migrate to SECURITY_PROVIDER_SOCKET or SECURITY_SOCKET; prefer CAPABILITY_SECURITY_ENDPOINT (capability-first)"
                    );
                }
                legacy_socket_path = Some(p);
                break;
            }
        }
        if let Some(socket_path) = legacy_socket_path
            && let Ok(security_client) = UnixRpcClient::new(PathBuf::from(socket_path))
        {
            match security_client.call_no_params::<Vec<u8>>("crypto.get_public_key").await {
                Ok(key_data) => {
                    let hash = crate::crypto_helpers::sha256_hash(Some(&crypto), &key_data).await;
                    return Ok(format!("sha256:{}", hex::encode(hash)));
                }
                Err(e) => {
                    debug!("Legacy direct-socket path failed: {e}; falling back to HMAC surrogate");
                }
            }
        }

        let node_info = self
            .node_info
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("CryptoUnavailable: rendezvous fingerprint requires security provider (CryptoProvider or SECURITY_PROVIDER_SOCKET) or node identity"))?;

        const DOMAIN_KEY: &[u8] = b"songbird.rendezvous.pkfp.v1";
        let tag = crate::crypto_helpers::hmac_sha256(
            Some(&crypto),
            DOMAIN_KEY,
            node_info.node_id.as_bytes(),
        )
        .await;

        Ok(format!("hmac-sha256:{}", hex::encode(tag)))
    }

    /// Sign registration message via crypto provider delegation.
    ///
    /// Uses `CryptoProvider` (discovered from environment) to sign the
    /// registration payload. Returns `None` if no crypto provider is available
    /// — registration proceeds unsigned (relay accepts with reduced trust tier).
    async fn sign_message_for_registration(&self) -> Option<String> {
        let crypto = songbird_crypto_provider::CryptoProvider::from_env();

        let node_id = self.node_info.as_ref().map_or("unknown", |n| n.node_id.as_str());
        let payload = format!("rendezvous:register:{node_id}");

        match crypto.call("crypto.sign.ed25519", serde_json::json!({ "data": payload })).await {
            Ok(v) => v.get("signature").and_then(serde_json::Value::as_str).map(String::from),
            Err(e) => {
                tracing::debug!("Registration signature unavailable: {e}");
                None
            }
        }
    }
}

// Message types (matching rendezvous server protocol)

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RegisterPresenceMessage {
    message_type: String,
    version: String,
    timestamp: chrono::DateTime<Utc>,
    node_identity: NodeIdentity,
    network_context: NetworkContext,
    security: SecurityInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NodeIdentity {
    node_id: String,
    ephemeral_session_id: String,
    public_key_fingerprint: String,
    capabilities: Vec<String>,
    protocols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkContext {
    pub nat_type: String,
    pub reachability: String,
    pub connection_quality: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SecurityInfo {
    signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RegisterPresenceResponse {
    status: String,
    session_id: String,
    expires_at: chrono::DateTime<Utc>,
    rendezvous_endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HeartbeatMessage {
    session_id: String,
    timestamp: chrono::DateTime<Utc>,
    signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueryPeersMessage {
    message_type: String,
    version: String,
    timestamp: chrono::DateTime<Utc>,
    requester: RequesterInfo,
    query: PeerQuery,
    filters: Option<QueryFilters>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RequesterInfo {
    session_id: String,
    signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PeerQuery {
    capabilities_required: Vec<String>,
    capabilities_optional: Vec<String>,
    exclude_node_ids: Vec<String>,
    max_results: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueryFilters {
    connection_quality_min: Option<String>,
    prefer_direct_connections: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueryPeersResponse {
    peers: Vec<PeerInfo>,
    total_matches: usize,
    returned: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub ephemeral_session_id: String,
    pub public_key_fingerprint: String,
    pub capabilities: Vec<String>,
    pub protocols: Vec<String>,
    pub network_context: NetworkContext,
    pub last_heartbeat: chrono::DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::{NetworkContext, PeerInfo, PeerQuery, RendezvousClient};
    use crate::state::NodeRegistration;
    use chrono::Utc;
    use serde_json::{from_value, to_value};
    use songbird_process_env::ScopedEnv;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;

    static ENV_LOCK: std::sync::OnceLock<Mutex<()>> = std::sync::OnceLock::new();
    fn env_mutex() -> &'static Mutex<()> {
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

    fn sample_node_info() -> NodeRegistration {
        NodeRegistration {
            node_id: "node-test-1".into(),
            node_name: "test-node".into(),
            node_address: "127.0.0.1:8080".into(),
            endpoints: None,
            cpu_cores: 4,
            memory_gb: 8,
            gpu_model: None,
            storage_gb: None,
            capabilities: vec!["compute".into()],
            status: crate::state::NodeStatus::Active,
            joined_at: Utc::now(),
            last_heartbeat: Utc::now(),
        }
    }

    fn temp_rendezvous_socket(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!("songbird_rdzv_{label}_{}.sock", uuid::Uuid::new_v4()))
    }

    async fn spawn_mock_rendezvous_server(path: PathBuf) {
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let listener = UnixListener::bind(&path).unwrap();
            let _ = ready_tx.send(());
            while let Ok((stream, _)) = listener.accept().await {
                tokio::spawn(async move {
                    let mut reader = BufReader::new(stream);
                    let mut line = String::new();
                    if reader.read_line(&mut line).await.is_err() {
                        return;
                    }
                    let request: serde_json::Value =
                        serde_json::from_str(line.trim()).unwrap_or_default();
                    let method = request["method"].as_str().unwrap_or("");
                    let id = request["id"].as_u64().unwrap_or(1);
                    let result = match method {
                        "rendezvous.register" => serde_json::json!({
                            "status": "ok",
                            "session_id": "sess-test-abc123",
                            "expires_at": (Utc::now() + chrono::Duration::hours(1)).to_rfc3339(),
                            "rendezvous_endpoint": null
                        }),
                        "rendezvous.heartbeat" => serde_json::json!({ "status": "ok" }),
                        "rendezvous.query" => serde_json::json!({
                            "peers": [{
                                "ephemeral_session_id": "peer-sess",
                                "public_key_fingerprint": "hmac-sha256:abc",
                                "capabilities": ["compute"],
                                "protocols": ["https"],
                                "network_context": {
                                    "nat_type": "unknown",
                                    "reachability": "unknown",
                                    "connection_quality": "unknown"
                                },
                                "last_heartbeat": Utc::now().to_rfc3339()
                            }],
                            "total_matches": 1,
                            "returned": 1
                        }),
                        _ => serde_json::json!({}),
                    };
                    let response = serde_json::json!({
                        "jsonrpc": "2.0",
                        "result": result,
                        "id": id
                    });
                    let mut stream = reader.into_inner();
                    let _ = stream.write_all(&serde_json::to_vec(&response).unwrap()).await;
                });
            }
        });
        ready_rx.await.unwrap();
    }

    #[test]
    fn network_context_serde_roundtrip() {
        let ctx = NetworkContext {
            nat_type: String::from("full_cone"),
            reachability: String::from("direct"),
            connection_quality: String::from("excellent"),
        };
        let v = to_value(&ctx).unwrap();
        let back: NetworkContext = from_value(v).unwrap();
        assert_eq!(ctx.nat_type, back.nat_type);
        assert_eq!(ctx.reachability, back.reachability);
        assert_eq!(ctx.connection_quality, back.connection_quality);
    }

    #[test]
    fn peer_info_serde_roundtrip() {
        let ts = Utc::now();
        let info = PeerInfo {
            ephemeral_session_id: String::from("sess-1"),
            public_key_fingerprint: String::from("sha256:abc"),
            capabilities: vec![String::from("a")],
            protocols: vec![String::from("https")],
            network_context: NetworkContext {
                nat_type: String::from("unknown"),
                reachability: String::from("unknown"),
                connection_quality: String::from("unknown"),
            },
            last_heartbeat: ts,
        };
        let v = to_value(&info).unwrap();
        let back: PeerInfo = from_value(v).unwrap();
        assert_eq!(info.ephemeral_session_id, back.ephemeral_session_id);
        assert_eq!(info.public_key_fingerprint, back.public_key_fingerprint);
        assert_eq!(info.capabilities, back.capabilities);
        assert_eq!(info.protocols, back.protocols);
        assert_eq!(info.network_context.nat_type, back.network_context.nat_type);
        assert_eq!(info.last_heartbeat, back.last_heartbeat);
    }

    #[test]
    fn peer_query_serde_roundtrip() {
        let q = PeerQuery {
            capabilities_required: vec![String::from("btsp")],
            capabilities_optional: vec![],
            exclude_node_ids: vec![String::from("self")],
            max_results: 25,
        };
        let v = to_value(&q).unwrap();
        let back: PeerQuery = from_value(v).unwrap();
        assert_eq!(q.capabilities_required, back.capabilities_required);
        assert_eq!(q.capabilities_optional, back.capabilities_optional);
        assert_eq!(q.exclude_node_ids, back.exclude_node_ids);
        assert_eq!(q.max_results, back.max_results);
    }

    #[test]
    fn network_context_debug_includes_fields() {
        let ctx = NetworkContext {
            nat_type: String::from("n"),
            reachability: String::from("r"),
            connection_quality: String::from("c"),
        };
        let s = format!("{ctx:?}");
        assert!(s.contains('n') && s.contains('r') && s.contains('c'));
    }

    #[test]
    fn rendezvous_client_new_uses_socket_env() {
        let _guard = env_mutex().lock().unwrap();
        let path = temp_rendezvous_socket("new");
        let _ = std::fs::remove_file(&path);
        let _env = ScopedEnv::new("RENDEZVOUS_SOCKET_PATH", &path);
        assert!(RendezvousClient::new("ignored-url".into()).is_ok());
    }

    #[test]
    fn set_node_info_accepts_registration_without_panic() {
        let _guard = env_mutex().lock().unwrap();
        let path = temp_rendezvous_socket("set_info");
        let _ = std::fs::remove_file(&path);
        let _env = ScopedEnv::new("RENDEZVOUS_SOCKET_PATH", &path);
        let mut client = RendezvousClient::new("url".into()).unwrap();
        client.set_node_info(sample_node_info());
    }

    #[tokio::test]
    async fn register_presence_requires_node_info() {
        let _guard = env_mutex().lock().unwrap();
        let path = temp_rendezvous_socket("no_info");
        let _ = std::fs::remove_file(&path);
        let _env = ScopedEnv::new("RENDEZVOUS_SOCKET_PATH", &path);
        let client = RendezvousClient::new("url".into()).unwrap();
        let err = client.register_presence().await.unwrap_err();
        assert!(err.to_string().contains("Node info not set"));
    }

    #[tokio::test]
    async fn register_presence_returns_session_id() {
        let _guard = env_mutex().lock().unwrap();
        let path = temp_rendezvous_socket("register");
        let _ = std::fs::remove_file(&path);
        spawn_mock_rendezvous_server(path.clone()).await;
        let _env = ScopedEnv::new("RENDEZVOUS_SOCKET_PATH", &path);
        let mut client = RendezvousClient::new("url".into()).unwrap();
        client.set_node_info(sample_node_info());
        let session = client.register_presence().await.unwrap();
        assert_eq!(session, "sess-test-abc123");
        client.heartbeat().await.unwrap();
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn heartbeat_requires_registration() {
        let _guard = env_mutex().lock().unwrap();
        let path = temp_rendezvous_socket("hb_no_reg");
        let _ = std::fs::remove_file(&path);
        let _env = ScopedEnv::new("RENDEZVOUS_SOCKET_PATH", &path);
        let client = RendezvousClient::new("url".into()).unwrap();
        let err = client.heartbeat().await.unwrap_err();
        assert!(err.to_string().contains("Not registered"));
    }

    #[tokio::test]
    async fn heartbeat_succeeds_after_register() {
        let _guard = env_mutex().lock().unwrap();
        let path = temp_rendezvous_socket("hb_ok");
        let _ = std::fs::remove_file(&path);
        spawn_mock_rendezvous_server(path.clone()).await;
        let _env = ScopedEnv::new("RENDEZVOUS_SOCKET_PATH", &path);
        let mut client = RendezvousClient::new("url".into()).unwrap();
        client.set_node_info(sample_node_info());
        client.register_presence().await.unwrap();
        client.heartbeat().await.unwrap();
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn query_peers_requires_registration() {
        let _guard = env_mutex().lock().unwrap();
        let path = temp_rendezvous_socket("query_no_reg");
        let _ = std::fs::remove_file(&path);
        let _env = ScopedEnv::new("RENDEZVOUS_SOCKET_PATH", &path);
        let mut client = RendezvousClient::new("url".into()).unwrap();
        client.set_node_info(sample_node_info());
        let err = client.query_peers(vec!["compute".into()]).await.unwrap_err();
        assert!(err.to_string().contains("Not registered"));
    }

    #[tokio::test]
    async fn query_peers_returns_matching_peers() {
        let _guard = env_mutex().lock().unwrap();
        let path = temp_rendezvous_socket("query_ok");
        let _ = std::fs::remove_file(&path);
        spawn_mock_rendezvous_server(path.clone()).await;
        let _env = ScopedEnv::new("RENDEZVOUS_SOCKET_PATH", &path);
        let mut client = RendezvousClient::new("url".into()).unwrap();
        client.set_node_info(sample_node_info());
        client.register_presence().await.unwrap();
        let peers = client.query_peers(vec!["compute".into()]).await.unwrap();
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].ephemeral_session_id, "peer-sess");
        assert_eq!(peers[0].capabilities, vec![String::from("compute")]);
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn register_uses_hmac_fingerprint_without_crypto_provider() {
        let _guard = env_mutex().lock().unwrap();
        let path = temp_rendezvous_socket("hmac_fp");
        let _ = std::fs::remove_file(&path);
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        tokio::spawn({
            let path = path.clone();
            async move {
                let listener = UnixListener::bind(&path).unwrap();
                let _ = ready_tx.send(());
                if let Ok((stream, _)) = listener.accept().await {
                    let mut reader = BufReader::new(stream);
                    let mut line = String::new();
                    let _ = reader.read_line(&mut line).await;
                    let request: serde_json::Value = serde_json::from_str(&line).unwrap();
                    let fp = request["params"]["node_identity"]["public_key_fingerprint"]
                        .as_str()
                        .unwrap_or("");
                    assert!(fp.starts_with("hmac-sha256:"));
                    let response = serde_json::json!({
                        "jsonrpc": "2.0",
                        "result": {
                            "status": "ok",
                            "session_id": "sess-hmac",
                            "expires_at": (Utc::now() + chrono::Duration::hours(1)).to_rfc3339(),
                            "rendezvous_endpoint": null
                        },
                        "id": request["id"]
                    });
                    let mut stream = reader.into_inner();
                    let _ = stream.write_all(&serde_json::to_vec(&response).unwrap()).await;
                }
            }
        });
        ready_rx.await.unwrap();
        let _env = ScopedEnv::new("RENDEZVOUS_SOCKET_PATH", &path);
        let mut client = RendezvousClient::new("url".into()).unwrap();
        client.set_node_info(sample_node_info());
        let session = client.register_presence().await.unwrap();
        assert_eq!(session, "sess-hmac");
        let _ = std::fs::remove_file(&path);
    }
}
