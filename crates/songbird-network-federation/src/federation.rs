// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # 🤝 Federation Coordination
//!
//! **MODERN FEDERATION SYSTEM** ✅

use serde::{Deserialize, Serialize};
use songbird_http_client::IpcHttpClient;
use songbird_types::{SongbirdError, SongbirdResult};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::discovery_mode::DiscoveryMode;
use crate::rendezvous::RendezvousClient;
use crate::security::{SecurityProviderFactory, SecurityProviderImpl};
use crate::state::{FederationState, FederationStatus, NodeRegistration};

/// Federation coordinator
#[derive(Clone)]
pub struct FederationCoordinator {
    state: Arc<FederationState>,
    client: IpcHttpClient,
    rendezvous_client: Arc<RwLock<Option<Arc<RendezvousClient>>>>,
    security_provider: Arc<RwLock<Option<SecurityProviderImpl>>>,
}

// Manual Debug implementation since SecurityProvider doesn't impl Debug
impl std::fmt::Debug for FederationCoordinator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FederationCoordinator")
            .field("state", &self.state)
            .field("client", &self.client)
            .field("rendezvous_client", &"<rendezvous>")
            .field("security_provider", &"<security_provider>")
            .finish()
    }
}

impl FederationCoordinator {
    /// Create a new federation coordinator
    ///
    /// # Errors
    /// Returns error if HTTP client creation fails
    pub async fn new() -> SongbirdResult<Self> {
        let client = IpcHttpClient::new()
            .await
            .map_err(|e| SongbirdError::network(format!("Failed to create HTTP client: {e}")))?;

        Ok(Self {
            state: Arc::new(FederationState::new(String::from("default"))),
            client,
            rendezvous_client: Arc::new(RwLock::new(None)),
            security_provider: Arc::new(RwLock::new(None)),
        })
    }

    /// Create coordinator with existing state
    ///
    /// # Errors
    /// Returns error if HTTP client creation fails
    pub async fn with_state(state: Arc<FederationState>) -> SongbirdResult<Self> {
        let client = IpcHttpClient::new()
            .await
            .map_err(|e| SongbirdError::network(format!("Failed to create HTTP client: {e}")))?;

        Ok(Self {
            state,
            client,
            rendezvous_client: Arc::new(RwLock::new(None)),
            security_provider: Arc::new(RwLock::new(None)),
        })
    }

    /// Whether a capability-discovered security provider is connected
    pub async fn has_security_provider(&self) -> bool {
        self.security_provider.read().await.is_some()
    }

    /// Get discovery mode based on security-provider availability
    pub async fn discovery_mode(&self) -> DiscoveryMode {
        if self.has_security_provider().await {
            DiscoveryMode::BirdSong
        } else {
            DiscoveryMode::Plaintext
        }
    }

    /// Get effective discovery mode (respecting config override)
    pub async fn effective_discovery_mode(&self, config: &FederationConfig) -> DiscoveryMode {
        // If config forces a mode, use it
        if let Some(mode) = config.discovery_mode {
            // Validate that we can support it
            if mode.requires_security_provider() && !self.has_security_provider().await {
                warn!(
                    "⚠️  BirdSong mode requested but security provider unavailable, falling back to plaintext"
                );
                return DiscoveryMode::Plaintext;
            }
            return mode;
        }

        // Otherwise auto-detect
        self.discovery_mode().await
    }

    /// Get the federation state
    #[must_use]
    pub fn state(&self) -> Arc<FederationState> {
        Arc::clone(&self.state)
    }

    /// Start federation coordination
    pub async fn coordinate(&self, config: &FederationConfig) -> SongbirdResult<()> {
        if !config.enabled {
            debug!("Federation disabled, skipping coordination");
            return Ok(());
        }

        info!("🌐 Starting federation coordination");

        // Register self if we have node info
        if let Some(node_info) = &config.self_registration {
            info!("📍 Registering self as node: {}", node_info.node_name);
            self.state.register_node(node_info.clone()).await;
        }

        info!("🔒 Attempting to discover security provider...");
        match SecurityProviderFactory::discover().await {
            Ok(Some(provider)) => {
                info!("✅ Security provider available - using encrypted birdSong discovery");
                *self.security_provider.write().await = Some(provider);
            }
            Ok(None) => {
                info!(
                    "ℹ️  Security provider not available - using plaintext discovery (trusted LAN only)"
                );
            }
            Err(e) => {
                warn!("⚠️  Security provider discovery failed: {} - using plaintext discovery", e);
            }
        }

        // Initialize rendezvous client if URL provided
        if let Some(rendezvous_url) = &config.rendezvous_url {
            info!("🌍 Initializing rendezvous client: {}", rendezvous_url);
            match self.initialize_rendezvous(rendezvous_url, config).await {
                Ok(()) => info!("✅ Connected to rendezvous server"),
                Err(e) => {
                    warn!("⚠️  Failed to connect to rendezvous: {}", e);
                    // Continue anyway - LAN discovery still works
                }
            }
        }

        // Join federation if bootstrap provided
        if let Some(bootstrap) = &config.bootstrap_address {
            info!("🤝 Joining federation via bootstrap: {}", bootstrap);
            match self.join_federation(bootstrap, config).await {
                Ok(()) => info!("✅ Successfully joined federation"),
                Err(e) => {
                    warn!("⚠️  Failed to join federation: {}", e);
                    // Continue anyway - we can still accept incoming joins
                }
            }
        }

        // Start background tasks
        self.start_heartbeat_loop(config).await?;
        self.start_health_monitor(config).await?;

        info!("✅ Federation coordination started");
        Ok(())
    }

    /// Join an existing federation
    pub async fn join_federation(
        &self,
        bootstrap_address: &str,
        config: &FederationConfig,
    ) -> SongbirdResult<()> {
        // Get our node registration
        let registration =
            config.self_registration.as_ref().ok_or_else(|| SongbirdError::Configuration {
                message: String::from("Cannot join federation without self registration info"),
                field: Some(String::from("self_registration")),
                suggestion: Some(String::from("Provide node registration information")),
            })?;

        // POST to bootstrap node's join endpoint
        let url = format!("http://{bootstrap_address}/api/federation/join");

        debug!("📡 Sending join request to: {}", url);

        let response = self
            .client
            .post(&url)
            .await
            .json(registration)
            .map_err(|e| SongbirdError::network(format!("Failed to serialize request: {e}")))?
            .send()
            .await
            .map_err(|e| SongbirdError::Network {
                message: format!("Failed to connect to bootstrap node: {e}"),
                interface: Some(bootstrap_address.to_string()),
                suggestion: Some(String::from("Check bootstrap node is running and accessible")),
            })?;

        if !response.is_success() {
            let status = response.status();
            return Err(SongbirdError::Network {
                message: format!("Bootstrap node rejected join request: {status}"),
                interface: Some(bootstrap_address.to_string()),
                suggestion: Some(String::from("Check bootstrap node is accepting new members")),
            });
        }

        // Parse response
        let federation_status: serde_json::Value =
            response.json().await.map_err(|e| SongbirdError::Network {
                message: format!("Failed to parse federation status: {e}"),
                interface: Some(bootstrap_address.to_string()),
                suggestion: Some(String::from("Check response format from bootstrap node")),
            })?;

        info!("✅ Joined federation successfully");
        debug!("Federation status: {:?}", federation_status);

        self.ingest_peers_from_join_response(&federation_status, registration).await;

        Ok(())
    }

    /// Register peers returned by the bootstrap join response (`FederationStatus`, `nodes`, or `peers`).
    async fn ingest_peers_from_join_response(
        &self,
        federation_status: &serde_json::Value,
        registration: &NodeRegistration,
    ) {
        if let Ok(status) = serde_json::from_value::<FederationStatus>(federation_status.clone()) {
            for node in status.nodes {
                if node.node_id != registration.node_id {
                    debug!("📍 Discovered peer node: {}", node.node_name);
                    self.state.register_node(node).await;
                }
            }
            return;
        }

        if let Some(nodes) = federation_status.get("nodes").and_then(|n| n.as_array()) {
            for node_value in nodes {
                match serde_json::from_value::<NodeRegistration>(node_value.clone()) {
                    Ok(node) if node.node_id != registration.node_id => {
                        debug!("📍 Discovered peer node: {}", node.node_name);
                        self.state.register_node(node).await;
                    }
                    Ok(_) => {}
                    Err(e) => {
                        warn!(
                            "Join response `nodes` entry could not be parsed as NodeRegistration: {e}"
                        );
                    }
                }
            }
        }

        if let Some(peers) = federation_status.get("peers").and_then(|p| p.as_array()) {
            for peer_value in peers {
                match serde_json::from_value::<NodeRegistration>(peer_value.clone()) {
                    Ok(node) if node.node_id != registration.node_id => {
                        debug!("📍 Discovered peer (peers[]): {}", node.node_name);
                        self.state.register_node(node).await;
                    }
                    Ok(_) => {}
                    Err(e) => {
                        warn!(
                            "Join response `peers` entry could not be parsed as NodeRegistration: {e}"
                        );
                    }
                }
            }
        }
    }

    /// Start heartbeat loop
    async fn start_heartbeat_loop(&self, config: &FederationConfig) -> SongbirdResult<()> {
        let state = Arc::clone(&self.state);
        let client = self.client.clone();
        let interval_secs = config.heartbeat_interval_secs;
        let self_node_id = config.self_registration.as_ref().map(|r| r.node_id.clone());

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));

            loop {
                interval.tick().await;

                // Get all active nodes
                let nodes = state.active_nodes().await;

                if let Some(node_id) = &self_node_id {
                    debug!("💓 Sending heartbeats to {} nodes", nodes.len());

                    // Send heartbeat to each node
                    for node in &nodes {
                        // Don't send heartbeat to ourselves
                        if &node.node_id == node_id {
                            continue;
                        }

                        // node_address may already include protocol (https://...) from discovery
                        let url = if node.node_address.starts_with("http://")
                            || node.node_address.starts_with("https://")
                        {
                            format!("{}/api/federation/heartbeat", node.node_address)
                        } else {
                            format!("http://{}/api/federation/heartbeat", node.node_address)
                        };
                        let heartbeat = serde_json::json!({
                            "node_id": node_id,
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                            "status": "active",
                            "metrics": {}
                        });

                        match client.post(&url).await.json(&heartbeat) {
                            Ok(builder) => match builder.send().await {
                                Ok(resp) if resp.is_success() => {
                                    debug!("💓 Heartbeat sent to {}", node.node_name);
                                }
                                Ok(resp) => {
                                    warn!(
                                        "⚠️  Heartbeat failed to {}: {}",
                                        node.node_name,
                                        resp.status()
                                    );
                                }
                                Err(e) => {
                                    warn!("⚠️  Heartbeat send failed to {}: {e}", node.node_name);
                                }
                            },
                            Err(e) => {
                                warn!("⚠️  Heartbeat error to {}: {}", node.node_name, e);
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Start health monitor
    async fn start_health_monitor(&self, config: &FederationConfig) -> SongbirdResult<()> {
        let federation_state = Arc::clone(&self.state);
        let timeout_secs = config.node_timeout_secs;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                songbird_types::defaults::timeouts::DEFAULT_FEDERATION_HEARTBEAT_INTERVAL,
            );

            loop {
                interval.tick().await;

                debug!("🏥 Checking node health");
                federation_state.check_node_health(timeout_secs).await;

                let health_stats = federation_state.get_stats().await;
                if health_stats.active_nodes != health_stats.total_nodes {
                    warn!(
                        "⚠️  Node health: {}/{} nodes active",
                        health_stats.active_nodes, health_stats.total_nodes
                    );
                }
            }
        });

        Ok(())
    }

    /// Initialize rendezvous client and register presence
    async fn initialize_rendezvous(
        &self,
        rendezvous_url: &str,
        config: &FederationConfig,
    ) -> SongbirdResult<()> {
        // Get our node registration
        let registration =
            config.self_registration.as_ref().ok_or_else(|| SongbirdError::Configuration {
                message: String::from("Cannot use rendezvous without self registration info"),
                field: Some(String::from("self_registration")),
                suggestion: Some(String::from("Provide node registration information")),
            })?;

        // Create rendezvous client
        let mut client = RendezvousClient::new(rendezvous_url.to_string()).map_err(|e| {
            SongbirdError::Network {
                message: format!("Failed to create rendezvous client: {e}"),
                interface: Some(rendezvous_url.to_string()),
                suggestion: Some(String::from("Check rendezvous server URL is valid")),
            }
        })?;

        // Set our node info
        client.set_node_info(registration.clone());

        // Register presence
        client.register_presence().await.map_err(|e| SongbirdError::Network {
            message: format!("Failed to register with rendezvous: {e}"),
            interface: Some(rendezvous_url.to_string()),
            suggestion: Some(String::from("Check rendezvous server is running and accessible")),
        })?;

        // Store client
        let client_arc = Arc::new(client);
        *self.rendezvous_client.write().await = Some(Arc::clone(&client_arc));

        // Start heartbeat loop
        tokio::spawn(async move {
            client_arc.start_heartbeat_loop().await;
        });

        // Start peer discovery loop
        let state = Arc::clone(&self.state);
        let rendezvous_client_lock = Arc::clone(&self.rendezvous_client);
        tokio::spawn(async move {
            Self::rendezvous_discovery_loop(state, rendezvous_client_lock).await;
        });

        Ok(())
    }

    /// Background loop for discovering peers via rendezvous
    async fn rendezvous_discovery_loop(
        _state: Arc<FederationState>,
        rendezvous_client: Arc<RwLock<Option<Arc<RendezvousClient>>>>,
    ) {
        let mut interval = tokio::time::interval(
            songbird_types::defaults::timeouts::DEFAULT_FEDERATION_RENDEZVOUS_INTERVAL,
        );

        loop {
            interval.tick().await;

            // Get client reference
            let client = {
                let lock = rendezvous_client.read().await;
                match &*lock {
                    Some(c) => Arc::clone(c),
                    None => continue,
                }
            };

            debug!("🔍 Discovering peers via rendezvous");

            // Query for orchestration capability (other Songbird instances)
            match client.query_peers(vec![String::from("orchestration")]).await {
                Ok(peers) => {
                    info!("🌍 Discovered {} peers via rendezvous", peers.len());

                    // Attempt to establish connections to discovered peers
                    for peer in peers {
                        debug!(
                            "  Peer: {} (capabilities: {:?})",
                            &peer.ephemeral_session_id[..8],
                            peer.capabilities
                        );

                        // Extract connection info - build endpoint from protocols
                        let endpoint = if peer.protocols.contains(&String::from("https")) {
                            // In production, would negotiate actual endpoint
                            format!("https://peer-{}", &peer.ephemeral_session_id[..8])
                        } else if peer.protocols.contains(&String::from("btsp")) {
                            format!("btsp://peer-{}", &peer.ephemeral_session_id[..8])
                        } else {
                            debug!(
                                "Peer {} has no compatible protocols",
                                &peer.ephemeral_session_id[..8]
                            );
                            continue;
                        };

                        // Log discovered peer for future connection attempts
                        // In a full implementation, would store peer info and attempt connection
                        // using NAT traversal techniques based on network_context
                        info!(
                            "💡 Peer available: {} at {} (protocols: {:?}, NAT: {})",
                            &peer.ephemeral_session_id[..8],
                            endpoint,
                            peer.protocols,
                            peer.network_context.nat_type
                        );
                    }
                }
                Err(e) => {
                    warn!("⚠️  Rendezvous peer query failed: {}", e);
                }
            }
        }
    }
}

/// Federation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationConfig {
    /// Whether federation is enabled
    pub enabled: bool,

    /// Bootstrap node address (IP:PORT or hostname:PORT)
    pub bootstrap_address: Option<String>,

    /// Self registration info (for joining federation)
    pub self_registration: Option<NodeRegistration>,

    /// Heartbeat interval in seconds
    #[serde(default = "default_heartbeat_interval")]
    pub heartbeat_interval_secs: u64,

    /// Node timeout in seconds (mark as inactive after this)
    #[serde(default = "default_node_timeout")]
    pub node_timeout_secs: i64,

    /// Rendezvous server URL for internet-wide discovery (optional)
    /// Example: "<http://rendezvous.songbird.network:8888>"
    pub rendezvous_url: Option<String>,

    /// Force discovery mode (if None, auto-detect based on security-provider availability)
    #[serde(default)]
    pub discovery_mode: Option<DiscoveryMode>,
}

const fn default_heartbeat_interval() -> u64 {
    30
}

const fn default_node_timeout() -> i64 {
    60
}

impl Default for FederationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bootstrap_address: None,
            self_registration: None,
            heartbeat_interval_secs: 30,
            node_timeout_secs: 60,
            rendezvous_url: None,
            discovery_mode: None, // Auto-detect
        }
    }
}

/// Node information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeInfo {
    pub node_id: String,
    pub address: String,
    pub status: String,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;
    use crate::state::{FederationStatus, NodeRegistration, NodeStatus};
    use chrono::Utc;

    fn sample_registration(node_id: &str, name: &str) -> NodeRegistration {
        NodeRegistration {
            node_id: node_id.to_string(),
            node_name: name.to_string(),
            node_address: String::from("192.168.0.1:8080"),
            endpoints: None,
            cpu_cores: 4,
            memory_gb: 8,
            gpu_model: None,
            storage_gb: None,
            capabilities: vec![],
            status: NodeStatus::Active,
            joined_at: Utc::now(),
            last_heartbeat: Utc::now(),
        }
    }

    #[test]
    fn federation_config_default_serde_roundtrip() {
        let c = FederationConfig::default();
        let json = serde_json::to_string(&c).unwrap();
        let back: FederationConfig = serde_json::from_str(&json).unwrap();
        assert!(!back.enabled);
        assert_eq!(back.heartbeat_interval_secs, 30);
        assert_eq!(back.node_timeout_secs, 60);
    }

    #[test]
    fn federation_config_with_discovery_mode_serializes() {
        let c = FederationConfig {
            discovery_mode: Some(DiscoveryMode::BirdSong),
            ..FederationConfig::default()
        };
        let json = serde_json::to_string(&c).unwrap();
        let back: FederationConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.discovery_mode, Some(DiscoveryMode::BirdSong));
    }

    #[test]
    fn node_info_roundtrip() {
        let n = NodeInfo {
            node_id: "a".into(),
            address: "b".into(),
            status: "c".into(),
        };
        let json = serde_json::to_string(&n).unwrap();
        let back: NodeInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(back.node_id, "a");
    }

    #[test]
    fn federation_config_default_heartbeat_and_timeout_match_helpers() {
        let c = FederationConfig::default();
        assert_eq!(c.heartbeat_interval_secs, 30);
        assert_eq!(c.node_timeout_secs, 60);
    }

    #[tokio::test]
    async fn discovery_mode_plaintext_without_security_provider() {
        let coord = FederationCoordinator::new().await.unwrap();
        assert!(!coord.has_security_provider().await);
        assert_eq!(coord.discovery_mode().await, DiscoveryMode::Plaintext);
    }

    #[tokio::test]
    async fn effective_discovery_mode_respects_override_and_fallback() {
        let coord = FederationCoordinator::new().await.unwrap();

        let auto = FederationConfig::default();
        assert_eq!(coord.effective_discovery_mode(&auto).await, DiscoveryMode::Plaintext);

        let forced_plain = FederationConfig {
            discovery_mode: Some(DiscoveryMode::Plaintext),
            ..FederationConfig::default()
        };
        assert_eq!(coord.effective_discovery_mode(&forced_plain).await, DiscoveryMode::Plaintext);

        let birdsong_without_security = FederationConfig {
            discovery_mode: Some(DiscoveryMode::BirdSong),
            ..FederationConfig::default()
        };
        assert_eq!(
            coord.effective_discovery_mode(&birdsong_without_security).await,
            DiscoveryMode::Plaintext
        );
    }

    #[tokio::test]
    async fn coordinator_debug_formats_without_panicking() {
        let coord = FederationCoordinator::new().await.unwrap();
        let s = format!("{coord:?}");
        assert!(s.contains("FederationCoordinator"));
    }

    #[tokio::test]
    async fn ingest_peers_from_federation_status_value() {
        let coord = FederationCoordinator::new().await.unwrap();
        let self_reg = sample_registration("self-node", "Self");
        let peer = sample_registration("peer-a", "Peer A");

        let status = FederationStatus {
            federation_id: String::from("fed-1"),
            active_nodes: 2,
            nodes: vec![self_reg.clone(), peer.clone()],
            total_cpu_cores: 0,
            total_memory_gb: 0,
            total_storage_gb: 0,
            uptime_seconds: 0,
        };
        let v = serde_json::to_value(&status).unwrap();
        coord.ingest_peers_from_join_response(&v, &self_reg).await;

        let state = coord.state();
        let nodes = state.nodes.read().await;
        assert!(nodes.contains_key("peer-a"));
        assert!(!nodes.contains_key("self-node"));
    }

    #[tokio::test]
    async fn ingest_peers_from_nodes_array_when_status_shape_unknown() {
        let coord = FederationCoordinator::new().await.unwrap();
        let self_reg = sample_registration("node-self", "Self");
        let peer = sample_registration("node-peer", "Peer B");

        let v = serde_json::json!({
            "not_federation_status": true,
            "nodes": [ serde_json::to_value(&peer).unwrap() ],
        });
        coord.ingest_peers_from_join_response(&v, &self_reg).await;

        let state = coord.state();
        assert!(state.nodes.read().await.contains_key("node-peer"));
    }

    #[tokio::test]
    async fn ingest_peers_from_peers_array() {
        let coord = FederationCoordinator::new().await.unwrap();
        let self_reg = sample_registration("id-self", "Self");
        let peer = sample_registration("id-peer", "Peer C");

        let v = serde_json::json!({
            "peers": [ serde_json::to_value(&peer).unwrap() ],
        });
        coord.ingest_peers_from_join_response(&v, &self_reg).await;

        let state = coord.state();
        assert!(state.nodes.read().await.contains_key("id-peer"));
    }
}
