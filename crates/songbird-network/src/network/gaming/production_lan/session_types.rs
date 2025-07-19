use crate::network::gaming::nat_traversal::types::NatType;
/// Production LAN Gaming Session Types
///
/// This module defines all data structures related to gaming sessions,
/// players, metrics, and session management.
use crate::network::gaming::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::time::Instant;

/// Production gaming session with full feature set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionGameSession {
    pub id: String,
    pub session_code: String,
    pub host_info: HostInfo,
    pub game_info: GameInfo,
    pub network_info: NetworkInfo,
    pub security_info: SecurityInfo,
    pub players: Vec<PlayerInfo>,
    pub status: SessionStatus,
    pub metrics: SessionMetrics,
    pub created_at: std::time::SystemTime,
    pub last_seen: std::time::SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostInfo {
    pub host_address: SocketAddr,
    pub host_name: String,
    pub host_version: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInfo {
    pub game_name: String,
    pub game_version: Option<String>,
    pub protocol_class: GameProtocolClass,
    pub detected_protocols: Vec<DetectedProtocol>,
    pub game_specific_data: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedProtocol {
    pub protocol_type: String,
    pub ports: Vec<u16>,
    pub confidence: f32,
    pub packet_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub primary_interface: String,
    pub available_ports: Vec<u16>,
    pub nat_type: NatType,
    pub bandwidth_estimate: Option<u64>,
    pub latency_estimate: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityInfo {
    pub encryption_enabled: bool,
    pub session_key: Option<String>,
    pub access_control: AccessControl,
    pub rate_limits: RateLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControl {
    pub is_public: bool,
    pub allowed_players: Vec<String>,
    pub banned_players: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimits {
    pub max_packets_per_second: u32,
    pub max_bandwidth_bytes_per_second: u64,
    pub max_connections_per_ip: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub player_id: String,
    pub display_name: String,
    pub address: SocketAddr,
    pub joined_at: std::time::SystemTime,
    pub last_activity: std::time::SystemTime,
    pub connection_quality: ConnectionQuality,
    pub permissions: PlayerPermissions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionQuality {
    pub ping_ms: Option<u32>,
    pub packet_loss_percent: Option<f32>,
    pub bandwidth_usage: Option<u64>,
    pub connection_stability: ConnectionStability,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionStability {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPermissions {
    pub can_invite_others: bool,
    pub can_kick_players: bool,
    pub is_moderator: bool,
    pub custom_permissions: HashMap<String, bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetrics {
    pub total_packets_sent: u64,
    pub total_packets_received: u64,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub average_latency_ms: Option<f32>,
    pub peak_bandwidth_usage: Option<u64>,
    pub uptime_seconds: u64,
    pub error_count: u32,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Initializing,
    Active,
    Paused,
    Recovering,
    Error(String),
    Shutdown,
}

/// Discovery message structure for network communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryMessage {
    pub session_code: String,
    pub session_id: String,
    pub host_info: HostInfo,
    pub game_info: GameInfo,
    pub network_info: NetworkInfo,
    pub security_info: DiscoverySecurityInfo,
    pub player_count: u8,
    pub max_players: u8,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverySecurityInfo {
    pub encryption_enabled: bool,
    pub requires_invitation: bool,
    pub is_public: bool,
}

/// Health monitoring structures
#[derive(Debug)]
pub struct HealthMonitor {
    pub last_health_check: Instant,
    pub failed_checks: u32,
    pub recovery_attempts: HashMap<String, u32>,
}

/// Network monitoring structures
#[derive(Debug)]
pub struct InterfaceStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub errors: u32,
    pub last_updated: Instant,
}

#[derive(Debug)]
pub struct BandwidthMonitor {
    pub current_upload_bps: u64,
    pub current_download_bps: u64,
    pub peak_upload_bps: u64,
    pub peak_download_bps: u64,
    pub last_measurement: Instant,
}
