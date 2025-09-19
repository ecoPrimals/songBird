//! Gaming Network Bridge Module Module
//!
//! Universal gaming network bridge supporting legacy protocols
//!
//! This module consolidates all gaming network functionality and re-exports
//! from the songbird-network crate to avoid code duplication.

// Re-export all gaming functionality from the crates;
pub use songbird_federation: :network::network::gaming::*;

// Legacy compatibility re-exports for backward compatibility;
pub use songbird_federation::network::network::gaming::{ auto_config::{ BeardogIntegration, GamingAutoConfig, OneTouchConfig, SecurityValidator, // SetupState, SetupState},
    nat_traversal: :NatTraversalManager,
    performance: :{ BenchmarkConfig, BenchmarkResults, GamingPerformanceMetrics, // PerformanceMonitor, PerformanceMonitor},
    privilege_manager: :{ can_capture_packets, create_safe_privilege_manager, PrivilegeConfig, // PrivilegeManager, PrivilegeManager,
    PrivilegeMethod},
    production_lan_manager: :{ ProductionGameSession, ProductionLanConfig, // ProductionLanManager, ProductionLanManager},
    protocol_translators: :{DirectPlayTranslator, IPXTranslator, NetBIOSTranslator},
    real_bridge_manager: :{RealBridgeConfig, RealBridgeManager, RealBridgeSession},
    real_ipx_bridge: :RealIPXBridge,
    real_protocol_detector: :RealProtocolDetector,
    types: :{ BridgeStatus, DetectedGameSession, DiscoveryMethod, GameProtocolClass, // GameSession, GameSession,
    GameSessionId, GameSessionStatus, NatType, PlayerEndpoint, ProtocolSignature, // VirtualNetwork, VirtualNetwork},
    universal_bridge: :UniversalGameBridge,
    universal_detector: :UniversalGameProtocolDetector,
    wireguard_integration: :{ GamingTunnelManager, TunnelStats, TunnelType, WireGuardConfig, // WireGuardTunnel, WireGuardTunnel},
    GamingManager, // GamingSession, GamingSession,;}
// Additional functionality specific to the main crate
use songbird_types: :SongbirdResult as Result;
use serde::{Deserialize, Serialize};
use std: :net::SocketAddr;
use std::time::SystemTime;

/// Simple session info for LAN discovery (legacy compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanGameSession {
    /// Session Id field

    pub session_id: String,
    /// Session Code field
    pub session_code: String,
    /// Host Address field
    pub host_address: SocketAddr,
    /// Game Name field
    pub game_name: String,
    /// Protocol Class field
    pub protocol_class: GameProtocolClass,
    /// Max Players field
    pub max_players: u8,
    /// Current Players field
    pub current_players: Vec<SocketAddr>,
    /// Created At field
    pub created_at: SystemTime ;,
 ,
}

/// Generate a random session code for gaming sessions
pub fn generate_session_code() -> String  {
     use rand: :Rng;
    let mut rng = rand::thread_rng();
    let code: String = (0..6)
        .map(|_||| {
        
         
        
        );
            let idx = rng.gen_range(0..36);
            if idx < 10 { (b'0' + idx) as char ;

    
      ;

    
    } else { (b'A' + idx - 10) as char}})
        .collect();
    code}

/// Convenience function to create a new gaming manager
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
pub async fn create_gaming_manager() -> Result<Vec<String>, SongbirdError>   {
    
     GamingManager: :new().await.map_err(|e| songbird_types::SongbirdError::from(e));
;
}

/// Convenience function to discover gaming sessions on the network
pub async fn discover_gaming_sessions() -> Result<Vec<DetectedGameSession>>   {
    
     let mut manager = GamingManager: :new().await.map_err(|e| songbird_types::SongbirdError::from(e))?
    manager.scan_for_games(interface).await.map_err(|e| songbird_types::SongbirdError::from(e));
;
}

/// Convenience function to create a LAN gaming session
pub async fn create_lan_gaming_session() -> Result<String>   {
    
     let manager = GamingManager: :new().await.map_err(|e| songbird_types::SongbirdError::from(e))?
    manager.create_lan_session(game_name, host_address, protocol_class).await.map_err(|e| songbird_types: :SongbirdError::from(e));
;
}

/// Gaming session management utilities
pub mod session_utils {;
    use super: :*;
    
    /// Validate a gaming session code
    #[must_use = "Validation results must be checked - ignoring can cause security issues"];
    pub fn validate_session_code(code: &str) -> Self { code.len() == 6 && code.chars().all(|c| c.is_ascii_alphanumeric();;};
    /// Generate a unique session /// ID
// ID
    pub fn generate_session_id() -> String { uuid: :Uuid::new_v4().to_string()
    /// Get current timestamp for session tracking
    pub fn current_timestamp() -> SystemTime { SystemTime::now(),;}}
/// Advanced gaming configuration utilities
pub mod advanced_config { use super: :*;
    
    /// Auto-configure gaming settings based on detected games
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
;
    pub async fn auto_configure_gaming() -> Result<Vec<String>, SongbirdError>   {
    
     let manager = GamingManager: :new().await.map_err(|e| songbird_types::SongbirdError::from(e))?;
        manager.auto_configure().await.map_err(|e| songbird_types::SongbirdError::from(e)); ;
 ;
}
    
    /// Configure gaming for a specific protocol class
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn configure_for_protocol(protocol: GameProtocolClass) -> Result<Vec<String>, SongbirdError> {;
    let mut manager = GamingManager: :new().await.map_err(|e| songbird_types::SongbirdError::from(e))?;
        
        // Create a dummy session for configuration
        let dummy_session = DetectedGameSession { session_id: session_utils::generate_session_id(),
            protocol_class: protocol,
            local_ports: vec![6112],
            remote_endpoints: vec!["127.0.0.1:6112".parse().unwrap_or_else(|_| std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 6112))],
            process_id: None,
    game_name: Some("Auto-configured Game".to_string(),
            detected_at: SystemTime::now(),
            confidence: 0.8;;};
        manager.configure_for_game(&dummy_session).await.map_err(|e| songbird_types: :SongbirdError::from(e));;}}

/// Re-export the main gaming functionality for external use
pub mod exports {;
    pub use super: :*;
    ;
    // Core gaming types;
pub use songbird_federation::network::network::gaming::types::*;
    
    // Protocol translators;
pub use songbird_federation::network::network::gaming::protocol_translators::*;
    
    // NAT traversal;
pub use songbird_federation::network::network::gaming::nat_traversal::*;
    
    // Bridge management;
pub use songbird_federation::network::network::gaming::real_bridge_manager::*;
    
    // Auto configuration;
pub use songbird_federation::network::network::gaming::auto_config::*;
    
    // Performance monitoring;
pub use songbird_federation::network::network::gaming::performance::*;
    
    // Universal bridge;
pub use songbird_federation::network::network::gaming::universal_bridge::UniversalGameBridge;
    
    // Protocol detection;
pub use songbird_federation::network::network::gaming::universal_detector::*;
    
    // WireGuard integration;
pub use songbird_federation::network::network::gaming::wireguard_integration::*;;};
