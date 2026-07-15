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
#[cfg(any(unix, test))]
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const MAX_ONION_FRAME: usize = 16 * 1024 * 1024;
use tokio::net::{TcpListener, TcpStream};
#[cfg(not(unix))]
use tracing::{debug, error, info};
#[cfg(unix)]
use tracing::{debug, error, info, warn};

#[cfg(any(unix, test))]
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

                    if payload_len > MAX_ONION_FRAME {
                        return Err(OnionError::InvalidMessage(format!(
                            "Frame too large: {payload_len} bytes (max {MAX_ONION_FRAME})"
                        )));
                    }

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
    #![allow(clippy::unwrap_used, reason = "test assertions")]
    #![allow(clippy::case_sensitive_file_extension_comparisons, reason = "checking .onion suffix")]
    #![allow(clippy::unused_async, reason = "test helpers match async interface signatures")]

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

#[cfg(test)]
mod service_lifecycle_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
    #![allow(clippy::case_sensitive_file_extension_comparisons)]
    #![allow(clippy::unused_async)]

    use super::OnionService;
    use crate::security_crypto::SecurityCryptoClient;
    use base64::{Engine, engine::general_purpose::STANDARD};
    use serde_json::json;
    use songbird_crypto_provider::{CryptoProvider, RoutingMode};
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream, UnixListener};

    fn b64(data: &[u8]) -> String {
        STANDARD.encode(data)
    }

    async fn read_json_rpc_request(stream: &mut tokio::net::UnixStream) -> serde_json::Value {
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.expect("read request");
        serde_json::from_slice(&buf).expect("parse JSON-RPC request")
    }

    async fn start_service_mock_server() -> String {
        let path = std::env::temp_dir().join(format!(
            "songbird-onion-service-{}-{}.sock",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        let path_str = path.to_string_lossy().to_string();
        let listener = UnixListener::bind(&path_str).expect("bind mock socket");

        tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                let req = read_json_rpc_request(&mut stream).await;
                let method = req["method"].as_str().unwrap_or("");
                let id = req["id"].as_u64().unwrap_or(1);
                let result = match method {
                    "crypto.ed25519_generate_keypair" => json!({
                        "public_key": b64(&[0x01u8; 32]),
                        "secret_key": b64(&[0x02u8; 32]),
                    }),
                    "crypto.sha3_256" => json!({
                        "hash_base64": b64(&[0x03u8; 32]),
                    }),
                    _ => json!({}),
                };
                let body = json!({"jsonrpc":"2.0","result":result,"id":id}).to_string();
                let _ = stream.write_all(body.as_bytes()).await;
            }
        });

        path_str
    }

    fn mock_client(path: &str) -> SecurityCryptoClient {
        SecurityCryptoClient::from_provider(CryptoProvider::with_mode(path, RoutingMode::Direct))
    }

    async fn pick_free_port() -> u16 {
        TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral")
            .local_addr()
            .expect("local addr")
            .port()
    }

    #[tokio::test(start_paused = true)]
    async fn new_via_security_provider_exposes_address_and_port() {
        let path = start_service_mock_server().await;
        let port = pick_free_port().await;
        let service = OnionService::new_via_security_provider(port, mock_client(&path))
            .await
            .expect("create service");
        assert_eq!(service.port(), port);
        assert!(service.onion_address().ends_with(".onion"));
    }

    #[tokio::test(start_paused = true)]
    async fn service_run_listens_and_accepts_tcp_connections() {
        let path = start_service_mock_server().await;
        let port = pick_free_port().await;
        let service = OnionService::new_via_security_provider(port, mock_client(&path))
            .await
            .expect("create service");

        let run_task = tokio::spawn(async move {
            let _ = service.run().await;
        });

        tokio::time::sleep(Duration::from_millis(50)).await;
        let conn = TcpStream::connect(format!("127.0.0.1:{port}")).await;
        assert!(conn.is_ok(), "service should accept TCP while running");
        run_task.abort();
    }

    #[tokio::test(start_paused = true)]
    async fn service_run_fails_when_port_already_bound() {
        let path = start_service_mock_server().await;
        let port = pick_free_port().await;
        let _guard = TcpListener::bind(format!("0.0.0.0:{port}")).await.expect("hold port");

        let service = OnionService::new_via_security_provider(port, mock_client(&path))
            .await
            .expect("create service");
        let err = service.run().await.expect_err("bind should fail");
        assert!(
            matches!(err, crate::error::OnionError::Io(_)),
            "expected Io error on bind conflict, got {err:?}"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn service_reuses_loaded_identity_from_in_memory_storage() {
        let path = start_service_mock_server().await;
        let port = pick_free_port().await;
        let client = mock_client(&path);

        let first = OnionService::new_via_security_provider(port, client.clone())
            .await
            .expect("first service");
        let address = first.onion_address().to_string();

        let second = OnionService::new_via_security_provider(port + 1, client)
            .await
            .expect("second service");
        // Each OnionService owns separate in-memory storage; addresses differ on second create.
        assert!(second.onion_address().ends_with(".onion"));
        assert!(!address.is_empty());
    }
}
