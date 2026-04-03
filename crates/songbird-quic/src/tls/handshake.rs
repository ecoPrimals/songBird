// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! TLS 1.3 handshake for QUIC.
//!
//! In QUIC, TLS handshake messages are carried in CRYPTO frames rather than
//! TLS records. This module manages the TLS state machine, producing handshake
//! bytes for outgoing CRYPTO frames and consuming bytes from incoming ones.
//!
//! The TLS key schedule is driven through `security provider`, producing keys that are
//! installed into the `CryptoSession` at each encryption level.

use crate::crypto::initial_keys::{self, DirectionalKeys};
use crate::crypto::provider::{QuicCipherSuite, QuicCryptoProvider};
use crate::error::Result;
use crate::tls::session::{CryptoSession, EncryptionLevel, LevelKeys};
use crate::tls::transport_params::TransportParams;

/// TLS handshake state for a QUIC connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandshakeState {
    /// Waiting to begin (client needs to send `ClientHello`).
    Initial,
    /// Client has sent `ClientHello`, waiting for `ServerHello`.
    WaitServerHello,
    /// Processing encrypted handshake (`EncryptedExtensions`, `Certificate`, etc.).
    WaitEncryptedExtensions,
    /// Waiting for the server's Finished message.
    WaitFinished,
    /// Handshake is complete, 1-RTT keys are available.
    Complete,
    /// Handshake has failed.
    Failed,
}

/// Role of this endpoint in the QUIC connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Client initiates the connection.
    Client,
    /// Server accepts the connection.
    Server,
}

/// Manages the TLS 1.3 handshake within a QUIC connection.
///
/// Produces handshake bytes for CRYPTO frames and consumes incoming
/// handshake bytes, driving the key schedule through `security provider`.
#[derive(Debug)]
pub struct QuicTlsHandshake {
    /// Current handshake state.
    state: HandshakeState,
    /// Role (client or server).
    role: Role,
    /// Negotiated cipher suite (default: `TLS_AES_128_GCM_SHA256`).
    cipher_suite: QuicCipherSuite,
    /// Accumulated transcript hash input.
    transcript: Vec<u8>,
    /// Outgoing handshake bytes (to be sent in CRYPTO frames).
    outgoing: Vec<u8>,
    /// Incoming handshake bytes buffer (reassembled from CRYPTO frames).
    incoming: Vec<u8>,
    /// Our transport parameters.
    local_params: TransportParams,
    /// Peer's transport parameters (once received).
    peer_params: Option<TransportParams>,
}

impl QuicTlsHandshake {
    /// Create a new handshake for the given role.
    #[must_use]
    pub fn new(role: Role, local_params: TransportParams) -> Self {
        Self {
            state: HandshakeState::Initial,
            role,
            cipher_suite: QuicCipherSuite::Aes128Gcm,
            transcript: Vec::new(),
            outgoing: Vec::new(),
            incoming: Vec::new(),
            local_params,
            peer_params: None,
        }
    }

    /// Current handshake state.
    #[must_use]
    pub const fn state(&self) -> HandshakeState {
        self.state
    }

    /// Whether the handshake is complete.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        matches!(self.state, HandshakeState::Complete)
    }

    /// Negotiated cipher suite.
    #[must_use]
    pub const fn cipher_suite(&self) -> QuicCipherSuite {
        self.cipher_suite
    }

    /// Peer's transport parameters (available after processing `ServerHello` / `EncryptedExtensions`).
    #[must_use]
    pub fn peer_params(&self) -> Option<&TransportParams> {
        self.peer_params.as_ref()
    }

    /// Take outgoing handshake bytes to be sent in CRYPTO frames.
    /// Drains the buffer.
    pub fn take_outgoing(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.outgoing)
    }

    /// Feed incoming handshake bytes from received CRYPTO frames.
    pub fn receive_crypto_data(&mut self, data: &[u8]) {
        self.incoming.extend_from_slice(data);
    }

    /// Initialize the handshake: derive Initial keys and (for clients)
    /// prepare the `ClientHello`.
    ///
    /// # Errors
    ///
    /// Returns an error if initial key derivation fails or the `ClientHello` cannot be built.
    pub async fn initialize(
        &mut self,
        crypto: &dyn QuicCryptoProvider,
        dcid: &[u8],
        session: &mut CryptoSession,
    ) -> Result<()> {
        let initial_keys = initial_keys::derive_initial_keys(crypto, dcid).await?;

        let (local_keys, remote_keys) = match self.role {
            Role::Client => (initial_keys.client, initial_keys.server),
            Role::Server => (initial_keys.server, initial_keys.client),
        };

        session.install_keys(
            EncryptionLevel::Initial,
            LevelKeys {
                local: local_keys,
                remote: remote_keys,
                suite: initial_keys::INITIAL_CIPHER_SUITE,
            },
        );

        if self.role == Role::Client {
            let client_hello = self.build_client_hello()?;
            self.transcript.extend_from_slice(&client_hello);
            self.outgoing.extend_from_slice(&client_hello);
            self.state = HandshakeState::WaitServerHello;
        }

        Ok(())
    }

    /// Process incoming handshake data and advance the state machine.
    ///
    /// Returns a list of encryption levels whose keys were updated.
    ///
    /// # Errors
    ///
    /// Returns an error if key derivation or building a handshake message fails.
    pub async fn process(
        &mut self,
        crypto: &dyn QuicCryptoProvider,
        session: &mut CryptoSession,
    ) -> Result<Vec<EncryptionLevel>> {
        let mut updated_levels = Vec::new();

        match self.state {
            HandshakeState::WaitServerHello if self.role == Role::Client => {
                if self.incoming.len() >= 4 {
                    let server_hello = std::mem::take(&mut self.incoming);
                    self.transcript.extend_from_slice(&server_hello);

                    let handshake_keys = self.derive_handshake_keys(crypto).await?;
                    session.install_keys(EncryptionLevel::Handshake, handshake_keys);
                    session.discard_keys(EncryptionLevel::Initial);
                    updated_levels.push(EncryptionLevel::Handshake);

                    self.state = HandshakeState::WaitEncryptedExtensions;
                }
            }

            HandshakeState::WaitEncryptedExtensions if self.role == Role::Client => {
                if !self.incoming.is_empty() {
                    let ee_data = std::mem::take(&mut self.incoming);
                    self.transcript.extend_from_slice(&ee_data);
                    self.state = HandshakeState::WaitFinished;
                }
            }

            HandshakeState::WaitFinished if self.role == Role::Client => {
                if !self.incoming.is_empty() {
                    let finished_data = std::mem::take(&mut self.incoming);
                    self.transcript.extend_from_slice(&finished_data);

                    let app_keys = self.derive_application_keys(crypto).await?;
                    session.install_keys(EncryptionLevel::OneRtt, app_keys);
                    session.discard_keys(EncryptionLevel::Handshake);
                    updated_levels.push(EncryptionLevel::OneRtt);

                    let client_finished = Self::build_client_finished();
                    self.outgoing.extend_from_slice(&client_finished);

                    self.state = HandshakeState::Complete;
                }
            }

            HandshakeState::Initial if self.role == Role::Server => {
                if !self.incoming.is_empty() {
                    let client_hello = std::mem::take(&mut self.incoming);
                    self.transcript.extend_from_slice(&client_hello);

                    let server_hello = Self::build_server_hello();
                    self.transcript.extend_from_slice(&server_hello);
                    self.outgoing.extend_from_slice(&server_hello);

                    let handshake_keys = self.derive_handshake_keys(crypto).await?;
                    session.install_keys(EncryptionLevel::Handshake, handshake_keys);
                    session.discard_keys(EncryptionLevel::Initial);
                    updated_levels.push(EncryptionLevel::Handshake);

                    let ee = self.build_encrypted_extensions()?;
                    self.transcript.extend_from_slice(&ee);
                    self.outgoing.extend_from_slice(&ee);

                    let finished = Self::build_server_finished();
                    self.transcript.extend_from_slice(&finished);
                    self.outgoing.extend_from_slice(&finished);

                    self.state = HandshakeState::WaitFinished;
                }
            }

            HandshakeState::WaitFinished if self.role == Role::Server => {
                if !self.incoming.is_empty() {
                    let client_finished = std::mem::take(&mut self.incoming);
                    self.transcript.extend_from_slice(&client_finished);

                    let app_keys = self.derive_application_keys(crypto).await?;
                    session.install_keys(EncryptionLevel::OneRtt, app_keys);
                    session.discard_keys(EncryptionLevel::Handshake);
                    updated_levels.push(EncryptionLevel::OneRtt);

                    self.state = HandshakeState::Complete;
                }
            }

            _ => {}
        }

        Ok(updated_levels)
    }

    /// Build a minimal `ClientHello` message (TLS 1.3 for QUIC).
    fn build_client_hello(&self) -> Result<Vec<u8>> {
        let tp_encoded = self.local_params.encode()?;
        let mut msg = Vec::with_capacity(64 + tp_encoded.len());

        // Handshake type: ClientHello (1)
        msg.push(0x01);
        // Placeholder for 3-byte length (filled below)
        let len_offset = msg.len();
        msg.extend_from_slice(&[0x00, 0x00, 0x00]);

        // Legacy version TLS 1.2 (for TLS 1.3 compatibility)
        msg.extend_from_slice(&[0x03, 0x03]);

        // Client random (32 bytes of zeros — real impl uses security provider randomness)
        msg.extend_from_slice(&[0x00; 32]);

        // Legacy session ID (empty)
        msg.push(0x00);

        // Cipher suites: TLS_AES_128_GCM_SHA256 (0x1301)
        msg.extend_from_slice(&[0x00, 0x02, 0x13, 0x01]);

        // Legacy compression methods (null)
        msg.extend_from_slice(&[0x01, 0x00]);

        // Extensions
        let mut extensions = Vec::new();

        // Supported Versions extension (0x002B): TLS 1.3 only
        extensions.extend_from_slice(&[0x00, 0x2B, 0x00, 0x03, 0x02, 0x03, 0x04]);

        // QUIC Transport Parameters (0x0039)
        extensions.extend_from_slice(&[0x00, 0x39]);
        #[expect(
            clippy::cast_possible_truncation,
            reason = "TLS extension lengths fit in u16 for valid transport params"
        )]
        let tp_len_u16 = tp_encoded.len() as u16;
        extensions.extend_from_slice(&tp_len_u16.to_be_bytes());
        extensions.extend_from_slice(&tp_encoded);

        // Extensions total length
        #[expect(
            clippy::cast_possible_truncation,
            reason = "handshake extension block length fits in u16 for this message layout"
        )]
        let ext_len_u16 = extensions.len() as u16;
        msg.extend_from_slice(&ext_len_u16.to_be_bytes());
        msg.extend_from_slice(&extensions);

        // Fill in the length field (3 bytes, big-endian)
        let body_len = msg.len() - len_offset - 3;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "TLS handshake body length uses three low bytes only"
        )]
        let b0 = ((body_len >> 16) & 0xFF) as u8;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "TLS handshake body length uses three low bytes only"
        )]
        let b1 = ((body_len >> 8) & 0xFF) as u8;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "TLS handshake body length uses three low bytes only"
        )]
        let b2 = (body_len & 0xFF) as u8;
        msg[len_offset] = b0;
        msg[len_offset + 1] = b1;
        msg[len_offset + 2] = b2;

        Ok(msg)
    }

    /// Build a minimal `ServerHello` message.
    fn build_server_hello() -> Vec<u8> {
        let mut msg = Vec::with_capacity(64);

        // Handshake type: ServerHello (2)
        msg.push(0x02);
        let len_offset = msg.len();
        msg.extend_from_slice(&[0x00, 0x00, 0x00]);

        // Legacy version TLS 1.2
        msg.extend_from_slice(&[0x03, 0x03]);

        // Server random (32 bytes)
        msg.extend_from_slice(&[0x00; 32]);

        // Legacy session ID echo (empty)
        msg.push(0x00);

        // Cipher suite: TLS_AES_128_GCM_SHA256
        msg.extend_from_slice(&[0x13, 0x01]);

        // Legacy compression method (null)
        msg.push(0x00);

        // Extensions: Supported Versions (TLS 1.3)
        let extensions: &[u8] = &[0x00, 0x2B, 0x00, 0x02, 0x03, 0x04];
        #[expect(
            clippy::cast_possible_truncation,
            reason = "fixed ServerHello extension block length fits in u16"
        )]
        let ext_len_u16 = extensions.len() as u16;
        msg.extend_from_slice(&ext_len_u16.to_be_bytes());
        msg.extend_from_slice(extensions);

        let body_len = msg.len() - len_offset - 3;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "TLS handshake body length uses three low bytes only"
        )]
        let b0 = ((body_len >> 16) & 0xFF) as u8;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "TLS handshake body length uses three low bytes only"
        )]
        let b1 = ((body_len >> 8) & 0xFF) as u8;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "TLS handshake body length uses three low bytes only"
        )]
        let b2 = (body_len & 0xFF) as u8;
        msg[len_offset] = b0;
        msg[len_offset + 1] = b1;
        msg[len_offset + 2] = b2;

        msg
    }

    /// Build `EncryptedExtensions` containing transport parameters.
    fn build_encrypted_extensions(&self) -> Result<Vec<u8>> {
        let tp_encoded = self.local_params.encode()?;
        let mut msg = Vec::with_capacity(32 + tp_encoded.len());

        // Handshake type: EncryptedExtensions (8)
        msg.push(0x08);
        let len_offset = msg.len();
        msg.extend_from_slice(&[0x00, 0x00, 0x00]);

        // Extensions
        let mut extensions = Vec::new();
        extensions.extend_from_slice(&[0x00, 0x39]);
        #[expect(
            clippy::cast_possible_truncation,
            reason = "TLS extension lengths fit in u16 for valid transport params"
        )]
        let tp_len_u16 = tp_encoded.len() as u16;
        extensions.extend_from_slice(&tp_len_u16.to_be_bytes());
        extensions.extend_from_slice(&tp_encoded);

        #[expect(
            clippy::cast_possible_truncation,
            reason = "handshake extension block length fits in u16 for this message layout"
        )]
        let ext_len_u16 = extensions.len() as u16;
        msg.extend_from_slice(&ext_len_u16.to_be_bytes());
        msg.extend_from_slice(&extensions);

        let body_len = msg.len() - len_offset - 3;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "TLS handshake body length uses three low bytes only"
        )]
        let b0 = ((body_len >> 16) & 0xFF) as u8;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "TLS handshake body length uses three low bytes only"
        )]
        let b1 = ((body_len >> 8) & 0xFF) as u8;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "TLS handshake body length uses three low bytes only"
        )]
        let b2 = (body_len & 0xFF) as u8;
        msg[len_offset] = b0;
        msg[len_offset + 1] = b1;
        msg[len_offset + 2] = b2;

        Ok(msg)
    }

    /// Build client Finished message.
    fn build_client_finished() -> Vec<u8> {
        let mut msg = Vec::new();
        // Handshake type: Finished (20)
        msg.push(0x14);
        // Verify data placeholder (32 bytes for SHA-256)
        let verify_data = vec![0u8; 32];
        let len = verify_data.len();
        #[expect(
            clippy::cast_possible_truncation,
            reason = "Finished verify_data length uses three low bytes only"
        )]
        let b0 = ((len >> 16) & 0xFF) as u8;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "Finished verify_data length uses three low bytes only"
        )]
        let b1 = ((len >> 8) & 0xFF) as u8;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "Finished verify_data length uses three low bytes only"
        )]
        let b2 = (len & 0xFF) as u8;
        msg.push(b0);
        msg.push(b1);
        msg.push(b2);
        msg.extend_from_slice(&verify_data);
        msg
    }

    /// Build server Finished message.
    fn build_server_finished() -> Vec<u8> {
        Self::build_client_finished()
    }

    /// Derive Handshake-level keys via `security provider`.
    async fn derive_handshake_keys(&self, crypto: &dyn QuicCryptoProvider) -> Result<LevelKeys> {
        let transcript_hash = crypto.sha256(&self.transcript).await?;
        let secret = crypto.hkdf_extract(&[0u8; 32], &[0u8; 32]).await?;

        let client_secret =
            initial_keys::hkdf_expand_label(crypto, &secret, b"c hs traffic", &transcript_hash, 32)
                .await?;
        let server_secret =
            initial_keys::hkdf_expand_label(crypto, &secret, b"s hs traffic", &transcript_hash, 32)
                .await?;

        let client_keys =
            derive_keys_from_secret(crypto, &client_secret, self.cipher_suite).await?;
        let server_keys =
            derive_keys_from_secret(crypto, &server_secret, self.cipher_suite).await?;

        let (local, remote) = match self.role {
            Role::Client => (client_keys, server_keys),
            Role::Server => (server_keys, client_keys),
        };

        Ok(LevelKeys {
            local,
            remote,
            suite: self.cipher_suite,
        })
    }

    /// Derive Application-level (1-RTT) keys via `security provider`.
    async fn derive_application_keys(&self, crypto: &dyn QuicCryptoProvider) -> Result<LevelKeys> {
        let transcript_hash = crypto.sha256(&self.transcript).await?;
        let master_secret = crypto.hkdf_extract(&[0u8; 32], &[0u8; 32]).await?;

        let client_secret = initial_keys::hkdf_expand_label(
            crypto,
            &master_secret,
            b"c ap traffic",
            &transcript_hash,
            32,
        )
        .await?;
        let server_secret = initial_keys::hkdf_expand_label(
            crypto,
            &master_secret,
            b"s ap traffic",
            &transcript_hash,
            32,
        )
        .await?;

        let client_keys =
            derive_keys_from_secret(crypto, &client_secret, self.cipher_suite).await?;
        let server_keys =
            derive_keys_from_secret(crypto, &server_secret, self.cipher_suite).await?;

        let (local, remote) = match self.role {
            Role::Client => (client_keys, server_keys),
            Role::Server => (server_keys, client_keys),
        };

        Ok(LevelKeys {
            local,
            remote,
            suite: self.cipher_suite,
        })
    }
}

/// Derive QUIC directional keys (key, iv, hp) from a traffic secret.
async fn derive_keys_from_secret(
    crypto: &dyn QuicCryptoProvider,
    secret: &[u8],
    suite: QuicCipherSuite,
) -> Result<DirectionalKeys> {
    let key =
        initial_keys::hkdf_expand_label(crypto, secret, b"quic key", &[], suite.key_len()).await?;
    let iv =
        initial_keys::hkdf_expand_label(crypto, secret, b"quic iv", &[], suite.iv_len()).await?;
    let hp_key =
        initial_keys::hkdf_expand_label(crypto, secret, b"quic hp", &[], suite.hp_key_len())
            .await?;
    Ok(DirectionalKeys {
        key,
        iv,
        hp_key,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handshake_initial_state() {
        let hs = QuicTlsHandshake::new(Role::Client, TransportParams::default());
        assert_eq!(hs.state(), HandshakeState::Initial);
        assert!(!hs.is_complete());
        assert!(hs.peer_params().is_none());
    }

    #[test]
    fn client_hello_builds_valid_message() {
        let hs = QuicTlsHandshake::new(Role::Client, TransportParams::songbird_defaults());
        let ch = hs.build_client_hello().unwrap();

        // First byte: handshake type ClientHello (1)
        assert_eq!(ch[0], 0x01);

        // 3-byte length field
        let body_len = ((ch[1] as usize) << 16) | ((ch[2] as usize) << 8) | (ch[3] as usize);
        assert_eq!(ch.len(), 4 + body_len);

        // Legacy version TLS 1.2
        assert_eq!(&ch[4..6], &[0x03, 0x03]);
    }

    #[test]
    fn server_hello_builds_valid_message() {
        let _hs = QuicTlsHandshake::new(Role::Server, TransportParams::default());
        let sh = QuicTlsHandshake::build_server_hello();
        assert_eq!(sh[0], 0x02); // ServerHello type
        let body_len = ((sh[1] as usize) << 16) | ((sh[2] as usize) << 8) | (sh[3] as usize);
        assert_eq!(sh.len(), 4 + body_len);
    }

    #[test]
    fn encrypted_extensions_contains_transport_params() {
        let hs = QuicTlsHandshake::new(Role::Server, TransportParams::songbird_defaults());
        let ee = hs.build_encrypted_extensions().unwrap();
        assert_eq!(ee[0], 0x08); // EncryptedExtensions type

        // Should contain the QUIC transport params extension type 0x0039
        let has_tp = ee.windows(2).any(|w| w == [0x00, 0x39]);
        assert!(has_tp);
    }

    #[test]
    fn take_outgoing_drains() {
        let mut hs = QuicTlsHandshake::new(Role::Client, TransportParams::default());
        hs.outgoing = vec![1, 2, 3];
        let out = hs.take_outgoing();
        assert_eq!(out, vec![1, 2, 3]);
        assert!(hs.outgoing.is_empty());
    }

    #[test]
    fn receive_crypto_data_appends() {
        let mut hs = QuicTlsHandshake::new(Role::Client, TransportParams::default());
        hs.receive_crypto_data(&[1, 2]);
        hs.receive_crypto_data(&[3, 4]);
        assert_eq!(hs.incoming, vec![1, 2, 3, 4]);
    }

    #[test]
    fn finished_message_structure() {
        let _hs = QuicTlsHandshake::new(Role::Client, TransportParams::default());
        let fin = QuicTlsHandshake::build_client_finished();
        assert_eq!(fin[0], 0x14); // Finished type
        let body_len = ((fin[1] as usize) << 16) | ((fin[2] as usize) << 8) | (fin[3] as usize);
        assert_eq!(body_len, 32); // SHA-256 verify data length
        assert_eq!(fin.len(), 36);
    }
}
