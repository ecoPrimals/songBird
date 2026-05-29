// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

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
    pub fn new(_federation_id: String) -> Self {
        Self {
            federation_id: Uuid::new_v4(), // Still generate a UUID, but accept string for API compatibility
            nodes: Arc::new(RwLock::new(HashMap::new())),
            created_at: Utc::now(),
        }
    }

    /// Add or update a node registration
    ///
    /// **Identity-Based Routing (Dec 20, 2025)**:
    /// - If `node_id` already exists, merge endpoints instead of replacing
    /// - This enables multi-interface coalescence (Ethernet + `WiFi` = 1 node)
    /// - Multiple Songbird subsystems per tower can coexist
    pub async fn register_node(&self, registration: NodeRegistration) {
        let mut nodes = self.nodes.write().await;

        // Check if this node_id already exists
        if let Some(existing) = nodes.get_mut(&registration.node_id) {
            // Node exists - coalesce endpoints
            tracing::debug!(
                "🔄 Coalescing endpoints for existing node '{}' ({})",
                existing.node_name,
                &existing.node_id[..8.min(existing.node_id.len())]
            );

            // Update heartbeat and status
            existing.last_heartbeat = Utc::now();
            existing.status = NodeStatus::Active;

            // Merge endpoints if new registration has any
            if let Some(new_endpoints) = registration.endpoints {
                for endpoint in new_endpoints {
                    existing.add_endpoint(endpoint);
                }
                tracing::info!(
                    "✅ Added {} endpoint(s) to '{}' (total: {})",
                    1,
                    existing.node_name,
                    existing.endpoints.as_ref().map_or(0, std::vec::Vec::len)
                );
            }

            // Update primary address if different (keep most recent)
            if existing.node_address != registration.node_address {
                tracing::debug!(
                    "🔄 Updated primary address for '{}': {} -> {}",
                    existing.node_name,
                    existing.node_address,
                    registration.node_address
                );
                existing.node_address = registration.node_address;
            }

            // Merge capabilities (union)
            for capability in registration.capabilities {
                if !existing.capabilities.contains(&capability) {
                    existing.capabilities.push(capability);
                }
            }
        } else {
            // New node - insert
            tracing::info!(
                "✅ Registering new node '{}' ({}) at {}",
                registration.node_name,
                &registration.node_id[..8.min(registration.node_id.len())],
                registration.node_address
            );
            nodes.insert(registration.node_id.clone(), registration);
        }
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

    /// Remove stale nodes that haven't sent heartbeat within TTL
    ///
    /// Deep Debt Fix (Dec 20, 2025):
    /// - Session IDs rotate every hour, creating new "nodes"
    /// - Old sessions were never removed, accumulating indefinitely
    /// - This led to 69 registered nodes for 4 physical towers (94% stale!)
    /// - Now: Remove nodes after TTL expiration (default 10 minutes)
    ///
    /// TTL Strategy:
    /// - Grace period: 2x heartbeat interval (10 min = 2 * 5 min)
    /// - Allows for network hiccups and temporary disconnections
    /// - But prevents indefinite accumulation of rotated sessions
    pub async fn cleanup_stale_nodes(&self, ttl_secs: i64) -> usize {
        let (removed_count, initial_count, final_count) = {
            let mut nodes = self.nodes.write().await;
            let now = Utc::now();
            let initial_count = nodes.len();

            // Retain only nodes that have sent heartbeat within TTL
            nodes.retain(|node_id, node| {
                let elapsed = (now - node.last_heartbeat).num_seconds();
                let should_keep = elapsed < ttl_secs;

                if !should_keep {
                    tracing::debug!(
                        "🧹 Removing stale node {} (last seen {} seconds ago)",
                        &node_id[..8.min(node_id.len())],
                        elapsed
                    );
                }

                should_keep
            });

            let removed_count = initial_count - nodes.len();
            let final_count = nodes.len();
            drop(nodes);
            (removed_count, initial_count, final_count)
        };

        if removed_count > 0 {
            tracing::info!(
                "🧹 Cleaned up {} stale nodes. Active: {} (was: {})",
                removed_count,
                final_count,
                initial_count
            );
        }

        removed_count
    }

    /// Get all active nodes
    pub async fn active_nodes(&self) -> Vec<NodeRegistration> {
        let nodes = self.nodes.read().await;
        nodes.values().filter(|n| matches!(n.status, NodeStatus::Active)).cloned().collect()
    }

    /// Get total federation stats
    pub async fn get_stats(&self) -> FederationStats {
        let nodes = self.nodes.read().await;
        let active_nodes: Vec<_> =
            nodes.values().filter(|n| matches!(n.status, NodeStatus::Active)).collect();

        let uptime = u64::try_from((Utc::now() - self.created_at).num_seconds().max(0)).ok();

        FederationStats {
            total_nodes: nodes.len(),
            active_nodes: active_nodes.len(),
            total_cpu_cores: active_nodes.iter().map(|n| n.cpu_cores).sum(),
            total_memory_gb: active_nodes.iter().map(|n| n.memory_gb).sum(),
            total_storage_gb: active_nodes.iter().filter_map(|n| n.storage_gb).sum(),
            uptime_seconds: uptime,
        }
    }

    /// Get best endpoint for a node (identity-based routing)
    ///
    /// **Routing Strategy**:
    /// 1. Prefer endpoints marked as active
    /// 2. Sort by preference value (highest first)
    /// 3. Fall back to primary `node_address` if no endpoints
    pub async fn get_best_endpoint(&self, node_id: &str) -> Option<String> {
        let node = self.nodes.read().await.get(node_id).cloned()?;

        // Try to get preferred endpoint
        if let Some(endpoint) = node.preferred_endpoint() {
            return Some(format!("https://{}", endpoint.address));
        }

        // Fall back to primary address
        Some(node.node_address)
    }

    /// Get all endpoints for a node (for connection fallback)
    pub async fn get_all_endpoints(&self, node_id: &str) -> Vec<String> {
        let Some(node) = self.nodes.read().await.get(node_id).cloned() else {
            return vec![];
        };

        let mut endpoints = vec![];

        // Add all active endpoints
        for endpoint in node.active_endpoints() {
            endpoints.push(format!("https://{}", endpoint.address));
        }

        // Add primary address as fallback
        if !endpoints.contains(&node.node_address) {
            endpoints.push(node.node_address);
        }

        endpoints
    }
}

impl Default for FederationState {
    fn default() -> Self {
        Self::new("default".to_string())
    }
}

/// Node registration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRegistration {
    /// Unique node identifier
    pub node_id: String,

    /// Human-readable node name
    pub node_name: String,

    /// Primary network address (IP:PORT or hostname:PORT)
    ///
    /// This is the preferred/primary endpoint for backward compatibility.
    /// For multi-path support, use `endpoints` field.
    pub node_address: String,

    /// All transport endpoints for this node (v3.0+)
    ///
    /// Each endpoint represents a different network interface (Ethernet, `WiFi`, etc.)
    /// For backward compatibility, this is optional. If None, use `node_address`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoints: Option<Vec<TransportEndpointInfo>>,

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

/// Transport endpoint information (v3.0+)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransportEndpointInfo {
    /// Interface type (e.g., "ethernet", "wifi", "bluetooth")
    pub interface_type: String,

    /// Network address for this endpoint
    pub address: String,

    /// Supported protocols on this endpoint
    pub protocols: Vec<String>,

    /// Relative preference (0-255, higher = more preferred)
    pub preference: u8,

    /// Endpoint status
    pub status: EndpointStatus,

    /// Last health check for this endpoint
    pub last_check: DateTime<Utc>,
}

/// Endpoint status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EndpointStatus {
    /// Endpoint is active and responding
    Active,

    /// Endpoint is on standby (not currently used)
    Standby,

    /// Endpoint is degraded (high latency, packet loss)
    Degraded,

    /// Endpoint is failed (unreachable)
    Failed,
}

impl NodeRegistration {
    /// Add or update a transport endpoint
    pub fn add_endpoint(&mut self, endpoint: TransportEndpointInfo) {
        if let Some(ref mut endpoints) = self.endpoints {
            // Remove existing endpoint with same address
            endpoints.retain(|e| e.address != endpoint.address);
            endpoints.push(endpoint);

            // Sort by preference (highest first)
            endpoints.sort_by(|a, b| b.preference.cmp(&a.preference));
        } else {
            // Create new endpoints vector
            self.endpoints = Some(vec![endpoint]);
        }
    }

    /// Get preferred endpoint (highest preference and active)
    #[must_use]
    pub fn preferred_endpoint(&self) -> Option<&TransportEndpointInfo> {
        self.endpoints
            .as_ref()?
            .iter()
            .filter(|e| matches!(e.status, EndpointStatus::Active))
            .max_by_key(|e| e.preference)
    }

    /// Get all active endpoints
    #[must_use]
    pub fn active_endpoints(&self) -> Vec<&TransportEndpointInfo> {
        self.endpoints
            .as_ref()
            .map(|endpoints| {
                endpoints.iter().filter(|e| matches!(e.status, EndpointStatus::Active)).collect()
            })
            .unwrap_or_default()
    }

    /// Update endpoint status by address
    pub fn update_endpoint_status(&mut self, address: &str, status: EndpointStatus) {
        if let Some(ref mut endpoints) = self.endpoints {
            for endpoint in endpoints.iter_mut() {
                if endpoint.address == address {
                    endpoint.status = status;
                    endpoint.last_check = Utc::now();
                    break;
                }
            }
        }
    }
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

    /// Federation uptime in seconds since creation
    pub uptime_seconds: Option<u64>,
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
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;

    #[tokio::test]
    async fn test_federation_state_creation() {
        let state = FederationState::new("test".to_string());
        assert_eq!(state.nodes.read().await.len(), 0);
    }

    #[tokio::test]
    async fn test_node_registration() {
        let state = FederationState::new("test".to_string());

        let registration = NodeRegistration {
            node_id: "test-node".to_string(),
            node_name: "Test Node".to_string(),
            node_address: "192.168.1.100:8080".to_string(),
            endpoints: None,
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
        let state = FederationState::new("test".to_string());

        let registration = NodeRegistration {
            node_id: "test-node".to_string(),
            node_name: "Test Node".to_string(),
            node_address: "192.168.1.100:8080".to_string(),
            endpoints: None,
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

    #[tokio::test]
    async fn register_node_merges_capabilities() {
        let state = FederationState::new("x".into());
        let mut r1 = make_registration("n", "addr1");
        r1.capabilities = vec!["a".into()];
        state.register_node(r1).await;
        let mut r2 = make_registration("n", "addr1");
        r2.capabilities = vec!["b".into()];
        state.register_node(r2).await;
        let nodes = state.nodes.read().await;
        let n = nodes.get("n").unwrap();
        assert!(n.capabilities.contains(&"a".into()));
        assert!(n.capabilities.contains(&"b".into()));
    }

    fn make_registration(id: &str, addr: &str) -> NodeRegistration {
        NodeRegistration {
            node_id: id.into(),
            node_name: id.into(),
            node_address: addr.into(),
            endpoints: None,
            cpu_cores: 1,
            memory_gb: 1,
            gpu_model: None,
            storage_gb: None,
            capabilities: vec![],
            status: NodeStatus::Active,
            joined_at: Utc::now(),
            last_heartbeat: Utc::now(),
        }
    }

    #[test]
    fn transport_endpoint_preference_sorts_in_add_endpoint() {
        let mut reg = make_registration_sync("n", "a");
        reg.add_endpoint(TransportEndpointInfo {
            interface_type: "e".into(),
            address: "192.168.1.1:1".into(),
            protocols: vec![],
            preference: 10,
            status: EndpointStatus::Active,
            last_check: Utc::now(),
        });
        reg.add_endpoint(TransportEndpointInfo {
            interface_type: "e".into(),
            address: "192.168.1.2:2".into(),
            protocols: vec![],
            preference: 200,
            status: EndpointStatus::Active,
            last_check: Utc::now(),
        });
        let pref = reg.preferred_endpoint().unwrap();
        assert_eq!(pref.address, "192.168.1.2:2");
    }

    fn make_registration_sync(id: &str, addr: &str) -> NodeRegistration {
        NodeRegistration {
            node_id: id.into(),
            node_name: id.into(),
            node_address: addr.into(),
            endpoints: None,
            cpu_cores: 0,
            memory_gb: 0,
            gpu_model: None,
            storage_gb: None,
            capabilities: vec![],
            status: NodeStatus::Active,
            joined_at: Utc::now(),
            last_heartbeat: Utc::now(),
        }
    }

    #[test]
    fn federation_stats_serde_roundtrip() {
        let s = FederationStats {
            total_nodes: 3,
            active_nodes: 2,
            total_cpu_cores: 4,
            total_memory_gb: 8,
            total_storage_gb: 16,
            uptime_seconds: Some(42),
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: FederationStats = serde_json::from_str(&json).unwrap();
        assert_eq!(back.active_nodes, 2);
    }

    #[tokio::test]
    async fn cleanup_stale_nodes_removes_old() {
        let state = FederationState::new("x".into());
        let mut r = make_registration("old", "a");
        r.last_heartbeat = Utc::now() - chrono::Duration::seconds(9999);
        state.register_node(r).await;
        let n = state.cleanup_stale_nodes(100).await;
        assert_eq!(n, 1);
        assert_eq!(state.nodes.read().await.len(), 0);
    }

    #[tokio::test]
    async fn federation_state_default_creates_empty() {
        let state = FederationState::default();
        assert_eq!(state.nodes.read().await.len(), 0);
    }

    #[test]
    fn node_status_display() {
        assert_eq!(NodeStatus::Active.to_string(), "active");
        assert_eq!(NodeStatus::Inactive.to_string(), "inactive");
    }

    #[tokio::test]
    async fn get_stats_aggregates_resources() {
        let state = FederationState::new("fed".into());
        let mut r = make_registration("n1", "https://a:1");
        r.cpu_cores = 4;
        r.memory_gb = 8;
        r.storage_gb = Some(100);
        state.register_node(r).await;
        let stats = state.get_stats().await;
        assert_eq!(stats.total_nodes, 1);
        assert_eq!(stats.active_nodes, 1);
        assert_eq!(stats.total_cpu_cores, 4);
        assert_eq!(stats.total_memory_gb, 8);
        assert_eq!(stats.total_storage_gb, 100);
    }

    #[tokio::test]
    async fn get_best_endpoint_prefers_https_wrapped_preferred() {
        let state = FederationState::new("test".into());
        let mut r = make_registration("node-x", "https://primary:443");
        r.endpoints = Some(vec![TransportEndpointInfo {
            interface_type: "eth".into(),
            address: "10.0.0.5:8443".into(),
            protocols: vec!["https".into()],
            preference: 200,
            status: EndpointStatus::Active,
            last_check: Utc::now(),
        }]);
        state.register_node(r).await;
        let best = state.get_best_endpoint("node-x").await.unwrap();
        assert!(best.contains("10.0.0.5:8443"));
    }

    #[tokio::test]
    async fn get_all_endpoints_includes_primary() {
        let state = FederationState::new("test".into());
        let mut r = make_registration("n", "https://only:1");
        r.endpoints = Some(vec![TransportEndpointInfo {
            interface_type: "eth".into(),
            address: "192.168.1.1:1".into(),
            protocols: vec![],
            preference: 10,
            status: EndpointStatus::Active,
            last_check: Utc::now(),
        }]);
        state.register_node(r).await;
        let eps = state.get_all_endpoints("n").await;
        assert!(eps.iter().any(|e| e.contains("192.168.1.1")));
    }

    #[test]
    fn federation_status_serde_roundtrip() {
        let s = FederationStatus {
            federation_id: "fid".into(),
            active_nodes: 1,
            nodes: vec![],
            total_cpu_cores: 2,
            total_memory_gb: 4,
            total_storage_gb: 8,
            uptime_seconds: 60,
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: FederationStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back.active_nodes, 1);
    }

    #[test]
    fn active_endpoints_returns_only_active() {
        let mut reg = make_registration_sync("n", "a");
        reg.endpoints = Some(vec![
            TransportEndpointInfo {
                interface_type: "eth".into(),
                address: "10.0.0.1:1".into(),
                protocols: vec![],
                preference: 100,
                status: EndpointStatus::Active,
                last_check: Utc::now(),
            },
            TransportEndpointInfo {
                interface_type: "eth".into(),
                address: "10.0.0.2:2".into(),
                protocols: vec![],
                preference: 50,
                status: EndpointStatus::Degraded,
                last_check: Utc::now(),
            },
            TransportEndpointInfo {
                interface_type: "wifi".into(),
                address: "10.0.0.3:3".into(),
                protocols: vec![],
                preference: 200,
                status: EndpointStatus::Failed,
                last_check: Utc::now(),
            },
            TransportEndpointInfo {
                interface_type: "eth".into(),
                address: "10.0.0.4:4".into(),
                protocols: vec![],
                preference: 80,
                status: EndpointStatus::Active,
                last_check: Utc::now(),
            },
        ]);
        let active = reg.active_endpoints();
        assert_eq!(active.len(), 2);
        assert!(active.iter().all(|e| e.status == EndpointStatus::Active));
    }

    #[test]
    fn active_endpoints_empty_when_none() {
        let reg = make_registration_sync("n", "a");
        assert!(reg.active_endpoints().is_empty());
    }

    #[test]
    fn active_endpoints_empty_when_all_failed() {
        let mut reg = make_registration_sync("n", "a");
        reg.endpoints = Some(vec![
            TransportEndpointInfo {
                interface_type: "eth".into(),
                address: "10.0.0.1:1".into(),
                protocols: vec![],
                preference: 100,
                status: EndpointStatus::Failed,
                last_check: Utc::now(),
            },
            TransportEndpointInfo {
                interface_type: "eth".into(),
                address: "10.0.0.2:2".into(),
                protocols: vec![],
                preference: 50,
                status: EndpointStatus::Standby,
                last_check: Utc::now(),
            },
        ]);
        assert!(reg.active_endpoints().is_empty());
    }

    #[test]
    fn update_endpoint_status_changes_matching_address() {
        let mut reg = make_registration_sync("n", "a");
        reg.endpoints = Some(vec![
            TransportEndpointInfo {
                interface_type: "eth".into(),
                address: "10.0.0.1:1".into(),
                protocols: vec![],
                preference: 100,
                status: EndpointStatus::Active,
                last_check: Utc::now() - chrono::Duration::seconds(60),
            },
            TransportEndpointInfo {
                interface_type: "eth".into(),
                address: "10.0.0.2:2".into(),
                protocols: vec![],
                preference: 50,
                status: EndpointStatus::Active,
                last_check: Utc::now() - chrono::Duration::seconds(60),
            },
        ]);
        reg.update_endpoint_status("10.0.0.1:1", EndpointStatus::Failed);
        let eps = reg.endpoints.as_ref().unwrap();
        assert_eq!(eps[0].status, EndpointStatus::Failed);
        assert_eq!(eps[1].status, EndpointStatus::Active);
        let elapsed = (Utc::now() - eps[0].last_check).num_seconds();
        assert!(elapsed < 2, "last_check should be updated to now");
    }

    #[test]
    fn update_endpoint_status_no_match_is_noop() {
        let mut reg = make_registration_sync("n", "a");
        reg.endpoints = Some(vec![TransportEndpointInfo {
            interface_type: "eth".into(),
            address: "10.0.0.1:1".into(),
            protocols: vec![],
            preference: 100,
            status: EndpointStatus::Active,
            last_check: Utc::now(),
        }]);
        reg.update_endpoint_status("nonexistent:999", EndpointStatus::Failed);
        assert_eq!(reg.endpoints.as_ref().unwrap()[0].status, EndpointStatus::Active);
    }

    #[test]
    fn update_endpoint_status_with_no_endpoints_is_noop() {
        let mut reg = make_registration_sync("n", "a");
        reg.update_endpoint_status("10.0.0.1:1", EndpointStatus::Failed);
        assert!(reg.endpoints.is_none());
    }

    #[test]
    fn preferred_endpoint_none_when_all_degraded() {
        let mut reg = make_registration_sync("n", "a");
        reg.endpoints = Some(vec![
            TransportEndpointInfo {
                interface_type: "eth".into(),
                address: "10.0.0.1:1".into(),
                protocols: vec![],
                preference: 200,
                status: EndpointStatus::Degraded,
                last_check: Utc::now(),
            },
            TransportEndpointInfo {
                interface_type: "eth".into(),
                address: "10.0.0.2:2".into(),
                protocols: vec![],
                preference: 100,
                status: EndpointStatus::Standby,
                last_check: Utc::now(),
            },
        ]);
        assert!(reg.preferred_endpoint().is_none());
    }

    #[test]
    fn preferred_endpoint_none_when_no_endpoints() {
        let reg = make_registration_sync("n", "a");
        assert!(reg.preferred_endpoint().is_none());
    }

    #[test]
    fn add_endpoint_replaces_existing_by_address() {
        let mut reg = make_registration_sync("n", "a");
        reg.add_endpoint(TransportEndpointInfo {
            interface_type: "eth".into(),
            address: "10.0.0.1:1".into(),
            protocols: vec!["http".into()],
            preference: 50,
            status: EndpointStatus::Active,
            last_check: Utc::now(),
        });
        reg.add_endpoint(TransportEndpointInfo {
            interface_type: "wifi".into(),
            address: "10.0.0.1:1".into(),
            protocols: vec!["https".into()],
            preference: 200,
            status: EndpointStatus::Active,
            last_check: Utc::now(),
        });
        let eps = reg.endpoints.as_ref().unwrap();
        assert_eq!(eps.len(), 1, "should deduplicate by address");
        assert_eq!(eps[0].interface_type, "wifi");
        assert_eq!(eps[0].preference, 200);
    }
}
