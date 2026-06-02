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
#[path = "state_tests.rs"]
mod tests;
