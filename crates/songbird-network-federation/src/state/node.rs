// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
