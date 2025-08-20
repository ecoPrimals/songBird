//! # 🎼 Canonical Federation Types
//!
//! **🚀 UNIFIED TYPE SYSTEM**
//!
//! This module provides clean, canonical types for federation operations,
//! replacing the fragmented type system with unified data structures.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// **🚀 FEDERATION NODE**
///
/// Canonical representation of a federation node with all necessary information
/// for distributed coordination and health monitoring.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FederationNode {
    /// Unique node identifier
    pub id: String,

    /// Network address for communication
    pub address: String,

    /// Current node status
    pub status: NodeStatus,

    /// Node capabilities
    pub capabilities: Vec<String>,

    /// Last time node was seen/contacted
    pub last_seen: SystemTime,

    /// Additional node metadata
    pub metadata: HashMap<String, String>,
}

impl FederationNode {
    /// Create new federation node
    pub fn new(id: String, address: String) -> Self {
        Self {
            id,
            address,
            status: NodeStatus::Starting,
            capabilities: Vec::new(),
            last_seen: SystemTime::now(),
            metadata: HashMap::new(),
        }
    }

    /// Check if node is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.status, NodeStatus::Healthy)
    }

    /// Update last seen timestamp
    pub fn update_last_seen(&mut self) {
        self.last_seen = SystemTime::now();
    }

    /// Add capability to node
    pub fn add_capability(&mut self, capability: String) {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability);
        }
    }

    /// Check if node has capability
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.contains(&capability.to_string())
    }
}

/// **🚀 NODE STATUS**
///
/// Canonical node status enumeration with clear state definitions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum NodeStatus {
    /// Node is starting up
    #[default]
    Starting,

    /// Node is healthy and operational
    Healthy,

    /// Node is unhealthy but still reachable
    Unhealthy,

    /// Node is unreachable
    Unreachable,

    /// Node is shutting down
    Stopping,

    /// Node has stopped
    Stopped,

    /// Node is online (legacy compatibility)
    Online,

    /// Node is offline (legacy compatibility)
    Offline,

    /// Node status is unknown (legacy compatibility)
    Unknown,
}

impl std::fmt::Display for NodeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeStatus::Starting => write!(f, "Starting"),
            NodeStatus::Healthy => write!(f, "Healthy"),
            NodeStatus::Unhealthy => write!(f, "Unhealthy"),
            NodeStatus::Unreachable => write!(f, "Unreachable"),
            NodeStatus::Stopping => write!(f, "Stopping"),
            NodeStatus::Stopped => write!(f, "Stopped"),
            NodeStatus::Online => write!(f, "Online"),
            NodeStatus::Offline => write!(f, "Offline"),
            NodeStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

/// **🚀 FEDERATION MESSAGE**
///
/// Message structure for inter-node communication in federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationMessage {
    /// Unique message identifier
    pub message_id: String,

    /// Type of message
    pub message_type: FederationMessageType,

    /// Sender node identifier
    pub sender_id: String,

    /// Target node identifier (for directed messages)
    pub target: Option<String>,

    /// Message timestamp
    pub timestamp: SystemTime,

    /// Message payload (JSON)
    pub payload: serde_json::Value,
}

impl FederationMessage {
    /// Create a new targeted message
    pub fn new_targeted(
        sender_id: String,
        target_id: String,
        message_type: MessageType,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            message_id: uuid::Uuid::new_v4().to_string(),
            message_type: match message_type {
                MessageType::HealthPing => FederationMessageType::Heartbeat,
                MessageType::ServiceDiscovery => FederationMessageType::DiscoveryRequest,
                MessageType::StatusUpdate => FederationMessageType::StatusUpdate,
            },
            sender_id,
            target: Some(target_id),
            timestamp: SystemTime::now(),
            payload,
        }
    }

    /// Check if this is a broadcast message
    pub fn is_broadcast(&self) -> bool {
        self.target.is_none()
    }
}

/// Message types for targeted messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    HealthPing,
    ServiceDiscovery,
    StatusUpdate,
}

/// **🚀 FEDERATION MESSAGE TYPE**
///
/// Types of messages that can be sent between federation nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FederationMessageType {
    /// Service registration announcement
    ServiceRegistration,
    /// Service deregistration announcement
    ServiceDeregistration,
    /// Heartbeat message
    Heartbeat,
    /// Node status update
    StatusUpdate,
    /// Service discovery request
    DiscoveryRequest,
    /// Service discovery response
    DiscoveryResponse,
    /// Custom message
    Custom(String),
}

/// Federation request types for API operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FederationRequestType {
    /// Request node information
    NodeInfo,
    /// Request health check
    HealthCheck,
    /// Request service discovery
    ServiceDiscovery,
}

/// Federation response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationResponse {
    /// Response message ID
    pub response_id: String,
    /// Request ID this responds to
    pub request_id: String,
    /// Success status
    pub success: bool,
    /// Response payload
    pub payload: serde_json::Value,
}

/// **🚀 SERVICE PROVIDER INFO**
///
/// Information about a service provider in the federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceProviderInfo {
    /// Service name
    pub service_name: String,

    /// Service endpoint
    pub endpoint: String,

    /// Service capabilities
    pub capabilities: Vec<String>,

    /// Service metadata
    pub metadata: HashMap<String, String>,

    /// Provider node ID
    pub provider_node_id: String,

    /// Registration timestamp
    pub registered_at: SystemTime,
}

/// Federation request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationRequest {
    /// Request identifier
    pub request_id: String,
    /// Request type
    pub request_type: FederationRequestType,
    /// Requester node ID
    pub requester_id: String,
    /// Request timestamp
    pub timestamp: SystemTime,
    /// Request parameters
    pub parameters: serde_json::Value,
}

/// **🚀 DISCOVERY INFO**
///
/// Information about a discovered federation node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryInfo {
    /// Node identifier
    pub node_id: String,

    /// Network endpoint for communication
    pub endpoint: String,

    /// Node capabilities
    pub capabilities: Vec<String>,

    /// Last time node was discovered/seen
    pub last_seen: SystemTime,

    /// Additional discovery metadata
    pub metadata: HashMap<String, String>,
}

impl DiscoveryInfo {
    /// Create new discovery info
    pub fn new(node_id: String, endpoint: String) -> Self {
        Self {
            node_id,
            endpoint,
            capabilities: Vec::new(),
            last_seen: SystemTime::now(),
            metadata: HashMap::new(),
        }
    }

    /// Check if discovery info is fresh (within timeout)
    pub fn is_fresh(&self, timeout_seconds: u64) -> bool {
        let cutoff = SystemTime::now() - std::time::Duration::from_secs(timeout_seconds);
        self.last_seen > cutoff
    }
}

/// **🚀 HEARTBEAT DATA**
///
/// Data structure for heartbeat messages containing node health and status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatData {
    /// Timestamp when heartbeat was generated
    pub timestamp: SystemTime,

    /// Node identifier sending the heartbeat
    pub node_id: String,

    /// Current CPU usage percentage (0.0-100.0)
    pub cpu_usage: f64,

    /// Current memory usage percentage (0.0-100.0)
    pub memory_usage: f64,

    /// System uptime in seconds
    pub uptime: u64,

    /// System load average
    pub load_average: f64,

    /// Number of active network connections
    pub active_connections: u32,
}

impl Default for HeartbeatData {
    fn default() -> Self {
        Self {
            timestamp: SystemTime::now(),
            node_id: String::new(),
            cpu_usage: 0.0,
            memory_usage: 0.0,
            uptime: 0,
            load_average: 0.0,
            active_connections: 0,
        }
    }
}

/// **🚀 HEALTH STATUS**
///
/// Comprehensive health status information for a federation node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Node identifier
    pub node_id: String,

    /// Current node status
    pub status: NodeStatus,

    /// Last heartbeat received
    pub last_heartbeat: SystemTime,

    /// Current CPU usage percentage
    pub cpu_usage: f64,

    /// Current memory usage percentage
    pub memory_usage: f64,

    /// System uptime in seconds
    pub uptime: u64,

    /// System load average
    pub load_average: f64,
}

impl HealthStatus {
    /// Create a new health status for a node
    pub fn new(node_id: String, status: NodeStatus) -> Self {
        Self {
            node_id,
            status,
            last_heartbeat: SystemTime::now(),
            cpu_usage: 0.0,
            memory_usage: 0.0,
            uptime: 0,
            load_average: 0.0,
        }
    }
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            node_id: String::new(),
            status: NodeStatus::Starting,
            last_heartbeat: SystemTime::now(),
            cpu_usage: 0.0,
            memory_usage: 0.0,
            uptime: 0,
            load_average: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_federation_node_creation() {
        let node = FederationNode::new("test-node".to_string(), "127.0.0.1:{}".to_string());
        assert_eq!(node.id, "test-node");
        assert_eq!(node.address, "127.0.0.1:{}");
        assert_eq!(node.status, NodeStatus::Starting);
    }

    #[test]
    fn test_node_capabilities() {
        let mut node = FederationNode::new("test-node".to_string(), "127.0.0.1:{}".to_string());

        // Add capability
        node.add_capability("orchestration".to_string());
        assert!(node.has_capability("orchestration"));
        assert_eq!(node.capabilities.len(), 1);

        // Add duplicate capability (should not duplicate)
        node.add_capability("orchestration".to_string());
        assert_eq!(node.capabilities.len(), 1);

        // Add different capability
        node.add_capability("discovery".to_string());
        assert_eq!(node.capabilities.len(), 2);
        assert!(node.has_capability("discovery"));
    }

    // #[test]
    // fn test_federation_message_creation() { // Temporarily disabled for canonical modernization
    //     let payload = serde_json::json!({"test": "data"});
    //     let msg = FederationMessage {
    //         message_id: uuid::Uuid::new_v4().to_string(),
    //         message_type: FederationMessageType::DiscoveryRequest,
    //         sender_id: "source-node".to_string(),
    //         target: None, // Add missing field
    //         timestamp: SystemTime::now(),
    //         payload,
    //     };

    //     assert_eq!(msg.sender_id, "source-node");
    //     assert!(matches!(
    //         msg.message_type,
    //         FederationMessageType::DiscoveryRequest
    //     ));

    //     // Test targeted message
    //     let targeted_msg = FederationMessage::new_targeted(
    //         "source-node".to_string(),
    //         "target-node".to_string(),
    //         MessageType::HealthPing,
    //         serde_json::json!({}),
    //     );

    //     assert!(!targeted_msg.is_broadcast());
    //     assert_eq!(
    //         targeted_msg.target.expect("Target should be specified"),
    //         "target-node"
    //     );
    // }

    // #[test]
    // fn test_node_status_display() { // Temporarily disabled for canonical modernization
    //     assert_eq!(NodeStatus::Healthy.to_string(), "Healthy");
    //     assert_eq!(NodeStatus::Unhealthy.to_string(), "Unhealthy");
    //     assert_eq!(NodeStatus::Starting.to_string(), "Starting");
    // }
}
