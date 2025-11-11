//! Federation State Management
//!
//! Manages the state of federated nodes and their registrations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Federation state - tracks all nodes in the federation
#[derive(Debug, Clone)]
pub struct FederationState {
    /// Unique federation identifier
    pub federation_id: Uuid,
    
    /// Map of `node_id` to node registration
    pub nodes: Arc<RwLock<HashMap<String, NodeRegistration>>>,
    
    /// When this federation was created
    pub created_at: DateTime<Utc>,
}

impl FederationState {
    /// Create a new federation state
    #[must_use]
    pub fn new() -> Self {
        Self {
            federation_id: Uuid::new_v4(),
            nodes: Arc::new(RwLock::new(HashMap::new())),
            created_at: Utc::now(),
        }
    }
    
    /// Add or update a node registration
    pub async fn register_node(&self, registration: NodeRegistration) {
        let mut nodes = self.nodes.write().await;
        nodes.insert(registration.node_id.clone(), registration);
    }
    
    /// Remove a node from the federation
    pub async fn remove_node(&self, node_id: &str) {
        let mut nodes = self.nodes.write().await;
        nodes.remove(node_id);
    }
    
    /// Update node heartbeat
    pub async fn update_heartbeat(&self, node_id: &str) {
        let mut nodes = self.nodes.write().await;
        if let Some(node) = nodes.get_mut(node_id) {
            node.last_heartbeat = Utc::now();
            node.status = NodeStatus::Active;
        }
    }
    
    /// Mark nodes as inactive if they haven't sent heartbeat
    pub async fn check_node_health(&self, timeout_secs: i64) {
        let mut nodes = self.nodes.write().await;
        let now = Utc::now();
        
        for node in nodes.values_mut() {
            let elapsed = (now - node.last_heartbeat).num_seconds();
            if elapsed > timeout_secs {
                node.status = NodeStatus::Inactive;
            }
        }
    }
    
    /// Get all active nodes
    pub async fn active_nodes(&self) -> Vec<NodeRegistration> {
        let nodes = self.nodes.read().await;
        nodes
            .values()
            .filter(|n| matches!(n.status, NodeStatus::Active))
            .cloned()
            .collect()
    }
    
    /// Get total federation stats
    pub async fn get_stats(&self) -> FederationStats {
        let nodes = self.nodes.read().await;
        let active_nodes: Vec<_> = nodes
            .values()
            .filter(|n| matches!(n.status, NodeStatus::Active))
            .collect();
        
        FederationStats {
            total_nodes: nodes.len(),
            active_nodes: active_nodes.len(),
            total_cpu_cores: active_nodes.iter().map(|n| n.cpu_cores).sum(),
            total_memory_gb: active_nodes.iter().map(|n| n.memory_gb).sum(),
            total_storage_gb: active_nodes
                .iter()
                .filter_map(|n| n.storage_gb)
                .sum(),
        }
    }
}

impl Default for FederationState {
    fn default() -> Self {
        Self::new()
    }
}

/// Node registration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRegistration {
    /// Unique node identifier
    pub node_id: String,
    
    /// Human-readable node name
    pub node_name: String,
    
    /// Network address (IP:PORT or hostname:PORT)
    pub node_address: String,
    
    /// Number of CPU cores
    pub cpu_cores: usize,
    
    /// Memory in GB
    pub memory_gb: usize,
    
    /// GPU model if available
    pub gpu_model: Option<String>,
    
    /// Storage in GB if available
    pub storage_gb: Option<usize>,
    
    /// Node capabilities
    pub capabilities: Vec<String>,
    
    /// Current node status
    pub status: NodeStatus,
    
    /// When node joined federation
    pub joined_at: DateTime<Utc>,
    
    /// Last heartbeat received
    pub last_heartbeat: DateTime<Utc>,
}

/// Node status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    /// Node is active and responsive
    Active,
    
    /// Node has not sent heartbeat recently
    Inactive,
    
    /// Node is experiencing issues
    Unhealthy,
}

impl std::fmt::Display for NodeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Inactive => write!(f, "inactive"),
            Self::Unhealthy => write!(f, "unhealthy"),
        }
    }
}

/// Federation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationStats {
    /// Total number of nodes (including inactive)
    pub total_nodes: usize,
    
    /// Number of active nodes
    pub active_nodes: usize,
    
    /// Total CPU cores across active nodes
    pub total_cpu_cores: usize,
    
    /// Total memory in GB across active nodes
    pub total_memory_gb: usize,
    
    /// Total storage in GB across active nodes
    pub total_storage_gb: usize,
}

/// Federation status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationStatus {
    /// Federation unique ID
    pub federation_id: String,
    
    /// Number of active nodes
    pub active_nodes: usize,
    
    /// All registered nodes
    pub nodes: Vec<NodeRegistration>,
    
    /// Total resources
    pub total_cpu_cores: usize,
    pub total_memory_gb: usize,
    pub total_storage_gb: usize,
    
    /// Federation uptime in seconds
    pub uptime_seconds: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_federation_state_creation() {
        let state = FederationState::new();
        assert_eq!(state.nodes.read().await.len(), 0);
    }
    
    #[tokio::test]
    async fn test_node_registration() {
        let state = FederationState::new();
        
        let registration = NodeRegistration {
            node_id: "test-node".to_string(),
            node_name: "Test Node".to_string(),
            node_address: "192.168.1.100:8080".to_string(),
            cpu_cores: 8,
            memory_gb: 16,
            gpu_model: Some("RTX 3070".to_string()),
            storage_gb: Some(500),
            capabilities: vec!["compute".to_string()],
            status: NodeStatus::Active,
            joined_at: Utc::now(),
            last_heartbeat: Utc::now(),
        };
        
        state.register_node(registration.clone()).await;
        
        let nodes = state.nodes.read().await;
        assert_eq!(nodes.len(), 1);
        assert!(nodes.contains_key("test-node"));
    }
    
    #[tokio::test]
    async fn test_heartbeat_update() {
        let state = FederationState::new();
        
        let registration = NodeRegistration {
            node_id: "test-node".to_string(),
            node_name: "Test Node".to_string(),
            node_address: "192.168.1.100:8080".to_string(),
            cpu_cores: 8,
            memory_gb: 16,
            gpu_model: None,
            storage_gb: None,
            capabilities: vec![],
            status: NodeStatus::Active,
            joined_at: Utc::now(),
            last_heartbeat: Utc::now() - chrono::Duration::seconds(100),
        };
        
        state.register_node(registration).await;
        state.update_heartbeat("test-node").await;
        
        let nodes = state.nodes.read().await;
        let node = nodes.get("test-node").unwrap();
        
        let elapsed = (Utc::now() - node.last_heartbeat).num_seconds();
        assert!(elapsed < 5); // Should be very recent
    }
}

