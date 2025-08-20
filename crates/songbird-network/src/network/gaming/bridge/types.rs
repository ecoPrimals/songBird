use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, SystemTime};

// use super::super::nat_traversal::types::{NatTraversalConfig, NatType, StunServerConfig};
use songbird_config::SongbirdConfig;

/// Configuration extracted from SongbirdConfig for gaming bridge
#[derive(Debug, Clone)]
pub struct GamingBridgeConfig {
    /// NAT traversal settings
    pub nat_traversal: NatTraversalConfig,
    /// Protocol detection settings
    pub protocol_detection_timeout: Duration,
    /// Maximum bridges per game
    pub max_bridges_per_game: usize,
    /// Bridge cleanup timeout
    pub bridge_cleanup_timeout: Duration,
    /// Enable bridge statistics
    pub enable_statistics: bool,
}

impl From<&SongbirdConfig> for GamingBridgeConfig {
    fn from(config: &SongbirdConfig) -> Self {
        Self {
            nat_traversal: NatTraversalConfig {
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
                hole_punch_attempts: 3,
                hole_punch_timeout_ms: 30000,
                discovery_timeout_ms: 10000,
                enable_upnp: config.network.gaming.enable_gaming_protocols,
                enable_nat_pmp: false,
            },
            protocol_detection_timeout: Duration::from_secs(10),
            max_bridges_per_game: 8,
            bridge_cleanup_timeout: Duration::from_secs(300),
            enable_statistics: true,
        }
    }
}

/// Real bridge session tracking active gaming sessions
#[derive(Debug, Clone)]
pub struct RealBridgeSession {
    pub session_id: String,
    pub game_id: String,
    pub host_info: RealHostInfo,
    pub players: HashMap<String, RealPlayerInfo>,
    pub bridge_sockets: BridgeSockets,
    pub nat_traversal_info: NatTraversalInfo,
    pub status: RealBridgeStatus,
    pub packet_stats: PacketStats,
    pub created_at: SystemTime,
    pub last_activity: SystemTime,
}

/// Host information for gaming sessions
#[derive(Debug, Clone)]
pub struct RealHostInfo {
    pub player_id: String,
    pub display_name: String,
    pub internal_addr: SocketAddr,
    pub external_addr: Option<SocketAddr>,
    pub nat_type: NatType,
    pub supports_upnp: bool,
}

/// Player information in gaming sessions
#[derive(Debug, Clone)]
pub struct RealPlayerInfo {
    pub player_id: String,
    pub display_name: String,
    pub internal_addr: SocketAddr,
    pub external_addr: Option<SocketAddr>,
    pub nat_type: NatType,
    pub connection_quality: f32,
    pub ping_ms: u32,
}

/// Bridge socket information
#[derive(Debug, Clone)]
pub struct BridgeSockets {
    pub host_bridge_port: u16,
    pub client_bridge_ports: HashMap<String, u16>,
}

/// NAT traversal information
#[derive(Debug, Clone)]
pub struct NatTraversalInfo {
    pub hole_punch_status: HashMap<String, HolePunchStatus>,
    pub upnp_mappings: Vec<u16>,
}

/// Packet statistics for monitoring
#[derive(Debug, Clone)]
pub struct PacketStats {
    pub packets_forwarded: u64,
    pub bytes_transferred: u64,
    pub packet_loss_rate: f32,
    pub average_latency_ms: f32,
}

/// Bridge metrics for monitoring and optimization
#[derive(Debug, Clone)]
pub struct RealBridgeMetrics {
    pub active_sessions: usize,
    pub total_packets_forwarded: u64,
    pub total_bytes_transferred: u64,
    pub average_session_duration_secs: f32,
    pub success_rate: f32,
}

/// Status of bridge operations
#[derive(Debug, Clone, PartialEq)]
pub enum RealBridgeStatus {
    Initializing,
    Active,
    Degraded(String),
    Failed(String),
    Shutdown,
}

/// Status of hole punching operations
#[derive(Debug, Clone, PartialEq)]
pub enum HolePunchStatus {
    NotStarted,
    InProgress,
    Success,
    Failed(String),
    TimedOut,
}

/// Forwarding task for packet processing
#[derive(Debug, Clone)]
pub struct ForwardingTask {
    pub session_id: String,
    pub packet_data: Vec<u8>,
    pub source_addr: SocketAddr,
    pub target_players: Vec<String>,
    pub protocol_class: super::super::types::GameProtocolClass,
}

/// Default implementations
impl Default for PacketStats {
    fn default() -> Self {
        Self {
            packets_forwarded: 0,
            bytes_transferred: 0,
            packet_loss_rate: 0.0,
            average_latency_ms: 0.0,
        }
    }
}

impl Default for RealBridgeMetrics {
    fn default() -> Self {
        Self {
            active_sessions: 0,
            total_packets_forwarded: 0,
            total_bytes_transferred: 0,
            average_session_duration_secs: 0.0,
            success_rate: 1.0,
        }
    }
}
