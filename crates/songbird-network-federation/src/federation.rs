//! # 🤝 Federation Coordination
//!
//! **MODERN FEDERATION SYSTEM** ✅

use serde::{Deserialize, Serialize};
use songbird_types::{SongbirdError, SongbirdResult};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::state::{FederationState, NodeRegistration};

/// Federation coordinator
#[derive(Debug, Clone)]
pub struct FederationCoordinator {
    state: Arc<FederationState>,
    client: reqwest::Client,
}

impl Default for FederationCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl FederationCoordinator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Arc::new(FederationState::new()),
            client: reqwest::Client::new(),
        }
    }

    /// Create coordinator with existing state
    #[must_use]
    pub fn with_state(state: Arc<FederationState>) -> Self {
        Self {
            state,
            client: reqwest::Client::new(),
        }
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
                message: "Cannot join federation without self registration info".to_string(),
                field: Some("self_registration".to_string()),
                suggestion: Some("Provide node registration information".to_string()),
            })?;

        // POST to bootstrap node's join endpoint
        let url = format!("http://{bootstrap_address}/api/federation/join");

        debug!("📡 Sending join request to: {}", url);

        let response = self
            .client
            .post(&url)
            .json(registration)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| SongbirdError::Network {
                message: format!("Failed to connect to bootstrap node: {e}"),
                interface: Some(bootstrap_address.to_string()),
                suggestion: Some("Check bootstrap node is running and accessible".to_string()),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(SongbirdError::Network {
                message: format!("Bootstrap node rejected join request: {status}"),
                interface: Some(bootstrap_address.to_string()),
                suggestion: Some("Check bootstrap node is accepting new members".to_string()),
            });
        }

        // Parse response
        let federation_status: serde_json::Value =
            response.json().await.map_err(|e| SongbirdError::Network {
                message: format!("Failed to parse federation status: {e}"),
                interface: Some(bootstrap_address.to_string()),
                suggestion: Some("Check response format from bootstrap node".to_string()),
            })?;

        info!("✅ Joined federation successfully");
        debug!("Federation status: {:?}", federation_status);

        // Extract and register peer nodes
        if let Some(nodes) = federation_status.get("nodes").and_then(|n| n.as_array()) {
            for node_value in nodes {
                if let Ok(node) = serde_json::from_value::<NodeRegistration>(node_value.clone()) {
                    // Don't register ourselves again
                    if node.node_id != registration.node_id {
                        debug!("📍 Discovered peer node: {}", node.node_name);
                        self.state.register_node(node).await;
                    }
                }
            }
        }

        Ok(())
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

                        let url = format!("http://{}/api/federation/heartbeat", node.node_address);
                        let heartbeat = serde_json::json!({
                            "node_id": node_id,
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                            "status": "active",
                            "metrics": {}
                        });

                        match client
                            .post(&url)
                            .json(&heartbeat)
                            .timeout(Duration::from_secs(5))
                            .send()
                            .await
                        {
                            Ok(resp) if resp.status().is_success() => {
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
            let mut interval = tokio::time::interval(Duration::from_secs(30));

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
}

fn default_heartbeat_interval() -> u64 {
    30
}

fn default_node_timeout() -> i64 {
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
        }
    }
}

/// Node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node_id: String,
    pub address: String,
    pub status: String,
}
