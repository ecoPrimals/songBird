use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, SystemTime};

/// Connection information for NAT traversal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub peer_id: String,
    pub local_addr: SocketAddr,
    pub public_addr: SocketAddr,
    pub public_address: SocketAddr,
    pub private_address: Option<SocketAddr>,
    pub connection_type: ConnectionType,
    pub latency: Option<u32>,
    pub bandwidth: Option<u64>,
    #[serde(with = "systemtime_serde")]
    pub last_seen: SystemTime,
}

/// Custom serialization for SystemTime
mod systemtime_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0));
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + Duration::from_secs(secs))
    }
}

/// Hole punching attempt information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolePunchAttempt {
    pub target_addr: SocketAddr,
    pub local_port: u16,
    pub attempt_count: u32,
    pub success: bool,
    #[serde(with = "systemtime_serde")]
    pub timestamp: SystemTime,
}

/// NAT type detection results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NatType {
    FullCone,
    RestrictedCone,
    PortRestrictedCone,
    Symmetric,
    Unknown,
}

impl NatType {
    /// Check if this NAT type supports hole punching
    pub fn supports_hole_punching(&self) -> bool {
        match self {
            NatType::FullCone | NatType::RestrictedCone | NatType::PortRestrictedCone => true,
            NatType::Symmetric | NatType::Unknown => false,
        }
    }
}

/// STUN server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StunServerConfig {
    pub address: String,
    pub port: u16,
    pub timeout_ms: u64,
    pub retries: u32,
}

/// TURN server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnServerConfig {
    pub address: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub realm: String,
    pub timeout_ms: u64,
}

/// NAT traversal configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatTraversalConfig {
    pub stun_servers: Vec<StunServerConfig>,
    pub turn_servers: Vec<TurnServerConfig>,
    pub hole_punch_attempts: u32,
    pub hole_punch_timeout_ms: u64,
    pub discovery_timeout_ms: u64,
    pub enable_upnp: bool,
    pub enable_nat_pmp: bool,
}

impl Default for NatTraversalConfig {
    fn default() -> Self {
        Self {
            stun_servers: vec![
                StunServerConfig {
                    address: "stun.l.google.com".to_string(),
                    port: 19302,
                    timeout_ms: 5000,
                    retries: 3,
                },
                StunServerConfig {
                    address: "stun1.l.google.com".to_string(),
                    port: 19302,
                    timeout_ms: 5000,
                    retries: 3,
                },
            ],
            turn_servers: vec![],
            hole_punch_attempts: 5,
            hole_punch_timeout_ms: 10000,
            discovery_timeout_ms: 30000,
            enable_upnp: true,
            enable_nat_pmp: true,
        }
    }
}

/// NAT traversal session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatTraversalSession {
    pub session_id: String,
    pub nat_type: NatType,
    pub local_addr: SocketAddr,
    pub public_addr: Option<SocketAddr>,
    pub connections: HashMap<String, ConnectionInfo>,
    pub hole_punch_attempts: Vec<HolePunchAttempt>,
    pub is_active: bool,
    #[serde(with = "systemtime_serde")]
    pub created_at: SystemTime,
    #[serde(with = "systemtime_serde")]
    pub last_activity: SystemTime,
}

/// NAT traversal statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatTraversalStats {
    pub total_sessions: u64,
    pub active_sessions: u64,
    pub active_connections: u64,
    pub successful_connections: u64,
    pub failed_connections: u64,
    pub hole_punch_successes: u64,
    pub hole_punch_failures: u64,
    pub stun_requests: u64,
    pub stun_responses: u64,
    pub turn_allocations: u64,
    pub upnp_mappings: u64,
    pub nat_pmp_mappings: u64,
}

impl Default for NatTraversalStats {
    fn default() -> Self {
        Self {
            total_sessions: 0,
            active_sessions: 0,
            active_connections: 0,
            successful_connections: 0,
            failed_connections: 0,
            hole_punch_successes: 0,
            hole_punch_failures: 0,
            stun_requests: 0,
            stun_responses: 0,
            turn_allocations: 0,
            upnp_mappings: 0,
            nat_pmp_mappings: 0,
        }
    }
}

/// Port mapping entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub protocol: String,
    pub internal_port: u16,
    pub external_port: u16,
    pub description: String,
    pub duration: u32,
    pub is_active: bool,
}

/// UPnP device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpnpDevice {
    pub device_type: String,
    pub friendly_name: String,
    pub manufacturer: String,
    pub model_name: String,
    pub control_url: String,
    pub service_type: String,
}

/// NAT-PMP gateway information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatPmpGateway {
    pub gateway_addr: SocketAddr,
    pub public_addr: SocketAddr,
    pub supported_version: u8,
    pub epoch_seconds: u32,
}

/// Relay allocation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayAllocation {
    pub relay_addr: SocketAddr,
    pub lifetime: u32,
    pub bandwidth: u32,
    pub permissions: Vec<SocketAddr>,
    pub channels: HashMap<u16, SocketAddr>,
}

/// Connection state for peer-to-peer connections
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Failed,
    Closed,
}

/// Peer connection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConnection {
    pub peer_id: String,
    pub state: ConnectionState,
    pub local_addr: SocketAddr,
    pub remote_addr: SocketAddr,
    pub nat_type: NatType,
    pub relay_addr: Option<SocketAddr>,
    pub bandwidth: u32,
    pub latency: Duration,
    #[serde(with = "systemtime_serde")]
    pub established_at: SystemTime,
    #[serde(with = "systemtime_serde")]
    pub last_activity: SystemTime,
}

/// Network topology information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTopology {
    pub local_interfaces: Vec<SocketAddr>,
    pub gateway_addr: Option<SocketAddr>,
    pub public_addr: Option<SocketAddr>,
    pub nat_type: NatType,
    pub upnp_available: bool,
    pub nat_pmp_available: bool,
    pub stun_servers: Vec<SocketAddr>,
    pub turn_servers: Vec<SocketAddr>,
}

/// Result of a hole punching operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HolePunchResult {
    Success {
        target_addr: SocketAddr,
        local_addr: SocketAddr,
        attempts: u32,
        duration: Duration,
    },
    Failed {
        target_addr: SocketAddr,
        reason: String,
        attempts: u32,
    },
    Timeout {
        target_addr: SocketAddr,
        attempts: u32,
    },
}

/// Result of a connection establishment operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionResult {
    Success {
        connection_type: ConnectionType,
        local_addr: SocketAddr,
        remote_addr: SocketAddr,
        latency: Duration,
    },
    Failed {
        reason: String,
        attempts: u32,
    },
    Timeout {
        attempts: u32,
    },
}

/// Type of connection established
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionType {
    Direct,
    HolePunch,
    Turn,
    Relay,
    Stun,
}

/// TURN server instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnServer {
    pub config: TurnServerConfig,
    pub allocation_id: Option<String>,
    pub relay_addr: Option<SocketAddr>,
    pub is_connected: bool,
    pub last_activity: SystemTime,
}

impl TurnServer {
    pub fn new(config: TurnServerConfig) -> Self {
        Self {
            config,
            allocation_id: None,
            relay_addr: None,
            is_connected: false,
            last_activity: SystemTime::now(),
        }
    }
}

/// TURN allocation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnAllocation {
    pub server: TurnServer,
    pub allocation_id: String,
    pub relay_addr: SocketAddr,
    pub lifetime: u32,
    pub permissions: Vec<SocketAddr>,
    pub is_active: bool,
    pub created_at: SystemTime,
}
