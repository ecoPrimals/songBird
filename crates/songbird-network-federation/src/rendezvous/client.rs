// SPDX-License-Identifier: AGPL-3.0-only
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

        // Get public key fingerprint (may involve BearDog call)
        let public_key_fingerprint = self.get_public_key_fingerprint().await?;

        // Get signature (may involve BearDog call)
        let signature = self.sign_message_for_registration().await;

        let msg = RegisterPresenceMessage {
            message_type: "register_presence".to_string(),
            version: "1.0".to_string(),
            timestamp: Utc::now(),
            node_identity: NodeIdentity {
                node_id: node_info.node_id.clone(),
                ephemeral_session_id: String::new(), // Server will generate
                public_key_fingerprint,
                capabilities: node_info.capabilities.clone(),
                protocols: vec!["https".to_string(), "btsp".to_string()],
            },
            network_context: NetworkContext {
                nat_type: "unknown".to_string(),
                reachability: "unknown".to_string(),
                connection_quality: "unknown".to_string(),
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
            message_type: "query_peers".to_string(),
            version: "1.0".to_string(),
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

    /// Get public key fingerprint from `BearDog`, or a deterministic HMAC-based surrogate.
    ///
    /// When `BEARDOG_SOCKET_PATH` is set and `crypto.get_public_key` succeeds, returns
    /// `sha256:` + hex(SHA-256(pubkey)). Otherwise derives
    /// `hmac-sha256:` + hex(HMAC-SHA256(key, `node_id`)) using a fixed domain key so the
    /// value is stable per node without silently using a global placeholder string.
    ///
    /// # Errors
    ///
    /// Returns an error when `BearDog` is unavailable or fails and there is no `node_id`
    /// to derive a surrogate fingerprint from (`CryptoUnavailable`).
    async fn get_public_key_fingerprint(&self) -> Result<String> {
        // Try to get from BearDog security service via RPC
        if let Ok(socket_path) = songbird_process_env::var("BEARDOG_SOCKET_PATH") {
            // Attempt to fetch public key via JSON-RPC
            if let Ok(beardog_client) = UnixRpcClient::new(PathBuf::from(socket_path)) {
                match beardog_client.call_no_params::<Vec<u8>>("crypto.get_public_key").await {
                    Ok(key_data) => {
                        // Compute SHA-256 fingerprint
                        use sha2::{Digest, Sha256};
                        let hash = Sha256::digest(&key_data);
                        return Ok(format!("sha256:{}", hex::encode(hash)));
                    }
                    Err(e) => {
                        debug!(
                            "Failed to fetch public key from BearDog: {e}; falling back to HMAC surrogate if node_id is set"
                        );
                    }
                }
            }
        }

        let node_info = self
            .node_info
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("CryptoUnavailable: rendezvous fingerprint requires BearDog (BEARDOG_SOCKET_PATH + crypto.get_public_key) or node identity"))?;

        // Deterministic surrogate: HMAC-SHA256(domain_key, node_id) — not a real pubkey hash.
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        const DOMAIN_KEY: &[u8] = b"songbird.rendezvous.pkfp.v1";
        let mut mac = HmacSha256::new_from_slice(DOMAIN_KEY)
            .map_err(|e| anyhow::anyhow!("HMAC key construction failed: {e}"))?;
        mac.update(node_info.node_id.as_bytes());
        let tag = mac.finalize().into_bytes();

        Ok(format!("hmac-sha256:{}", hex::encode(tag)))
    }

    /// Sign registration message with `BearDog` or return None
    ///
    /// In production, this would use the `BearDog` security service to
    /// cryptographically sign the registration message.
    async fn sign_message_for_registration(&self) -> Option<String> {
        // Try to sign with BearDog security service
        if let Ok(beardog_url) = songbird_process_env::var("BEARDOG_ENDPOINT") {
            // In production, would serialize msg and send to BearDog for signing
            // For now, return None to indicate unsigned (but ready for integration)
            debug!("BearDog endpoint configured at {}, signature integration pending", beardog_url);
        }

        // Return None for now - server should accept unsigned messages in development
        None
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

    use super::{NetworkContext, PeerInfo, PeerQuery};
    use chrono::Utc;
    use serde_json::{from_value, to_value};

    #[test]
    fn network_context_serde_roundtrip() {
        let ctx = NetworkContext {
            nat_type: "full_cone".to_string(),
            reachability: "direct".to_string(),
            connection_quality: "excellent".to_string(),
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
            ephemeral_session_id: "sess-1".to_string(),
            public_key_fingerprint: "sha256:abc".to_string(),
            capabilities: vec!["a".to_string()],
            protocols: vec!["https".to_string()],
            network_context: NetworkContext {
                nat_type: "unknown".to_string(),
                reachability: "unknown".to_string(),
                connection_quality: "unknown".to_string(),
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
            capabilities_required: vec!["btsp".to_string()],
            capabilities_optional: vec![],
            exclude_node_ids: vec!["self".to_string()],
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
            nat_type: "n".to_string(),
            reachability: "r".to_string(),
            connection_quality: "c".to_string(),
        };
        let s = format!("{ctx:?}");
        assert!(s.contains('n') && s.contains('r') && s.contains('c'));
    }
}
