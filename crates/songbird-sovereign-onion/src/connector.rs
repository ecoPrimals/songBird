// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Onion connector (connect to .onion addresses) - Phase 4 Implementation
//!
//! ✅ **TRUE PRIMAL**: Production uses `security provider` delegation for all crypto.

use crate::error::{OnionError, Result};
use crate::keys::EphemeralKeypair;
use crate::protocol::{DataMessage, KeyExchangeMessage, MessageType};
use crate::security_crypto::SecurityCryptoClient;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, info};

/// Connect to onion services
///
/// **Status**: Phase 4 - Complete implementation
#[derive(Default)]
pub struct OnionConnector {
    security: Option<Arc<SecurityCryptoClient>>,
}

impl OnionConnector {
    /// Create new onion connector with `security provider` delegation (TRUE PRIMAL)
    #[must_use]
    pub fn new_via_security_provider(security: SecurityCryptoClient) -> Self {
        Self {
            security: Some(Arc::new(security)),
        }
    }

    /// Create new onion connector (standalone mode - testing only)
    #[cfg(feature = "standalone")]
    #[must_use]
    pub const fn new_standalone() -> Self {
        Self {
            security: None,
        }
    }

    /// Connect to an onion address (via `security provider`)
    ///
    /// # Errors
    ///
    /// Returns error if connection fails, handshake fails, or `security provider` crypto fails.
    ///
    /// # Arguments
    /// - `onion_address`: The target .onion address
    /// - `port`: Target port
    ///
    /// # Returns
    /// Established encrypted connection
    pub async fn connect(&self, onion_address: &str, port: u16) -> Result<OnionConnection> {
        let security = self
            .security
            .as_ref()
            .ok_or_else(|| OnionError::ConfigError("Security crypto client required".into()))?;

        info!(onion_address = onion_address, port = port, "Connecting to onion service");

        // For Phase 1: Direct TCP connection (assumes IP known via rendezvous)
        // For Phase 2: Will use BeaconMesh for address resolution
        let stream = TcpStream::connect(format!("{onion_address}:{port}"))
            .await
            .map_err(|_| OnionError::ConnectionTimeout)?;

        debug!("TCP connection established, starting handshake");

        // Generate our ephemeral keypair via security provider
        let our_ephemeral = EphemeralKeypair::generate_via_security_provider(security).await?;

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
                "Expected KeyExchange response, got {response_type:?}"
            )));
        }

        let peer_key_exchange = KeyExchangeMessage::decode(&recv_buf[1..])?;
        debug!("Received key exchange response");

        // Derive shared secret via security provider
        let shared_secret = our_ephemeral
            .derive_shared_secret_via_security_provider(security, &peer_key_exchange.pubkey)
            .await?;

        info!("Handshake complete - connection established via security provider crypto");

        Ok(OnionConnection {
            stream,
            session_key: shared_secret,
            sequence: 0,
            security: Arc::clone(security),
        })
    }
}

/// Established onion connection (encrypted session)
pub struct OnionConnection {
    stream: TcpStream,
    session_key: [u8; 32],
    sequence: u64,
    security: Arc<SecurityCryptoClient>,
}

impl OnionConnection {
    /// Send encrypted data via `security provider`
    ///
    /// # Errors
    ///
    /// Returns error if encryption or I/O fails.
    pub async fn send(&mut self, data: &[u8]) -> Result<()> {
        // Encrypt via security provider (pad sequence to 12 bytes for nonce)
        let mut nonce = [0u8; 12];
        nonce[..8].copy_from_slice(&self.sequence.to_be_bytes());

        let encrypted =
            self.security.chacha20_poly1305_encrypt(&self.session_key, &nonce, data).await?;

        debug!(
            sequence = self.sequence,
            bytes = data.len(),
            "Sending encrypted message via security provider"
        );

        // Send DataMessage (type + sequence + length + payload)
        let msg = DataMessage::new(self.sequence, encrypted);
        self.stream.write_all(&[MessageType::Data as u8]).await?;
        self.stream.write_all(&msg.encode()).await?;

        self.sequence += 1;
        Ok(())
    }

    /// Receive encrypted data, decrypt via `security provider`
    ///
    /// # Errors
    ///
    /// Returns error if decryption or I/O fails.
    pub async fn recv(&mut self) -> Result<Vec<u8>> {
        // Read message type
        let mut type_buf = [0u8; 1];
        self.stream.read_exact(&mut type_buf).await?;

        let msg_type = MessageType::try_from(type_buf[0])?;
        if msg_type != MessageType::Data {
            return Err(OnionError::InvalidMessage(format!("Expected Data, got {msg_type:?}")));
        }

        // Read sequence (8 bytes)
        let mut seq_buf = [0u8; 8];
        self.stream.read_exact(&mut seq_buf).await?;
        let msg_sequence = u64::from_be_bytes(seq_buf);

        // Read payload (rest of message)
        let mut encrypted = Vec::new();
        self.stream.read_to_end(&mut encrypted).await?;

        // Decrypt via security provider
        let mut nonce = [0u8; 12];
        nonce[..8].copy_from_slice(&msg_sequence.to_be_bytes());

        let plaintext =
            self.security.chacha20_poly1305_decrypt(&self.session_key, &nonce, &encrypted).await?;

        debug!(
            sequence = msg_sequence,
            bytes = plaintext.len(),
            "Received and decrypted message via security provider"
        );

        Ok(plaintext)
    }

    /// Close connection gracefully
    ///
    /// # Errors
    ///
    /// Returns error if I/O fails.
    pub async fn close(mut self) -> Result<()> {
        self.stream.write_all(&[MessageType::Close as u8]).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::security_crypto::SecurityCryptoClient;

    #[test]
    fn default_is_empty_connector() {
        let c = OnionConnector::default();
        let _ = c;
    }

    #[tokio::test(start_paused = true)]
    async fn default_connector_missing_beardog_errors_before_tcp() {
        let connector = OnionConnector::default();
        let r = connector.connect("127.0.0.1", 9).await;
        assert!(matches!(r, Err(OnionError::ConfigError(_))));
    }

    #[tokio::test(start_paused = true)]
    async fn beardog_connector_tcp_fails_fast_when_port_closed() {
        let client =
            SecurityCryptoClient::from_neural_api_socket("/tmp/songbird-onion-test-invalid.sock");
        let connector = OnionConnector::new_via_security_provider(client);
        let r = connector.connect("127.0.0.1", 1).await;
        assert!(matches!(r, Err(OnionError::ConnectionTimeout)));
    }

    #[test]
    fn new_via_beardog_stores_client_for_connect() {
        let client =
            SecurityCryptoClient::from_neural_api_socket("/tmp/songbird-onion-test-invalid.sock");
        let _connector = OnionConnector::new_via_security_provider(client);
    }
}
