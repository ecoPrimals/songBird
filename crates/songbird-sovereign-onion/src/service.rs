// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Onion service (listen mode) - Phase 3 Implementation
//!
//! ✅ **TRUE PRIMAL**: Production uses `security provider` delegation for all crypto.

#[cfg(feature = "sled-storage")]
use crate::OnionStorage;
use crate::error::{OnionError, Result};
use crate::keys::{EphemeralKeypair, OnionIdentity};
use crate::protocol::{DataMessage, KeyExchangeMessage, MessageType};
use crate::security_crypto::SecurityCryptoClient;
#[cfg(not(feature = "sled-storage"))]
use crate::storage::InMemoryOnionStorage;
use crate::storage::OnionStorageBackend;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info};

#[cfg(any(test, feature = "sled-storage"))]
fn onion_data_dir() -> String {
    songbird_process_env::var("SONGBIRD_ONION_DATA_DIR").unwrap_or_else(|_| {
        dirs::data_local_dir().map_or_else(
            || "./data/sovereign-onion".to_string(),
            |d| d.join("songbird").join("sovereign-onion").to_string_lossy().into_owned(),
        )
    })
}

/// Onion service (creates reachable .onion address)
///
/// **Status**: Phase 3 - Complete implementation with `security provider` delegation
pub struct OnionService {
    identity: OnionIdentity,
    storage: Arc<dyn OnionStorageBackend>,
    port: u16,
    security: Arc<SecurityCryptoClient>,
}

impl OnionService {
    /// Create new onion service via `security provider` (TRUE PRIMAL - production)
    ///
    /// Loads existing identity or generates new one via `security provider`.
    ///
    /// # Errors
    ///
    /// Returns an error if storage open, identity load/generate, or persistence fails.
    pub async fn new_via_security_provider(
        port: u16,
        security: SecurityCryptoClient,
    ) -> Result<Self> {
        #[cfg(feature = "sled-storage")]
        let storage: Arc<dyn OnionStorageBackend> = Arc::new(OnionStorage::open(onion_data_dir())?);
        #[cfg(not(feature = "sled-storage"))]
        let storage: Arc<dyn OnionStorageBackend> = Arc::new(InMemoryOnionStorage::new());

        // Load or generate identity via security provider
        let identity = if let Some(stored) = storage.load_identity()? {
            debug!("Loaded existing onion identity");
            stored
        } else {
            info!("Generating new onion identity via security provider");
            let identity = OnionIdentity::generate_via_security_provider(&security).await?;
            storage.store_identity(&identity)?;
            identity
        };

        info!(
            onion_address = %identity.onion_address(),
            port = port,
            "Onion service created (delegated security-provider crypto)"
        );

        Ok(Self {
            identity,
            storage,
            port,
            security: Arc::new(security),
        })
    }

    /// Create new onion service (standalone mode - testing only)
    ///
    /// ⚠️ **Testing only** - Uses direct crypto without `security provider`
    ///
    /// # Errors
    ///
    /// Returns an error if storage open or identity load/generate fails.
    #[cfg(feature = "standalone")]
    pub fn new_standalone(port: u16) -> Result<Self> {
        #[cfg(feature = "sled-storage")]
        let (storage, identity): (Arc<dyn OnionStorageBackend>, OnionIdentity) = {
            let inner = OnionStorage::open(onion_data_dir())?;
            let identity = inner.load_or_generate_identity()?;
            (Arc::new(inner), identity)
        };
        #[cfg(not(feature = "sled-storage"))]
        let (storage, identity): (Arc<dyn OnionStorageBackend>, OnionIdentity) = {
            let inner = InMemoryOnionStorage::new();
            let identity = inner.load_or_generate_identity()?;
            (Arc::new(inner), identity)
        };

        info!(
            onion_address = %identity.onion_address(),
            port = port,
            "Onion service created (STANDALONE - testing mode)"
        );

        // Session handshakes and data-plane crypto (`ChaCha20Poly1305`, X25519 ephemeral)
        // still delegate to this client; only long-lived identity material above is local.
        let security = SecurityCryptoClient::from_env();

        Ok(Self {
            identity,
            storage,
            port,
            security: Arc::new(security),
        })
    }

    /// Get our .onion address
    #[must_use]
    pub fn onion_address(&self) -> &str {
        self.identity.onion_address()
    }

    /// Get listen port
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// Start listening for incoming connections
    ///
    /// Binds to 0.0.0.0:port and accepts connections indefinitely.
    ///
    /// # Errors
    ///
    /// Returns error if bind fails.
    pub async fn run(&self) -> Result<()> {
        let bind_addr = format!("0.0.0.0:{}", self.port);
        info!("Starting onion service on {}", bind_addr);

        let listener = TcpListener::bind(&bind_addr).await.map_err(|e| {
            OnionError::Io(std::io::Error::new(
                std::io::ErrorKind::AddrInUse,
                format!("Failed to bind to {bind_addr}: {e}"),
            ))
        })?;

        info!(
            onion_address = %self.onion_address(),
            port = self.port,
            "Onion service listening - ready for connections"
        );

        // Accept connections loop
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    debug!("Accepted connection from {}", addr);
                    let service = Self {
                        identity: self.identity.clone(),
                        storage: self.storage.clone(),
                        port: self.port,
                        security: Arc::clone(&self.security),
                    };

                    tokio::spawn(async move {
                        if let Err(e) = service.handle_connection(stream).await {
                            error!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle incoming connection (handshake + data transfer)
    async fn handle_connection(&self, mut stream: TcpStream) -> Result<()> {
        debug!("Handling new connection - starting handshake");

        // Read key exchange message (1 byte type + 57 bytes payload)
        let mut buf = [0u8; 58];
        stream.read_exact(&mut buf).await?;

        // Parse message type
        let msg_type = MessageType::try_from(buf[0])?;
        if msg_type != MessageType::KeyExchange {
            return Err(OnionError::InvalidMessage(format!(
                "Expected KeyExchange, got {msg_type:?}"
            )));
        }

        // Parse key exchange
        let key_exchange = KeyExchangeMessage::decode(&buf[1..])?;
        debug!("Received key exchange from peer");

        // Generate our ephemeral keypair via security provider
        let our_ephemeral =
            EphemeralKeypair::generate_via_security_provider(&self.security).await?;
        let our_public_key = *our_ephemeral.public_bytes();

        // Derive shared secret via security provider (consumes our_ephemeral)
        let shared_secret = our_ephemeral
            .derive_shared_secret_via_security_provider(&self.security, &key_exchange.pubkey)
            .await?;

        debug!("Derived shared secret via security provider - handshake complete");

        // Send our key exchange response
        let our_key_exchange = KeyExchangeMessage::new(our_public_key, [0u8; 24]);
        stream.write_all(&[MessageType::KeyExchange as u8]).await?;
        stream.write_all(&our_key_exchange.encode()).await?;

        debug!("Sent key exchange response");

        // Now handle encrypted data messages
        self.handle_data_transfer(&mut stream, &shared_secret).await?;

        Ok(())
    }

    /// Handle encrypted data transfer after handshake
    async fn handle_data_transfer(
        &self,
        stream: &mut TcpStream,
        session_key: &[u8; 32],
    ) -> Result<()> {
        let mut sequence: u64 = 0;

        loop {
            // Read message type
            let mut type_buf = [0u8; 1];
            if stream.read_exact(&mut type_buf).await.is_err() {
                debug!("Connection closed by peer");
                break;
            }

            let msg_type = MessageType::try_from(type_buf[0])?;

            match msg_type {
                MessageType::Data => {
                    // Read data message header (8 bytes sequence + 4 bytes length)
                    let mut header = [0u8; 12];
                    stream.read_exact(&mut header).await?;

                    let msg_sequence = u64::from_be_bytes([
                        header[0], header[1], header[2], header[3], header[4], header[5],
                        header[6], header[7],
                    ]);
                    let payload_len =
                        u32::from_be_bytes([header[8], header[9], header[10], header[11]]) as usize;

                    // Read encrypted payload
                    let mut encrypted = vec![0u8; payload_len];
                    stream.read_exact(&mut encrypted).await?;

                    // Decrypt via security provider (pad sequence to 12 bytes for nonce)
                    let mut nonce = [0u8; 12];
                    nonce[..8].copy_from_slice(&msg_sequence.to_be_bytes());

                    let plaintext = self
                        .security
                        .chacha20_poly1305_decrypt(session_key, &nonce, &encrypted)
                        .await?;

                    debug!(
                        sequence = msg_sequence,
                        bytes = plaintext.len(),
                        "Received and decrypted message via security provider"
                    );

                    // Echo back (for testing - replace with actual logic)
                    let response_encrypted = self
                        .security
                        .chacha20_poly1305_encrypt(session_key, &nonce, &plaintext)
                        .await?;

                    let response_data = DataMessage::new(sequence, response_encrypted);
                    stream.write_all(&[MessageType::Data as u8]).await?;
                    stream.write_all(&response_data.encode()).await?;

                    sequence += 1;
                }
                MessageType::Close => {
                    debug!("Received close message");
                    break;
                }
                MessageType::KeyExchange => {
                    error!("Unexpected message type: {msg_type:?}");
                    break;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use std::time::Duration;

    #[test]
    fn onion_data_dir_returns_non_empty_path() {
        let dir = super::onion_data_dir();
        assert!(!dir.is_empty());
        assert!(
            dir.contains("sovereign-onion") || dir.ends_with("sovereign-onion"),
            "expected default or env-based path segment: {dir:?}"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn virtual_time_advances_for_sleep() {
        let start = tokio::time::Instant::now();
        let sleep = tokio::time::sleep(Duration::from_secs(2));
        tokio::time::advance(Duration::from_secs(2)).await;
        sleep.await;
        assert!(
            start.elapsed() >= Duration::from_secs(2),
            "paused timer should advance with tokio::time::advance"
        );
    }
}
