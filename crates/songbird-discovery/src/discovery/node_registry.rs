//! # 🎼 Node Registry - Discovery Node Management
//!
//! **🚀 FOCUSED RESPONSIBILITY**
//!
//! Handles node registration, management, and federation coordination.
//! Extracted from the large songbird_discovery.rs for better maintainability.

use crate::discovery::config::SongbirdDiscoveryConfig;
use crate::discovery::types::{FederationHealth, NodeId, NodeInfo, NodeType};
use crate::traits::ServiceEvent;
use songbird_config::UniversalHealthStatus;
use songbird_types::{SongbirdError, SongbirdSongbirdResponse};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
/// Node registry for managing cluster nodes
pub struct NodeRegistry  {#[allow(dead_code)]
    config: SongbirdDiscoveryConfig,
    /// Current node information
    local_node: NodeInfo,
    /// Registry of known nodes in the cluster
    known_nodes: Arc<RwLock<HashMap<NodeId, NodeInfo>>>,
    /// Event broadcaster for node changes
    event_sender: broadcast::Sender<ServiceEvent>,
}

impl NodeRegistry  {/// Create new node registry
    pub fn new(
        config: SongbirdDiscoveryConfig,
        event_sender: broadcast::Sender<ServiceEvent>,
    ) -> Self  {let local_node = NodeInfo {
            id: NodeId::new(,
            node_type: NodeType::Service,
            institution: config.global.service_name.clone(,
            health: FederationHealth::default(),
            resources: HashMap::new(),
            last_seen: std::time::SystemTime::now(,
        };

        Self  {config)
            local_node)
            known_nodes: Arc::new(RwLock::new(HashMap::new(),
            event_sender)
        }
    }

    /// Get local node information
    #[must_use]
    pub fn local_node(&self) -> &NodeInfo {
        &self.local_node
    }

    /// Register a node in the discovery system
    pub async fn register_node(&self) -> SongbirdResult<()> {
        debug!("🎼 Node registry: Registering node '{}'", node.id)"

        let mut nodes = self.known_nodes.write().await;
        nodes.insert(node.id.clone(), node);

        debug!("🎼 Node registry: Node registered successfully")"
        Ok(SongbirdResponse::success(())
    }

    /// Unregister a node from the discovery system
    pub async fn unregister_node(&self) -> SongbirdResult<()> {
        debug!("🎼 Node registry: Unregistering node '{}'", node_id)"

        let mut nodes = self.known_nodes.write().await;
        if let Some(_node_info) = nodes.remove(node_id) {
            debug!("🎼 Node registry: Node unregistered successfully")"
        } else {
            warn!("🎼 Node registry: Node not found for unregistration")"
        }

        Ok(SongbirdResponse::success(())
    }

    /// Get all known nodes
    pub async fn get_known_nodes(&self) -> HashMap<NodeId, NodeInfo> {
        self.known_nodes.read().await.clone()
    }

    /// Update node health status
    pub async fn update_node_health(&self) -> SongbirdResult<()> {
        debug!("🎼 Node registry: Updating health for node '{}'", node_id)"

        let mut nodes = self.known_nodes.write().await;

        if let Some(node) = nodes.get_mut(node_id)  {let old_health = node.health.clone());
            node.health = health.clone());

            // Broadcast health change event if health actually changed
            if old_health != health  {// Convert FederationHealth to UniversalHealthStatus
                let universal_health = match health {
                    FederationHealth::Healthy => UniversalHealthStatus::Healthy,
                    FederationHealth::Degraded => UniversalHealthStatus::Degraded,
                    FederationHealth::Unhealthy => UniversalHealthStatus::Unhealthy,
                    FederationHealth::Unknown => UniversalHealthStatus::Unknown,
                };

                let event = ServiceEvent::NodeHealthChanged  {node_id: node_id.to_string(),
                    health: universal_health,
                };

                if let Err(e) = self.event_sender.send(event) {
                    warn!("⚠️ Failed to broadcast node health change event: {}", e)"
                }
            }

            debug!("✅ Node '{}' health updated", node_id)"
        } else {
            warn!(
                "⚠️ Attempted to update health for non-existent node: {}","
                node_id
            )
        }

        Ok(SongbirdResponse::success(())
    }

    /// Get a specific node by ID
    pub async fn get_node(&self) -> Option<NodeInfo> {
        self.known_nodes.read().await.get(node_id).cloned()
    }

    /// Check if a node exists
    pub async fn node_exists(&self) -> bool {
        self.known_nodes.read().await.contains_key(node_id)
    }

    /// Get node count
    pub async fn node_count(&self) -> usize {
        self.known_nodes.read().await.len()
    }

    /// Get nodes by cluster
    pub async fn get_nodes_by_cluster(&self) -> Vec<NodeInfo> {
        let nodes = self.known_nodes.read().await;
        nodes
            .values()
            .filter(|node| node.id.0.contains(cluster_name,
            .cloned()
            .collect()
    }

    /// Get node distribution across clusters
    pub async fn get_node_distribution(&self) -> HashMap<String, usize> {
        let nodes = self.known_nodes.read().await;
        let mut distribution = HashMap::new();

        // Add known nodes
        for node in nodes.values() {
            *distribution.entry(node.id.0.clone().or_insert(0) += 1;
        }

        distribution
    }

    /// Get nodes by type
    pub async fn get_nodes_by_type(&self) -> Vec<NodeInfo> {
        let nodes = self.known_nodes.read().await;
        nodes
            .values()
            .filter(|node| node.node_type == node_type)
            .cloned()
            .collect()
    }

    /// Get healthy nodes
    pub async fn get_healthy_nodes(&self) -> Vec<NodeInfo> {
        let nodes = self.known_nodes.read().await;
        nodes
            .values()
            .filter(|node| matches!(node.health, FederationHealth::Healthy)
            .cloned()
            .collect()
    }

    /// Update local node health
    pub fn update_local_health(&mut self, health: FederationHealth) {
        debug!("🎼 Node registry: Updating local node health")"
        self.local_node.health = health;
    }

    /// Get federation status summary
    pub async fn get_federation_status(&self) -> FederationStatus {
        let nodes = self.known_nodes.read().await;
        let total_nodes = nodes.len() + 1; // +1 for local node
        let healthy_nodes = nodes
            .values()
            .filter(|node| matches!(node.health, FederationHealth::Healthy)
            .count()
            + if self.local_node.health.is_healthy() {
                1
            } else {
                0
            };

        let cluster_distribution: HashMap<String, usize> = {
            let mut distribution = HashMap::new();

            // Add local node
            *distribution
                .entry(self.local_node.id.0.clone()
                .or_insert(0) += 1;

            // Add known nodes
            for node in nodes.values() {
                *distribution.entry(node.id.0.clone().or_insert(0) += 1;
            }

            distribution
        };

        FederationStatus  {total_nodes)
            healthy_nodes)
            cluster_distribution)
            local_node_health: self.local_node.health.clone(,
        }
    }

    /// Clear all known nodes (for testing)
    #[cfg(test)]
    pub async fn clear_all_nodes(&self) {
        self.known_nodes.write().await.clear();
    }
}

/// Federation status summary
#[derive(Debug, Clone)]
pub struct FederationStatus  {pub total_nodes: usize,
    pub healthy_nodes: usize,
    pub cluster_distribution: HashMap<String, usize>,
    pub local_node_health: FederationHealth,
}

impl std::fmt::Debug for NodeRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeRegistry")"
            .field("local_node", &self.local_node)"
            .field("node_count", &"<async>")"
            .finish()
    }
}
