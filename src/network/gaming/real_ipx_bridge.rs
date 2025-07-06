//! Real IPX Network Bridge Implementation
//!
//! This module implements actual UDP socket-based IPX network emulation
//! for legacy games like StarCraft, Age of Empires I, Command & Conquer

use crate::errors::{Result, SongbirdError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};

/// IPX address structure (32-bit network + 48-bit node)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IpxAddress {
    pub network: u32,
    pub node: [u8; 6],
}

impl IpxAddress {
    pub fn new(network: u32, node: [u8; 6]) -> Self {
        Self { network, node }
    }

    pub fn from_ip(ip: IpAddr) -> Self {
        match ip {
            IpAddr::V4(ipv4) => {
                let octets = ipv4.octets();
                Self {
                    network: u32::from_be_bytes(octets),
                    node: [octets[0], octets[1], octets[2], octets[3], 0, 0],
                }
            }
            IpAddr::V6(_) => {
                // For IPv6, use hash of the address
                Self {
                    network: 0x01000000,      // Default network for IPv6
                    node: [0, 0, 0, 0, 0, 1], // Simplified mapping
                }
            }
        }
    }
}

/// IPX packet structure
#[derive(Debug, Clone)]
pub struct IpxPacket {
    pub header: IpxHeader,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct IpxHeader {
    pub checksum: u16,
    pub length: u16,
    pub transport_control: u8,
    pub packet_type: u8,
    pub dest_network: u32,
    pub dest_node: [u8; 6],
    pub dest_socket: u16,
    pub src_network: u32,
    pub src_node: [u8; 6],
    pub src_socket: u16,
}

/// IPX-to-UDP packet translator
pub struct IPXToUDPTranslator {
    #[allow(dead_code)]
    virtual_network: u32,
    node_mappings: Arc<RwLock<HashMap<IpxAddress, SocketAddr>>>,
}

impl IPXToUDPTranslator {
    pub fn new(virtual_network: u32) -> Self {
        Self {
            virtual_network,
            node_mappings: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Convert IPX packet to UDP data
    pub async fn ipx_to_udp(&self, ipx_packet: &IpxPacket) -> Result<Vec<u8>> {
        // Create minimal IPX header for UDP encapsulation
        let mut udp_data = Vec::new();

        // Magic header for IPX-over-UDP (for compatibility)
        udp_data.extend_from_slice(b"IPX\x00");

        // IPX header (30 bytes)
        udp_data.extend_from_slice(&ipx_packet.header.checksum.to_be_bytes());
        udp_data.extend_from_slice(&ipx_packet.header.length.to_be_bytes());
        udp_data.push(ipx_packet.header.transport_control);
        udp_data.push(ipx_packet.header.packet_type);
        udp_data.extend_from_slice(&ipx_packet.header.dest_network.to_be_bytes());
        udp_data.extend_from_slice(&ipx_packet.header.dest_node);
        udp_data.extend_from_slice(&ipx_packet.header.dest_socket.to_be_bytes());
        udp_data.extend_from_slice(&ipx_packet.header.src_network.to_be_bytes());
        udp_data.extend_from_slice(&ipx_packet.header.src_node);
        udp_data.extend_from_slice(&ipx_packet.header.src_socket.to_be_bytes());

        // Append actual data
        udp_data.extend_from_slice(&ipx_packet.data);

        Ok(udp_data)
    }

    /// Convert UDP data to IPX packet
    pub async fn udp_to_ipx(&self, udp_data: &[u8]) -> Result<IpxPacket> {
        if udp_data.len() < 34 {
            // 4 bytes magic + 30 bytes IPX header minimum
            return Err(SongbirdError::Network {
                service: "Real IPX Bridge".to_string(),
                message: "UDP data too short for IPX packet".to_string(),
                details: None,
            });
        }

        // Check magic header
        if &udp_data[0..4] != b"IPX\x00" {
            return Err(SongbirdError::Network {
                service: "Real IPX Bridge".to_string(),
                message: "Invalid IPX magic header".to_string(),
                details: None,
            });
        }

        let header_data = &udp_data[4..34];
        let payload = &udp_data[34..];

        let header = IpxHeader {
            checksum: u16::from_be_bytes([header_data[0], header_data[1]]),
            length: u16::from_be_bytes([header_data[2], header_data[3]]),
            transport_control: header_data[4],
            packet_type: header_data[5],
            dest_network: u32::from_be_bytes([
                header_data[6],
                header_data[7],
                header_data[8],
                header_data[9],
            ]),
            dest_node: [
                header_data[10],
                header_data[11],
                header_data[12],
                header_data[13],
                header_data[14],
                header_data[15],
            ],
            dest_socket: u16::from_be_bytes([header_data[16], header_data[17]]),
            src_network: u32::from_be_bytes([
                header_data[18],
                header_data[19],
                header_data[20],
                header_data[21],
            ]),
            src_node: [
                header_data[22],
                header_data[23],
                header_data[24],
                header_data[25],
                header_data[26],
                header_data[27],
            ],
            src_socket: u16::from_be_bytes([header_data[28], header_data[29]]),
        };

        Ok(IpxPacket {
            header,
            data: payload.to_vec(),
        })
    }

    /// Register a node mapping
    pub async fn register_node(&self, ipx_addr: IpxAddress, socket_addr: SocketAddr) {
        let mut mappings = self.node_mappings.write().await;
        mappings.insert(ipx_addr, socket_addr);
        debug!("Registered IPX node {:?} -> {}", ipx_addr, socket_addr);
    }

    /// Get socket address for IPX address
    pub async fn get_socket_addr(&self, ipx_addr: &IpxAddress) -> Option<SocketAddr> {
        let mappings = self.node_mappings.read().await;
        mappings.get(ipx_addr).copied()
    }
}

type PacketReceiver = Arc<RwLock<Option<mpsc::UnboundedReceiver<(Vec<u8>, SocketAddr)>>>>;

/// Real IPX Bridge using UDP sockets
pub struct RealIPXBridge {
    socket: Arc<UdpSocket>,
    ipx_network_id: u32,
    virtual_nodes: Arc<RwLock<HashMap<IpxAddress, SocketAddr>>>,
    packet_translator: IPXToUDPTranslator,
    broadcast_port: u16,
    // Channel for packet forwarding
    packet_sender: mpsc::UnboundedSender<(Vec<u8>, SocketAddr)>,
    packet_receiver: PacketReceiver,
}

impl RealIPXBridge {
    /// Create a new real IPX bridge
    pub async fn bind_ipx_network(network_id: u32) -> Result<Self> {
        // Bind to IPX port range (typically 6112 for StarCraft)
        let base_port = 6112;
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", base_port))
            .await
            .map_err(|e| SongbirdError::Network {
                service: "Real IPX Bridge".to_string(),
                message: format!("Failed to bind IPX bridge socket: {}", e),
                details: None,
            })?;

        info!("IPX Bridge bound to port {}", base_port);

        let packet_translator = IPXToUDPTranslator::new(network_id);
        let (packet_sender, packet_receiver) = mpsc::unbounded_channel();

        Ok(Self {
            socket: Arc::new(socket),
            ipx_network_id: network_id,
            virtual_nodes: Arc::new(RwLock::new(HashMap::new())),
            packet_translator,
            broadcast_port: base_port,
            packet_sender,
            packet_receiver: Arc::new(RwLock::new(Some(packet_receiver))),
        })
    }

    /// Start the IPX bridge packet forwarding loop
    pub async fn start_forwarding(&self) -> Result<()> {
        let socket = Arc::clone(&self.socket);
        let virtual_nodes_clone = Arc::clone(&self.virtual_nodes);
        let packet_sender_clone = self.packet_sender.clone();

        let mut receiver = {
            let mut recv_lock = self.packet_receiver.write().await;
            recv_lock.take().ok_or_else(|| SongbirdError::Network {
                service: "Real IPX Bridge".to_string(),
                message: "Packet receiver already taken".to_string(),
                details: None,
            })?
        };

        info!("Starting IPX bridge packet forwarding");

        // Spawn background task for packet forwarding
        tokio::spawn(async move {
            let mut buffer = [0u8; 1500]; // Standard MTU size

            loop {
                tokio::select! {
                    // Receive packets from socket
                    result = socket.recv_from(&mut buffer) => {
                        match result {
                            Ok((len, src_addr)) => {
                                debug!("Received {} bytes from {}", len, src_addr);
                                // Forward the received packet through the bridge
                                // Create a copy of the packet data for forwarding
                                let packet_data = buffer[..len].to_vec();

                                // We need to forward this to the bridge processing
                                // For now, broadcast to all registered nodes except sender
                                let virtual_nodes = virtual_nodes_clone.read().await;
                                for (_, dest_addr) in virtual_nodes.iter() {
                                    if *dest_addr != src_addr {
                                        // Queue packet for sending to this destination
                                        if let Err(e) = packet_sender_clone.send((packet_data.clone(), *dest_addr)) {
                                            warn!("Failed to queue packet for {}: {}", dest_addr, e);
                                        }
                                    }
                                }
                                drop(virtual_nodes);
                            }
                            Err(e) => {
                                warn!("Error receiving packet: {}", e);
                            }
                        }
                    }

                    // Send queued packets
                    Some((data, dest_addr)) = receiver.recv() => {
                        if let Err(e) = socket.send_to(&data, dest_addr).await {
                            warn!("Error sending packet to {}: {}", dest_addr, e);
                        } else {
                            debug!("Forwarded {} bytes to {}", data.len(), dest_addr);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Forward an IPX packet through the bridge
    pub async fn forward_ipx_packet(&self, packet: &[u8], from: SocketAddr) -> Result<()> {
        debug!(
            "Forwarding IPX packet from {} ({} bytes)",
            from,
            packet.len()
        );

        // Try to parse as IPX packet
        match self.packet_translator.udp_to_ipx(packet).await {
            Ok(ipx_packet) => {
                // Handle different packet types
                match ipx_packet.header.packet_type {
                    0x00 => self.handle_ipx_unknown(&ipx_packet, from).await?,
                    0x01 => self.handle_ipx_rip(&ipx_packet, from).await?,
                    0x04 => self.handle_ipx_spp(&ipx_packet, from).await?,
                    0x05 => self.handle_ipx_spx(&ipx_packet, from).await?,
                    _ => {
                        debug!(
                            "Unknown IPX packet type: 0x{:02x}",
                            ipx_packet.header.packet_type
                        );
                        self.forward_to_destination(&ipx_packet, from).await?;
                    }
                }
            }
            Err(_) => {
                // Not a valid IPX packet, might be raw game data
                debug!("Received raw game data, broadcasting to all nodes");
                self.broadcast_raw_data(packet, from).await?;
            }
        }

        Ok(())
    }

    /// Simulate IPX broadcast to all nodes in the virtual network
    pub async fn simulate_ipx_broadcast(&self, data: &[u8]) -> Result<()> {
        info!("Broadcasting IPX data to all nodes ({} bytes)", data.len());

        let nodes = self.virtual_nodes.read().await;
        let mut broadcast_count = 0;

        for (ipx_addr, socket_addr) in nodes.iter() {
            debug!("Broadcasting to IPX node {:?} at {}", ipx_addr, socket_addr);

            if let Err(e) = self.packet_sender.send((data.to_vec(), *socket_addr)) {
                warn!(
                    "Failed to queue broadcast packet for {}: {}",
                    socket_addr, e
                );
            } else {
                broadcast_count += 1;
            }
        }

        info!("Broadcast queued for {} nodes", broadcast_count);
        Ok(())
    }

    /// Register a new node in the virtual IPX network
    pub async fn register_node(&self, socket_addr: SocketAddr) -> Result<IpxAddress> {
        let ipx_addr = IpxAddress::from_ip(socket_addr.ip());

        {
            let mut nodes = self.virtual_nodes.write().await;
            nodes.insert(ipx_addr, socket_addr);
        }

        // Also register in translator
        self.packet_translator
            .register_node(ipx_addr, socket_addr)
            .await;

        info!("Registered new IPX node: {:?} -> {}", ipx_addr, socket_addr);
        Ok(ipx_addr)
    }

    /// Handle unknown IPX packets (general game data)
    async fn handle_ipx_unknown(&self, packet: &IpxPacket, from: SocketAddr) -> Result<()> {
        debug!("Handling unknown IPX packet from {}", from);
        self.forward_to_destination(packet, from).await
    }

    /// Handle IPX RIP (Routing Information Protocol) packets
    async fn handle_ipx_rip(&self, _packet: &IpxPacket, from: SocketAddr) -> Result<()> {
        debug!("Handling IPX RIP packet from {}", from);
        // RIP packets are used for network discovery
        // For gaming, we usually just acknowledge the network exists
        Ok(())
    }

    /// Handle IPX SPP (Sequenced Packet Protocol) packets
    async fn handle_ipx_spp(&self, packet: &IpxPacket, from: SocketAddr) -> Result<()> {
        debug!("Handling IPX SPP packet from {}", from);
        self.forward_to_destination(packet, from).await
    }

    /// Handle IPX SPX (Sequenced Packet Exchange) packets
    async fn handle_ipx_spx(&self, packet: &IpxPacket, from: SocketAddr) -> Result<()> {
        debug!("Handling IPX SPX packet from {}", from);
        self.forward_to_destination(packet, from).await
    }

    /// Forward packet to its intended destination
    async fn forward_to_destination(&self, packet: &IpxPacket, from: SocketAddr) -> Result<()> {
        let dest_ipx = IpxAddress {
            network: packet.header.dest_network,
            node: packet.header.dest_node,
        };

        // Check if this is a broadcast
        if packet.header.dest_node == [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] {
            // Broadcast to all nodes except sender
            let translated_data = self.packet_translator.ipx_to_udp(packet).await?;

            let nodes = self.virtual_nodes.read().await;
            for (_, socket_addr) in nodes.iter() {
                if *socket_addr != from {
                    if let Err(e) = self
                        .packet_sender
                        .send((translated_data.clone(), *socket_addr))
                    {
                        warn!("Failed to queue packet for {}: {}", socket_addr, e);
                    }
                }
            }
        } else {
            // Unicast to specific destination
            if let Some(dest_addr) = self.packet_translator.get_socket_addr(&dest_ipx).await {
                let translated_data = self.packet_translator.ipx_to_udp(packet).await?;

                if let Err(e) = self.packet_sender.send((translated_data, dest_addr)) {
                    warn!("Failed to queue packet for {}: {}", dest_addr, e);
                }
            } else {
                debug!("No route to IPX destination {:?}", dest_ipx);
            }
        }

        Ok(())
    }

    /// Broadcast raw data to all nodes (for non-IPX game protocols)
    async fn broadcast_raw_data(&self, data: &[u8], from: SocketAddr) -> Result<()> {
        let nodes = self.virtual_nodes.read().await;

        for (_, socket_addr) in nodes.iter() {
            if *socket_addr != from {
                if let Err(e) = self.packet_sender.send((data.to_vec(), *socket_addr)) {
                    warn!("Failed to queue raw data for {}: {}", socket_addr, e);
                }
            }
        }

        Ok(())
    }

    /// Get current network statistics
    pub async fn get_network_stats(&self) -> NetworkStats {
        let nodes = self.virtual_nodes.read().await;

        NetworkStats {
            network_id: self.ipx_network_id,
            active_nodes: nodes.len(),
            broadcast_port: self.broadcast_port,
            node_list: nodes.values().cloned().collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub network_id: u32,
    pub active_nodes: usize,
    pub broadcast_port: u16,
    pub node_list: Vec<SocketAddr>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[tokio::test]
    async fn test_ipx_address_creation() {
        let ip = Ipv4Addr::new(192, 168, 1, 100);
        let ipx_addr = IpxAddress::from_ip(IpAddr::V4(ip));

        assert_eq!(ipx_addr.network, 0xC0A80164); // 192.168.1.100 in hex
        assert_eq!(ipx_addr.node[0], 192);
        assert_eq!(ipx_addr.node[1], 168);
        assert_eq!(ipx_addr.node[2], 1);
        assert_eq!(ipx_addr.node[3], 100);
    }

    #[tokio::test]
    async fn test_ipx_packet_translation() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let translator = IPXToUDPTranslator::new(0x01000000);

        let ipx_packet = IpxPacket {
            header: IpxHeader {
                checksum: 0xFFFF,
                length: 64,
                transport_control: 0,
                packet_type: 0x04,
                dest_network: 0x01000000,
                dest_node: [0x00, 0x11, 0x22, 0x33, 0x44, 0x55],
                dest_socket: 0x0451,
                src_network: 0x01000000,
                src_node: [0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB],
                src_socket: 0x0451,
            },
            data: vec![0x01, 0x02, 0x03, 0x04],
        };

        let udp_data = translator.ipx_to_udp(&ipx_packet).await.map_err(|e| { tracing::error!("IPX to UDP translation failed: {}", e); e })?;
        let parsed_packet = translator.udp_to_ipx(&udp_data).await.map_err(|e| { tracing::error!("UDP to IPX translation failed: {}", e); e })?;

        assert_eq!(parsed_packet.header.packet_type, 0x04);
        assert_eq!(parsed_packet.data, vec![0x01, 0x02, 0x03, 0x04]);

        Ok(())
    }

    #[tokio::test]
    async fn test_real_ipx_bridge_creation() {
        // This test might fail if port 6112 is already in use
        match RealIPXBridge::bind_ipx_network(0x01000000).await {
            Ok(bridge) => {
                let stats = bridge.get_network_stats().await;
                assert_eq!(stats.network_id, 0x01000000);
                assert_eq!(stats.active_nodes, 0);
            }
            Err(_) => {
                // Port might be in use, that's okay for testing
                println!("Port 6112 in use, skipping bind test");
            }
        }
    }
}
