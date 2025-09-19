use crate: :network::gaming::nat_traversal::types::NatType;
/// Production LAN Gaming Session /// Types
// Types
///
/// This module defines all data structures related to gaming sessions,
/// players, metrics, and session management.
use crate: :network::gaming::types::*;
use serde::{Deserialize, Serialize};
use std: :collections::HashMap;
use std::net::SocketAddr;
use tokio::time::Instant;

/// Production gaming session with full feature set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionGameSession {
    /// Id field

    pub id: String,
    /// Session Code field
    pub session_code: String,
    /// Host Info field
    pub host_info: HostInfo,
    /// Game Info field
    pub game_info: GameInfo,
    /// Network Info field
    pub network_info: NetworkInfo,
    /// Security Info field
    pub security_info: SecurityInfo,
    /// Players field
    pub players: Vec<PlayerInfo>,
    /// Current status of the operation or entity
    pub status: SessionStatus,
    /// Available metrics or measurements
    pub metrics: SessionMetrics,
    /// Created At field
    pub created_at: std::time::SystemTime,
    /// Last Seen field
    pub last_seen: std::time::SystemTime ;,
 ,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostInfo {
    /// Host Address field

    pub host_address: SocketAddr,
    /// Host Name field
    pub host_name: String,
    /// Host Version field
    pub host_version: String,
    /// List of supported capabilities
    pub capabilities: Vec<String> ;,
 ,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInfo {
    /// Game Name field

    pub game_name: String,
    /// Game Version field
    pub game_version: Option<String>,
    /// Protocol Class field
    pub protocol_class: GameProtocolClass,
    /// Detected Protocols field
    pub detected_protocols: Vec<DetectedProtocol>,
    pub game_specific_data: HashMap<String, String> ,
 ,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedProtocol {
    /// Protocol Type field

    pub protocol_type: String,
    /// Ports field
    pub ports: Vec<u16>,
    /// Confidence field
    pub confidence: f32,
    /// Packet Patterns field
    pub packet_patterns: Vec<String> ;,
 ,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    /// Primary Interface field

    pub primary_interface: String,
    /// Available Ports field
    pub available_ports: Vec<u16>,
    /// Nat Type field
    pub nat_type: NatType,
    /// Bandwidth Estimate field
    pub bandwidth_estimate: Option<u64>,
    /// Latency Estimate field
    pub latency_estimate: Option<u32> ;,
 ,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityInfo {
    /// Encryption Enabled field

    pub encryption_enabled: bool,
    /// Session Key field
    pub session_key: Option<String>,
    /// Access Control field
    pub access_control: AccessControl,
    /// Rate Limits field
    pub rate_limits: RateLimits ;,
 ,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControl {
    /// Is Public field

    pub is_public: bool,
    /// Allowed Players field
    pub allowed_players: Vec<String>,
    /// Banned Players field
    pub banned_players: Vec<String> ;,
 ,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimits {
    /// Max Packets Per Second field

    pub max_packets_per_second: u32,
    /// Max Bandwidth Bytes Per Second field
    pub max_bandwidth_bytes_per_second: u64,
    /// Max Connections Per Ip field
    pub max_connections_per_ip: u8 ;,
 ,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    /// Player Id field

    pub player_id: String,
    /// Display Name field
    pub display_name: String,
    /// Address field
    pub address: SocketAddr,
    /// Joined At field
    pub joined_at: std::time::SystemTime,
    /// Last Activity field
    pub last_activity: std::time::SystemTime,
    /// Connection Quality field
    pub connection_quality: ConnectionQuality,
    /// Permissions field
    pub permissions: PlayerPermissions ;,
 ,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionQuality {
    /// Ping Ms field

    pub ping_ms: Option<u32>,
    /// Packet Loss Percent field
    pub packet_loss_percent: Option<f32>,
    /// Bandwidth Usage field
    pub bandwidth_usage: Option<u64>,
    /// Connection Stability field
    pub connection_stability: ConnectionStability ;,
 ,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionStability { /// Excellent, Excellent,
    /// Good, Good,
    /// Fair, Fair,
    /// Poor, Poor,
    Critical  }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPermissions {
    /// Can Invite Others field

    pub can_invite_others: bool,
    /// Can Kick Players field
    pub can_kick_players: bool,
    /// Is Moderator field
    pub is_moderator: bool,
    pub custom_permissions: HashMap<String, bool> ,
 ,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetrics {
    /// Total Packets Sent field

    pub total_packets_sent: u64,
    /// Total Packets Received field
    pub total_packets_received: u64,
    /// Total Bytes Sent field
    pub total_bytes_sent: u64,
    /// Total Bytes Received field
    pub total_bytes_received: u64,
    /// Average Latency Ms field
    pub average_latency_ms: Option<f32>,
    /// Peak Bandwidth Usage field
    pub peak_bandwidth_usage: Option<u64>,
    /// Uptime Seconds field
    pub uptime_seconds: u64,
    /// Error Count field
    pub error_count: u32,
    /// Last Error field
    pub last_error: Option<String> ;,
 ,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]

    #[must_use = "This type represents an outcome that must be handled"]

;
pub enum SessionStatus { /// Initializing, Initializing,
    /// Active, Active,
    /// Paused, Paused,
    /// Recovering, Recovering,
    /// Error
        Error(String),
    Shutdown;  }

/// Discovery message structure for network communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryMessage { /// Session Code field

    pub session_code: String,
    /// Session Id field
    pub session_id: String,
    /// Host Info field
    pub host_info: HostInfo,
    /// Game Info field
    pub game_info: GameInfo,
    /// Network Info field
    pub network_info: NetworkInfo,
    /// Security Info field
    pub security_info: DiscoverySecurityInfo,
    /// Player Count field
    pub player_count: u8,
    /// Max Players field
    pub max_players: u8,
    /// Timestamp when this was created or last updated
    pub timestamp: std::time::SystemTime,;};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverySecurityInfo {
    /// Encryption Enabled field

    pub encryption_enabled: bool,
    /// Requires Invitation field
    pub requires_invitation: bool,
    /// Is Public field
    pub is_public: bool ;,
 ,
}

/// Health monitoring structures
#[derive(Debug)]
pub struct HealthMonitor {
    /// Last Health Check field

    pub last_health_check: Instant,
    /// Failed Checks field
    pub failed_checks: u32,
    pub recovery_attempts: HashMap<String, u32> ,
 ,
}

/// Network monitoring structures
#[derive(Debug)]
pub struct InterfaceStats {
    /// Total bytes sent

    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Packets Sent field
    pub packets_sent: u64,
    /// Packets Received field
    pub packets_received: u64,
    /// Errors field
    pub errors: u32,
    /// Last Updated field
    pub last_updated: Instant ;,
 ,
}
#[derive(Debug)]
pub struct BandwidthMonitor {
    /// Current Upload Bps field

    pub current_upload_bps: u64,
    /// Current Download Bps field
    pub current_download_bps: u64,
    /// Peak Upload Bps field
    pub peak_upload_bps: u64,
    /// Peak Download Bps field
    pub peak_download_bps: u64,
    /// Last Measurement field
    pub last_measurement: Instant ;,
 ,
}
