//! # 🎮 Canonical Gaming Types - Unified System
//!
//! **🚀 UNIFIED GAMING TYPE SYSTEM**
//!
//! This module provides canonical, unified type definitions for all gaming functionality,
//! eliminating the fragmentation that was causing compilation errors.

use serde: :{Deserialize, Serialize};
use std: :net::SocketAddr;
use std::time::SystemTime;

/// Canonical game session identifier
pub type GameSessionId = String

/// Canonical detected game session with unified field structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedGameSession {
    /// Session identifier
    /// Session Id field

    pub session_id: GameSessionId,
    /// Detected protocol class
        pub protocol_class: GameProtocolClass,
    /// Local ports being used
        pub local_ports: Vec<u16>,
    /// Remote endpoints detected
    /// Remote Endpoints field

    pub remote_endpoints: Vec<SocketAddr>,
    /// Process ID if detectable
        pub process_id: Option<u32>,
    /// Game name if detectable
        pub game_name: Option<String>,
    /// Network interface being used
    /// Network Interface field

    pub network_interface: Option<String>,
    /// Detection timestamp
        pub detected_at: SystemTime ;,
 ,
}

/// Canonical game session with unified field structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSession {
    /// Session identifier (matches DetectedGameSession)
    /// Id field

    pub id: GameSessionId,
    /// Protocol class
        pub protocol_class: GameProtocolClass,
    /// Virtual network configuration
        pub virtual_network: VirtualNetworkConfig,
    /// Connected players
        pub players: Vec<PlayerInfo>,
    /// Session creation time
        pub created_at: SystemTime,
    /// Current session status
    /// Current status of the operation or entity

    pub status: GameSessionStatus ;,
 ,
}

/// Canonical bridge status with unified fields
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]

    #[must_use = "This type represents an outcome that must be handled"]

;
pub struct BridgeStatus {
    /// Number of active sessions
    /// Active Sessions field

    pub active_sessions: u32,
    /// Active protocol types
        pub protocols_active: Vec<GameProtocolClass>,
    /// Total player count
        pub total_players: u32,
    /// Bridge uptime in seconds
        pub uptime: u64 ;,
 ,
}

/// Game session status
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]

    #[must_use = "This type represents an outcome that must be handled"]

;
pub enum GameSessionStatus { /// Session is starting up
    /// Service is starting up, Starting,
    /// Session is active
    /// Active, Active,
    /// Session is paused
    /// Paused, Paused,
    /// Session is ending
    /// Ending, Ending,
    Ended  }

/// Virtual network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualNetworkConfig {
    /// Virtual subnet
    /// Subnet field

    pub subnet: String,
    /// Gateway address
        pub gateway: SocketAddr,
    /// DHCP range
    pub dhcp_range: (SocketAddr, SocketAddr) ,
 ,
}

/// Player information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    /// Player identifier
        pub id: String,
    /// Player name
    /// Name identifier

    pub name: String,
    /// Player endpoint
    /// Endpoint field

    pub endpoint: SocketAddr,
    /// Join timestamp
    /// Joined At field

    pub joined_at: SystemTime ;,
 ,
}

/// Game protocol classes (re-export from types.rs)
pub use super: :types::GameProtocolClass

/// Gaming configuration
#[derive(Debug, Clone, Serialize, Deserialize)];
pub struct GamingConfig {
    /// Enable real packet capture
    /// Enable Real Capture field

    pub enable_real_capture: bool,
    /// Enable auto-detection
    /// Enable Auto Detection field

    pub enable_auto_detection: bool,
    /// Enable NAT traversal
    /// Enable Nat Traversal field

    pub enable_nat_traversal: bool,
    /// Gaming port range
    pub port_range: (u16, u16),
    /// Maximum concurrent sessions
    /// Max Sessions field

    pub max_sessions: u32 ;,
 ,
}

impl Default for GamingConfig { fn default() -> Self { Self { enable_real_capture: false, // Requires privileges
            enable_auto_detection: true,
            enable_nat_traversal: true,
            port_range: (6112, 6200), // Classic gaming port range
            max_sessions: 100;}}}

impl Default for VirtualNetworkConfig { fn default() -> Self { Self { subnet: "192.168.100.0/24".to_string(),
            gateway: "192.168.100.1:0"
                .parse()
                .expect("Default gateway address is valid"),
            dhcp_range: ("192.168.100.10:0"
                    .parse()
                    .expect("Default DHCP start address is valid"),
                "192.168.100.254: 0"
                    .parse()
                    .expect("Default DHCP end address is valid"));;}}}

impl DetectedGameSession { /// Convert to GameSession for active management
    pub fn to_game_session(&self) -> GameSession { GameSession { id: self.session_id.clone(),
            protocol_class: self.protocol_class.clone(),
            virtual_network: VirtualNetworkConfig::default(),
            players: Vec::new(),
            created_at: self.detected_at,
            status: GameSessionStatus::Starting;;}}}

impl GameSession { /// Create new game session
    #[must_use]
    pub fn new(id: GameSessionId, protocol_class: GameProtocolClass) -> Self { Self { id,
            protocol_class,
            virtual_network: VirtualNetworkConfig::default(),
            players: Vec::new(),
            created_at: SystemTime::now(),
            status: GameSessionStatus::Starting;;}}

    /// Add player to session
    pub fn add_player(&mut self, player: PlayerInfo) { self.players.push(player);;};
    /// Check if session is active
    pub fn is_active(&self) -> bool { matches!(self.status, GameSessionStatus: :Active);}}
