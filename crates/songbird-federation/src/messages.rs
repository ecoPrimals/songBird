/*!
 * Federation Message Types and Data Structures
 *
 * This module contains all message-related structures for federation communication:
 * - Request/Response structures
 * - Message types and enums
 * - Service information structures
 */

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Service provider information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceProviderInfo {
    /// Provider name
    pub name: String,

    /// Provider description
    pub description: String,

    /// Service capabilities
    pub capabilities: Vec<String>,

    /// Service endpoints
    pub endpoints: Vec<String>,

    /// Provider version
    pub version: String,

    /// Provider metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Federation request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationRequest {
    /// Request ID
    pub request_id: String,

    /// Request type
    pub request_type: FederationRequestType,

    /// Request data
    pub data: serde_json::Value,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Source node ID
    pub source_node: Option<String>,

    /// Target node ID (None for broadcast)
    pub target_node: Option<String>,
}

/// Federation request types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FederationRequestType {
    /// Service discovery request
    ServiceDiscovery,

    /// Data replication request
    DataReplication,

    /// Health check request
    HealthCheck,

    /// Configuration update
    ConfigUpdate,

    /// Load balancing request
    LoadBalancing,

    /// Resource allocation request
    ResourceAllocation,

    /// Node join request
    NodeJoin,

    /// Node leave request
    NodeLeave,
}

/// Federation response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationResponse {
    /// Request ID this response is for
    pub request_id: String,

    /// Whether the request was successful
    pub success: bool,

    /// Response data
    pub data: serde_json::Value,

    /// Error message if unsuccessful
    pub error_message: Option<String>,
}

/// Federated service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedServiceInfo {
    /// Service ID
    pub service_id: String,

    /// Service name
    pub service_name: String,

    /// Node ID hosting the service
    pub node_id: String,

    /// Service endpoints
    pub endpoints: Vec<String>,

    /// Service capabilities
    pub capabilities: Vec<String>,

    /// Service health status
    pub health_status: String,

    /// Service metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Federation message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationMessage {
    /// Message ID
    pub message_id: String,

    /// Message type
    pub message_type: FederationMessageType,

    /// Message data
    pub data: serde_json::Value,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Source node ID
    pub source_node: String,
}

/// Federation message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FederationMessageType {
    /// Service status update
    ServiceStatusUpdate,

    /// Node status update
    NodeStatusUpdate,

    /// Configuration change notification
    ConfigurationChange,

    /// Emergency alert
    EmergencyAlert,

    /// Load balancing update
    LoadBalancingUpdate,

    /// General announcement
    Announcement,
}
