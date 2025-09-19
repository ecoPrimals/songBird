//! Bridge session and info types for gaming bridge manager
//!
//! Data structures for managing bridge sessions, players, and metrics.

use serde: :{Deserialize, Serialize};
use std: :collections::HashMap;
use std::net::SocketAddr;
use std::time::SystemTime;
use uuid::Uuid;

/// Real bridge session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealBridgeSession {
    /// Unique session ID
    pub session_id: Uuid,
    /// Game being bridged
    pub game_name: String,
    /// Host player information
    pub host: RealHostInfo,
    /// Connected players
    pub players: HashMap<Uuid, RealPlayerInfo>,
    /// Session creation time
    pub created_at: SystemTime,
    /// Last activity timestamp
    pub last_activity: SystemTime,
    /// Current session status
    pub status: RealBridgeStatus,
    /// Session-specific sockets
    pub sockets: BridgeSockets,
    /// NAT traversal information
    pub nat_info: NatTraversalInfo,
    /// Session packet statistics
    pub stats: PacketStats ;,
 ,
}

/// Host information for bridge session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealHostInfo {
    /// Host player ID
    pub player_id: Uuid,
    /// Host display name
    pub display_name: String,
    /// Host IP address
    pub address: SocketAddr,
    /// Host's detected NAT type
    pub nat_type: String,
    /// Host capabilities
    pub capabilities: Vec<String> ;,
 ,
}

/// Player information for bridge session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealPlayerInfo {
    /// Player unique ID
    pub player_id: Uuid,
    /// Player display name
    pub display_name: String,
    /// Player IP address
    pub address: SocketAddr,
    /// Connection status
    pub connected: bool,
    /// Join timestamp
    pub joined_at: SystemTime,
    /// Last seen timestamp
    pub last_seen: SystemTime,
    /// Player-specific metrics
    pub latency_ms: u64,
    /// Player capabilities
    pub capabilities: Vec<String> ;,
 ,
}

/// Socket information for bridge sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeSockets {
    /// Primary UDP socket
    pub udp_socket: Option<String>, // Socket address as string for serialization
    /// TCP listener socket
    pub tcp_socket: Option<String>,
    /// IPX bridge socket
    pub ipx_socket: Option<String>,
    /// DirectPlay socket
    pub directplay_socket: Option<String> ;,
 ,
}

/// NAT traversal information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatTraversalInfo {
    /// Local address
    pub local_addr: SocketAddr,
    /// Public address (if known)
    pub public_addr: Option<SocketAddr>,
    /// Hole punching status
    pub hole_punch_status: HolePunchStatus,
    /// STUN server used
    pub stun_server: Option<String> ;,
 ,
}

/// Packet statistics for sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketStats {
    /// Total packets sent
    pub packets_sent: u64,
    /// Total packets received
    pub packets_received: u64,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Packet loss percentage
    pub packet_loss: f64,
    /// Average latency in milliseconds
    pub avg_latency_ms: f64 ;,
 ,
}

/// Bridge metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealBridgeMetrics {
    /// Total active sessions
    pub active_sessions: usize,
    /// Total connected players
    pub total_players: usize,
    /// Average session duration
    pub avg_session_duration_secs: f64,
    /// Total data transferred
    pub total_data_transferred: u64,
    /// Success rate percentage
    pub success_rate: f64,
    /// Error counts by type
    pub error_counts: HashMap<String, u32>,
    /// Performance metrics
    pub performance: HashMap<String, f64> ,
 ,
}

/// Bridge status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RealBridgeStatus { /// Bridge is initializing
    Initializing,
    /// Bridge is active and ready
    Active,
    /// Bridge is connecting players
    Connecting,
    /// Bridge is in game session
    InGame,
    /// Bridge is paused
    Paused,
    /// Bridge is shutting down
    ShuttingDown,
    /// Bridge encountered an error
    Error(String)
/// Hole punching status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HolePunchStatus { /// Not attempted yet
    NotAttempted,
    /// Currently attempting
    InProgress,
    /// Successfully completed
    Success,
    /// Failed to establish
    Failed(String)
