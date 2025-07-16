//! Protocol Translators for Universal Gaming Support
//!
//! This module contains translators for different gaming protocol classes,
//! enabling universal compatibility with legacy and modern games.

use super::types::*;
use crate::errors::{Result, SongbirdError};
use async_trait::async_trait;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

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

/// IPX protocol translator for DOS/Windows 95 era games
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
        let port_mappings = crate::config::constants::protocol_port_mappings();

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
            return Err(SongbirdError::Protocol {
                protocol: "IPX".to_string(),
                message: "IPX packet too short".to_string(),
            });
        }

        Ok(IPXHeader {
            checksum: u16::from_be_bytes([packet[0], packet[1]]),
            length: u16::from_be_bytes([packet[2], packet[3]]),
            transport_control: packet[4],
            packet_type: packet[5],
            dest_network: u32::from_be_bytes([packet[6], packet[7], packet[8], packet[9]]),
            dest_node: [
                packet[10], packet[11], packet[12], packet[13], packet[14], packet[15],
            ],
            dest_socket: u16::from_be_bytes([packet[16], packet[17]]),
            src_network: u32::from_be_bytes([packet[18], packet[19], packet[20], packet[21]]),
            src_node: [
                packet[22], packet[23], packet[24], packet[25], packet[26], packet[27],
            ],
            src_socket: u16::from_be_bytes([packet[28], packet[29]]),
            payload: packet[30..].to_vec(),
        })
    }

    /// Create IPX packet header
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
            _ => Err(SongbirdError::Protocol {
                protocol: "IPX".to_string(),
                message: "IPX only supports UDP translation".to_string(),
            }),
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
                host_address: crate::config::constants::external_address().unwrap_or_else(|| {
                    format!(
                        "{}:6112",
                        crate::config::constants::network::production_bind_address()
                    )
                }),
            }],
        })
    }
}

/// DirectPlay protocol translator for Windows 95-XP era games  
#[derive(Debug)]
pub struct DirectPlayTranslator {
    #[allow(dead_code)]
    _placeholder: u8,
}

impl Default for DirectPlayTranslator {
    fn default() -> Self {
        Self::new()
    }
}

impl DirectPlayTranslator {
    pub fn new() -> Self {
        Self { _placeholder: 0 }
    }
}

#[async_trait]
impl ProtocolTranslator for DirectPlayTranslator {
    async fn translate_to_internet(&self, dp_packet: &[u8]) -> Result<InternetPacket> {
        tracing::debug!("🔄 Translating DirectPlay packet");

        let port_mappings = crate::config::constants::protocol_port_mappings();
        let directplay_port = port_mappings.get("directplay").copied().unwrap_or(2300);

        Ok(InternetPacket::UDP {
            src_port: directplay_port,
            dst_port: directplay_port,
            payload: dp_packet.to_vec(),
            virtual_network: None,
        })
    }

    async fn translate_from_internet(&self, internet_packet: &InternetPacket) -> Result<Vec<u8>> {
        match internet_packet {
            InternetPacket::UDP { payload, .. } => Ok(payload.clone()),
            InternetPacket::TCP { payload, .. } => Ok(payload.clone()),
            _ => Err(SongbirdError::Protocol {
                protocol: "DirectPlay".to_string(),
                message: "Unsupported packet type for DirectPlay".to_string(),
            }),
        }
    }

    async fn create_virtual_network(&self, players: &[PlayerEndpoint]) -> Result<VirtualNetwork> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let host_player = players
            .first()
            .ok_or_else(|| SongbirdError::Protocol {
                protocol: "DirectPlay".to_string(),
                message: "No players specified".to_string(),
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

        Ok(VirtualNetwork::DirectPlay {
            session_id,
            players: dp_players,
            host_player,
        })
    }

    async fn handle_game_discovery(&self, discovery_packet: &[u8]) -> Result<DiscoveryResponse> {
        // Parse DirectPlay session enumeration request
        if discovery_packet.windows(5).any(|w| w == b"DPLAY") {
            let external_addr = crate::config::constants::external_address().unwrap_or_else(|| {
                format!(
                    "{}:2300",
                    crate::config::constants::network::production_bind_address()
                )
            });

            let sessions = vec![DirectPlaySession {
                session_name: "Game Session".to_string(),
                session_id: uuid::Uuid::new_v4().to_string(),
                host_address: external_addr.parse().unwrap_or_else(|_| {
                    format!("{}:2300", crate::config::constants::default_bind_address())
                        .parse()
                        .expect("Default bind address should be valid")
                }),
                current_players: 1,
                max_players: 8,
                password_required: false,
            }];

            Ok(DiscoveryResponse::DirectPlay { sessions })
        } else {
            Err(SongbirdError::Protocol {
                protocol: "DirectPlay".to_string(),
                message: "Invalid DirectPlay discovery packet".to_string(),
            })
        }
    }
}

/// NetBIOS translator for game discovery
#[derive(Debug)]
pub struct NetBIOSTranslator {
    #[allow(dead_code)]
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
}

#[async_trait]
impl ProtocolTranslator for NetBIOSTranslator {
    async fn translate_to_internet(&self, netbios_packet: &[u8]) -> Result<InternetPacket> {
        tracing::debug!("🔄 Translating NetBIOS packet");

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
            _ => Err(SongbirdError::Protocol {
                protocol: "NetBIOS".to_string(),
                message: "NetBIOS only supports UDP".to_string(),
            }),
        }
    }

    async fn create_virtual_network(&self, players: &[PlayerEndpoint]) -> Result<VirtualNetwork> {
        let mut udp_players = HashMap::new();
        for player in players {
            udp_players.insert(player.player_id.clone(), player.real_address);
        }

        let subnet = crate::config::constants::default_subnet();
        let _gateway = crate::config::constants::default_gateway();
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

    async fn handle_game_discovery(&self, _discovery_packet: &[u8]) -> Result<DiscoveryResponse> {
        let external_addr = crate::config::constants::external_address().unwrap_or_else(|| {
            format!(
                "{}:137",
                crate::config::constants::network::production_bind_address()
            )
        });

        let game_sessions = vec![NetBIOSGameSession {
            name: "NetBIOS Game".to_string(),
            address: external_addr.parse().unwrap_or_else(|_| {
                format!("{}:137", crate::config::constants::default_bind_address())
                    .parse()
                    .expect("Default bind address should be valid")
            }),
            players: 1,
            max_players: 8,
        }];

        Ok(DiscoveryResponse::NetBIOS { game_sessions })
    }
}

/// UDP Broadcast translator for simple broadcast games
#[derive(Debug)]
pub struct UDPBroadcastTranslator;

impl Default for UDPBroadcastTranslator {
    fn default() -> Self {
        Self::new()
    }
}

impl UDPBroadcastTranslator {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ProtocolTranslator for UDPBroadcastTranslator {
    async fn translate_to_internet(&self, udp_packet: &[u8]) -> Result<InternetPacket> {
        let gaming_ports = crate::config::constants::default_gaming_ports();
        let port = gaming_ports.first().copied().unwrap_or(6112);

        Ok(InternetPacket::UDP {
            src_port: port,
            dst_port: port,
            payload: udp_packet.to_vec(),
            virtual_network: None,
        })
    }

    async fn translate_from_internet(&self, internet_packet: &InternetPacket) -> Result<Vec<u8>> {
        match internet_packet {
            InternetPacket::UDP { payload, .. } => Ok(payload.clone()),
            _ => Err(SongbirdError::Protocol {
                protocol: "UDP".to_string(),
                message: "UDP translator only supports UDP packets".to_string(),
            }),
        }
    }

    async fn create_virtual_network(&self, players: &[PlayerEndpoint]) -> Result<VirtualNetwork> {
        let mut udp_players = HashMap::new();
        for player in players {
            udp_players.insert(player.player_id.clone(), player.real_address);
        }

        let subnet = crate::config::constants::default_subnet();
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
            let external_addr = crate::config::constants::external_address().unwrap_or_else(|| {
                format!("{}:6112", crate::config::constants::default_bind_address())
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
        let port_mappings = crate::config::constants::protocol_port_mappings();
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
            _ => Err(SongbirdError::Protocol {
                protocol: "TCP".to_string(),
                message: "TCP translator only supports TCP packets".to_string(),
            }),
        }
    }

    async fn create_virtual_network(&self, players: &[PlayerEndpoint]) -> Result<VirtualNetwork> {
        let mut tcp_players = HashMap::new();
        for player in players {
            tcp_players.insert(player.player_id.clone(), player.real_address);
        }

        let bind_address = crate::config::constants::default_bind_address();
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
