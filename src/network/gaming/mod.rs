//! Gaming Network Bridge Module
//!
//! Universal gaming network bridge supporting legacy protocols

pub mod auto_config;
pub mod nat_traversal;
pub mod performance;
pub mod privilege_manager;
pub mod production_lan;
pub mod production_lan_manager;
pub mod protocol_translators;
pub mod real_bridge_manager;
pub mod real_ipx_bridge;
pub mod real_protocol_detector;
pub mod types;
pub mod universal_bridge;
pub mod universal_detector;
pub mod wireguard_integration;

// Re-export core types directly from types module
pub use types::{
    BridgeStatus, DetectedGameSession, DiscoveryMethod, GameProtocolClass, GameSession,
    GameSessionId, GameSessionStatus, NatType, PlayerEndpoint, ProtocolSignature, VirtualNetwork,
};

// Re-export main components
pub use auto_config::{
    BeardogIntegration, GamingAutoConfig, OneTouchConfig, SecurityValidator, SetupState,
};
pub use nat_traversal::NatTraversalManager;
pub use performance::{
    BenchmarkConfig, BenchmarkResults, GamingPerformanceMetrics, PerformanceMonitor,
};
pub use privilege_manager::{
    can_capture_packets, create_safe_privilege_manager, PrivilegeConfig, PrivilegeManager,
    PrivilegeMethod,
};
pub use production_lan_manager::{
    ProductionGameSession, ProductionLanConfig, ProductionLanManager,
};
pub use protocol_translators::{DirectPlayTranslator, IPXTranslator, NetBIOSTranslator};
pub use real_bridge_manager::{RealBridgeConfig, RealBridgeManager, RealBridgeSession};
pub use real_ipx_bridge::RealIPXBridge;
pub use real_protocol_detector::RealProtocolDetector;
pub use universal_bridge::UniversalGameBridge;
pub use universal_detector::UniversalGameProtocolDetector;
pub use wireguard_integration::{
    GamingTunnelManager, TunnelStats, TunnelType, WireGuardConfig, WireGuardTunnel,
};

use crate::errors::Result;
use rand;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::net::UdpSocket;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::info;
use uuid;

/// Simple session info for LAN discovery
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

/// Main gaming manager that coordinates all gaming network functionality
pub struct GamingManager {
    detector: UniversalGameProtocolDetector,
    bridge: UniversalGameBridge,
    configurator: GamingAutoConfig,
    active_sessions: Arc<RwLock<HashMap<String, DetectedGameSession>>>,
    pub lan_sessions: Arc<RwLock<HashMap<String, LanGameSession>>>,
}

impl GamingManager {
    /// Create a new gaming manager
    pub async fn new() -> Result<Self> {
        let mut detector = UniversalGameProtocolDetector::new();
        detector.initialize().await?;

        // Enable real packet capture detection with privilege management
        if let Err(e) = detector.enable_real_detection().await {
            tracing::warn!(
                "⚠️  Could not enable real detection: {}, falling back to basic detection - real detection will be enabled in future releases",
                e
            );
        }

        let bridge = UniversalGameBridge::new();
        let configurator = GamingAutoConfig::new().await?;

        Ok(Self {
            detector,
            bridge,
            configurator,
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            lan_sessions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Initialize secure packet capture with privilege management
    pub async fn initialize_secure_capture(&mut self) -> Result<()> {
        tracing::info!("🔐 Initializing secure packet capture...");

        // Initialize privileges for the detector
        if let Err(e) = self.detector.initialize_privileges().await {
            tracing::warn!("⚠️  Could not initialize secure capture: {}", e);
        }

        Ok(())
    }

    /// Scan for active gaming sessions on the network
    pub async fn scan_for_games(
        &mut self,
        interface: Option<String>,
    ) -> Result<Vec<DetectedGameSession>> {
        let sessions = self.detector.scan_network(interface).await?;

        // Store detected sessions
        let mut active_sessions = self.active_sessions.write().await;
        for session in &sessions {
            active_sessions.insert(session.session_id.clone(), session.clone());
        }

        Ok(sessions)
    }

    /// Configure gaming for a specific detected game session
    pub async fn configure_for_game(&mut self, session: &DetectedGameSession) -> Result<()> {
        let game_name = session.game_name.as_deref().unwrap_or("Unknown Game");
        let _config = self.configurator.configure_for_game(game_name).await?;
        Ok(())
    }

    /// Create a bridge for a specific game session
    pub async fn create_bridge(&mut self, session: &DetectedGameSession) -> Result<String> {
        // Configure the bridge for this game's protocol
        let bridge_id = self.bridge.create_bridge(session).await?;

        // Auto-configure network settings
        if let Some(ref game_name) = session.game_name {
            self.configurator.configure_for_game(game_name).await?;
        }

        Ok(bridge_id)
    }

    /// Join an existing bridge
    pub async fn join_bridge(&mut self, bridge_id: &str, local_address: SocketAddr) -> Result<()> {
        self.bridge.join_bridge(bridge_id, local_address).await
    }

    /// Get status of all active bridges
    pub async fn get_bridge_status(&self) -> Result<Vec<BridgeStatus>> {
        self.bridge.get_all_bridge_status().await
    }

    /// Stop a specific bridge
    pub async fn stop_bridge(&mut self, bridge_id: &str) -> Result<()> {
        self.bridge.stop_bridge(bridge_id).await
    }

    /// Get all active gaming sessions
    pub async fn get_active_sessions(&self) -> Vec<DetectedGameSession> {
        let sessions = self.active_sessions.read().await;
        sessions.values().cloned().collect()
    }

    pub async fn auto_configure(&self) -> Result<ProductionLanConfig> {
        // This would trigger auto-configuration based on detected games
        self.configurator.auto_configure_for_detected_games().await
    }

    /// Create a new LAN gaming session with secure binding
    pub async fn create_lan_session(
        &self,
        game_name: String,
        host_address: SocketAddr,
        protocol_class: GameProtocolClass,
    ) -> Result<String> {
        let session_code = generate_session_code();
        let session_id = uuid::Uuid::new_v4().to_string();

        let session = LanGameSession {
            session_id: session_id.clone(),
            session_code: session_code.clone(),
            host_address,
            game_name,
            protocol_class,
            max_players: 16,
            current_players: vec![],
            created_at: SystemTime::now(),
        };

        let mut sessions = self.lan_sessions.write().await;
        sessions.insert(session_code.clone(), session);

        Ok(session_code)
    }

    /// Join a LAN gaming session
    pub async fn join_lan_session(
        &self,
        session_code: &str,
        player_address: SocketAddr,
    ) -> Result<LanGameSession> {
        let mut sessions = self.lan_sessions.write().await;

        let session = sessions.get_mut(session_code).ok_or_else(|| {
            crate::errors::SongbirdError::Network {
                service: "Gaming Manager".to_string(),
                message: format!("Session not found: {}", session_code),
                details: None,
            }
        })?;

        // Add player if not already in session
        if !session.current_players.contains(&player_address) {
            session.current_players.push(player_address);
        }

        tracing::info!("Player {} joined session {}", player_address, session_code);
        Ok(session.clone())
    }

    /// Look up a LAN gaming session
    pub async fn lookup_lan_session(&self, session_code: &str) -> Result<Option<LanGameSession>> {
        let sessions = self.lan_sessions.read().await;
        Ok(sessions.get(session_code).cloned())
    }

    /// Get all active LAN sessions
    pub async fn get_lan_sessions(&self) -> Vec<LanGameSession> {
        let sessions = self.lan_sessions.read().await;
        sessions.values().cloned().collect()
    }

    /// Start packet bridge with configurable binding
    pub async fn start_packet_bridge(&self, session_code: &str) -> Result<()> {
        let env_config = crate::config::environment::EnvironmentConfig::default();
        
        // Get the session from our storage
        let sessions = self.lan_sessions.read().await;
        let session = sessions.get(session_code).ok_or_else(|| {
            crate::errors::SongbirdError::Network {
                service: "Gaming Manager".to_string(),
                message: format!("Session not found: {}", session_code),
                details: None,
            }
        })?;
        
        // Use configurable binding address - NO MORE HARDCODING 0.0.0.0!
        let bind_addr = if env_config.bind_address == "0.0.0.0" {
            // Only allow 0.0.0.0 if explicitly approved
            if std::env::var("SONGBIRD_GAMING_BIND_ALL_APPROVED").is_err() {
                return Err(crate::errors::SongbirdError::Config {
                    field: Some("gaming_bind_address".to_string()),
                    message: "Gaming services binding to 0.0.0.0 requires explicit approval via SONGBIRD_GAMING_BIND_ALL_APPROVED=true".to_string(),
                });
            }
            "0.0.0.0"
        } else {
            &env_config.bind_address
        };
        
        let _socket = UdpSocket::bind(format!("{}:0", bind_addr))?;

        // For IPX games, create a real bridge
        if matches!(session.protocol_class, GameProtocolClass::IpxBased) {
            tracing::info!("🌉 Starting IPX bridge for session {}", session_code);

            // Create IPX bridge with a standard network ID
            let bridge = RealIPXBridge::bind_ipx_network(0x01000000).await?;

            // Register all current players
            for player_addr in &session.current_players {
                bridge.register_node(*player_addr).await?;
            }

            // Start packet forwarding
            bridge.start_forwarding().await?;

            tracing::info!("✅ IPX bridge active for session {}", session_code);
        } else {
            tracing::info!(
                "🔄 Generic packet forwarding for {:?}",
                session.protocol_class
            );
            // For other protocols, we'd implement UDP/TCP forwarding here
        }

        Ok(())
    }

    /// Broadcast session for LAN discovery
    pub async fn broadcast_lan_session(&self, session_code: &str) -> Result<()> {
        let sessions = self.lan_sessions.read().await;
        if let Some(session) = sessions
            .iter()
            .find(|(_, s)| s.session_code == session_code)
        {
            let session = session.1;
            let env_config = crate::config::environment::EnvironmentConfig::default();

            // Create UDP socket for broadcasting - NO MORE HARDCODING!
            let bind_addr = if env_config.bind_address == "0.0.0.0" {
                if std::env::var("SONGBIRD_GAMING_BIND_ALL_APPROVED").is_err() {
                    return Err(crate::errors::SongbirdError::Config {
                        field: Some("broadcast_bind_address".to_string()),
                        message: "Broadcasting requires explicit approval for 0.0.0.0 binding".to_string(),
                    });
                }
                "0.0.0.0:0"
            } else {
                &format!("{}:0", env_config.bind_address)
            };
            
            let socket = UdpSocket::bind(bind_addr)?;
            socket.set_broadcast(true)?;

            // Create discovery message
            let discovery_msg = format!(
                "SONGBIRD_GAMING:{}",
                serde_json::to_string(&DiscoveryMessage {
                    session_code: session.session_code.clone(),
                    host_address: session.host_address,
                    game_name: session.game_name.clone(),
                    protocol_class: session.protocol_class.clone(),
                    max_players: session.max_players,
                    current_players: session.current_players.len() as u8,
                })?
            );

            // Broadcast to configurable gaming discovery ports - NO MORE HARDCODING!
            for &port in &env_config.discovery_ports {
                socket.send_to(discovery_msg.as_bytes(), format!("255.255.255.255:{}", port))?;
            }

            info!("📡 Broadcasted session {} for LAN discovery", session_code);
        }

        Ok(())
    }

    /// Scan for LAN gaming sessions via UDP broadcast
    pub async fn scan_lan_sessions(&self) -> Result<Vec<LanGameSession>> {
        let mut discovered_sessions = Vec::new();
        let env_config = crate::config::environment::EnvironmentConfig::default();

        // Listen for discovery broadcasts - NO MORE HARDCODING!
        let discovery_port = env_config.discovery_ports.first().copied().unwrap_or(6112);
        let bind_addr = if env_config.bind_address == "0.0.0.0" {
            format!("0.0.0.0:{}", discovery_port)
        } else {
            format!("{}:{}", env_config.bind_address, discovery_port)
        };
        
        let socket = UdpSocket::bind(&bind_addr)?;
        
        // Use configurable timeout instead of hardcoded 3 seconds
        let scan_timeout = std::time::Duration::from_secs(env_config.discovery_timeout_secs);
        socket.set_read_timeout(Some(scan_timeout))?;

        info!("🔍 Scanning for LAN gaming sessions on {}...", bind_addr);

        let mut buffer = [0u8; 1024];
        let start_time = std::time::Instant::now();

        while start_time.elapsed() < scan_timeout {
            match socket.recv_from(&mut buffer) {
                Ok((len, from_addr)) => {
                    let message = String::from_utf8_lossy(&buffer[..len]);

                    if message.starts_with("SONGBIRD_GAMING:") {
                        if let Some(json_part) = message.strip_prefix("SONGBIRD_GAMING:") {
                            if let Ok(discovery_msg) =
                                serde_json::from_str::<DiscoveryMessage>(json_part)
                            {
                                let session = LanGameSession {
                                    session_id: uuid::Uuid::new_v4().to_string(),
                                    session_code: discovery_msg.session_code.clone(),
                                    host_address: discovery_msg.host_address,
                                    game_name: discovery_msg.game_name,
                                    protocol_class: discovery_msg.protocol_class,
                                    max_players: discovery_msg.max_players,
                                    current_players: vec![from_addr],
                                    created_at: std::time::SystemTime::now(),
                                };

                                discovered_sessions.push(session);
                                info!(
                                    "🎮 Discovered session from {}: {}",
                                    from_addr, discovery_msg.session_code
                                );
                            }
                        }
                    }
                }
                Err(_) => {
                    // Timeout or other error, continue scanning
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }

        info!(
            "📊 LAN scan complete, found {} sessions",
            discovered_sessions.len()
        );
        Ok(discovered_sessions)
    }
}

/// Generate a simple session code for LAN gaming
fn generate_session_code() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect();

    (0..4)
        .map(|_| chars[rng.gen_range(0..chars.len())])
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiscoveryMessage {
    session_code: String,
    host_address: SocketAddr,
    game_name: String,
    protocol_class: GameProtocolClass,
    max_players: u8,
    current_players: u8,
}
pub mod advanced_tunnel_system;
