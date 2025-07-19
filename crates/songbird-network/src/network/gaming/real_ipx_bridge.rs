//! Real IPX Network Bridge Implementation
//!
//! This module implements actual UDP socket-based IPX network emulation
//! for legacy games like StarCraft, Age of Empires I, Command & Conquer

use serde::{Deserialize, Serialize};
use songbird_errors::{NetworkError, Result, SongbirdError};
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

/// IPX packet structure - now uses borrowed data where possible
#[derive(Debug)]
pub struct IpxPacket<'a> {
    pub header: IpxHeader,
    pub data: &'a [u8], // Zero-copy: use slice instead of Vec
}

/// Owned IPX packet for when we need to store the data
#[derive(Debug, Clone)]
pub struct OwnedIpxPacket {
    pub header: IpxHeader,
    pub data: Vec<u8>,
}

impl<'a> IpxPacket<'a> {
    /// Convert to owned packet when necessary
    pub fn to_owned(&self) -> OwnedIpxPacket {
        OwnedIpxPacket {
            header: self.header.clone(),
            data: self.data.to_vec(),
        }
    }
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

/// Buffer pool for zero-copy packet operations
pub struct PacketBufferPool {
    available_buffers: Vec<Vec<u8>>,
    buffer_size: usize,
    max_buffers: usize,
}

impl PacketBufferPool {
    pub fn new(buffer_size: usize, initial_count: usize) -> Self {
        let mut buffers = Vec::with_capacity(initial_count);
        for _ in 0..initial_count {
            buffers.push(vec![0u8; buffer_size]);
        }

        Self {
            available_buffers: buffers,
            buffer_size,
            max_buffers: initial_count * 2,
        }
    }

    pub fn get_buffer(&mut self) -> Vec<u8> {
        self.available_buffers
            .pop()
            .unwrap_or_else(|| vec![0u8; self.buffer_size])
    }

    pub fn return_buffer(&mut self, mut buffer: Vec<u8>) {
        if self.available_buffers.len() < self.max_buffers {
            buffer.clear();
            buffer.resize(self.buffer_size, 0);
            self.available_buffers.push(buffer);
        }
    }
}

/// Type alias for the complex packet receiver type
type PacketReceiver = Arc<RwLock<Option<mpsc::Receiver<(Vec<u8>, SocketAddr)>>>>;

/// Real IPX bridge implementation with zero-copy optimizations
pub struct RealIpxBridge {
    socket: Arc<UdpSocket>,
    virtual_nodes: Arc<RwLock<HashMap<IpxAddress, SocketAddr>>>,
    packet_translator: IpxPacketTranslator,
    packet_sender: mpsc::Sender<(Vec<u8>, SocketAddr)>,
    packet_receiver: PacketReceiver,
    buffer_pool: Arc<RwLock<PacketBufferPool>>,
}

impl RealIpxBridge {
    pub async fn new(bind_address: SocketAddr, buffer_pool_size: usize) -> Result<Self> {
        let socket = UdpSocket::bind(bind_address).await.map_err(|e| {
            SongbirdError::Network(Box::new(NetworkError {
                message: format!(
                    "Real IPX Bridge - Failed to bind to address {}: {}",
                    bind_address, e
                ),
                endpoint: Some(bind_address.to_string()),
                port: Some(bind_address.port()),
                protocol: None,
            }))
        })?;

        info!("Real IPX Bridge bound to: {}", bind_address);

        let (packet_sender, packet_receiver) = mpsc::channel(1000);
        let buffer_pool = PacketBufferPool::new(1500, buffer_pool_size);

        Ok(Self {
            socket: Arc::new(socket),
            virtual_nodes: Arc::new(RwLock::new(HashMap::new())),
            packet_translator: IpxPacketTranslator::new(),
            packet_sender,
            packet_receiver: Arc::new(RwLock::new(Some(packet_receiver))),
            buffer_pool: Arc::new(RwLock::new(buffer_pool)),
        })
    }

    /// Register a virtual IPX node
    pub async fn register_virtual_node(
        &self,
        ipx_addr: IpxAddress,
        socket_addr: SocketAddr,
    ) -> Result<()> {
        let mut nodes = self.virtual_nodes.write().await;
        nodes.insert(ipx_addr, socket_addr);
        info!(
            "Registered virtual IPX node: {:?} -> {}",
            ipx_addr, socket_addr
        );
        Ok(())
    }

    /// Start packet forwarding with zero-copy optimizations
    pub async fn start_forwarding(&self) -> Result<()> {
        let socket = Arc::clone(&self.socket);
        let virtual_nodes_clone = Arc::clone(&self.virtual_nodes);
        let packet_sender_clone = self.packet_sender.clone();
        let buffer_pool_clone = Arc::clone(&self.buffer_pool);

        let mut receiver = {
            let mut recv_lock = self.packet_receiver.write().await;
            recv_lock.take().ok_or_else(|| {
                SongbirdError::Network(Box::new(NetworkError {
                    message: "Packet receiver already taken".to_string(),
                    endpoint: None,
                    port: None,
                    protocol: None,
                }))
            })?
        };

        info!("Starting IPX bridge packet forwarding");

        // Spawn background task for packet forwarding
        tokio::spawn(async move {
            // Use buffer pool for receive operations
            let mut current_buffer = {
                let mut pool = buffer_pool_clone.write().await;
                pool.get_buffer()
            };

            loop {
                tokio::select! {
                    // Receive packets from socket with zero-copy buffer reuse
                    result = socket.recv_from(&mut current_buffer) => {
                        match result {
                            Ok((len, src_addr)) => {
                                debug!("Received {} bytes from {}", len, src_addr);

                                // Process packet in-place without copying
                                let packet_data = &current_buffer[..len];

                                // Forward to all registered nodes except sender
                                let virtual_nodes = virtual_nodes_clone.read().await;
                                for (_, dest_addr) in virtual_nodes.iter() {
                                    if *dest_addr != src_addr {
                                        // Only copy when we need to send to multiple destinations
                                        if let Err(e) = packet_sender_clone.send((packet_data.to_vec(), *dest_addr)).await {
                                            warn!("Failed to queue packet for {}: {}", dest_addr, e);
                                        }
                                    }
                                }
                                drop(virtual_nodes);

                                // Get a new buffer for next receive
                                let mut pool = buffer_pool_clone.write().await;
                                let old_buffer = std::mem::replace(&mut current_buffer, pool.get_buffer());
                                pool.return_buffer(old_buffer);
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

    /// Forward an IPX packet through the bridge with zero-copy optimizations
    pub async fn forward_ipx_packet(&self, packet: &[u8], from: SocketAddr) -> Result<()> {
        debug!(
            "Forwarding IPX packet from {} ({} bytes)",
            from,
            packet.len()
        );

        // Try to parse as IPX packet without copying
        match self.packet_translator.parse_ipx_packet(packet) {
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

    /// Handle IPX packet forwarding with zero-copy where possible
    async fn forward_to_destination(&self, packet: &IpxPacket<'_>, from: SocketAddr) -> Result<()> {
        let dest_ipx = IpxAddress {
            network: packet.header.dest_network,
            node: packet.header.dest_node,
        };

        if packet.header.dest_node == [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] {
            // Broadcast to all nodes except sender
            let translated_data = self.packet_translator.ipx_to_udp_borrowed(packet)?;

            let nodes = self.virtual_nodes.read().await;
            for (_, socket_addr) in nodes.iter() {
                if *socket_addr != from {
                    if let Err(e) = self
                        .packet_sender
                        .send((translated_data.clone(), *socket_addr))
                        .await
                    {
                        warn!("Failed to queue packet for {}: {}", socket_addr, e);
                    }
                }
            }
        } else {
            // Unicast to specific destination
            if let Some(dest_addr) = self.packet_translator.get_socket_addr(&dest_ipx).await {
                let translated_data = self.packet_translator.ipx_to_udp_borrowed(packet)?;

                if let Err(e) = self.packet_sender.send((translated_data, dest_addr)).await {
                    warn!("Failed to queue packet for {}: {}", dest_addr, e);
                }
            } else {
                debug!("No route to IPX destination {:?}", dest_ipx);
            }
        }

        Ok(())
    }

    /// Handle unknown IPX packets (general game data)
    async fn handle_ipx_unknown(&self, packet: &IpxPacket<'_>, from: SocketAddr) -> Result<()> {
        debug!("Handling unknown IPX packet from {}", from);
        self.forward_to_destination(packet, from).await
    }

    /// Handle IPX RIP (Routing Information Protocol) packets
    async fn handle_ipx_rip(&self, _packet: &IpxPacket<'_>, from: SocketAddr) -> Result<()> {
        debug!("Handling IPX RIP packet from {}", from);
        // RIP packets are used for network discovery
        Ok(())
    }

    /// Handle IPX SPP (Sequenced Packet Protocol) packets
    async fn handle_ipx_spp(&self, packet: &IpxPacket<'_>, from: SocketAddr) -> Result<()> {
        debug!("Handling IPX SPP packet from {}", from);
        self.forward_to_destination(packet, from).await
    }

    /// Handle IPX SPX (Sequenced Packet Exchange) packets
    async fn handle_ipx_spx(&self, packet: &IpxPacket<'_>, from: SocketAddr) -> Result<()> {
        debug!("Handling IPX SPX packet from {}", from);
        self.forward_to_destination(packet, from).await
    }

    /// Broadcast raw data to all nodes with minimal copying
    async fn broadcast_raw_data(&self, data: &[u8], from: SocketAddr) -> Result<()> {
        let nodes = self.virtual_nodes.read().await;
        let data_vec = data.to_vec(); // Only clone once for all destinations

        for (_, socket_addr) in nodes.iter() {
            if *socket_addr != from {
                if let Err(e) = self
                    .packet_sender
                    .send((data_vec.clone(), *socket_addr))
                    .await
                {
                    warn!("Failed to queue raw data for {}: {}", socket_addr, e);
                }
            }
        }

        Ok(())
    }

    /// Get current network statistics
    pub async fn get_stats(&self) -> Result<serde_json::Value> {
        let nodes = self.virtual_nodes.read().await;
        let stats = serde_json::json!({
            "virtual_nodes": nodes.len(),
            "bridge_status": "active"
        });
        Ok(stats)
    }
}

/// IPX packet translator with zero-copy optimizations
pub struct IpxPacketTranslator {
    socket_mappings: Arc<RwLock<HashMap<IpxAddress, SocketAddr>>>,
}

impl IpxPacketTranslator {
    pub fn new() -> Self {
        Self {
            socket_mappings: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Parse IPX packet from buffer without copying
    pub fn parse_ipx_packet<'a>(&self, data: &'a [u8]) -> Result<IpxPacket<'a>> {
        if data.len() < 30 {
            return Err(SongbirdError::Network(Box::new(NetworkError {
                message: format!("IPX Parser - Need at least 30 bytes, got {}", data.len()),
                endpoint: None,
                port: None,
                protocol: Some("IPX".to_string()),
            })));
        }

        let header = IpxHeader {
            checksum: u16::from_be_bytes([data[0], data[1]]),
            length: u16::from_be_bytes([data[2], data[3]]),
            transport_control: data[4],
            packet_type: data[5],
            dest_network: u32::from_be_bytes([data[6], data[7], data[8], data[9]]),
            dest_node: [data[10], data[11], data[12], data[13], data[14], data[15]],
            dest_socket: u16::from_be_bytes([data[16], data[17]]),
            src_network: u32::from_be_bytes([data[18], data[19], data[20], data[21]]),
            src_node: [data[22], data[23], data[24], data[25], data[26], data[27]],
            src_socket: u16::from_be_bytes([data[28], data[29]]),
        };

        Ok(IpxPacket {
            header,
            data: &data[30..], // Zero-copy: use slice
        })
    }

    /// Convert IPX packet to UDP using borrowed data
    pub fn ipx_to_udp_borrowed(&self, packet: &IpxPacket<'_>) -> Result<Vec<u8>> {
        let mut udp_data = Vec::with_capacity(packet.data.len() + 8);

        // Add minimal UDP header simulation
        udp_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // Source port placeholder
        udp_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // Dest port placeholder

        // Add IPX data
        udp_data.extend_from_slice(packet.data);

        Ok(udp_data)
    }

    /// Get socket address for IPX address
    pub async fn get_socket_addr(&self, ipx_addr: &IpxAddress) -> Option<SocketAddr> {
        self.socket_mappings.read().await.get(ipx_addr).copied()
    }

    /// Legacy method for backwards compatibility
    pub async fn udp_to_ipx(&self, data: &[u8]) -> Result<OwnedIpxPacket> {
        let packet = self.parse_ipx_packet(data)?;
        Ok(packet.to_owned())
    }

    /// Legacy method for backwards compatibility
    pub async fn ipx_to_udp(&self, packet: &OwnedIpxPacket) -> Result<Vec<u8>> {
        let borrowed_packet = IpxPacket {
            header: packet.header.clone(),
            data: &packet.data,
        };
        self.ipx_to_udp_borrowed(&borrowed_packet)
    }
}

impl Default for IpxPacketTranslator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    #[tokio::test]
    async fn test_ipx_bridge_creation() {
        let bind_addr: SocketAddr = format!(
            "{}:0",
            songbird_config::config::constants::network::DEFAULT_BIND_ADDRESS
        )
        .parse()
        .expect("Default IPX bridge bind address should be valid");
        let bridge = RealIpxBridge::new(bind_addr, 10).await;
        assert!(bridge.is_ok());
    }

    #[tokio::test]
    async fn test_ipx_packet_parsing() {
        let translator = IpxPacketTranslator::new();

        // Create test packet data
        let test_data = vec![
            0x00, 0x00, // checksum
            0x00, 0x20, // length
            0x00, // transport control
            0x00, // packet type
            0x00, 0x00, 0x00, 0x01, // dest network
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // dest node
            0x04, 0x51, // dest socket
            0x00, 0x00, 0x00, 0x01, // src network
            0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // src node
            0x04, 0x51, // src socket
            0x01, 0x02, 0x03, 0x04, // data
        ];

        let packet = translator
            .parse_ipx_packet(&test_data)
            .expect("Test IPX packet should be valid");
        assert_eq!(packet.header.packet_type, 0x00);
        assert_eq!(packet.data, &[0x01, 0x02, 0x03, 0x04]);
    }
}
