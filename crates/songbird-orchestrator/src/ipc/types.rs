//! Type definitions for Unix socket JSON-RPC APIs
//!
//! v3.19.1: Modern idiomatic Rust types for biomeOS integration
//! v3.20.0: Service registry types for primal discovery

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Request to discover nodes by genetic family tags
///
/// ## Example
///
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "method": "discover_by_family",
///   "params": {
///     "family_tags": ["nat0", "lan0"],
///     "timeout_ms": 5000
///   },
///   "id": 1
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverByFamilyRequest {
    /// Family tags to filter by (e.g., ["nat0", "lan0"])
    pub family_tags: Vec<String>,

    /// Timeout in milliseconds (optional, default: 5000)
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

fn default_timeout() -> u64 {
    5000
}

/// Response containing discovered nodes filtered by family
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverByFamilyResponse {
    /// Discovered nodes matching family tags
    pub nodes: Vec<DiscoveredNode>,
}

/// Discovered node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredNode {
    /// Unique node identifier
    pub node_id: String,

    /// Human-readable node name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_name: Option<String>,

    /// Genetic families this node belongs to
    pub genetic_families: Vec<String>,

    /// Sub-federations this node is part of (optional)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sub_federations: Vec<String>,

    /// Capabilities offered by this node
    pub capabilities: Vec<String>,

    /// BTSP endpoint for encrypted tunnels (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub btsp_endpoint: Option<String>,

    /// HTTPS endpoint (fallback)
    pub https_endpoint: String,

    /// When this node was last seen (ISO 8601)
    pub last_seen: String,
}

/// Request to create an encrypted BTSP tunnel using genetic proof
///
/// ## Example
///
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "method": "create_genetic_tunnel",
///   "params": {
///     "peer_node_id": "node-beta",
///     "peer_endpoint": "udp://192.168.1.101:4433",
///     "genetic_proof": {
///       "family_id": "nat0",
///       "parent_seed_hash": "abc123",
///       "relationship": "sibling"
///     }
///   },
///   "id": 2
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGeneticTunnelRequest {
    /// Target peer node ID
    pub peer_node_id: String,

    /// Peer endpoint (optional, will use discovered endpoint if not provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_endpoint: Option<String>,

    /// Genetic proof from BearDog (optional, will verify via BearDog if not provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genetic_proof: Option<GeneticProof>,
}

/// Genetic lineage proof from BearDog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneticProof {
    /// Family identifier (e.g., "nat0")
    pub family_id: String,

    /// Parent seed hash (from BearDog verification)
    pub parent_seed_hash: String,

    /// Relationship (e.g., "sibling", "parent", "child")
    pub relationship: String,
}

/// Response after creating a BTSP tunnel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGeneticTunnelResponse {
    /// Unique tunnel identifier
    pub tunnel_id: String,

    /// Tunnel status ("establishing", "established", "failed")
    pub status: String,

    /// Local endpoint for this tunnel
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_endpoint: Option<String>,

    /// Remote peer endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_endpoint: Option<String>,

    /// Encryption algorithm (from BearDog)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption: Option<String>,

    /// When the tunnel was created (ISO 8601)
    pub created_at: String,
}

/// Request to announce capabilities and genetic families
///
/// ## Example
///
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "method": "announce_capabilities",
///   "params": {
///     "capabilities": ["storage", "compute"],
///     "sub_federations": ["gaming", "family"],
///     "genetic_families": ["nat0", "lan0"]
///   },
///   "id": 3
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnounceCapabilitiesRequest {
    /// Capabilities to announce
    pub capabilities: Vec<String>,

    /// Sub-federations this node is part of (optional)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sub_federations: Vec<String>,

    /// Genetic families this node belongs to (optional)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub genetic_families: Vec<String>,
}

/// Response after updating capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnounceCapabilitiesResponse {
    /// Status ("updated", "broadcasting", "failed")
    pub status: String,

    /// Whether broadcasting is active
    pub broadcasting: bool,

    /// When the update was applied (ISO 8601)
    pub updated_at: String,
}

// ============================================================================
// Service Registry Types (v3.20.0)
// ============================================================================

/// Request to register a primal service with Songbird
///
/// ## Example
///
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "method": "register_service",
///   "params": {
///     "primal_name": "BearDog",
///     "capabilities": ["encryption", "identity", "trust"],
///     "endpoint": "/run/user/1000/beardog-nat0.sock",
///     "protocol": "json-rpc",
///     "health_check_interval": 30
///   },
///   "id": 4
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterServiceRequest {
    /// Primal name (e.g., "BearDog", "ToadStool")
    pub primal_name: String,

    /// Capabilities provided (e.g., ["encryption", "identity"])
    pub capabilities: Vec<String>,

    /// Endpoint (e.g., "/run/user/1000/beardog-nat0.sock")
    pub endpoint: String,

    /// Protocol (e.g., "json-rpc", "tarpc", "http")
    pub protocol: String,

    /// Health check interval in seconds (optional, default: 30)
    #[serde(default = "default_health_interval")]
    pub health_check_interval: u64,
}

fn default_health_interval() -> u64 {
    30
}

/// Response after registering a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterServiceResponse {
    /// Unique service ID (auto-generated by Songbird)
    pub service_id: String,

    /// Status ("registered", "updated", "failed")
    pub status: String,

    /// When the service was registered (ISO 8601)
    pub registered_at: String,
}

/// Request to discover primals by capability
///
/// ## Example
///
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "method": "discover_by_capability",
///   "params": {
///     "capability": "encryption",
///     "protocol": "json-rpc"
///   },
///   "id": 5
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverByCapabilityRequest {
    /// Capability to search for (e.g., "encryption", "storage", "*" for all)
    pub capability: String,

    /// Optional protocol filter (e.g., "json-rpc", "tarpc")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
}

/// Response containing discovered primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverByCapabilityResponse {
    /// Discovered primals matching the capability
    pub primals: Vec<PrimalEndpoint>,
}

/// Primal endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalEndpoint {
    /// Unique service ID (auto-generated by Songbird)
    pub service_id: String,

    /// Primal name
    pub primal_name: String,

    /// Capabilities
    pub capabilities: Vec<String>,

    /// Endpoint (Unix socket path or URL)
    pub endpoint: String,

    /// Protocol
    pub protocol: String,

    /// Last health check timestamp (ISO 8601)
    pub last_health_check: String,

    /// Health status ("healthy", "degraded", "down", "unknown")
    pub health_status: String,
}

/// Request to get health status of a specific service
///
/// ## Example
///
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "method": "get_service_health",
///   "params": {
///     "service_id": "beardog-12345"
///   },
///   "id": 6
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetServiceHealthRequest {
    /// Service ID to check
    pub service_id: String,
}

/// Response with health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetServiceHealthResponse {
    /// Health status
    pub health: HealthStatus,
}

/// Health status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Service ID (or "songbird" for Songbird itself)
    pub service_id: String,

    /// Status ("healthy", "degraded", "down", "unknown")
    pub status: String,

    /// Optional message (e.g., error reason)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Timestamp (ISO 8601)
    pub timestamp: String,
}

/// Request for Songbird's own health check
///
/// ## Example
///
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "method": "health_check",
///   "params": {},
///   "id": 7
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckRequest {}

/// Response with Songbird's health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    /// Health status
    pub health: HealthStatus,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Helper to convert SystemTime to ISO 8601 string
pub fn system_time_to_iso8601(time: SystemTime) -> String {
    let duration = time.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();

    let secs = duration.as_secs();
    let nanos = duration.subsec_nanos();

    // Simple ISO 8601 format: YYYY-MM-DDTHH:MM:SSZ
    chrono::DateTime::from_timestamp(secs as i64, nanos).map_or_else(|| "1970-01-01T00:00:00Z".to_string(), |dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // P2P Discovery Tests (v3.19.3)
    #[test]
    fn test_discover_request_deserialization() {
        let json = r#"{
            "family_tags": ["nat0", "lan0"],
            "timeout_ms": 3000
        }"#;

        let req: DiscoverByFamilyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.family_tags, vec!["nat0", "lan0"]);
        assert_eq!(req.timeout_ms, 3000);
    }

    #[test]
    fn test_discover_request_default_timeout() {
        let json = r#"{"family_tags": ["nat0"]}"#;

        let req: DiscoverByFamilyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.timeout_ms, 5000); // Default
    }

    #[test]
    fn test_genetic_proof_serialization() {
        let proof = GeneticProof {
            family_id: "nat0".to_string(),
            parent_seed_hash: "abc123".to_string(),
            relationship: "sibling".to_string(),
        };

        let json = serde_json::to_string(&proof).unwrap();
        assert!(json.contains("nat0"));
        assert!(json.contains("abc123"));
        assert!(json.contains("sibling"));
    }

    // Service Registry Tests (v3.20.0)
    #[test]
    fn test_register_service_request_deserialization() {
        let json = r#"{
            "primal_name": "BearDog",
            "capabilities": ["encryption", "identity"],
            "endpoint": "/run/user/1000/beardog-nat0.sock",
            "protocol": "json-rpc",
            "health_check_interval": 60
        }"#;

        let req: RegisterServiceRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.primal_name, "BearDog");
        assert_eq!(req.capabilities, vec!["encryption", "identity"]);
        assert_eq!(req.endpoint, "/run/user/1000/beardog-nat0.sock");
        assert_eq!(req.protocol, "json-rpc");
        assert_eq!(req.health_check_interval, 60);
    }

    #[test]
    fn test_register_service_request_default_health_interval() {
        let json = r#"{
            "primal_name": "ToadStool",
            "capabilities": ["compute"],
            "endpoint": "/tmp/toadstool.sock",
            "protocol": "json-rpc"
        }"#;

        let req: RegisterServiceRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.health_check_interval, 30); // Default
    }

    #[test]
    fn test_discover_by_capability_request_deserialization() {
        let json = r#"{
            "capability": "encryption",
            "protocol": "json-rpc"
        }"#;

        let req: DiscoverByCapabilityRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.capability, "encryption");
        assert_eq!(req.protocol, Some("json-rpc".to_string()));
    }

    #[test]
    fn test_discover_by_capability_wildcard() {
        let json = r#"{"capability": "*"}"#;

        let req: DiscoverByCapabilityRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.capability, "*");
        assert!(req.protocol.is_none());
    }

    #[test]
    fn test_primal_endpoint_serialization() {
        let endpoint = PrimalEndpoint {
            service_id: "beardog-12345".to_string(),
            primal_name: "BearDog".to_string(),
            capabilities: vec!["encryption".to_string()],
            endpoint: "/run/user/1000/beardog-nat0.sock".to_string(),
            protocol: "json-rpc".to_string(),
            last_health_check: "2026-01-10T12:00:00Z".to_string(),
            health_status: "healthy".to_string(),
        };

        let json = serde_json::to_string(&endpoint).unwrap();
        assert!(json.contains("beardog-12345"));
        assert!(json.contains("BearDog"));
        assert!(json.contains("encryption"));
        assert!(json.contains("healthy"));
    }

    #[test]
    fn test_health_status_serialization() {
        let health = HealthStatus {
            service_id: "songbird".to_string(),
            status: "healthy".to_string(),
            message: None,
            timestamp: "2026-01-10T12:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&health).unwrap();
        assert!(json.contains("songbird"));
        assert!(json.contains("healthy"));
        // Message should not be in JSON when None
        assert!(!json.contains("message"));
    }
}
