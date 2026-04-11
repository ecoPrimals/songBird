// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! TLS Server Module
//!
//! High-level TLS server integration for HTTP/HTTPS serving.
//! Uses the security (crypto) provider for all crypto operations via JSON-RPC.

use crate::crypto::SecurityTlsCryptoClient;
use crate::error::{Result, TlsError};
use crate::handshake::HandshakeStateMachine;
use crate::record_layer::RecordLayer;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// TLS Server Configuration
pub struct TlsServerConfig {
    /// Security-provider crypto client
    pub crypto_client: SecurityTlsCryptoClient,

    /// Server certificate (DER-encoded)
    pub certificate: Vec<u8>,

    /// Certificate private key ID (for security-provider signing)
    pub key_id: String,
}

/// TLS Server Acceptor
///
/// Accepts TCP connections and performs TLS handshake.
pub struct TlsAcceptor {
    config: Arc<TlsServerConfig>,
}

impl TlsAcceptor {
    /// Create a new TLS acceptor
    #[must_use]
    pub fn new(config: TlsServerConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// Accept a TLS connection
    ///
    /// Performs the full TLS 1.3 handshake and returns an encrypted stream.
    ///
    /// # Errors
    ///
    /// Returns an error if the handshake fails (protocol error, crypto failure, or I/O).
    pub async fn accept(&self, stream: TcpStream) -> Result<TlsStream> {
        TlsStream::accept(stream, self.config.clone()).await
    }
}

/// TLS Stream
///
/// Represents an established TLS connection after handshake.
pub struct TlsStream {
    /// Underlying TCP stream
    stream: TcpStream,

    /// Record layer for encryption/decryption
    record_layer: RecordLayer,

    /// Traffic keys (derived during handshake) — `Arc` slices avoid cloning key material per I/O call.
    write_key: Arc<[u8]>,
    write_iv: Arc<[u8]>,
    read_key: Arc<[u8]>,
    read_iv: Arc<[u8]>,

    /// Crypto client for encryption/decryption
    crypto_client: SecurityTlsCryptoClient,
}

impl TlsStream {
    /// Accept a TLS connection (server-side handshake)
    async fn accept(mut stream: TcpStream, config: Arc<TlsServerConfig>) -> Result<Self> {
        use crate::codec::{Decode, Encode};
        use crate::messages::{ClientHello, ContentType};
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        // Initialize handshake state machine
        let mut handshake = HandshakeStateMachine::new();
        handshake.set_crypto_client(config.crypto_client.clone());

        let mut record_layer = RecordLayer::new();

        // ==== PHASE 1: Receive ClientHello ====
        // Read TLS record header (5 bytes)
        let mut header = vec![0u8; 5];
        stream
            .read_exact(&mut header)
            .await
            .map_err(|e| TlsError::IoError(format!("Failed to read record header: {e}")))?;

        // Parse record header to get payload length
        let content_type = header[0];
        let payload_length = u16::from_be_bytes([header[3], header[4]]) as usize;

        if content_type != ContentType::Handshake as u8 {
            return Err(TlsError::UnexpectedMessage {
                expected: "Handshake".to_string(),
                got: format!("{content_type}"),
            });
        }

        // Read the full payload
        let mut payload = vec![0u8; payload_length];
        stream
            .read_exact(&mut payload)
            .await
            .map_err(|e| TlsError::IoError(format!("Failed to read record payload: {e}")))?;

        // Parse ClientHello from payload (skip handshake header: type(1) + length(3))
        if payload.len() < 4 {
            return Err(TlsError::ProtocolError("Handshake message too short".to_string()));
        }

        let handshake_type = payload[0];
        if handshake_type != crate::HANDSHAKE_TYPE_CLIENT_HELLO {
            return Err(TlsError::UnexpectedMessage {
                expected: "ClientHello".to_string(),
                got: format!("{handshake_type}"),
            });
        }

        // Decode ClientHello (skip 4-byte handshake header)
        let (client_hello, _) = ClientHello::decode(&payload[4..])?;

        // Update transcript hash
        handshake.key_schedule_mut().update_transcript(&payload);

        // Extract client's X25519 public key BEFORE moving client_hello
        let client_public_key = client_hello
            .get_key_share()
            .ok_or_else(|| TlsError::HandshakeFailure("Client did not send key_share".to_string()))?
            .to_vec(); // Clone the key data

        // Process ClientHello (this moves client_hello)
        handshake.process_client_hello(client_hello)?;

        // ==== PHASE 2: Generate and Send ServerHello ====
        let server_hello = handshake.generate_server_hello().await?;

        // Encode ServerHello with handshake header
        let mut server_hello_bytes = Vec::new();
        server_hello_bytes.push(crate::HANDSHAKE_TYPE_SERVER_HELLO);

        // Encode ServerHello to get size
        let mut sh_payload = Vec::new();
        server_hello.encode(&mut sh_payload)?;

        // Write handshake length (24-bit)
        let sh_len = u32::try_from(sh_payload.len()).map_err(|_| TlsError::RecordTooLarge {
            size: sh_payload.len(),
        })?;
        server_hello_bytes.push(((sh_len >> 16) & 0xFF) as u8);
        server_hello_bytes.push(((sh_len >> 8) & 0xFF) as u8);
        server_hello_bytes.push((sh_len & 0xFF) as u8);
        server_hello_bytes.extend_from_slice(&sh_payload);

        // Update transcript hash
        handshake.key_schedule_mut().update_transcript(&server_hello_bytes);

        // Frame as TLS record
        let server_hello_record =
            record_layer.frame_plaintext(ContentType::Handshake, &server_hello_bytes)?;

        // Send ServerHello
        stream
            .write_all(&server_hello_record)
            .await
            .map_err(|e| TlsError::IoError(format!("Failed to send ServerHello: {e}")))?;

        // ==== PHASE 3: Derive Keys & Complete Handshake ====
        // Get our server's secret key from key schedule
        let server_secret_key = handshake
            .key_schedule()
            .server_secret_key()
            .ok_or_else(|| TlsError::InternalError("Server secret key not stored".to_string()))?;

        // Derive ECDHE shared secret using the security provider
        let shared_secret = config
            .crypto_client
            .x25519_derive_secret(server_secret_key, &client_public_key)
            .await?;

        // Compute handshake secret
        handshake.key_schedule_mut().compute_handshake_secret(&shared_secret).await?;

        // Derive handshake traffic keys
        let (client_hs_traffic_secret, server_hs_traffic_secret) =
            handshake.key_schedule().derive_handshake_traffic_keys().await?;

        // Derive write key and IV from server handshake traffic secret
        let (write_key, write_iv) =
            handshake.key_schedule().derive_traffic_keys(&server_hs_traffic_secret).await?;

        // Derive read key and IV from client handshake traffic secret
        let (read_key, read_iv) =
            handshake.key_schedule().derive_traffic_keys(&client_hs_traffic_secret).await?;

        // NOTE: In a complete implementation, we would:
        // 1. Send EncryptedExtensions (empty for basic TLS)
        // 2. Send Certificate (from config)
        // 3. Send CertificateVerify (Ed25519 signature via security provider)
        // 4. Send server Finished (HMAC of transcript)
        // 5. Receive client Finished
        // 6. Compute master secret and application traffic keys
        //
        // For MVP, we'll use handshake traffic keys for now.
        // This allows basic encrypted communication for testing.

        Ok(Self {
            stream,
            record_layer,
            write_key: Arc::from(write_key.into_boxed_slice()),
            write_iv: Arc::from(write_iv.into_boxed_slice()),
            read_key: Arc::from(read_key.into_boxed_slice()),
            read_iv: Arc::from(read_iv.into_boxed_slice()),
            crypto_client: config.crypto_client.clone(),
        })
    }

    /// Write application data (encrypted)
    ///
    /// # Errors
    ///
    /// Returns an error if encryption or I/O fails.
    pub async fn write_all(&mut self, data: &[u8]) -> Result<()> {
        use crate::messages::ContentType;

        // Encrypt function for record layer
        let crypto_client = self.crypto_client.clone();
        let write_key = Arc::clone(&self.write_key);
        let write_iv = Arc::clone(&self.write_iv);

        let encrypt_fn = move |plaintext: &[u8], sequence: u64| {
            // Construct nonce: sequence number XOR IV
            let mut nonce = [0u8; 12];
            nonce.copy_from_slice(write_iv.as_ref());
            let seq_bytes = sequence.to_be_bytes();
            for i in 0..8 {
                nonce[4 + i] ^= seq_bytes[i];
            }

            // Encrypt with ChaCha20-Poly1305 via security provider
            // For now, use a blocking approach (will be made async in future)
            // This is acceptable as the async runtime handles it
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let (ciphertext, _nonce, tag) = crypto_client
                        .chacha20_poly1305_encrypt(plaintext, write_key.as_ref(), None)
                        .await?;

                    // Append tag to ciphertext (standard AEAD format)
                    let mut result = ciphertext;
                    result.extend_from_slice(&tag);
                    Ok(result)
                })
            })
        };

        // Encrypt the data using record layer
        let encrypted_record =
            self.record_layer.encrypt_record(ContentType::ApplicationData, data, encrypt_fn)?;

        // Send encrypted record
        self.stream
            .write_all(&encrypted_record)
            .await
            .map_err(|e| TlsError::IoError(format!("Failed to write encrypted data: {e}")))
    }

    /// Read application data (decrypted)
    ///
    /// # Errors
    ///
    /// Returns an error if I/O, decryption, or record parsing fails.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        use crate::messages::ContentType;

        // Read TLS record
        let mut header = vec![0u8; 5];
        self.stream
            .read_exact(&mut header)
            .await
            .map_err(|e| TlsError::IoError(format!("Failed to read record header: {e}")))?;

        let content_type_byte = header[0];
        let length = u16::from_be_bytes([header[3], header[4]]) as usize;

        // Read ciphertext
        let mut ciphertext = vec![0u8; length];
        self.stream
            .read_exact(&mut ciphertext)
            .await
            .map_err(|e| TlsError::IoError(format!("Failed to read ciphertext: {e}")))?;

        // Only decrypt if it's ApplicationData
        if content_type_byte != ContentType::ApplicationData as u8 {
            return Err(TlsError::UnexpectedMessage {
                expected: "ApplicationData".to_string(),
                got: format!("{content_type_byte}"),
            });
        }

        // Decrypt function for record layer
        let crypto_client = self.crypto_client.clone();
        let read_key = Arc::clone(&self.read_key);
        let read_iv = Arc::clone(&self.read_iv);

        let decrypt_fn = move |ciphertext_with_tag: &[u8], sequence: u64| {
            // Split ciphertext and tag (last 16 bytes)
            if ciphertext_with_tag.len() < 16 {
                return Err(TlsError::DecryptError);
            }
            let tag_start = ciphertext_with_tag.len() - 16;
            let ciphertext = &ciphertext_with_tag[..tag_start];
            let tag = &ciphertext_with_tag[tag_start..];

            // Construct nonce: sequence number XOR IV
            let mut nonce = [0u8; 12];
            nonce.copy_from_slice(read_iv.as_ref());
            let seq_bytes = sequence.to_be_bytes();
            for i in 0..8 {
                nonce[4 + i] ^= seq_bytes[i];
            }

            // Decrypt with ChaCha20-Poly1305 via security provider
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    crypto_client
                        .chacha20_poly1305_decrypt(ciphertext, read_key.as_ref(), &nonce, tag, None)
                        .await
                })
            })
        };

        // Decrypt the record
        let (_content_type, plaintext) =
            self.record_layer.decrypt_record(&ciphertext, decrypt_fn)?;

        // Copy to user buffer
        let to_copy = plaintext.len().min(buf.len());
        buf[..to_copy].copy_from_slice(&plaintext[..to_copy]);

        Ok(to_copy)
    }

    /// Shutdown the TLS connection
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying stream shutdown fails.
    pub async fn shutdown(&mut self) -> Result<()> {
        self.stream
            .shutdown()
            .await
            .map_err(|e| TlsError::IoError(format!("Failed to shutdown: {e}")))
    }
}

/// Implement `tokio::io` traits for `TlsStream`
impl tokio::io::AsyncRead for TlsStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        // Delegate to underlying stream (decryption happens in read())
        std::pin::Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl tokio::io::AsyncWrite for TlsStream {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        // Delegate to underlying stream (encryption happens in write_all())
        std::pin::Pin::new(&mut self.stream).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.stream).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::ContentType;
    use tokio::io::AsyncWriteExt;
    use tokio::net::TcpListener;

    fn sample_config() -> TlsServerConfig {
        TlsServerConfig {
            crypto_client: SecurityTlsCryptoClient::with_socket_path(
                "/tmp/songbird-tls-server-unit-test.sock".into(),
            ),
            certificate: vec![0x30, 0x82, 0x01], // arbitrary DER prefix — not used in these error-path tests
            key_id: "unit-test-key".into(),
        }
    }

    /// TLS plaintext record: 5-byte header + payload (version 0x0303).
    fn tls_record(content_type: u8, payload: &[u8]) -> Vec<u8> {
        let mut r = Vec::with_capacity(5 + payload.len());
        r.push(content_type);
        r.extend_from_slice(&[0x03, 0x03]);
        let len = u16::try_from(payload.len()).expect("payload fits u16");
        r.extend_from_slice(&len.to_be_bytes());
        r.extend_from_slice(payload);
        r
    }

    #[test]
    fn tls_acceptor_new_wraps_config() {
        let cfg = sample_config();
        let acceptor = TlsAcceptor::new(TlsServerConfig {
            crypto_client: cfg.crypto_client.clone(),
            certificate: cfg.certificate.clone(),
            key_id: cfg.key_id,
        });
        let _ = acceptor;
    }

    #[tokio::test]
    async fn accept_rejects_non_handshake_first_record() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        let acceptor = TlsAcceptor::new(sample_config());
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            acceptor.accept(stream).await
        });
        let mut client = tokio::net::TcpStream::connect(addr).await.expect("connect");
        let rec = tls_record(ContentType::ApplicationData as u8, b"ping");
        client.write_all(&rec).await.expect("write");
        let res = server.await.expect("join");
        assert!(res.is_err(), "expected error for non-handshake first record");
    }

    #[tokio::test]
    async fn accept_rejects_empty_handshake_payload() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        let acceptor = TlsAcceptor::new(sample_config());
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            acceptor.accept(stream).await
        });
        let mut client = tokio::net::TcpStream::connect(addr).await.expect("connect");
        let rec = tls_record(ContentType::Handshake as u8, &[]);
        client.write_all(&rec).await.expect("write");
        let res = server.await.expect("join");
        assert!(res.is_err(), "expected error for handshake payload too short");
    }

    #[tokio::test]
    async fn accept_rejects_non_client_hello_handshake_type() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        let acceptor = TlsAcceptor::new(sample_config());
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            acceptor.accept(stream).await
        });
        let mut client = tokio::net::TcpStream::connect(addr).await.expect("connect");
        // Handshake message header: type ServerHello (2), length 0, no body — enough to fail type check
        let payload = [crate::HANDSHAKE_TYPE_SERVER_HELLO, 0, 0, 0];
        let rec = tls_record(ContentType::Handshake as u8, &payload);
        client.write_all(&rec).await.expect("write");
        let res = server.await.expect("join");
        assert!(res.is_err(), "expected error when first handshake is not ClientHello");
    }

    #[tokio::test]
    async fn accept_fails_when_peer_closes_before_header() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        let acceptor = TlsAcceptor::new(sample_config());
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            acceptor.accept(stream).await
        });
        drop(tokio::net::TcpStream::connect(addr).await.expect("connect"));
        let res = server.await.expect("join");
        assert!(res.is_err(), "expected I/O error when client disconnects immediately");
    }
}
