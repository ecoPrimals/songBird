use byteorder::{NetworkEndian, ReadBytesExt};
use songbird_errors::{Result, SongbirdError};
use std::io::Cursor;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::UdpSocket;
use tokio::time::{timeout, Duration};
use tracing::{debug, warn};

/// STUN message types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StunMessageType {
    Request = 0x0001,
    Response = 0x0101,
    ErrorResponse = 0x0111,
}

/// STUN attribute types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StunAttributeType {
    Mapped = 0x0001,
    Source = 0x0004,
    Changed = 0x0005,
    XorMapped = 0x0020,
}

/// STUN message structure
#[derive(Debug, Clone)]
pub struct StunMessage {
    pub message_type: StunMessageType,
    pub transaction_id: [u8; 12],
    pub attributes: Vec<StunAttribute>,
}

/// STUN attribute structure
#[derive(Debug, Clone)]
pub struct StunAttribute {
    pub attribute_type: StunAttributeType,
    pub value: Vec<u8>,
}

/// STUN client for NAT traversal
pub struct StunClient {
    socket: UdpSocket,
    timeout_duration: Duration,
}

impl StunClient {
    /// Create a new STUN client
    pub async fn new(local_addr: SocketAddr) -> Result<Self> {
        let socket = UdpSocket::bind(local_addr).await.map_err(|e| {
            SongbirdError::network_error(format!("STUN Client - Failed to bind socket: {}", e))
        })?;

        Ok(Self {
            socket,
            timeout_duration: Duration::from_secs(5),
        })
    }

    /// Perform STUN binding request to discover external address
    pub async fn binding_request(&self, stun_server: SocketAddr) -> Result<SocketAddr> {
        let transaction_id = self.generate_transaction_id();
        let message = StunMessage {
            message_type: StunMessageType::Request,
            transaction_id,
            attributes: vec![],
        };

        let request_data = self.encode_message(&message)?;

        // Send request
        self.socket
            .send_to(&request_data, stun_server)
            .await
            .map_err(|e| {
                SongbirdError::network_error(format!(
                    "STUN Client - Failed to send request to {}: {}",
                    stun_server, e
                ))
            })?;

        // Receive response
        let mut buffer = vec![0u8; 1024];
        let len = match timeout(self.timeout_duration, self.socket.recv_from(&mut buffer)).await {
            Ok(Ok((len, _))) => {
                if len < 20 {
                    return Err(SongbirdError::network_error(format!(
                        "STUN Client - Response too short from {}",
                        stun_server
                    )));
                }
                len
            }
            Ok(Err(e)) => {
                return Err(SongbirdError::network_error(format!(
                    "STUN Client - Failed to receive response from {}: {}",
                    stun_server, e
                )));
            }
            Err(_) => {
                return Err(SongbirdError::network_error(format!(
                    "STUN Client - Request timed out to {}",
                    stun_server
                )));
            }
        };

        let response = self.decode_message(&buffer[..len])?;

        if response.transaction_id != transaction_id {
            return Err(SongbirdError::network_error(format!(
                "STUN Client - Transaction ID mismatch in STUN response from {}",
                stun_server
            )));
        }

        self.extract_mapped_address(&response)
    }

    /// Generate a random transaction ID
    fn generate_transaction_id(&self) -> [u8; 12] {
        let mut transaction_id = [0u8; 12];
        // In a real implementation, this would use proper random generation
        for (i, byte) in transaction_id.iter_mut().enumerate() {
            *byte = (i as u8).wrapping_mul(17).wrapping_add(42);
        }
        transaction_id
    }

    /// Encode STUN message to bytes
    fn encode_message(&self, message: &StunMessage) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        // Message type (2 bytes)
        buffer.extend_from_slice(&(message.message_type as u16).to_be_bytes());

        // Message length (2 bytes) - calculate attributes length
        let attrs_len: u16 = message
            .attributes
            .iter()
            .map(|attr| 4 + attr.value.len() as u16)
            .sum();
        buffer.extend_from_slice(&attrs_len.to_be_bytes());

        // Magic cookie (4 bytes)
        buffer.extend_from_slice(&0x2112A442u32.to_be_bytes());

        // Transaction ID (12 bytes)
        buffer.extend_from_slice(&message.transaction_id);

        // Attributes
        for attr in &message.attributes {
            buffer.extend_from_slice(&(attr.attribute_type as u16).to_be_bytes());
            buffer.extend_from_slice(&(attr.value.len() as u16).to_be_bytes());
            buffer.extend_from_slice(&attr.value);

            // Pad to 4-byte boundary
            while buffer.len() % 4 != 0 {
                buffer.push(0);
            }
        }

        Ok(buffer)
    }

    /// Decode STUN message from bytes
    fn decode_message(&self, data: &[u8]) -> Result<StunMessage> {
        if data.len() < 20 {
            return Err(SongbirdError::network_error(
                "STUN Client - STUN message too short",
            ));
        }

        let _cursor = Cursor::new(data);

        // Message type
        let msg_type = u16::from_be_bytes([data[0], data[1]]);

        if msg_type != 0x0101 {
            return Err(SongbirdError::network_error(format!(
                "STUN Client - Unknown STUN message type: {}",
                msg_type
            )));
        }

        let msg_length = u16::from_be_bytes([data[2], data[3]]);

        let _magic_cookie = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);

        let transaction_id = &data[8..20];
        if transaction_id.len() != 12 {
            return Err(SongbirdError::network_error(format!(
                "STUN Client - Failed to read transaction ID: invalid length"
            )));
        }

        // Attributes
        let mut attributes = Vec::new();
        let mut remaining = msg_length as usize;
        let mut i = 20; // Start after the header (20 bytes)

        while remaining > 0 && i + 4 <= data.len() {
            if remaining < 4 {
                break;
            }

            let attr_type = u16::from_be_bytes([data[i], data[i + 1]]);
            let attr_len = u16::from_be_bytes([data[i + 2], data[i + 3]]);

            let attribute_type = match attr_type {
                0x0001 => StunAttributeType::Mapped,
                0x0004 => StunAttributeType::Source,
                0x0005 => StunAttributeType::Changed,
                0x0020 => StunAttributeType::XorMapped,
                _ => {
                    // Skip unknown attributes
                    let skip_len = (attr_len as usize + 3) & !3; // Round up to 4-byte boundary
                    i += 4 + skip_len;
                    remaining = remaining.saturating_sub(4 + skip_len);
                    continue;
                }
            };

            if i + 4 + attr_len as usize > data.len() {
                break;
            }

            let attr_value = &data[i + 4..i + 4 + attr_len as usize];

            let attribute = StunAttribute {
                attribute_type,
                value: attr_value.to_vec(),
            };

            attributes.push(attribute);

            // Move to next attribute, respecting padding
            let padded_len = (attr_len as usize + 3) & !3;
            i += 4 + padded_len;
            remaining = remaining.saturating_sub(4 + padded_len);
        }

        Ok(StunMessage {
            message_type: StunMessageType::Response, // Assuming it's a response based on msg_type
            transaction_id: transaction_id.try_into().unwrap(), // Convert slice to array
            attributes,
        })
    }

    /// Extract mapped address from STUN response
    fn extract_mapped_address(&self, response: &StunMessage) -> Result<SocketAddr> {
        // First try XOR-MAPPED-ADDRESS, then fall back to MAPPED-ADDRESS
        for attr_type in [StunAttributeType::XorMapped, StunAttributeType::Mapped] {
            if let Some(attr) = response
                .attributes
                .iter()
                .find(|a| a.attribute_type == attr_type)
            {
                if attr.value.len() < 8 {
                    continue;
                }

                let mut cursor = Cursor::new(&attr.value);

                // Skip reserved byte
                let _reserved = cursor.read_u8().map_err(|e| {
                    SongbirdError::network_error(format!(
                        "STUN Client - Failed to read reserved byte: {}",
                        e
                    ))
                })?;

                // Address family
                let family = cursor.read_u8().map_err(|e| {
                    SongbirdError::network_error(format!(
                        "STUN Client - Failed to read address family: {}",
                        e
                    ))
                })?;

                if family != 0x01 {
                    // Only IPv4 supported for now
                    continue;
                }

                // Port
                let mut port = cursor.read_u16::<NetworkEndian>().map_err(|e| {
                    SongbirdError::network_error(format!(
                        "STUN Client - Failed to read port: {}",
                        e
                    ))
                })?;

                // Address
                let mut addr_bytes = [0u8; 4];
                std::io::Read::read_exact(&mut cursor, &mut addr_bytes).map_err(|e| {
                    SongbirdError::network_error(format!(
                        "STUN Client - Failed to read address bytes: {}",
                        e
                    ))
                })?;

                // Apply XOR for XOR-MAPPED-ADDRESS
                if attr_type == StunAttributeType::XorMapped {
                    let magic_cookie = 0x2112A442u32.to_be_bytes();
                    port ^= (magic_cookie[0] as u16) << 8 | magic_cookie[1] as u16;
                    for i in 0..4 {
                        addr_bytes[i] ^= magic_cookie[i];
                    }
                }

                let ip = IpAddr::V4(Ipv4Addr::new(
                    addr_bytes[0],
                    addr_bytes[1],
                    addr_bytes[2],
                    addr_bytes[3],
                ));

                return Ok(SocketAddr::new(ip, port));
            }
        }

        Err(SongbirdError::network_error(format!(
            "STUN Client - No mapped address found in STUN response from {}",
            "unknown_server"
        )))
    }
}

/// STUN server for testing and development
pub struct StunServer {
    socket: UdpSocket,
}

impl StunServer {
    /// Create a new STUN server
    pub async fn new(bind_addr: SocketAddr) -> Result<Self> {
        let socket = UdpSocket::bind(bind_addr).await.map_err(|e| {
            SongbirdError::network_error(format!("STUN Server - Failed to bind socket: {}", e))
        })?;

        Ok(Self { socket })
    }

    /// Run the STUN server
    pub async fn run(&self) -> Result<()> {
        let mut buffer = vec![0u8; 1024];

        loop {
            let (size, client_addr) = self.socket.recv_from(&mut buffer).await.map_err(|e| {
                SongbirdError::network_error(format!(
                    "STUN Server - Failed to receive request: {}",
                    e
                ))
            })?;

            if let Err(e) = self.handle_request(&buffer[..size], client_addr).await {
                warn!("Failed to handle STUN request: {}", e);
            }
        }
    }

    /// Handle a STUN request
    async fn handle_request(&self, data: &[u8], client_addr: SocketAddr) -> Result<()> {
        debug!("Received STUN request from {}", client_addr);

        // Simple implementation - just echo back the client address
        if data.len() < 20 {
            return Err(SongbirdError::network_error(
                "STUN Server - STUN request too short",
            ));
        }

        // Extract transaction ID
        let transaction_id = &data[8..20];

        // Create response with mapped address
        let response = self.create_binding_response(transaction_id, client_addr)?;

        self.socket
            .send_to(&response, client_addr)
            .await
            .map_err(|e| {
                SongbirdError::network_error(format!(
                    "STUN Server - Failed to send response: {}",
                    e
                ))
            })?;

        Ok(())
    }

    /// Create a binding response
    fn create_binding_response(
        &self,
        transaction_id: &[u8],
        client_addr: SocketAddr,
    ) -> Result<Vec<u8>> {
        let mut response = Vec::new();

        // Message type (Binding Response)
        response.extend_from_slice(&0x0101u16.to_be_bytes());

        // Message length (will be updated later)
        let length_pos = response.len();
        response.extend_from_slice(&0u16.to_be_bytes());

        // Magic cookie
        response.extend_from_slice(&0x2112A442u32.to_be_bytes());

        // Transaction ID
        response.extend_from_slice(transaction_id);

        // Add MAPPED-ADDRESS attribute
        let attr_start = response.len();
        response.extend_from_slice(&0x0001u16.to_be_bytes()); // MAPPED-ADDRESS
        response.extend_from_slice(&0x0008u16.to_be_bytes()); // Length
        response.push(0x00); // Reserved
        response.push(0x01); // IPv4
        response.extend_from_slice(&client_addr.port().to_be_bytes());

        match client_addr.ip() {
            IpAddr::V4(ipv4) => {
                response.extend_from_slice(&ipv4.octets());
            }
            IpAddr::V6(_) => {
                return Err(SongbirdError::network_error(
                    "STUN Server - IPv6 not supported",
                ));
            }
        }

        // Update message length
        let attr_length = response.len() - attr_start;
        let total_length = (attr_length as u16).to_be_bytes();
        response[length_pos..length_pos + 2].copy_from_slice(&total_length);

        Ok(response)
    }
}
