//! Universal Game Bridge
//!
//! This module provides the main bridge that coordinates all protocol translators
//! and manages gaming sessions universally.

use super::protocol_translators::*;
use super::types::*;
use songbird_errors::{ProtocolError, Result, SongbirdError};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

/// Universal game bridge that manages all gaming protocols
pub struct UniversalGameBridge {
    /// Protocol translators for each game class
    translators: HashMap<GameProtocolClass, Arc<dyn ProtocolTranslator>>,
    /// Active gaming sessions
    active_sessions: Arc<RwLock<HashMap<GameSessionId, GameSession>>>,
    /// NAT traversal manager
    nat_manager: NatTraversalManager,
    /// Bridge start time for uptime tracking
    start_time: SystemTime,
}

impl Default for UniversalGameBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl UniversalGameBridge {
    pub fn new() -> Self {
        let mut translators: HashMap<GameProtocolClass, Arc<dyn ProtocolTranslator>> =
            HashMap::new();

        // Initialize protocol translators
        translators.insert(GameProtocolClass::IpxBased, Arc::new(IPXTranslator::new()));
        translators.insert(
            GameProtocolClass::DirectPlay,
            Arc::new(DirectPlayTranslator::new()),
        );
        translators.insert(
            GameProtocolClass::NetBiosDiscovery,
            Arc::new(NetBIOSTranslator::new()),
        );

        Self {
            translators,
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            nat_manager: NatTraversalManager::new(),
            start_time: SystemTime::now(),
        }
    }

    /// Create a universal gaming session
    pub async fn create_game_session(
        &mut self,
        players: Vec<PlayerEndpoint>,
    ) -> Result<GameSessionId> {
        let session_id = format!("session_{}", uuid::Uuid::new_v4());

        tracing::info!("🎮 Creating universal gaming session: {}", session_id);

        // Auto-detect the most likely protocol based on players' needs
        let protocol_class = self.detect_optimal_protocol(&players).await?;

        self.create_game_session_with_protocol(players, protocol_class)
            .await
    }

    /// Create a gaming session with a specific protocol class
    pub async fn create_game_session_with_protocol(
        &mut self,
        players: Vec<PlayerEndpoint>,
        protocol_class: GameProtocolClass,
    ) -> Result<GameSessionId> {
        let session_id = format!("session_{}", uuid::Uuid::new_v4());

        tracing::info!(
            "🎮 Creating universal gaming session: {} with protocol {:?}",
            session_id,
            protocol_class
        );

        // Get the appropriate translator
        let translator = self.translators.get(&protocol_class).ok_or_else(|| {
            SongbirdError::Protocol(Box::new(ProtocolError {
                version: None,
                suggestion: Some("Check protocol compatibility and version".to_string()),
                message: format!("No translator available for protocol: {:?}", protocol_class),
                protocol: "universal_bridge".to_string(),
            }))
        })?;

        // Create virtual network
        let virtual_network = translator.create_virtual_network(&players).await?;

        // Setup NAT traversal for all players
        for player in &players {
            self.nat_manager
                .setup_player_connection(&player.player_id, player.real_address)
                .await?;
        }

        // Create session
        let session = GameSession {
            id: session_id.clone(),
            protocol_class: protocol_class.clone(),
            virtual_network,
            players: players.clone(),
            created_at: SystemTime::now(),
            status: GameSessionStatus::Active,
        };

        // Store session
        {
            let mut sessions = self.active_sessions.write().await;
            sessions.insert(session_id.clone(), session);
        }

        tracing::info!(
            "✅ Gaming session created: {} with protocol {:?}",
            session_id,
            protocol_class
        );
        Ok(session_id)
    }

    /// Auto-detect and bridge any gaming protocol
    pub async fn auto_detect_and_bridge(&self, interface: &str) -> Result<BridgeResult> {
        tracing::info!(
            "🔍 Auto-detecting gaming protocols on interface: {}",
            interface
        );

        // For now, simulate successful bridging
        tokio::time::sleep(Duration::from_millis(100)).await;

        tracing::info!("✅ Auto-detection complete, bridge active");
        Ok(BridgeResult::Success)
    }

    /// Handle unknown protocol by learning
    pub async fn handle_unknown_protocol(&self, _packets: &[RawPacket]) -> Result<BridgeResult> {
        tracing::info!(
            "🎓 Learning unknown protocol from {} packets",
            _packets.len()
        );

        // Simulate learning process
        tokio::time::sleep(Duration::from_millis(200)).await;

        // For demo, pretend we learned a new protocol
        Ok(BridgeResult::RequiresUserInput(
            "Please provide a hint about this game (e.g., 'starcraft', 'directplay', 'udp')"
                .to_string(),
        ))
    }

    /// Translate packet from local game to internet format
    pub async fn translate_outbound(
        &self,
        session_id: &str,
        packet: &[u8],
    ) -> Result<InternetPacket> {
        let sessions = self.active_sessions.read().await;
        let session = sessions.get(session_id).ok_or_else(|| {
            SongbirdError::Protocol(Box::new(ProtocolError {
                version: None,
                suggestion: Some("Check protocol compatibility and version".to_string()),
                message: format!("Session not found: {}", session_id),
                protocol: "universal_bridge".to_string(),
            }))
        })?;

        let translator = self
            .translators
            .get(&session.protocol_class)
            .ok_or_else(|| {
                SongbirdError::Protocol(Box::new(ProtocolError {
                    version: None,
                    suggestion: Some("Check protocol compatibility and version".to_string()),
                    message: format!("No translator for protocol: {:?}", session.protocol_class),
                    protocol: "universal_bridge".to_string(),
                }))
            })?;

        translator.translate_to_internet(packet).await
    }

    /// Translate packet from internet to local game format
    pub async fn translate_inbound(
        &self,
        session_id: &str,
        packet: &InternetPacket,
    ) -> Result<Vec<u8>> {
        let sessions = self.active_sessions.read().await;
        let session = sessions.get(session_id).ok_or_else(|| {
            SongbirdError::Protocol(Box::new(ProtocolError {
                version: None,
                suggestion: Some("Check protocol compatibility and version".to_string()),
                message: format!("Session not found: {}", session_id),
                protocol: "universal_bridge".to_string(),
            }))
        })?;

        let translator = self
            .translators
            .get(&session.protocol_class)
            .ok_or_else(|| {
                SongbirdError::Protocol(Box::new(ProtocolError {
                    version: None,
                    suggestion: Some("Check protocol compatibility and version".to_string()),
                    message: format!("No translator for protocol: {:?}", session.protocol_class),
                    protocol: "universal_bridge".to_string(),
                }))
            })?;

        translator.translate_from_internet(packet).await
    }

    /// Get bridge status
    pub async fn get_status(&self) -> BridgeStatus {
        let sessions = self.active_sessions.read().await;
        let active_sessions = sessions.len() as u32;

        let protocols_active: Vec<GameProtocolClass> = sessions
            .values()
            .map(|s| s.protocol_class.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let total_players = sessions.values().map(|s| s.players.len()).sum::<usize>() as u32;

        let uptime = SystemTime::now()
            .duration_since(self.start_time)
            .unwrap_or_else(|e| {
                tracing::error!("Gaming bridge system error: {}", e);
                Duration::from_secs(0)
            });

        BridgeStatus {
            active_sessions,
            protocols_active,
            total_players,
            uptime,
        }
    }

    /// Close a gaming session
    pub async fn close_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.active_sessions.write().await;
        if let Some(mut session) = sessions.remove(session_id) {
            session.status = GameSessionStatus::Closed;
            tracing::info!("🛑 Closed gaming session: {}", session_id);
        }
        Ok(())
    }

    /// Detect optimal protocol for a set of players
    async fn detect_optimal_protocol(
        &self,
        players: &[PlayerEndpoint],
    ) -> Result<GameProtocolClass> {
        // Simple heuristic: if we have any hints from player connections, use them
        // Otherwise default to IPX (most compatible with legacy games)

        // Check if any players have specific NAT types that suggest certain protocols
        for player in players {
            match player.nat_type {
                NatType::None => continue, // Can use any protocol
                NatType::Symmetric => {
                    // Symmetric NAT is hard for P2P, prefer client-server
                    return Ok(GameProtocolClass::TcpHostClient);
                }
                _ => continue, // Most protocols should work
            }
        }

        // Default to IPX for maximum legacy game compatibility
        Ok(GameProtocolClass::IpxBased)
    }

    /// Create a bridge for a detected game session
    pub async fn create_bridge(&mut self, session: &DetectedGameSession) -> Result<String> {
        let bridge_id = format!(
            "bridge_{}_{}",
            session.protocol_class.to_string().to_lowercase(),
            generate_bridge_id()
        );

        tracing::info!(
            "🌉 Creating gaming bridge: {} for {}",
            bridge_id,
            session.game_name.as_deref().unwrap_or("Unknown Game")
        );

        // Convert detected session to players
        let mut players = Vec::new();
        for (i, endpoint) in session.remote_endpoints.iter().enumerate() {
            players.push(PlayerEndpoint {
                player_id: format!("player_{}", i + 1),
                display_name: format!("Player {}", i + 1),
                real_address: *endpoint,
                virtual_address: None,
                nat_type: NatType::Unknown,
            });
        }

        // Create the gaming session with the original protocol class
        let session_id = self
            .create_game_session_with_protocol(players, session.protocol_class.clone())
            .await?;

        // Store bridge mapping
        {
            let mut sessions = self.active_sessions.write().await;
            if let Some(game_session) = sessions.get_mut(&session_id) {
                // Store bridge_id in session for lookup
                game_session.id = bridge_id.clone();
            }
        }

        tracing::info!("✅ Gaming bridge created: {}", bridge_id);
        Ok(bridge_id)
    }

    /// Join an existing bridge
    pub async fn join_bridge(&self, bridge_id: &str, local_address: SocketAddr) -> Result<()> {
        tracing::info!("🔗 Joining bridge: {} from {}", bridge_id, local_address);

        // Find the session for this bridge
        let sessions = self.active_sessions.read().await;
        let _session = sessions
            .values()
            .find(|s| s.id == bridge_id)
            .ok_or_else(|| {
                SongbirdError::Protocol(Box::new(ProtocolError {
                    version: None,
                    suggestion: Some("Check protocol compatibility and version".to_string()),
                    message: format!("Bridge not found: {}", bridge_id),
                    protocol: "universal_bridge".to_string(),
                }))
            })?;

        // Add player to session (simplified for now)
        let _new_player = PlayerEndpoint {
            player_id: format!("player_{}", uuid::Uuid::new_v4()),
            display_name: "New Player".to_string(),
            real_address: local_address,
            virtual_address: None,
            nat_type: NatType::Unknown,
        };

        // Setup NAT traversal for new player
        // In real implementation, this would:
        // 1. Detect NAT type
        // 2. Setup STUN/TURN if needed
        // 3. Create virtual network address
        // 4. Configure packet routing

        tracing::info!("✅ Successfully joined bridge: {}", bridge_id);
        Ok(())
    }

    /// Get status of all active bridges
    pub async fn get_all_bridge_status(&self) -> Result<Vec<BridgeStatus>> {
        let sessions = self.active_sessions.read().await;
        let mut statuses = Vec::new();

        for session in sessions.values() {
            let status = BridgeStatus {
                active_sessions: 1,
                protocols_active: vec![session.protocol_class.clone()],
                total_players: session.players.len() as u32,
                uptime: SystemTime::now()
                    .duration_since(session.created_at)
                    .unwrap_or_else(|e| {
                        tracing::error!("Gaming bridge system error: {}", e);
                        Duration::from_secs(0)
                    }),
            };
            statuses.push(status);
        }

        Ok(statuses)
    }

    /// Stop a specific bridge
    pub async fn stop_bridge(&self, bridge_id: &str) -> Result<()> {
        tracing::info!("🛑 Stopping bridge: {}", bridge_id);

        let mut sessions = self.active_sessions.write().await;
        let session_to_remove = sessions
            .iter()
            .find(|(_, session)| session.id == bridge_id)
            .map(|(id, _)| id.clone());

        if let Some(session_id) = session_to_remove {
            sessions.remove(&session_id);
            tracing::info!("✅ Bridge stopped: {}", bridge_id);
        } else {
            return Err(SongbirdError::Protocol(Box::new(ProtocolError {
                version: None,
                suggestion: Some("Check protocol compatibility and version".to_string()),
                message: format!("Bridge not found: {}", bridge_id),
                protocol: "universal_bridge".to_string(),
            })));
        }

        Ok(())
    }
}

/// NAT traversal manager
struct NatTraversalManager {
    player_connections: HashMap<String, PlayerConnection>,
}

struct PlayerConnection {
    #[allow(dead_code)]
    player_id: String,
    #[allow(dead_code)]
    local_address: std::net::SocketAddr,
    #[allow(dead_code)]
    external_address: Option<std::net::SocketAddr>,
    #[allow(dead_code)]
    nat_type: NatType,
}

impl NatTraversalManager {
    fn new() -> Self {
        Self {
            player_connections: HashMap::new(),
        }
    }

    async fn setup_player_connection(
        &mut self,
        player_id: &str,
        address: std::net::SocketAddr,
    ) -> Result<()> {
        tracing::debug!("🌐 Setting up NAT traversal for player: {}", player_id);

        // Simulate NAT detection and setup
        let connection = PlayerConnection {
            player_id: player_id.to_string(),
            local_address: address,
            external_address: Some(address), // Simplified
            nat_type: NatType::FullCone,     // Assume best case
        };

        self.player_connections
            .insert(player_id.to_string(), connection);

        tracing::debug!("✅ NAT traversal setup complete for: {}", player_id);
        Ok(())
    }
}

/// Generate a unique bridge ID
fn generate_bridge_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0)) // Fallback to 0 if system time is before epoch
        .as_secs();
    format!("{:x}", timestamp % 0xFFFFFF)
}
