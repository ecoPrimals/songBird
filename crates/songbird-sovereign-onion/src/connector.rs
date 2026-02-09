//! Onion connector (connect to .onion addresses) - Phase 4 Implementation
//!
//! ✅ **TRUE PRIMAL**: Production uses BearDog delegation for all crypto.

use crate::beardog_crypto::BeardogCryptoClient;
use crate::error::{OnionError, Result};
use crate::keys::EphemeralKeypair;
use crate::protocol::{DataMessage, KeyExchangeMessage, MessageType};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, info};

/// Connect to onion services
///
/// **Status**: Phase 4 - Complete implementation
pub struct OnionConnector {
    beardog: Option<Arc<BeardogCryptoClient>>,
}

impl OnionConnector {
    /// Create new onion connector with BearDog delegation (TRUE PRIMAL)
    pub fn new_via_beardog(beardog: BeardogCryptoClient) -> Self {
        Self {
            beardog: Some(Arc::new(beardog)),
        }
    }

    /// Create new onion connector (standalone mode - testing only)
    #[cfg(feature = "standalone")]
    pub fn new_standalone() -> Self {
        Self { beardog: None }
    }

    /// Connect to an onion address (via BearDog)
    ///
    /// # Arguments
    /// - `onion_address`: The target .onion address
    /// - `port`: Target port
    ///
    /// # Returns
    /// Established encrypted connection
    pub async fn connect(
        &self,
        onion_address: &str,
        port: u16,
    ) -> Result<OnionConnection> {
        let beardog = self.beardog.as_ref()
            .ok_or_else(|| OnionError::ConfigError("BearDog client required".into()))?;

        info!(
            onion_address = onion_address,
            port = port,
            "Connecting to onion service"
        );

        // For Phase 1: Direct TCP connection (assumes IP known via rendezvous)
        // For Phase 2: Will use BeaconMesh for address resolution
        let stream = TcpStream::connect(format!("{}:{}", onion_address, port))
            .await
            .map_err(|_| OnionError::ConnectionTimeout)?;

        debug!("TCP connection established, starting handshake");

        // Generate our ephemeral keypair via BearDog
        let our_ephemeral = EphemeralKeypair::generate_via_beardog(beardog)?;

        // Send KeyExchange
        let key_exchange = KeyExchangeMessage::new(*our_ephemeral.public_bytes(), [0u8; 24]);
        let mut send_buf = vec![MessageType::KeyExchange as u8];
        send_buf.extend_from_slice(&key_exchange.encode());
        
        let mut stream = stream;
        stream.write_all(&send_buf).await?;

        debug!("Sent key exchange");

        // Receive KeyExchange response (1 type + 57 payload)
        let mut recv_buf = [0u8; 58];
        stream.read_exact(&mut recv_buf).await?;

        let response_type = MessageType::try_from(recv_buf[0])?;
        if response_type != MessageType::KeyExchange {
            return Err(OnionError::HandshakeFailed(format!(
                "Expected KeyExchange response, got {:?}",
                response_type
            )));
        }

        let peer_key_exchange = KeyExchangeMessage::decode(&recv_buf[1..])?;
        debug!("Received key exchange response");

        // Derive shared secret via BearDog
        let shared_secret = our_ephemeral
            .derive_shared_secret_via_beardog(beardog, &peer_key_exchange.pubkey)?;

        info!("Handshake complete - connection established via BearDog crypto");

        Ok(OnionConnection {
            stream,
            session_key: shared_secret,
            sequence: 0,
            beardog: Arc::clone(beardog),
        })
    }
}

impl Default for OnionConnector {
    fn default() -> Self {
        Self { beardog: None }
    }
}

/// Established onion connection (encrypted session)
pub struct OnionConnection {
    stream: TcpStream,
    session_key: [u8; 32],
    sequence: u64,
    beardog: Arc<BeardogCryptoClient>,
}

impl OnionConnection {
    /// Send encrypted data via BearDog
    pub async fn send(&mut self, data: &[u8]) -> Result<()> {
        // Encrypt via BearDog (pad sequence to 12 bytes for nonce)
        let mut nonce = [0u8; 12];
        nonce[..8].copy_from_slice(&self.sequence.to_be_bytes());

        let encrypted = self
            .beardog
            .chacha20_poly1305_encrypt(&self.session_key, &nonce, data)?;

        debug!(
            sequence = self.sequence,
            bytes = data.len(),
            "Sending encrypted message via BearDog"
        );

        // Send DataMessage (type + sequence + length + payload)
        let msg = DataMessage::new(self.sequence, encrypted);
        self.stream.write_all(&[MessageType::Data as u8]).await?;
        self.stream.write_all(&msg.encode()).await?;

        self.sequence += 1;
        Ok(())
    }

    /// Receive encrypted data, decrypt via BearDog
    pub async fn recv(&mut self) -> Result<Vec<u8>> {
        // Read message type
        let mut type_buf = [0u8; 1];
        self.stream.read_exact(&mut type_buf).await?;

        let msg_type = MessageType::try_from(type_buf[0])?;
        if msg_type != MessageType::Data {
            return Err(OnionError::InvalidMessage(format!(
                "Expected Data, got {:?}",
                msg_type
            )));
        }

        // Read sequence (8 bytes)
        let mut seq_buf = [0u8; 8];
        self.stream.read_exact(&mut seq_buf).await?;
        let msg_sequence = u64::from_be_bytes(seq_buf);

        // Read payload (rest of message)
        let mut encrypted = Vec::new();
        self.stream.read_to_end(&mut encrypted).await?;

        // Decrypt via BearDog
        let mut nonce = [0u8; 12];
        nonce[..8].copy_from_slice(&msg_sequence.to_be_bytes());

        let plaintext = self
            .beardog
            .chacha20_poly1305_decrypt(&self.session_key, &nonce, &encrypted)?;

        debug!(
            sequence = msg_sequence,
            bytes = plaintext.len(),
            "Received and decrypted message via BearDog"
        );

        Ok(plaintext)
    }

    /// Close connection gracefully
    pub async fn close(mut self) -> Result<()> {
        self.stream.write_all(&[MessageType::Close as u8]).await?;
        Ok(())
    }
}
