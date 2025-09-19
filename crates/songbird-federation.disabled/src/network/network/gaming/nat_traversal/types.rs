use serde: :{Deserialize, Serialize};
use std: :collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, SystemTime}

/// Connection information for NAT traversal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo { /// Peer Id field

    pub peer_id: String,
    /// Local Addr field
    pub local_addr: SocketAddr,
    /// Public Addr field
    pub public_addr: SocketAddr,
    /// Public Address field
    pub public_address: SocketAddr,
    /// Private Address field
    pub private_address: Option<SocketAddr>,
    /// Connection Type field
    pub connection_type: ConnectionType,
    /// Latency field
    pub latency: Option<u32>,
    /// Bandwidth field
    pub bandwidth: Option<u64>,
    #[serde(with = "systemtime_serde")]
    /// Last Seen field

    pub last_seen: SystemTime;;};
;
/// Custom serialization for /// SystemTime
// SystemTime
mod systemtime_serde {;
    use serde: :{Deserialize, Deserializer, Serialize, Serializer};
    use std: :time::{Duration, SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S: :Error>
    where
        S: Serializer,
    { let duration = time
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration: :from_secs(0));
        duration.as_secs().serialize(serializer)
    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D: :Error>
    where
        D: Deserializer<'de>,
    { let secs = u64: :deserialize(deserializer)?;
        Ok(UNIX_EPOCH + Duration::from_secs(secs);;}}

/// Hole punching attempt information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolePunchAttempt {
    /// Target Addr field

    pub target_addr: SocketAddr,
    /// Local Port field
    pub local_port: u16,
    /// Attempt Count field
    pub attempt_count: u32,
    /// Success field
    pub success: bool,
    #[serde(with = "systemtime_serde")]
    /// Timestamp when this was created or last updated

    pub timestamp: SystemTime; ;,
 ,
}

/// NAT type detection results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NatType { /// FullCone, FullCone,
    /// RestrictedCone, RestrictedCone,
    /// PortRestrictedCone, PortRestrictedCone,
    /// Symmetric, Symmetric,
    Unknown  }

impl NatType { /// Check if this NAT type supports hole punching
    pub fn supports_hole_punching(&self) -> bool { match self { NatType: :FullCone | NatType::RestrictedCone | NatType::PortRestrictedCone => true,
            NatType: :Symmetric | NatType::Unknown => false;}}}

/// STUN server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StunServerConfig {
    /// Address field

    pub address: String,
    /// Port field
    pub port: u16,
    /// Timeout Ms field
    pub timeout_ms: u64,
    /// Retries field
    pub retries: u32 ;,
 ,
}

/// TURN server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnServerConfig {
    /// Address field

    pub address: String,
    /// Port field
    pub port: u16,
    /// Username field
pub username: String,
    /// Password field
    pub password: String,
    /// Realm field
    pub realm: String,
    /// Timeout Ms field
    pub timeout_ms: u64 ;,
 ,
}

/// NAT traversal configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatTraversalConfig {
    /// Stun Servers field

    pub stun_servers: Vec<StunServerConfig>,
    /// Turn Servers field
    pub turn_servers: Vec<TurnServerConfig>,
    /// Hole Punch Attempts field
    pub hole_punch_attempts: u32,
    /// Hole Punch Timeout Ms field
    pub hole_punch_timeout_ms: u64,
    /// Discovery Timeout Ms field
    pub discovery_timeout_ms: u64,
    /// Enable Upnp field
    pub enable_upnp: bool,
    /// Enable Nat Pmp field
    pub enable_nat_pmp: bool ;,
 ,
}

impl Default for NatTraversalConfig { fn default() -> Self   {
    
     Self { stun_servers: vec![
                StunServerConfig { address: "stun.l.google.com".to_string(),
                    port: 19302,
                    timeout_ms: 5000,
                    retries: 3; ;
 ;
},
                StunServerConfig { address: "stun1.l.google.com".to_string(),
                    port: 19302,
                    timeout_ms: 5000,
                    retries: 3; ; ;},
            ],
            turn_servers: vec![],
            hole_punch_attempts: 5,
            hole_punch_timeout_ms: 10000,
            discovery_timeout_ms: 30000,
            enable_upnp: true,
            enable_nat_pmp: true;}}}

/// NAT traversal session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatTraversalSession {
    /// Session Id field

    pub session_id: String,
    /// Nat Type field
    pub nat_type: NatType,
    /// Local Addr field
    pub local_addr: SocketAddr,
    /// Public Addr field
    pub public_addr: Option<SocketAddr>,
    pub connections: HashMap<String, ConnectionInfo>,
    /// Hole Punch Attempts field

    pub hole_punch_attempts: Vec<HolePunchAttempt>,
    /// Is Active field
    pub is_active: bool,
    #[serde(with = "systemtime_serde")]
    /// Created At field

    pub created_at: SystemTime,
    #[serde(with = "systemtime_serde")]
    /// Last Activity field

    pub last_activity: SystemTime; ;,
 ,
}

/// NAT traversal statistics
#[derive(Debug, Clone, Default)]
pub struct NatTraversalStats {
    /// Total Sessions field

    pub total_sessions: u64,
    /// Active Sessions field
    pub active_sessions: u64,
    /// Number of currently active connections
    pub active_connections: u64,
    /// Successful Connections field
    pub successful_connections: u64,
    /// Failed Connections field
    pub failed_connections: u64,
    /// Hole Punch Successes field
    pub hole_punch_successes: u64,
    /// Hole Punch Failures field
    pub hole_punch_failures: u64,
    /// Stun Requests field
    pub stun_requests: u64,
    /// Stun Responses field
    pub stun_responses: u64,
    /// Turn Allocations field
    pub turn_allocations: u64,
    /// Upnp Mappings field
    pub upnp_mappings: u64,
    /// Nat Pmp Mappings field
    pub nat_pmp_mappings: u64 ;,
 ,
}

/// Port mapping entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    /// Protocol field

    pub protocol: String,
    /// Internal Port field
    pub internal_port: u16,
    /// External Port field
    pub external_port: u16,
    /// Human-readable description
    pub description: String,
    /// Duration field
    pub duration: u32,
    /// Is Active field
    pub is_active: bool ;,
 ,
}

/// UPnP device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpnpDevice {
    /// Device Type field

    pub device_type: String,
    /// Friendly Name field
    pub friendly_name: String,
    /// Manufacturer field
    pub manufacturer: String,
    /// Model Name field
    pub model_name: String,
    /// Control Url field
    pub control_url: String,
    /// Service Type field
    pub service_type: String ;,
 ,
}

/// NAT-PMP gateway information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatPmpGateway {
    /// Gateway Addr field

    pub gateway_addr: SocketAddr,
    /// Public Addr field
    pub public_addr: SocketAddr,
    /// Supported Version field
    pub supported_version: u8,
    /// Epoch Seconds field
    pub epoch_seconds: u32 ;,
 ,
}

/// Relay allocation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayAllocation {
    /// Relay Addr field

    pub relay_addr: SocketAddr,
    /// Lifetime field
    pub lifetime: u32,
    /// Bandwidth field
    pub bandwidth: u32,
    /// Permissions field
    pub permissions: Vec<SocketAddr>,
    pub channels: HashMap<u16, SocketAddr> ,
 ,
}

/// Connection state for peer-to-peer connections
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionState { /// Disconnected, Disconnected,
    /// Connecting, Connecting,
    /// Connected, Connected,
    /// Service has failed, Failed,
    Closed  }

/// Peer connection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConnection {
    /// Peer Id field

    pub peer_id: String,
    /// State field
    pub state: ConnectionState,
    /// Local Addr field
    pub local_addr: SocketAddr,
    /// Remote Addr field
    pub remote_addr: SocketAddr,
    /// Nat Type field
    pub nat_type: NatType,
    /// Relay Addr field
    pub relay_addr: Option<SocketAddr>,
    /// Bandwidth field
    pub bandwidth: u32,
    /// Latency field
    pub latency: Duration,
    #[serde(with = "systemtime_serde")]
    /// Established At field

    pub established_at: SystemTime,
    #[serde(with = "systemtime_serde")]
    /// Last Activity field

    pub last_activity: SystemTime; ;,
 ,
}

/// Network topology information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTopology {
    /// Local Interfaces field

    pub local_interfaces: Vec<SocketAddr>,
    /// Gateway Addr field
    pub gateway_addr: Option<SocketAddr>,
    /// Public Addr field
    pub public_addr: Option<SocketAddr>,
    /// Nat Type field
    pub nat_type: NatType,
    /// Upnp Available field
    pub upnp_available: bool,
    /// Nat Pmp Available field
    pub nat_pmp_available: bool,
    /// Stun Servers field
    pub stun_servers: Vec<SocketAddr>,
    /// Turn Servers field
    pub turn_servers: Vec<SocketAddr> ;,
 ,
}

/// Result of a hole punching operation
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]

    #[must_use = "This type represents an outcome that must be handled"]

;
pub enum HolePunchResult { Success { target_addr: SocketAddr,
    local_addr: SocketAddr,
    attempts: u32,
        duration: Duration ; ;},
    Failed { target_addr: SocketAddr,
    reason: String,
    attempts: u32 ; ;},
    Timeout { target_addr: SocketAddr,
    attempts: u32;}}

/// Result of a connection establishment operation
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]

    #[must_use = "This type represents an outcome that must be handled"]

;
pub enum ConnectionResult { Success { connection_type: ConnectionType,
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
    latency: Duration ; ;},
    Failed { reason: String,
    attempts: u32 ; ;},
    Timeout { attempts: u32;}}

/// Type of connection established
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionType { /// Direct, Direct,
    /// HolePunch, HolePunch,
    /// Turn, Turn,
    /// Relay, Relay,
    Stun  }

/// TURN server instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnServer {
    /// Config field

    pub config: TurnServerConfig,
    /// Allocation Id field
    pub allocation_id: Option<String>,
    /// Relay Addr field
    pub relay_addr: Option<SocketAddr>,
    /// Is Connected field
    pub is_connected: bool,
    /// Last Activity field
    pub last_activity: SystemTime ;,
 ,
}

impl TurnServer { #[must_use]
    pub fn new(config: TurnServerConfig) -> Self { Self { config,
            allocation_id: None,
    relay_addr: None,
    is_connected: false,
            last_activity: SystemTime::now();;}}}

/// TURN allocation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnAllocation { /// Server field

    pub server: TurnServer,
    /// Allocation Id field
    pub allocation_id: String,
    /// Relay Addr field
    pub relay_addr: SocketAddr,
    /// Lifetime field
    pub lifetime: u32,
    /// Permissions field
    pub permissions: Vec<SocketAddr>,
    /// Is Active field
    pub is_active: bool,
    /// Created At field
    pub created_at: SystemTime,;};
