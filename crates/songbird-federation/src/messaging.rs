/// Federation Message Broadcasting and Communication System
///
/// This module implements the core messaging functionality for the federation system,
/// providing real message broadcasting, routing, and inter-node communication.
use crate::types::{FederationNode, NodeAddress};
use serde::{Deserialize, Serialize};
use songbird_errors::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Federation message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FederationMessage {
    /// Heartbeat message to maintain node connectivity
    Heartbeat {
        node_id: String,
        timestamp: chrono::DateTime<chrono::Utc>,
        load_metrics: NodeLoadMetrics,
    },
    /// Service registration broadcast
    ServiceRegistration {
        service_id: String,
        service_info: ServiceInfo,
        node_id: String,
    },
    /// Service deregistration broadcast
    ServiceDeregistration { service_id: String, node_id: String },
    /// Load balancing coordination
    LoadBalancing {
        node_id: String,
        current_load: f64,
        available_capacity: f64,
        request_redistribution: bool,
    },
    /// Health status update
    HealthUpdate {
        node_id: String,
        health_status: NodeHealthStatus,
        affected_services: Vec<String>,
    },
    /// Custom application message
    Application {
        message_type: String,
        payload: serde_json::Value,
        target_nodes: Option<Vec<String>>, // None = broadcast to all
    },
}

/// Node health status for federation messaging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeHealthStatus {
    /// Node is healthy and operational
    Healthy,
    /// Node is degraded but functional
    Degraded,
    /// Node is in warning state
    Warning,
    /// Node is critical and may fail
    Critical,
    /// Node is offline or unreachable
    Offline,
}

/// Node load metrics for load balancing decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeLoadMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub active_connections: u64,
    pub request_rate_per_second: f64,
    pub response_time_ms: f64,
}

/// Service information for federation-wide service registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub operation: String,
    pub capabilities: Vec<String>,
    pub health_status: String,
    pub metadata: HashMap<String, String>,
}

/// Federation message broadcaster and router
pub struct FederationMessenger {
    /// Local node ID
    node_id: String,

    /// Broadcast channel for outgoing messages
    broadcast_tx: broadcast::Sender<FederationMessage>,

    /// Known federation nodes
    nodes: Arc<RwLock<HashMap<String, FederationNode>>>,

    /// Message routing table
    routing_table: Arc<RwLock<HashMap<String, Vec<NodeAddress>>>>,

    /// Message statistics
    stats: Arc<RwLock<MessagingStats>>,
}

/// Messaging statistics
#[derive(Debug, Clone, Default)]
pub struct MessagingStats {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub broadcasts_sent: u64,
    pub routing_failures: u64,
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
}

impl FederationMessenger {
    /// Create new federation messenger
    pub fn new(node_id: String) -> Self {
        let (broadcast_tx, _) = broadcast::channel(10000); // Large buffer for high throughput

        Self {
            node_id,
            broadcast_tx,
            nodes: Arc::new(RwLock::new(HashMap::new())),
            routing_table: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(MessagingStats::default())),
        }
    }

    /// Register a federation node
    pub async fn register_node(&self) -> SongbirdResult<()> {
        let node_id = node.cluster_name.to_string();

        // Add to nodes registry
        {
            let mut nodes = self.nodes.write().await;
            nodes.insert(node_id.clone(), node.clone());
        }

        // Update routing table
        {
            let mut routing_table = self.routing_table.write().await;
            routing_table.insert(node_id.clone(), node.addresses.clone());
        }

        info!("📝 Registered federation node: {}", node_id);
        Ok(())
    }

    /// Unregister a federation node
    pub async fn unregister_node(&self) -> SongbirdResult<()> {
        // Remove from nodes registry
        {
            let mut nodes = self.nodes.write().await;
            nodes.remove(node_id);
        }

        // Remove from routing table
        {
            let mut routing_table = self.routing_table.write().await;
            routing_table.remove(node_id);
        }

        info!("🗑️ Unregistered federation node: {}", node_id);
        Ok(())
    }

    /// Broadcast message to all federation nodes
    pub async fn broadcast_message(&self, message: FederationMessage) -> SongbirdResult<u64> {
        let nodes = self.nodes.read().await;
        let node_count = nodes.len();

        if node_count == 0 {
            debug!("No federation nodes to broadcast to");
            return Ok(0);
        }

        // Send to broadcast channel (for local listeners)
        if let Err(_) = self.broadcast_tx.send(message.clone()) {
            warn!("No local broadcast listeners");
        }

        // Send to all federation nodes via network
        let mut successful_sends = 0u64;
        for (node_id, node) in nodes.iter() {
            if node_id == &self.node_id {
                continue; // Don't send to ourselves
            }

            match self.send_message_to_node(node_id, &message).await {
                Ok(_) => {
                    successful_sends += 1;
                    debug!("📡 Message broadcasted to node: {}", node_id);
                }
                Err(e) => {
                    warn!("Failed to broadcast to node {}: {}", node_id, e);
                    // Update stats
                    let mut stats = self.stats.write().await;
                    stats.routing_failures += 1;
                }
            }
        }

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.broadcasts_sent += 1;
            stats.messages_sent += successful_sends;
            stats.last_activity = Some(chrono::Utc::now());
        }

        info!(
            "📢 Broadcasted message to {}/{} nodes",
            successful_sends,
            node_count - 1
        );
        Ok(successful_sends)
    }

    /// Send message to specific node
    pub async fn send_message_to_node(&self, target_node_id: &str, message: &FederationMessage) -> SongbirdResult<()> {
        let routing_table = self.routing_table.read().await;

        let addresses = routing_table.get(target_node_id).ok_or_else(|| {
            SongbirdError::network(format!(
                "Node not found in routing table: {}",
                target_node_id
            ))
        })?;

        // Try each address until one succeeds
        let mut last_error = None;
        for address in addresses {
            match self.deliver_message_to_address(message, address).await {
                Ok(_) => {
                    // Update stats
                    let mut stats = self.stats.write().await;
                    stats.messages_sent += 1;
                    stats.last_activity = Some(chrono::Utc::now());

                    debug!(
                        "✉️ Message delivered to {} via {}",
                        target_node_id, address.addr
                    );
                    return Ok(());
                }
                Err(e) => {
                    last_error = Some(format!("{e}"));
                    debug!("Failed to deliver via {}: {:?}", address.addr, last_error);
                }
            }
        }

        // All addresses failed
        let error = last_error
            .unwrap_or_else(|| SongbirdError::network("No addresses available".to_string()));
        Err(songbird_errors::SongbirdError::Federation { service: "federation".to_string(), message: error, peer: None, recovery_actions: vec!["retry_operation".to_string(), "check_network".to_string()] })
    }

    /// Deliver message to specific network address
    async fn deliver_message_to_address(&self, message: &FederationMessage, address: &NodeAddress) -> SongbirdResult<()> {
        use tokio::time::{Duration, timeout};

        // Serialize message
        let message_bytes = serde_json::to_vec(message).map_err(|e| {
            SongbirdError::operation_error(format!("Message serialization failed: {}", e))
        })?;

        // Create HTTP client for message delivery
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| SongbirdError::network(format!("HTTP client creation failed: {}", e)))?;

        // Build federation message endpoint URL
        let url = format!(
            "http://{}:{}/federation/messages",
            address.addr.ip(),
            address.addr.port()
        );

        // Send POST request with message
        let response = timeout(
            Duration::from_secs(5),
            client
                .post(&url)
                .header("Content-Type", "application/json")
                .header("X-Federation-Node", &self.node_id)
                .body(message_bytes)
                .send(),
        )
        .await
        .map_err(|_| SongbirdError::network("Message delivery timeout".to_string()))?
        .map_err(|e| SongbirdError::network(format!("HTTP request failed: {}", e)))?;

        if response.status().is_) {
            Ok(())
        } else {
            Err(songbird_errors::SongbirdError::Federation { 
                service: "federation".to_string(), 
                message: format!("Message delivery failed with status: {}", response.status()),
                peer: None, 
                recovery_actions: vec!["retry_operation".to_string(), "check_network".to_string()] 
            })
        }
    }

    /// Subscribe to federation messages
    pub fn subscribe(&self) -> broadcast::Receiver<FederationMessage> {
        self.broadcast_tx.subscribe()
    }

    /// Send heartbeat to all nodes
    pub async fn send_heartbeat(&self) -> SongbirdResult<u64> {
        let heartbeat = FederationMessage::Heartbeat {
            node_id: self.node_id.clone(),
            timestamp: chrono::Utc::now(),
            load_metrics,
        };

        self.broadcast_message(heartbeat).await
    }

    /// Broadcast service registration
    pub async fn broadcast_service_registration(&self) -> SongbirdResult<u64> {
        let message = FederationMessage::ServiceRegistration {
            service_id,
            service_info,
            node_id: self.node_id.clone(),
        };

        self.broadcast_message(message).await
    }

    /// Broadcast service deregistration
    pub async fn broadcast_service_deregistration(&self) -> SongbirdResult<u64> {
        let message = FederationMessage::ServiceDeregistration {
            service_id,
            node_id: self.node_id.clone(),
        };

        self.broadcast_message(message).await
    }

    /// Send load balancing coordination message
    pub async fn send_load_balancing_update(&self) -> SongbirdResult<u64> {
        let message = FederationMessage::LoadBalancing {
            node_id: self.node_id.clone(),
            current_load,
            available_capacity,
            request_redistribution,
        };

        self.broadcast_message(message).await
    }

    /// Get messaging statistics
    pub async fn get_stats(&self) -> MessagingStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Get list of registered nodes
    pub async fn get_registered_nodes(&self) -> Vec<String> {
        let nodes = self.nodes.read().await;
        nodes.keys().cloned().collect()
    }
}
