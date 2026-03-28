// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! # 🎮 Gaming Network Protocol Support
//!
//! **MODERN GAMING PROTOCOLS** ✅
//!
//! This module provides gaming-specific networking functionality with support
//! for legacy gaming protocols and modern gaming network patterns.
//!
//! ## Native Async Traits
//! This module uses native async trait methods (Rust 1.75+) for zero-cost abstractions.
//! No boxing overhead, better optimization, maximum gaming performance!

#![expect(async_fn_in_trait, reason = "async fn in trait (edition / trait-object compatibility)")]

use super::{GamingConfig, GamingHealth, NetworkStatus};
use serde::{Deserialize, Serialize};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Protocol handler enum - zero-cost dispatch instead of `Box<dyn Trait>`
#[derive(Debug)]
pub enum ProtocolHandlerImpl {
    Udp(UdpProtocolHandler),
    Tcp(TcpProtocolHandler),
    Ipx(IpxProtocolHandler),
    DirectPlay(DirectPlayProtocolHandler),
    NetBios(NetBiosProtocolHandler),
}

impl ProtocolHandler for ProtocolHandlerImpl {
    fn protocol_type(&self) -> GameProtocolType {
        match self {
            Self::Udp(h) => h.protocol_type(),
            Self::Tcp(h) => h.protocol_type(),
            Self::Ipx(h) => h.protocol_type(),
            Self::DirectPlay(h) => h.protocol_type(),
            Self::NetBios(h) => h.protocol_type(),
        }
    }

    async fn handle_packet(&mut self, data: &[u8], source: SocketAddr) -> SongbirdResult<Vec<u8>> {
        match self {
            Self::Udp(h) => h.handle_packet(data, source).await,
            Self::Tcp(h) => h.handle_packet(data, source).await,
            Self::Ipx(h) => h.handle_packet(data, source).await,
            Self::DirectPlay(h) => h.handle_packet(data, source).await,
            Self::NetBios(h) => h.handle_packet(data, source).await,
        }
    }

    async fn initialize(&mut self) -> SongbirdResult<()> {
        match self {
            Self::Udp(h) => h.initialize().await,
            Self::Tcp(h) => h.initialize().await,
            Self::Ipx(h) => h.initialize().await,
            Self::DirectPlay(h) => h.initialize().await,
            Self::NetBios(h) => h.initialize().await,
        }
    }

    async fn shutdown(&mut self) -> SongbirdResult<()> {
        match self {
            Self::Udp(h) => h.shutdown().await,
            Self::Tcp(h) => h.shutdown().await,
            Self::Ipx(h) => h.shutdown().await,
            Self::DirectPlay(h) => h.shutdown().await,
            Self::NetBios(h) => h.shutdown().await,
        }
    }
}

/// Gaming manager - handles gaming-specific networking (now with zero-cost dispatch)
pub struct GamingManager {
    config: GamingConfig,
    active_sessions: HashMap<Uuid, GameSession>,
    protocol_handlers: HashMap<GameProtocolType, ProtocolHandlerImpl>,
}

impl GamingManager {
    /// Create a new gaming manager
    #[must_use]
    pub fn new(config: GamingConfig) -> Self {
        Self {
            config,
            active_sessions: HashMap::new(),
            protocol_handlers: HashMap::new(),
        }
    }

    /// Initialize the gaming manager
    pub async fn initialize(&mut self) -> SongbirdResult<()> {
        // Register protocol handlers for supported protocols
        for protocol in &self.config.protocols {
            let handler = create_protocol_handler(protocol.clone())?;
            self.protocol_handlers.insert(protocol.clone(), handler);
        }

        Ok(())
    }

    /// Create a new gaming session
    pub async fn create_session(
        &mut self,
        protocol: GameProtocolType,
        config: SessionConfig,
    ) -> SongbirdResult<Uuid> {
        let session_id = Uuid::new_v4();
        let session = GameSession::new(session_id, protocol, config);

        self.active_sessions.insert(session_id, session);

        Ok(session_id)
    }

    /// Get session by ID
    #[must_use]
    pub fn get_session(&self, session_id: &Uuid) -> Option<&GameSession> {
        self.active_sessions.get(session_id)
    }

    /// Remove session
    pub async fn remove_session(&mut self, session_id: &Uuid) -> SongbirdResult<()> {
        self.active_sessions.remove(session_id);
        Ok(())
    }

    /// Get health status
    pub async fn health_check(&self) -> SongbirdResult<GamingHealth> {
        // Status is Healthy regardless of session count - simplify the logic
        Ok(GamingHealth {
            status: NetworkStatus::Healthy,
            active_sessions: self.active_sessions.len() as u32,
            supported_protocols: self.config.protocols.clone(),
        })
    }
}

/// Game protocol types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum GameProtocolType {
    /// UDP protocol
    UDP,
    /// TCP protocol
    TCP,
    /// IPX protocol (legacy)
    IPX,
    /// `DirectPlay` protocol (legacy)
    DirectPlay,
    /// `NetBIOS` protocol (legacy)
    NetBIOS,
    /// Custom protocol
    Custom(String),
}

/// Game session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSession {
    /// Session ID
    pub session_id: Uuid,

    /// Protocol type
    pub protocol: GameProtocolType,

    /// Session configuration
    pub config: SessionConfig,

    /// Session status
    pub status: SessionStatus,

    /// Creation time
    pub created_at: SystemTime,

    /// Last activity time
    pub last_activity: SystemTime,

    /// Connected players
    pub players: Vec<PlayerInfo>,
}

impl GameSession {
    /// Create a new game session
    #[must_use]
    pub fn new(session_id: Uuid, protocol: GameProtocolType, config: SessionConfig) -> Self {
        let now = SystemTime::now();

        Self {
            session_id,
            protocol,
            config,
            status: SessionStatus::Active,
            created_at: now,
            last_activity: now,
            players: Vec::new(),
        }
    }

    /// Add a player to the session
    pub fn add_player(&mut self, player: PlayerInfo) {
        self.players.push(player);
        self.last_activity = SystemTime::now();
    }

    /// Remove a player from the session
    pub fn remove_player(&mut self, player_id: &Uuid) {
        self.players.retain(|p| p.player_id != *player_id);
        self.last_activity = SystemTime::now();
    }

    /// Check if session is expired
    #[must_use]
    pub fn is_expired(&self, timeout: Duration) -> bool {
        self.last_activity.elapsed().unwrap_or(Duration::ZERO) > timeout
    }
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Session name
    pub name: String,

    /// Maximum players
    pub max_players: u32,

    /// Session password (optional)
    pub password: Option<String>,

    /// Public session flag
    pub public: bool,

    /// Custom properties
    pub properties: HashMap<String, String>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            name: "Game Session".to_string(),
            max_players: 8,
            password: None,
            public: true,
            properties: HashMap::new(),
        }
    }
}

/// Session status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionStatus {
    /// Session is active
    Active,
    /// Session is paused
    Paused,
    /// Session is ending
    Ending,
    /// Session has ended
    Ended,
}

/// Player information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    /// Player ID
    pub player_id: Uuid,

    /// Player name
    pub name: String,

    /// Player address
    pub address: SocketAddr,

    /// Join time
    pub joined_at: SystemTime,

    /// Last seen time
    pub last_seen: SystemTime,

    /// Player properties
    pub properties: HashMap<String, String>,
}

/// Protocol handler trait - uses native async methods for zero-cost abstractions
pub trait ProtocolHandler: Send + Sync {
    /// Get protocol type
    fn protocol_type(&self) -> GameProtocolType;

    /// Handle incoming packet
    async fn handle_packet(&mut self, data: &[u8], source: SocketAddr) -> SongbirdResult<Vec<u8>>;

    /// Initialize protocol handler
    async fn initialize(&mut self) -> SongbirdResult<()>;

    /// Shutdown protocol handler
    async fn shutdown(&mut self) -> SongbirdResult<()>;
}

/// Create a protocol handler for the given protocol type (returns enum for zero-cost dispatch)
pub fn create_protocol_handler(protocol: GameProtocolType) -> SongbirdResult<ProtocolHandlerImpl> {
    match protocol {
        GameProtocolType::UDP => Ok(ProtocolHandlerImpl::Udp(UdpProtocolHandler::new())),
        GameProtocolType::TCP => Ok(ProtocolHandlerImpl::Tcp(TcpProtocolHandler::new())),
        GameProtocolType::IPX => Ok(ProtocolHandlerImpl::Ipx(IpxProtocolHandler::new())),
        GameProtocolType::DirectPlay => {
            Ok(ProtocolHandlerImpl::DirectPlay(DirectPlayProtocolHandler::new()))
        }
        GameProtocolType::NetBIOS => {
            Ok(ProtocolHandlerImpl::NetBios(NetBiosProtocolHandler::new()))
        }
        GameProtocolType::Custom(name) => Err(SongbirdError::Network {
            message: format!("Custom protocol '{name}' not supported"),
            interface: None,
            suggestion: Some(
                "Use a standard protocol (UDP, TCP, IPX, DirectPlay, NetBIOS)".to_string(),
            ),
        }),
    }
}

// Protocol handler implementations

/// UDP protocol handler
#[derive(Debug)]
pub struct UdpProtocolHandler;

impl Default for UdpProtocolHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl UdpProtocolHandler {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

// Native async trait implementation (no boxing overhead)
impl ProtocolHandler for UdpProtocolHandler {
    fn protocol_type(&self) -> GameProtocolType {
        GameProtocolType::UDP
    }

    async fn handle_packet(&mut self, data: &[u8], _source: SocketAddr) -> SongbirdResult<Vec<u8>> {
        // Simple echo for now - real implementation would handle UDP gaming protocols
        Ok(data.to_vec())
    }

    async fn initialize(&mut self) -> SongbirdResult<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> SongbirdResult<()> {
        Ok(())
    }
}

/// TCP protocol handler
#[derive(Debug)]
pub struct TcpProtocolHandler;

impl Default for TcpProtocolHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl TcpProtocolHandler {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

// Native async trait implementation (no boxing overhead)
impl ProtocolHandler for TcpProtocolHandler {
    fn protocol_type(&self) -> GameProtocolType {
        GameProtocolType::TCP
    }

    async fn handle_packet(&mut self, data: &[u8], _source: SocketAddr) -> SongbirdResult<Vec<u8>> {
        // Simple echo for now - real implementation would handle TCP gaming protocols
        Ok(data.to_vec())
    }

    async fn initialize(&mut self) -> SongbirdResult<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> SongbirdResult<()> {
        Ok(())
    }
}

/// IPX protocol handler (legacy)
#[derive(Debug)]
pub struct IpxProtocolHandler;

impl Default for IpxProtocolHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl IpxProtocolHandler {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

// Native async trait implementation (no boxing overhead)
impl ProtocolHandler for IpxProtocolHandler {
    fn protocol_type(&self) -> GameProtocolType {
        GameProtocolType::IPX
    }

    async fn handle_packet(&mut self, data: &[u8], _source: SocketAddr) -> SongbirdResult<Vec<u8>> {
        // IPX protocol translation logic would go here
        Ok(data.to_vec())
    }

    async fn initialize(&mut self) -> SongbirdResult<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> SongbirdResult<()> {
        Ok(())
    }
}

/// `DirectPlay` protocol handler (legacy)
#[derive(Debug)]
pub struct DirectPlayProtocolHandler;

impl Default for DirectPlayProtocolHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl DirectPlayProtocolHandler {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

// Native async trait implementation (no boxing overhead)
impl ProtocolHandler for DirectPlayProtocolHandler {
    fn protocol_type(&self) -> GameProtocolType {
        GameProtocolType::DirectPlay
    }

    async fn handle_packet(&mut self, data: &[u8], _source: SocketAddr) -> SongbirdResult<Vec<u8>> {
        // DirectPlay protocol translation logic would go here
        Ok(data.to_vec())
    }

    async fn initialize(&mut self) -> SongbirdResult<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> SongbirdResult<()> {
        Ok(())
    }
}

/// `NetBIOS` protocol handler (legacy)
#[derive(Debug)]
pub struct NetBiosProtocolHandler;

impl Default for NetBiosProtocolHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl NetBiosProtocolHandler {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

// Native async trait implementation (no boxing overhead)
impl ProtocolHandler for NetBiosProtocolHandler {
    fn protocol_type(&self) -> GameProtocolType {
        GameProtocolType::NetBIOS
    }

    async fn handle_packet(&mut self, data: &[u8], _source: SocketAddr) -> SongbirdResult<Vec<u8>> {
        // NetBIOS protocol translation logic would go here
        Ok(data.to_vec())
    }

    async fn initialize(&mut self) -> SongbirdResult<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> SongbirdResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use uuid::Uuid;

    #[test]
    fn create_protocol_handler_custom_errors() {
        let err = create_protocol_handler(GameProtocolType::Custom("x".into())).unwrap_err();
        let s = err.to_string();
        assert!(s.contains("Custom") || s.contains("not supported"));
    }

    #[test]
    fn create_protocol_handler_udp_and_tcp_variants() {
        assert!(matches!(
            create_protocol_handler(GameProtocolType::UDP).unwrap(),
            ProtocolHandlerImpl::Udp(_)
        ));
        assert!(matches!(
            create_protocol_handler(GameProtocolType::TCP).unwrap(),
            ProtocolHandlerImpl::Tcp(_)
        ));
    }

    #[tokio::test]
    async fn udp_handler_echoes_packet() {
        let mut h = UdpProtocolHandler::new();
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 1);
        let out = h.handle_packet(b"ping", addr).await.unwrap();
        assert_eq!(out, b"ping");
    }

    #[test]
    fn game_session_add_remove_player() {
        let sid = Uuid::new_v4();
        let mut session = GameSession::new(sid, GameProtocolType::UDP, SessionConfig::default());
        let pid = Uuid::new_v4();
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9000);
        session.add_player(PlayerInfo {
            player_id: pid,
            name: "p".into(),
            address: addr,
            joined_at: std::time::SystemTime::UNIX_EPOCH,
            last_seen: std::time::SystemTime::UNIX_EPOCH,
            properties: std::collections::HashMap::new(),
        });
        assert_eq!(session.players.len(), 1);
        session.remove_player(&pid);
        assert!(session.players.is_empty());
    }

    #[test]
    fn game_session_is_expired() {
        let sid = Uuid::new_v4();
        let mut session = GameSession::new(sid, GameProtocolType::TCP, SessionConfig::default());
        assert!(!session.is_expired(Duration::from_secs(3600)));
        session.last_activity = std::time::SystemTime::UNIX_EPOCH;
        assert!(session.is_expired(Duration::from_secs(1)));
    }

    #[tokio::test]
    async fn gaming_manager_health_lists_protocols() {
        let mut mgr = GamingManager::new(GamingConfig {
            protocols: vec![GameProtocolType::UDP],
            ..GamingConfig::default()
        });
        mgr.initialize().await.unwrap();
        let h = mgr.health_check().await.unwrap();
        assert_eq!(h.status, NetworkStatus::Healthy);
        assert!(h.supported_protocols.contains(&GameProtocolType::UDP));
    }

    #[test]
    fn game_protocol_type_custom_is_hashable() {
        use std::collections::HashMap;
        let mut m = HashMap::new();
        m.insert(GameProtocolType::Custom("z".into()), 1u8);
        assert_eq!(m[&GameProtocolType::Custom("z".into())], 1);
    }

    #[test]
    fn game_protocol_type_serde_roundtrip() {
        let p = GameProtocolType::DirectPlay;
        let json = serde_json::to_string(&p).unwrap();
        let back: GameProtocolType = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
        let c = GameProtocolType::Custom("lobby".into());
        let json = serde_json::to_string(&c).unwrap();
        let back: GameProtocolType = serde_json::from_str(&json).unwrap();
        assert_eq!(c, back);
    }

    #[test]
    fn session_config_default_and_serde() {
        let c = SessionConfig::default();
        assert!(c.public);
        let json = serde_json::to_string(&c).unwrap();
        let back: SessionConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.max_players, c.max_players);
    }

    #[test]
    fn session_status_variants() {
        assert_ne!(SessionStatus::Active, SessionStatus::Ended);
    }

    #[tokio::test]
    async fn tcp_protocol_handler_echoes() {
        let mut h = TcpProtocolHandler::new();
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 2);
        let out = h.handle_packet(b"data", addr).await.unwrap();
        assert_eq!(out, b"data");
        assert_eq!(h.protocol_type(), GameProtocolType::TCP);
    }

    #[tokio::test]
    async fn gaming_manager_create_session() {
        let mut mgr = GamingManager::new(GamingConfig {
            protocols: vec![GameProtocolType::UDP],
            ..GamingConfig::default()
        });
        mgr.initialize().await.unwrap();
        let id = mgr.create_session(GameProtocolType::UDP, SessionConfig::default()).await.unwrap();
        assert!(mgr.get_session(&id).is_some());
    }

    #[test]
    fn ipx_handler_protocol_type() {
        use super::ProtocolHandler;
        let h = IpxProtocolHandler::new();
        assert_eq!(h.protocol_type(), GameProtocolType::IPX);
    }
}
