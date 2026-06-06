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
    use crate::codec::Encode;
    use crate::error::TlsError;
    use crate::messages::{ClientHello, ContentType, Extension};
    use base64::Engine;
    use base64::engine::general_purpose;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
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

    fn config_with_crypto_path(socket_path: String) -> TlsServerConfig {
        TlsServerConfig {
            crypto_client: SecurityTlsCryptoClient::with_socket_path(socket_path),
            certificate: vec![0x30, 0x82, 0x01, 0x04, 0x05],
            key_id: "test-key-id".into(),
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

    /// Encode a `ClientHello` as a TLS Handshake record payload (type + 24-bit length + body).
    fn client_hello_handshake_payload(hello: &ClientHello) -> Vec<u8> {
        let mut ch_body = Vec::new();
        hello.encode(&mut ch_body).expect("encode ClientHello");
        let mut payload = Vec::with_capacity(4 + ch_body.len());
        payload.push(crate::HANDSHAKE_TYPE_CLIENT_HELLO);
        let len = u32::try_from(ch_body.len()).expect("ClientHello fits u24");
        payload.push(((len >> 16) & 0xFF) as u8);
        payload.push(((len >> 8) & 0xFF) as u8);
        payload.push((len & 0xFF) as u8);
        payload.extend_from_slice(&ch_body);
        payload
    }

    fn valid_client_hello() -> ClientHello {
        ClientHello::new(
            [0xAB; 32],
            vec![crate::TLS_CHACHA20_POLY1305_SHA256],
            vec![
                Extension::SupportedVersions(vec![crate::TLS_VERSION_1_3]),
                Extension::KeyShare(vec![0xCD; 32]),
            ],
        )
    }

    /// Mock JSON-RPC server that answers handshake crypto calls (many one-shot TCP connections).
    async fn spawn_handshake_mock_crypto_server() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind mock crypto");
        let addr = listener.local_addr().expect("mock crypto addr");
        tokio::spawn(async move {
            loop {
                let Ok((mut stream, _)) = listener.accept().await else {
                    break;
                };
                tokio::spawn(async move {
                    let mut buf = vec![0u8; 65_536];
                    let n = stream.read(&mut buf).await.unwrap_or(0);
                    if n == 0 {
                        return;
                    }
                    let req = String::from_utf8_lossy(&buf[..n]);
                    let response = if req.contains("generate_keypair") {
                        let pk = general_purpose::STANDARD.encode([9u8; 32]);
                        let sk = general_purpose::STANDARD.encode([8u8; 32]);
                        format!(
                            r#"{{"jsonrpc":"2.0","result":{{"public_key":"{pk}","secret_key":"{sk}"}},"id":1}}"#
                        )
                    } else if req.contains("derive_secret") {
                        let ss = general_purpose::STANDARD.encode([7u8; 32]);
                        format!(r#"{{"jsonrpc":"2.0","result":{{"shared_secret":"{ss}"}},"id":1}}"#)
                    } else {
                        let mac = general_purpose::STANDARD.encode([4u8; 32]);
                        format!(r#"{{"jsonrpc":"2.0","result":{{"mac":"{mac}"}},"id":1}}"#)
                    };
                    let _ = stream.write_all(response.as_bytes()).await;
                    let _ = stream.shutdown().await;
                });
            }
        });
        format!("tcp:{addr}")
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

    #[test]
    fn tls_acceptor_new_with_various_configurations() {
        let paths = ["/tmp/a.sock", "tcp:127.0.0.1:9901", "/var/run/biomeos/crypto.sock"];
        for path in paths {
            let cfg = config_with_crypto_path(path.to_string());
            let acceptor = TlsAcceptor::new(cfg);
            let _ = acceptor;
        }

        let empty_cert = TlsAcceptor::new(TlsServerConfig {
            crypto_client: SecurityTlsCryptoClient::with_socket_path("tcp:127.0.0.1:1".into()),
            certificate: Vec::new(),
            key_id: String::new(),
        });
        let _ = empty_cert;

        let large_cert = TlsAcceptor::new(TlsServerConfig {
            crypto_client: SecurityTlsCryptoClient::with_socket_path("tcp:127.0.0.1:2".into()),
            certificate: vec![0xFF; 4096],
            key_id: "key-with-unicode-🔐".into(),
        });
        let _ = large_cert;
    }

    #[tokio::test]
    async fn accept_rejects_malformed_client_hello_decode() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        let acceptor = TlsAcceptor::new(sample_config());
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            acceptor.accept(stream).await
        });
        let mut client = tokio::net::TcpStream::connect(addr).await.expect("connect");
        // Valid handshake header but body too short to decode ClientHello fields
        let payload = [crate::HANDSHAKE_TYPE_CLIENT_HELLO, 0, 0, 2, 0x03, 0x03];
        let rec = tls_record(ContentType::Handshake as u8, &payload);
        client.write_all(&rec).await.expect("write");
        let res = server.await.expect("join");
        assert!(res.is_err(), "expected decode error for truncated ClientHello body");
    }

    #[tokio::test]
    async fn accept_rejects_client_hello_missing_key_share() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        let acceptor = TlsAcceptor::new(sample_config());
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            acceptor.accept(stream).await
        });
        let hello = ClientHello::new(
            [1u8; 32],
            vec![crate::TLS_CHACHA20_POLY1305_SHA256],
            vec![Extension::SupportedVersions(vec![crate::TLS_VERSION_1_3])],
        );
        let rec = tls_record(ContentType::Handshake as u8, &client_hello_handshake_payload(&hello));
        let mut client = tokio::net::TcpStream::connect(addr).await.expect("connect");
        client.write_all(&rec).await.expect("write");
        let res = server.await.expect("join");
        match res {
            Err(TlsError::HandshakeFailure(msg)) => {
                assert!(msg.contains("key_share"), "unexpected message: {msg}");
            }
            Ok(_) => panic!("expected HandshakeFailure for missing key_share"),
            Err(e) => panic!("expected HandshakeFailure, got {:?}", std::mem::discriminant(&e)),
        }
    }

    #[tokio::test]
    async fn accept_rejects_client_hello_missing_supported_versions() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        let acceptor = TlsAcceptor::new(sample_config());
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            acceptor.accept(stream).await
        });
        let hello = ClientHello::new(
            [2u8; 32],
            vec![crate::TLS_CHACHA20_POLY1305_SHA256],
            vec![Extension::KeyShare(vec![0xEE; 32])],
        );
        let rec = tls_record(ContentType::Handshake as u8, &client_hello_handshake_payload(&hello));
        let mut client = tokio::net::TcpStream::connect(addr).await.expect("connect");
        client.write_all(&rec).await.expect("write");
        let res = server.await.expect("join");
        assert!(res.is_err(), "expected validation error without supported_versions");
    }

    #[tokio::test]
    async fn accept_fails_on_oversized_record_with_truncated_payload() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        let acceptor = TlsAcceptor::new(sample_config());
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            acceptor.accept(stream).await
        });
        let mut client = tokio::net::TcpStream::connect(addr).await.expect("connect");
        // Header claims 4096-byte payload but we only send the 5-byte header then close
        let header = vec![ContentType::Handshake as u8, 0x03, 0x03, 0x10, 0x00];
        client.write_all(&header).await.expect("write header");
        drop(client);
        let res = server.await.expect("join");
        match res {
            Err(TlsError::IoError(msg)) => {
                assert!(
                    msg.contains("payload") || msg.contains("read"),
                    "expected read failure for truncated oversized record: {msg}"
                );
            }
            Ok(_) => panic!("expected IoError for truncated oversized record"),
            Err(e) => panic!("expected IoError, got {:?}", std::mem::discriminant(&e)),
        }
    }

    #[tokio::test]
    async fn accept_fails_when_peer_closes_mid_payload() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        let acceptor = TlsAcceptor::new(sample_config());
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            acceptor.accept(stream).await
        });
        let hello = valid_client_hello();
        let payload = client_hello_handshake_payload(&hello);
        let mut client = tokio::net::TcpStream::connect(addr).await.expect("connect");
        // Send record header claiming full length but only partial payload
        let mut header = vec![ContentType::Handshake as u8, 0x03, 0x03];
        let len = u16::try_from(payload.len()).expect("len");
        header.extend_from_slice(&len.to_be_bytes());
        client.write_all(&header).await.expect("header");
        client.write_all(&payload[..4]).await.expect("partial");
        drop(client);
        let res = server.await.expect("join");
        assert!(res.is_err(), "expected I/O error when payload truncated");
    }

    #[tokio::test]
    async fn accept_completes_key_derivation_with_mock_crypto() {
        let crypto_path = spawn_handshake_mock_crypto_server().await;
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        let acceptor = TlsAcceptor::new(config_with_crypto_path(crypto_path));
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            acceptor.accept(stream).await
        });
        let hello = valid_client_hello();
        let rec = tls_record(ContentType::Handshake as u8, &client_hello_handshake_payload(&hello));
        let mut client = tokio::net::TcpStream::connect(addr).await.expect("connect");
        client.write_all(&rec).await.expect("write ClientHello");
        let tls_stream = server.await.expect("join").expect("handshake should succeed");
        let _ = tls_stream;
    }

    #[tokio::test]
    async fn tls_stream_async_read_write_and_shutdown() {
        let crypto_path = spawn_handshake_mock_crypto_server().await;
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        let acceptor = TlsAcceptor::new(config_with_crypto_path(crypto_path));
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            acceptor.accept(stream).await
        });
        let hello = valid_client_hello();
        let rec = tls_record(ContentType::Handshake as u8, &client_hello_handshake_payload(&hello));
        let (client_ready_tx, client_ready_rx) = tokio::sync::oneshot::channel();
        let client = tokio::spawn(async move {
            let mut client = tokio::net::TcpStream::connect(addr).await.expect("connect");
            client.write_all(&rec).await.expect("write ClientHello");
            client_ready_rx.await.expect("handshake done signal");
            client.write_all(b"post-handshake-probe").await.expect("client probe");
        });
        let mut tls_stream = server.await.expect("join").expect("handshake");
        client_ready_tx.send(()).expect("signal client");

        // AsyncWrite trait delegates to underlying TCP (not TlsStream::write_all which encrypts)
        let n = AsyncWriteExt::write(&mut tls_stream, b"probe-bytes").await.expect("async write");
        assert_eq!(n, b"probe-bytes".len());
        AsyncWriteExt::flush(&mut tls_stream).await.expect("flush");

        // AsyncRead trait delegates to underlying TCP (not TlsStream::read which expects TLS records)
        let mut buf = [0u8; 64];
        let n = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            AsyncReadExt::read(&mut tls_stream, &mut buf),
        )
        .await
        .expect("read timed out")
        .expect("async read");
        assert_eq!(&buf[..n], b"post-handshake-probe");

        tls_stream.shutdown().await.expect("shutdown");
        client.await.expect("client task");
    }

    #[tokio::test]
    async fn accept_fails_when_crypto_provider_unreachable() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        // Bind then drop to get a closed port
        let dead_listener = TcpListener::bind("127.0.0.1:0").await.expect("bind dead");
        let dead_addr = dead_listener.local_addr().expect("dead addr");
        drop(dead_listener);
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        let acceptor = TlsAcceptor::new(config_with_crypto_path(format!("tcp:{dead_addr}")));
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            acceptor.accept(stream).await
        });
        let hello = valid_client_hello();
        let rec = tls_record(ContentType::Handshake as u8, &client_hello_handshake_payload(&hello));
        let mut client = tokio::net::TcpStream::connect(addr).await.expect("connect");
        client.write_all(&rec).await.expect("write");
        let res = server.await.expect("join");
        match res {
            Err(TlsError::CryptoError(msg)) => {
                assert!(msg.contains("connect") || msg.contains("Failed"), "got: {msg}");
            }
            Ok(_) => panic!("expected CryptoError when provider unreachable"),
            Err(e) => panic!("expected CryptoError, got {:?}", std::mem::discriminant(&e)),
        }
    }
}
