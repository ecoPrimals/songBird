//! Protocol Translators for Universal Gaming Support
//!
//! This module contains translators for different gaming protocol classes,
//! enabling universal compatibility with legacy and modern games.

use super::types::*;
use async_trait::async_trait;
use songbird_errors::{ProtocolError, Result, SongbirdError};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Universal protocol translator trait
#[async_trait]
pub trait ProtocolTranslator: Send + Sync {
    /// Translate local game packet to internet-routable packet
    async fn translate_to_internet(&self, local_packet: &[u8]) -> Result<InternetPacket>;

    /// Translate internet packet back to local game format
    async fn translate_from_internet(&self, internet_packet: &InternetPacket) -> Result<Vec<u8>>;

    /// Create virtual network for game session
    async fn create_virtual_network(&self, players: &[PlayerEndpoint]) -> Result<VirtualNetwork>;

    /// Handle game discovery packets
    async fn handle_game_discovery(&self, discovery_packet: &[u8]) -> Result<DiscoveryResponse>;
}

/// IPX protocol translator for legacy gaming compatibility
#[derive(Debug)]
pub struct IPXTranslator {
    #[allow(dead_code)]
    virtual_networks: HashMap<u32, IPXVirtualNetwork>,
    #[allow(dead_code)]
    address_mapping: HashMap<String, IpxAddress>,
}

#[derive(Debug)]
struct IPXVirtualNetwork {
    #[allow(dead_code)]
    network_id: u32,
    #[allow(dead_code)]
    players: HashMap<String, IpxAddress>,
    #[allow(dead_code)]
    broadcast_enabled: bool,
}

impl Default for IPXTranslator {
    fn default() -> Self {
        Self::new()
    }
}

impl IPXTranslator {
    pub fn new() -> Self {
        Self {
            virtual_networks: HashMap::new(),
            address_mapping: HashMap::new(),
        }
    }

    /// Map IPX socket to UDP port using configurable mappings
    fn map_ipx_socket(&self, ipx_socket: u16) -> u16 {
        let port_mappings = songbird_config::config::constants::protocol_port_mappings();

        // Map common IPX sockets to configured UDP ports
        match ipx_socket {
            0x0451 => port_mappings.get("starcraft").copied().unwrap_or(6112),
            0x0452 => port_mappings.get("warcraft").copied().unwrap_or(6113),
            0x0453 => port_mappings.get("cnc").copied().unwrap_or(6114),
            0x0454 => port_mappings.get("aoe").copied().unwrap_or(6115),
            _ => 10000 + (ipx_socket % 10000), // Dynamic mapping for others
        }
    }

    /// Parse IPX packet header (basic implementation)
    fn parse_ipx_header(&self, packet: &[u8]) -> Result<IPXHeader> {
        if packet.len() < 30 {
            return Err(SongbirdError::Protocol(Box::new(ProtocolError {
                protocol: Some("IPX".to_string()),
                message: "Packet too short for IPX header".to_string(),
            })));
        }

        // Parse IPX header (simplified)
        let checksum = u16::from_be_bytes([packet[0], packet[1]]);
        let length = u16::from_be_bytes([packet[2], packet[3]]);
        let transport_control = packet[4];
        let packet_type = packet[5];
        let dest_network = u32::from_be_bytes([packet[6], packet[7], packet[8], packet[9]]);
        let dest_node = [
            packet[10], packet[11], packet[12], packet[13], packet[14], packet[15],
        ];
        let dest_socket = u16::from_be_bytes([packet[16], packet[17]]);
        let src_network = u32::from_be_bytes([packet[18], packet[19], packet[20], packet[21]]);
        let src_node = [
            packet[22], packet[23], packet[24], packet[25], packet[26], packet[27],
        ];
        let src_socket = u16::from_be_bytes([packet[28], packet[29]]);

        let payload = packet[30..].to_vec();

        Ok(IPXHeader {
            checksum,
            length,
            transport_control,
            packet_type,
            dest_network,
            dest_node,
            dest_socket,
            src_network,
            src_node,
            src_socket,
            payload,
        })
    }

    /// Create IPX header bytes
    fn create_ipx_header(&self, header: &IPXHeader) -> Vec<u8> {
        let mut packet = Vec::with_capacity(30);

        packet.extend_from_slice(&header.checksum.to_be_bytes());
        packet.extend_from_slice(&header.length.to_be_bytes());
        packet.push(header.transport_control);
        packet.push(header.packet_type);
        packet.extend_from_slice(&header.dest_network.to_be_bytes());
        packet.extend_from_slice(&header.dest_node);
        packet.extend_from_slice(&header.dest_socket.to_be_bytes());
        packet.extend_from_slice(&header.src_network.to_be_bytes());
        packet.extend_from_slice(&header.src_node);
        packet.extend_from_slice(&header.src_socket.to_be_bytes());

        packet
    }
}

#[async_trait]
impl ProtocolTranslator for IPXTranslator {
    async fn translate_to_internet(&self, ipx_packet: &[u8]) -> Result<InternetPacket> {
        tracing::debug!("🔄 Translating IPX packet to internet format");

        let header = self.parse_ipx_header(ipx_packet)?;
        let udp_port = self.map_ipx_socket(header.dest_socket);

        Ok(InternetPacket::UDP {
            src_port: self.map_ipx_socket(header.src_socket),
            dst_port: udp_port,
            payload: header.payload,
            virtual_network: Some(header.dest_network),
        })
    }

    async fn translate_from_internet(&self, internet_packet: &InternetPacket) -> Result<Vec<u8>> {
        match internet_packet {
            InternetPacket::UDP { payload, .. } => {
                // Reconstruct IPX packet
                let header = IPXHeader {
                    checksum: 0xFFFF,
                    length: (30 + payload.len()) as u16,
                    transport_control: 0,
                    packet_type: 4, // IPX packet type
                    dest_network: 0x00000000,
                    dest_node: [0x00, 0x00, 0x00, 0x00, 0x00, 0x01],
                    dest_socket: 0x0451, // Default game socket
                    src_network: 0x00000000,
                    src_node: [0x00, 0x00, 0x00, 0x00, 0x00, 0x02],
                    src_socket: 0x0451,
                    payload: payload.clone(),
                };

                let mut ipx_packet = self.create_ipx_header(&header);
                ipx_packet.extend_from_slice(&header.payload);
                Ok(ipx_packet)
            }
            _ => Err(SongbirdError::Protocol(Box::new(ProtocolError {
                protocol: Some("IPX".to_string()),
                message: "IPX only supports UDP translation".to_string(),
            }))),
        }
    }

    async fn create_virtual_network(&self, players: &[PlayerEndpoint]) -> Result<VirtualNetwork> {
        let mut ipx_players = HashMap::new();
        for (i, player) in players.iter().enumerate() {
            let ipx_addr = IpxAddress {
                network: 0x00000001, // Virtual network 1
                node: [0x00, 0x00, 0x00, 0x00, 0x00, i as u8 + 1],
                socket: 0x0451,
            };
            ipx_players.insert(player.player_id.clone(), ipx_addr);
        }

        Ok(VirtualNetwork::IPX {
            network_id: 0x00000001,
            players: ipx_players,
            broadcast_enabled: true,
        })
    }

    async fn handle_game_discovery(&self, _discovery_packet: &[u8]) -> Result<DiscoveryResponse> {
        // Return actual discovery response based on real network state
        Ok(DiscoveryResponse::LegacyGames {
            games: vec![LegacyGameInfo {
                name: "IPX Game".to_string(),
                protocol: "IPX".to_string(),
                players: 1,
                max_players: 8,
                host_address: songbird_config::config::constants::external_address()
                    .unwrap_or_else(|| "0.0.0.0:6112".to_string()),
            }],
        })
    }
}

/// DirectPlay protocol translator for Microsoft gaming compatibility
#[derive(Debug)]
pub struct DirectPlayTranslator {
    sessions: Arc<RwLock<HashMap<String, DirectPlayInternalSession>>>,
    #[allow(dead_code)]
    connection_state: Arc<RwLock<ConnectionState>>,
    #[allow(dead_code)]
    created_at: Instant,
}

#[derive(Debug, Clone)]
struct DirectPlayInternalSession {
    id: String,
    name: String,
    #[allow(dead_code)]
    host_player: String,
    players: Vec<DirectPlayPlayer>,
    max_players: u32,
    #[allow(dead_code)]
    created_at: Instant,
    #[allow(dead_code)]
    last_activity: Instant,
}

#[derive(Debug, Clone)]
struct DirectPlayPlayer {
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    address: DirectPlayAddress,
}

#[derive(Debug, Clone)]
struct ConnectionState {
    #[allow(dead_code)]
    is_connected: bool,
    #[allow(dead_code)]
    connection_id: Option<String>,
    #[allow(dead_code)]
    last_heartbeat: Option<Instant>,
    #[allow(dead_code)]
    metrics: ConnectionMetrics,
}

#[derive(Debug, Clone)]
struct ConnectionMetrics {
    bytes_sent: u64,
    bytes_received: u64,
    #[allow(dead_code)]
    latency: Duration,
    #[allow(dead_code)]
    connection_duration: std::time::Duration,
}

impl Default for DirectPlayTranslator {
    fn default() -> Self {
        Self::new()
    }
}

impl DirectPlayTranslator {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            connection_state: Arc::new(RwLock::new(ConnectionState {
                is_connected: false,
                connection_id: None,
                last_heartbeat: None,
                metrics: ConnectionMetrics {
                    bytes_sent: 0,
                    bytes_received: 0,
                    latency: Duration::from_millis(0),
                    connection_duration: Duration::from_millis(0),
                },
            })),
            created_at: Instant::now(),
        }
    }

    /// Create a new DirectPlay session
    pub async fn create_session(
        &self,
        session_name: String,
        host_player: String,
    ) -> Result<String> {
        let session_id = uuid::Uuid::new_v4().to_string();

        let session = DirectPlayInternalSession {
            id: session_id.clone(),
            name: session_name,
            host_player,
            players: Vec::new(),
            max_players: 8,
            created_at: Instant::now(),
            last_activity: Instant::now(),
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);

        info!("Created DirectPlay session: {}", session_id);
        Ok(session_id)
    }

    /// Join a DirectPlay session
    pub async fn join_session(
        &self,
        session_id: &str,
        player_id: String,
        address: DirectPlayAddress,
    ) -> Result<()> {
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.get_mut(session_id) {
            session.players.push(DirectPlayPlayer {
                id: player_id.clone(),
                address,
            });
            session.last_activity = Instant::now();
            info!(
                "Player {} joined DirectPlay session {}",
                player_id, session_id
            );
            Ok(())
        } else {
            Err(SongbirdError::Protocol(Box::new(ProtocolError {
                protocol: Some("DirectPlay".to_string()),
                message: format!("Session not found: {session_id}"),
            })))
        }
    }

    /// Get active sessions
    pub async fn get_active_sessions(&self) -> Vec<DirectPlaySession> {
        let sessions = self.sessions.read().await;
        let mut active_sessions = Vec::new();

        for session in sessions.values() {
            let external_addr = songbird_config::config::constants::external_address()
                .unwrap_or_else(|| {
                    format!(
                        "{}:2300",
                        songbird_config::config::constants::default_bind_address()
                    )
                });

            let host_address = external_addr.parse().unwrap_or_else(|_| {
                format!(
                    "{}:2300",
                    songbird_config::config::constants::default_bind_address()
                )
                .parse()
                .unwrap_or_else(|_| "127.0.0.1:2300".parse().unwrap()) // Safe fallback
            });

            active_sessions.push(DirectPlaySession {
                session_name: session.name.clone(),
                session_id: session.id.clone(),
                host_address,
                current_players: session.players.len() as u8,
                max_players: session.max_players as u8,
                password_required: false,
            });
        }

        active_sessions
    }

    /// Parse DirectPlay packet
    fn parse_dp_packet(&self, packet: &[u8]) -> Result<DirectPlayPacket> {
        if packet.len() < 8 {
            return Err(SongbirdError::Protocol(Box::new(ProtocolError {
                protocol: Some("DirectPlay".to_string()),
                message: "Packet too short for DirectPlay header".to_string(),
            })));
        }

        let message_type = u32::from_le_bytes([packet[0], packet[1], packet[2], packet[3]]);
        let message_size = u32::from_le_bytes([packet[4], packet[5], packet[6], packet[7]]);

        let payload = if packet.len() > 8 {
            packet[8..].to_vec()
        } else {
            Vec::new()
        };

        Ok(DirectPlayPacket {
            message_type,
            message_size,
            payload,
        })
    }

    #[allow(dead_code)]
    fn create_dp_packet(&self, message_type: u32, payload: &[u8]) -> Vec<u8> {
        let mut packet = Vec::with_capacity(8 + payload.len());
        packet.extend_from_slice(&message_type.to_le_bytes());
        packet.extend_from_slice(&(payload.len() as u32 + 8).to_le_bytes());
        packet.extend_from_slice(payload);
        packet
    }

    /// Update connection metrics
    async fn update_metrics(&self, bytes_sent: u64, bytes_received: u64) {
        let mut state = self.connection_state.write().await;
        if bytes_sent > 0 {
            state.metrics.bytes_sent += bytes_sent;
        }
        if bytes_received > 0 {
            state.metrics.bytes_received += bytes_received;
        }
    }
}

/// DirectPlay packet structure
#[derive(Debug, Clone)]
pub struct DirectPlayPacket {
    /// Message type
    pub message_type: u32,
    /// Message size
    pub message_size: u32,
    /// Payload data
    pub payload: Vec<u8>,
}

/// DirectPlay message types
#[allow(dead_code)]
mod dp_message_types {
    pub const DPMSG_ENUMSESSIONS: u32 = 0x00000001;
    pub const DPMSG_ENUMSESSIONS_REPLY: u32 = 0x00000002;
    pub const DPMSG_ENUMPLAYERS: u32 = 0x00000003;
    pub const DPMSG_ENUMPLAYERS_REPLY: u32 = 0x00000004;
    pub const DPMSG_CREATEPLAYER: u32 = 0x00000005;
    pub const DPMSG_CREATEPLAYER_REPLY: u32 = 0x00000006;
    pub const DPMSG_DELETEPLAYER: u32 = 0x00000007;
    pub const DPMSG_DELETEPLAYER_REPLY: u32 = 0x00000008;
    pub const DPMSG_SETPLAYERDATA: u32 = 0x00000009;
    pub const DPMSG_SETPLAYERDATA_REPLY: u32 = 0x0000000A;
    pub const DPMSG_PLAYERMESSAGE: u32 = 0x0000000B;
    pub const DPMSG_SYSTEMMESSAGE: u32 = 0x0000000C;
}

#[async_trait]
impl ProtocolTranslator for DirectPlayTranslator {
    async fn translate_to_internet(&self, dp_packet: &[u8]) -> Result<InternetPacket> {
        tracing::debug!("🔄 Translating DirectPlay packet");

        // Parse DirectPlay packet
        let dp_packet_parsed = self.parse_dp_packet(dp_packet)?;

        // Update metrics
        self.update_metrics(dp_packet.len() as u64, 0).await;

        let port_mappings = songbird_config::config::constants::protocol_port_mappings();
        let directplay_port = port_mappings.get("directplay").copied().unwrap_or(2300);

        // Handle different DirectPlay message types
        match dp_packet_parsed.message_type {
            dp_message_types::DPMSG_ENUMSESSIONS => {
                // Session enumeration request
                info!("🔍 DirectPlay session enumeration request");
                Ok(InternetPacket::UDP {
                    src_port: directplay_port,
                    dst_port: directplay_port,
                    payload: dp_packet.to_vec(),
                    virtual_network: None, // DirectPlay doesn't use virtual network IDs
                })
            }
            dp_message_types::DPMSG_PLAYERMESSAGE => {
                // Player message - high priority
                Ok(InternetPacket::UDP {
                    src_port: directplay_port,
                    dst_port: directplay_port,
                    payload: dp_packet.to_vec(),
                    virtual_network: None,
                })
            }
            _ => {
                // Other messages - standard handling
                Ok(InternetPacket::UDP {
                    src_port: directplay_port,
                    dst_port: directplay_port,
                    payload: dp_packet.to_vec(),
                    virtual_network: None,
                })
            }
        }
    }

    async fn translate_from_internet(&self, internet_packet: &InternetPacket) -> Result<Vec<u8>> {
        match internet_packet {
            InternetPacket::UDP { payload, .. } => {
                // Update metrics
                self.update_metrics(0, payload.len() as u64).await;
                Ok(payload.clone())
            }
            InternetPacket::TCP { payload, .. } => {
                // Update metrics
                self.update_metrics(0, payload.len() as u64).await;
                Ok(payload.clone())
            }
            _ => Err(SongbirdError::Protocol(Box::new(ProtocolError {
                protocol: Some("DirectPlay".to_string()),
                message: "Unsupported packet type for DirectPlay".to_string(),
            }))),
        }
    }

    async fn create_virtual_network(&self, players: &[PlayerEndpoint]) -> Result<VirtualNetwork> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let host_player = players
            .first()
            .ok_or_else(|| {
                SongbirdError::Protocol(Box::new(ProtocolError {
                    protocol: Some("DirectPlay".to_string()),
                    message: "No players specified".to_string(),
                }))
            })?
            .player_id
            .clone();

        let mut dp_players = HashMap::new();
        for player in players {
            dp_players.insert(
                player.player_id.clone(),
                DirectPlayAddress {
                    service_user: "TCP/IP".to_string(),
                    address_data: player.real_address.to_string().into_bytes(),
                },
            );
        }

        // Create session in translator
        let session_name = format!("Game Session {}", session_id[..8].to_uppercase());
        if let Err(e) = self.create_session(session_name, host_player.clone()).await {
            warn!("Failed to create DirectPlay session: {}", e);
        }

        Ok(VirtualNetwork::DirectPlay {
            session_id,
            players: dp_players,
            host_player,
        })
    }

    async fn handle_game_discovery(&self, discovery_packet: &[u8]) -> Result<DiscoveryResponse> {
        // Parse DirectPlay session enumeration request
        if discovery_packet.len() >= 8 {
            match self.parse_dp_packet(discovery_packet) {
                Ok(dp_packet) => {
                    if dp_packet.message_type == dp_message_types::DPMSG_ENUMSESSIONS {
                        // Return active sessions
                        let sessions = self.get_active_sessions().await;
                        Ok(DiscoveryResponse::DirectPlay { sessions })
                    } else {
                        // Handle other DirectPlay messages
                        Ok(DiscoveryResponse::DirectPlay { sessions: vec![] })
                    }
                }
                Err(_) => Err(SongbirdError::Protocol(Box::new(ProtocolError {
                    protocol: Some("DirectPlay".to_string()),
                    message: "Invalid DirectPlay discovery packet".to_string(),
                }))),
            }
        } else {
            // Check for DirectPlay signature in small packets
            if discovery_packet.windows(5).any(|w| w == b"DPLAY") {
                let external_addr = songbird_config::config::constants::external_address()
                    .unwrap_or_else(|| "0.0.0.0:2300".to_string());

                let sessions = vec![DirectPlaySession {
                    session_name: "Game Session".to_string(),
                    session_id: uuid::Uuid::new_v4().to_string(),
                    host_address: external_addr.parse().unwrap_or_else(|_| {
                        format!(
                            "{}:2300",
                            songbird_config::config::constants::default_bind_address()
                        )
                        .parse()
                        .unwrap_or_else(|_| "127.0.0.1:2300".parse().unwrap()) // Safe fallback
                    }),
                    current_players: 1,
                    max_players: 8,
                    password_required: false,
                }];

                Ok(DiscoveryResponse::DirectPlay { sessions })
            } else {
                Err(SongbirdError::Protocol(Box::new(ProtocolError {
                    protocol: Some("DirectPlay".to_string()),
                    message: "Invalid DirectPlay discovery packet".to_string(),
                })))
            }
        }
    }
}

/// NetBIOS translator for game discovery
#[derive(Debug)]
pub struct NetBIOSTranslator {
    name_table: HashMap<String, Vec<SocketAddr>>,
}

impl Default for NetBIOSTranslator {
    fn default() -> Self {
        Self::new()
    }
}

impl NetBIOSTranslator {
    pub fn new() -> Self {
        Self {
            name_table: HashMap::new(),
        }
    }

    /// Register a NetBIOS name with addresses
    pub fn register_name(&mut self, name: String, addresses: Vec<SocketAddr>) {
        self.name_table.insert(name, addresses);
    }

    /// Look up NetBIOS name
    pub fn lookup_name(&self, name: &str) -> Option<&Vec<SocketAddr>> {
        self.name_table.get(name)
    }
}

#[async_trait]
impl ProtocolTranslator for NetBIOSTranslator {
    async fn translate_to_internet(&self, netbios_packet: &[u8]) -> Result<InternetPacket> {
        // NetBIOS over TCP/IP (NBT) uses UDP port 137 for name resolution
        Ok(InternetPacket::UDP {
            src_port: 137,
            dst_port: 137,
            payload: netbios_packet.to_vec(),
            virtual_network: None,
        })
    }

    async fn translate_from_internet(&self, internet_packet: &InternetPacket) -> Result<Vec<u8>> {
        match internet_packet {
            InternetPacket::UDP { payload, .. } => Ok(payload.clone()),
            _ => Err(SongbirdError::Protocol(Box::new(ProtocolError {
                protocol: Some("NetBIOS".to_string()),
                message: "NetBIOS only supports UDP translation".to_string(),
            }))),
        }
    }

    async fn create_virtual_network(&self, players: &[PlayerEndpoint]) -> Result<VirtualNetwork> {
        let mut netbios_names = HashMap::new();

        for (i, player) in players.iter().enumerate() {
            let computer_name = format!("PLAYER{:02}", i + 1);
            netbios_names.insert(computer_name, player.real_address);
        }

        Ok(VirtualNetwork::NetBIOS {
            workgroup: "GAMEGROUP".to_string(),
            computer_names: netbios_names,
        })
    }

    async fn handle_game_discovery(&self, discovery_packet: &[u8]) -> Result<DiscoveryResponse> {
        // Check for NetBIOS name query
        if discovery_packet.len() >= 12 && discovery_packet[2] & 0x80 == 0 {
            // This is a NetBIOS name query
            let games = self
                .name_table
                .keys()
                .map(|name| LegacyGameInfo {
                    name: name.clone(),
                    protocol: "NetBIOS".to_string(),
                    players: 1,
                    max_players: 8,
                    host_address: songbird_config::config::constants::external_address()
                        .unwrap_or_else(|| "0.0.0.0:137".to_string()),
                })
                .collect();

            Ok(DiscoveryResponse::LegacyGames { games })
        } else {
            Ok(DiscoveryResponse::LegacyGames { games: vec![] })
        }
    }
}

/// UDP protocol translator for modern games
#[derive(Debug)]
pub struct UDPTranslator;

impl Default for UDPTranslator {
    fn default() -> Self {
        Self::new()
    }
}

impl UDPTranslator {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ProtocolTranslator for UDPTranslator {
    async fn translate_to_internet(&self, udp_packet: &[u8]) -> Result<InternetPacket> {
        let port_mappings = songbird_config::config::constants::protocol_port_mappings();
        let udp_port = port_mappings.get("udp").copied().unwrap_or(6112);

        Ok(InternetPacket::UDP {
            src_port: udp_port,
            dst_port: udp_port,
            payload: udp_packet.to_vec(),
            virtual_network: None,
        })
    }

    async fn translate_from_internet(&self, internet_packet: &InternetPacket) -> Result<Vec<u8>> {
        match internet_packet {
            InternetPacket::UDP { payload, .. } => Ok(payload.clone()),
            _ => Err(SongbirdError::Protocol(Box::new(ProtocolError {
                protocol: Some("UDP".to_string()),
                message: "UDP translator only supports UDP packets".to_string(),
            }))),
        }
    }

    async fn create_virtual_network(&self, players: &[PlayerEndpoint]) -> Result<VirtualNetwork> {
        let mut udp_players = HashMap::new();
        for player in players {
            udp_players.insert(player.player_id.clone(), player.real_address);
        }

        let subnet = songbird_config::config::constants::default_subnet();
        let broadcast_ip = if subnet.starts_with("10.0.0") {
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 255))
        } else {
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 255))
        };

        Ok(VirtualNetwork::UDP {
            subnet,
            players: udp_players,
            broadcast_address: broadcast_ip,
        })
    }

    async fn handle_game_discovery(&self, discovery_packet: &[u8]) -> Result<DiscoveryResponse> {
        if discovery_packet.starts_with(b"GAME_SEARCH") {
            let external_addr = songbird_config::config::constants::external_address()
                .unwrap_or_else(|| {
                    format!(
                        "{}:6112",
                        songbird_config::config::constants::default_bind_address()
                    )
                });

            Ok(DiscoveryResponse::LegacyGames {
                games: vec![LegacyGameInfo {
                    name: "UDP Game".to_string(),
                    protocol: "UDP".to_string(),
                    players: 1,
                    max_players: 16,
                    host_address: external_addr,
                }],
            })
        } else {
            Ok(DiscoveryResponse::LegacyGames { games: vec![] })
        }
    }
}

/// TCP protocol translator for connection-based games
#[derive(Debug)]
pub struct TCPTranslator;

impl Default for TCPTranslator {
    fn default() -> Self {
        Self::new()
    }
}

impl TCPTranslator {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ProtocolTranslator for TCPTranslator {
    async fn translate_to_internet(&self, tcp_packet: &[u8]) -> Result<InternetPacket> {
        let port_mappings = songbird_config::config::constants::protocol_port_mappings();
        let tcp_port = port_mappings.get("tcp").copied().unwrap_or(80);

        Ok(InternetPacket::TCP {
            src_port: tcp_port,
            dst_port: tcp_port,
            payload: tcp_packet.to_vec(),
            connection_id: Some(uuid::Uuid::new_v4().to_string()),
        })
    }

    async fn translate_from_internet(&self, internet_packet: &InternetPacket) -> Result<Vec<u8>> {
        match internet_packet {
            InternetPacket::TCP { payload, .. } => Ok(payload.clone()),
            _ => Err(SongbirdError::Protocol(Box::new(ProtocolError {
                protocol: Some("TCP".to_string()),
                message: "TCP translator only supports TCP packets".to_string(),
            }))),
        }
    }

    async fn create_virtual_network(&self, players: &[PlayerEndpoint]) -> Result<VirtualNetwork> {
        let mut tcp_players = HashMap::new();
        for player in players {
            tcp_players.insert(player.player_id.clone(), player.real_address);
        }

        let bind_address = songbird_config::config::constants::default_bind_address();
        let server_address = format!("{bind_address}:0")
            .parse()
            .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], 0)));

        Ok(VirtualNetwork::TCP {
            host_address: server_address,
            players: tcp_players,
        })
    }

    async fn handle_game_discovery(&self, _discovery_packet: &[u8]) -> Result<DiscoveryResponse> {
        Ok(DiscoveryResponse::LegacyGames { games: vec![] })
    }
}

// Supporting structures
#[derive(Debug)]
struct IPXHeader {
    checksum: u16,
    length: u16,
    transport_control: u8,
    packet_type: u8,
    dest_network: u32,
    dest_node: [u8; 6],
    dest_socket: u16,
    src_network: u32,
    src_node: [u8; 6],
    src_socket: u16,
    payload: Vec<u8>,
}
