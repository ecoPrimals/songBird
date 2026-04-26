// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Onion service (listen mode) - Phase 3 Implementation
//!
//! ✅ **TRUE PRIMAL**: Production uses `security provider` delegation for all crypto.

use crate::error::{OnionError, Result};
use crate::keys::{EphemeralKeypair, OnionIdentity};
use crate::protocol::{DataMessage, KeyExchangeMessage, MessageType};
use crate::security_crypto::SecurityCryptoClient;
use crate::storage::{InMemoryOnionStorage, OnionStorage, OnionStorageBackend};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info, warn};

fn storage_socket_from_endpoint(endpoint: &str) -> Option<PathBuf> {
    let t = endpoint.trim();
    if let Some(p) = t.strip_prefix("unix://") {
        return Some(PathBuf::from(p));
    }
    if t.starts_with('/') {
        return Some(PathBuf::from(t));
    }
    None
}

/// Onion service (creates reachable .onion address)
///
/// **Status**: Phase 3 - Complete implementation with `security provider` delegation
pub struct OnionService {
    identity: OnionIdentity,
    storage: Arc<OnionStorage>,
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
        let storage: Arc<OnionStorage> = {
            #[cfg(unix)]
            {
                if let Ok(ep) = songbird_config::primal_discovery::get_storage_endpoint().await {
                    if let Some(path) = storage_socket_from_endpoint(&ep) {
                        match tokio::net::UnixStream::connect(&path).await {
                            Ok(_) => {
                                info!(
                                    path = %path.display(),
                                    "Onion service: IPC storage (storage.* JSON-RPC)"
                                );
                                Arc::new(OnionStorage::Ipc(
                                    crate::storage_ipc::IpcOnionStorage::new(path),
                                ))
                            }
                            Err(e) => {
                                warn!(
                                    error = %e,
                                    path = %path.display(),
                                    "storage provider unreachable; using in-memory onion storage"
                                );
                                Arc::new(OnionStorage::InMemory(InMemoryOnionStorage::new()))
                            }
                        }
                    } else {
                        Arc::new(OnionStorage::InMemory(InMemoryOnionStorage::new()))
                    }
                } else {
                    Arc::new(OnionStorage::InMemory(InMemoryOnionStorage::new()))
                }
            }
            #[cfg(not(unix))]
            {
                Arc::new(OnionStorage::InMemory(InMemoryOnionStorage::new()))
            }
        };

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
        let (storage, identity): (Arc<OnionStorage>, OnionIdentity) = {
            let inner = InMemoryOnionStorage::new();
            let identity = inner.load_or_generate_identity()?;
            (Arc::new(OnionStorage::InMemory(inner)), identity)
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
        let bind_addr =
            format!("{}:{}", songbird_types::constants::PRODUCTION_BIND_ADDRESS, self.port);
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
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use std::path::PathBuf;

    use super::storage_socket_from_endpoint;

    #[test]
    fn storage_socket_from_endpoint_unix_triple_slash() {
        assert_eq!(
            storage_socket_from_endpoint("unix:///run/storage.sock"),
            Some(PathBuf::from("/run/storage.sock"))
        );
    }

    #[test]
    fn storage_socket_from_endpoint_absolute_path() {
        assert_eq!(
            storage_socket_from_endpoint("/var/songbird/storage.sock"),
            Some(PathBuf::from("/var/songbird/storage.sock"))
        );
    }

    #[test]
    fn storage_socket_from_endpoint_trims_whitespace() {
        assert_eq!(
            storage_socket_from_endpoint("  unix:///tmp/x  "),
            Some(PathBuf::from("/tmp/x"))
        );
    }

    #[test]
    fn storage_socket_from_endpoint_http_not_socket_path() {
        assert_eq!(storage_socket_from_endpoint("http://127.0.0.1:8080"), None);
        assert_eq!(storage_socket_from_endpoint(""), None);
        assert_eq!(storage_socket_from_endpoint("relative/path"), None);
    }
}
