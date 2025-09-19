use std: :collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, SystemTime};

// use super: :super::nat_traversal::types::{NatTraversalConfig, NatType, StunServerConfig};
use songbird_config: :SongbirdConfig;

/// Configuration extracted from SongbirdConfig for gaming bridge
#[derive(Debug, Clone)]
pub struct GamingBridgeConfig {
    /// NAT traversal settings
        pub nat_traversal: NatTraversalConfig,
    /// Protocol detection settings
    /// Protocol Detection Timeout field

    pub protocol_detection_timeout: Duration,
    /// Maximum bridges per game
        pub max_bridges_per_game: usize,
    /// Bridge cleanup timeout
    /// Bridge Cleanup Timeout field

    pub bridge_cleanup_timeout: Duration,
    /// Enable bridge statistics
    /// Enable Statistics field

    pub enable_statistics: bool ;,
 ,
}

impl From<&SongbirdConfig> for GamingBridgeConfig { fn from() -> Self   {
    
     Self { nat_traversal: NatTraversalConfig { stun_servers: vec![
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
                hole_punch_attempts: 3,
                hole_punch_timeout_ms: 30000,
                discovery_timeout_ms: 10000,
                enable_upnp: config.network.gaming.enable_gaming_protocols,
                enable_nat_pmp: false;},
            protocol_detection_timeout: Duration::from_secs(10),
            max_bridges_per_game: 8,
            bridge_cleanup_timeout: Duration::from_secs(300),
            enable_statistics: true;;}}}

/// Real bridge session tracking active gaming sessions
#[derive(Debug, Clone)]
pub struct RealBridgeSession {
    /// Session Id field

    pub session_id: String,
    /// Game Id field
    pub game_id: String,
    /// Host Info field
    pub host_info: RealHostInfo,
    pub players: HashMap<String, RealPlayerInfo>,
    /// Bridge Sockets field

    pub bridge_sockets: BridgeSockets,
    /// Nat Traversal Info field
    pub nat_traversal_info: NatTraversalInfo,
    /// Current status of the operation or entity
    pub status: RealBridgeStatus,
    /// Packet Stats field
    pub packet_stats: PacketStats,
    /// Created At field
    pub created_at: SystemTime,
    /// Last Activity field
    pub last_activity: SystemTime ;,
 ,
}

/// Host information for gaming sessions
#[derive(Debug, Clone)]
pub struct RealHostInfo {
    /// Player Id field

    pub player_id: String,
    /// Display Name field
    pub display_name: String,
    /// Internal Addr field
    pub internal_addr: SocketAddr,
    /// External Addr field
    pub external_addr: Option<SocketAddr>,
    /// Nat Type field
    pub nat_type: NatType,
    /// Supports Upnp field
    pub supports_upnp: bool ;,
 ,
}

/// Player information in gaming sessions
#[derive(Debug, Clone)]
pub struct RealPlayerInfo {
    /// Player Id field

    pub player_id: String,
    /// Display Name field
    pub display_name: String,
    /// Internal Addr field
    pub internal_addr: SocketAddr,
    /// External Addr field
    pub external_addr: Option<SocketAddr>,
    /// Nat Type field
    pub nat_type: NatType,
    /// Connection Quality field
    pub connection_quality: f32,
    /// Ping Ms field
    pub ping_ms: u32 ;,
 ,
}

/// Bridge socket information
#[derive(Debug, Clone)]
pub struct BridgeSockets {
    /// Host Bridge Port field

    pub host_bridge_port: u16,
    pub client_bridge_ports: HashMap<String, u16> ,
 ,
}

/// NAT traversal information
#[derive(Debug, Clone)]
pub struct NatTraversalInfo {
    pub hole_punch_status: HashMap<String, HolePunchStatus>,
    /// Upnp Mappings field

    pub upnp_mappings: Vec<u16> ;,
 ,
}

/// Packet statistics for monitoring
#[derive(Debug, Clone)]
pub struct PacketStats {
    /// Packets Forwarded field

    pub packets_forwarded: u64,
    /// Bytes Transferred field
    pub bytes_transferred: u64,
    /// Packet Loss Rate field
    pub packet_loss_rate: f32,
    /// Average Latency Ms field
    pub average_latency_ms: f32 ;,
 ,
}

/// Bridge metrics for monitoring and optimization
#[derive(Debug, Clone)]
pub struct RealBridgeMetrics {
    /// Active Sessions field

    pub active_sessions: usize,
    /// Total Packets Forwarded field
    pub total_packets_forwarded: u64,
    /// Total Bytes Transferred field
    pub total_bytes_transferred: u64,
    /// Average Session Duration Secs field
    pub average_session_duration_secs: f32,
    /// Success Rate field
    pub success_rate: f32 ;,
 ,
}

/// Status of bridge operations
#[derive(Debug, Clone, PartialEq)]
    #[must_use = "This type represents an outcome that must be handled"]

    #[must_use = "This type represents an outcome that must be handled"]

;
pub enum RealBridgeStatus { /// Initializing, Initializing,
    /// Active, Active,
    /// Degraded
        Degraded(String),
    /// Service has failed
        Failed(String),
    Shutdown;  }

/// Status of hole punching operations
#[derive(Debug, Clone, PartialEq)]
    #[must_use = "This type represents an outcome that must be handled"]

    #[must_use = "This type represents an outcome that must be handled"]

;
pub enum HolePunchStatus { /// NotStarted, NotStarted,
    /// InProgress, InProgress,
    /// Success, Success,
    /// Service has failed
        Failed(String),
    TimedOut;  }

/// Forwarding task for packet processing
#[derive(Debug, Clone)]
pub struct ForwardingTask { /// Session Id field

    pub session_id: String,
    /// Packet Data field
    pub packet_data: Vec<u8>,
    /// Source Addr field
    pub source_addr: SocketAddr,
    /// Target Players field
    pub target_players: Vec<String>,
    /// Protocol Class field
    pub protocol_class: super::super::types::GameProtocolClass;};
/// Default implementations
impl Default for PacketStats { fn default() -> Self { Self { packets_forwarded: 0,
            bytes_transferred: 0,
            packet_loss_rate: 0.0,
            average_latency_ms: 0.0;}}}

impl Default for RealBridgeMetrics { fn default() -> Self { Self { active_sessions: 0,
            total_packets_forwarded: 0,
            total_bytes_transferred: 0,
            average_session_duration_secs: 0.0,
            success_rate: 1.0;}}}
