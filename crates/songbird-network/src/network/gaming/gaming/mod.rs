//! Gaming Network Bridge Module
//!
//! Universal gaming network bridge supporting legacy protocols
//!
//! This module consolidates all gaming network functionality and re-exports
//! from the songbird-network crate to avoid code duplication.

// Re-export all gaming functionality from the crates
pub use songbird_network::network::gaming::*;

// Legacy compatibility re-exports for backward compatibility
pub use songbird_network::network::gaming::{
    auto_config::{
        BeardogIntegration, GamingAutoConfig, OneTouchConfig, SecurityValidator, SetupState,
    },
    nat_traversal::NatTraversalManager,
    performance::{
        BenchmarkConfig, BenchmarkResults, GamingPerformanceMetrics, PerformanceMonitor,
    },
    privilege_manager::{
        can_capture_packets, create_safe_privilege_manager, PrivilegeConfig, PrivilegeManager,
        PrivilegeMethod,
    },
    production_lan_manager::{
        ProductionGameSession, ProductionLanConfig, ProductionLanManager,
    },
    protocol_translators::{DirectPlayTranslator, IPXTranslator, NetBIOSTranslator},
    real_bridge_manager::{RealBridgeConfig, RealBridgeManager, RealBridgeSession},
    real_ipx_bridge::RealIPXBridge,
    real_protocol_detector::RealProtocolDetector,
    types::{
        BridgeStatus, DetectedGameSession, DiscoveryMethod, GameProtocolClass, GameSession,
        GameSessionId, GameSessionStatus, NatType, PlayerEndpoint, ProtocolSignature, VirtualNetwork,
    },
    universal_bridge::UniversalGameBridge,
    universal_detector::UniversalGameProtocolDetector,
    wireguard_integration::{
        GamingTunnelManager, TunnelStats, TunnelType, WireGuardConfig, WireGuardTunnel,
    },
    GamingManager, GamingSession,
};

// Additional functionality specific to the main crate
use songbird_errors::SongbirdResult as Result;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::SystemTime;

/// Simple session info for LAN discovery (legacy compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanGameSession {
    pub session_id: String,
    pub session_code: String,
    pub host_address: SocketAddr,
    pub game_name: String,
    pub protocol_class: GameProtocolClass,
    pub max_players: u8,
    pub current_players: Vec<SocketAddr>,
    pub created_at: SystemTime,
}

/// Generate a random session code for gaming sessions
pub fn generate_session_code() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let code: String = (0..6)
        .map(|_| {
            let idx = rng.gen_range(0..36);
            if idx < 10 {
                (b'0' + idx) as char
            } else {
                (b'A' + idx - 10) as char
            }
        })
        .collect();
    code
}

/// Convenience function to create a new gaming manager
pub async fn create_gaming_manager() -> Result<GamingManager> {
    GamingManager::new().await.map_err(|e| songbird_errors::SongbirdError::from(e))
}

/// Convenience function to discover gaming sessions on the network
pub async fn discover_gaming_sessions(
    interface: Option<String>,
) -> Result<Vec<DetectedGameSession>> {
    let mut manager = GamingManager::new().await.map_err(|e| songbird_errors::SongbirdError::from(e))?;
    manager.scan_for_games(interface).await.map_err(|e| songbird_errors::SongbirdError::from(e))
}

/// Convenience function to create a LAN gaming session
pub async fn create_lan_gaming_session(
    game_name: String,
    host_address: SocketAddr,
    protocol_class: GameProtocolClass,
) -> Result<String> {
    let manager = GamingManager::new().await.map_err(|e| songbird_errors::SongbirdError::from(e))?;
    manager.create_lan_session(game_name, host_address, protocol_class).await.map_err(|e| songbird_errors::SongbirdError::from(e))
}

/// Gaming session management utilities
pub mod session_utils {
    use super::*;
    
    /// Validate a gaming session code
    pub fn validate_session_code(code: &str) -> bool {
        code.len() == 6 && code.chars().all(|c| c.is_ascii_alphanumeric())
    }
    
    /// Generate a unique session ID
    pub fn generate_session_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }
    
    /// Get current timestamp for session tracking
    pub fn current_timestamp() -> SystemTime {
        SystemTime::now()
    }
}

/// Advanced gaming configuration utilities
pub mod advanced_config {
    use super::*;
    
    /// Auto-configure gaming settings based on detected games
    pub async fn auto_configure_gaming() -> Result<ProductionLanConfig> {
        let manager = GamingManager::new().await.map_err(|e| songbird_errors::SongbirdError::from(e))?;
        manager.auto_configure().await.map_err(|e| songbird_errors::SongbirdError::from(e))
    }
    
    /// Configure gaming for a specific protocol class
    pub async fn configure_for_protocol(protocol: GameProtocolClass) -> Result<()> {
        let mut manager = GamingManager::new().await.map_err(|e| songbird_errors::SongbirdError::from(e))?;
        
        // Create a dummy session for configuration
        let dummy_session = DetectedGameSession {
            session_id: session_utils::generate_session_id(),
            protocol_class: protocol,
            local_ports: vec![6112],
            remote_endpoints: vec!["127.0.0.1:6112".parse().unwrap_or_else(|_| std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 6112))],
            process_id: None,
            game_name: Some("Auto-configured Game".to_string()),
            detected_at: SystemTime::now(),
            confidence: 0.8,
        };
        
        manager.configure_for_game(&dummy_session).await.map_err(|e| songbird_errors::SongbirdError::from(e))
    }
}

/// Re-export the main gaming functionality for external use
pub mod exports {
    pub use super::*;
    
    // Core gaming types
    pub use songbird_network::network::gaming::types::*;
    
    // Protocol translators
    pub use songbird_network::network::gaming::protocol_translators::*;
    
    // NAT traversal
    pub use songbird_network::network::gaming::nat_traversal::*;
    
    // Bridge management
    pub use songbird_network::network::gaming::real_bridge_manager::*;
    
    // Auto configuration
    pub use songbird_network::network::gaming::auto_config::*;
    
    // Performance monitoring
    pub use songbird_network::network::gaming::performance::*;
    
    // Universal bridge
    pub use songbird_network::network::gaming::universal_bridge::UniversalGameBridge;
    
    // Protocol detection
    pub use songbird_network::network::gaming::universal_detector::*;
    
    // WireGuard integration
    pub use songbird_network::network::gaming::wireguard_integration::*;
}
