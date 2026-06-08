// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! TLS 1.3 Handshake State Machine
//!
//! Manages the TLS handshake protocol state transitions.

use crate::crypto::SecurityTlsCryptoClient;
use crate::error::{Result, TlsError};
use crate::key_schedule::KeySchedule;
use crate::messages::{ClientHello, ServerHello};

/// Handshake state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandshakeState {
    /// Initial state - waiting for `ClientHello`
    Start,

    /// Received `ClientHello` - ready to send `ServerHello`
    ReceivedClientHello,

    /// Sent `ServerHello` - handshake in progress
    SentServerHello,

    /// Handshake complete - ready for application data
    Connected,

    /// Error state
    Error,
}

/// Handshake state machine
///
/// Coordinates TLS 1.3 handshake protocol.
pub struct HandshakeStateMachine {
    /// Current state
    state: HandshakeState,

    /// Key schedule for key derivation
    key_schedule: KeySchedule,

    /// Crypto client for security-provider delegation
    crypto_client: Option<SecurityTlsCryptoClient>,

    /// Cached `ClientHello` (for transcript)
    client_hello: Option<ClientHello>,

    /// Cached `ServerHello` (for transcript)
    server_hello: Option<ServerHello>,
}

impl HandshakeStateMachine {
    /// Create a new handshake state machine
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: HandshakeState::Start,
            key_schedule: KeySchedule::new(),
            crypto_client: None,
            client_hello: None,
            server_hello: None,
        }
    }

    /// Set the security-provider crypto client
    pub fn set_crypto_client(&mut self, client: SecurityTlsCryptoClient) {
        self.key_schedule.set_crypto_client(client.clone());
        self.crypto_client = Some(client);
    }

    /// Get current state
    #[must_use]
    pub const fn state(&self) -> HandshakeState {
        self.state
    }

    /// Is handshake complete?
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.state == HandshakeState::Connected
    }

    /// Process received `ClientHello`
    ///
    /// # Errors
    ///
    /// Returns an error if not in Start state or `ClientHello` validation fails.
    pub fn process_client_hello(&mut self, client_hello: ClientHello) -> Result<()> {
        if self.state != HandshakeState::Start {
            return Err(TlsError::UnexpectedMessage {
                expected: "Start".to_string(),
                got: "ClientHello".to_string(),
            });
        }

        // Validate ClientHello
        client_hello.validate()?;

        // Store for transcript
        self.client_hello = Some(client_hello);

        // Transition state
        self.state = HandshakeState::ReceivedClientHello;

        Ok(())
    }

    /// Generate `ServerHello` response
    ///
    /// # Errors
    ///
    /// Returns an error if not in `ReceivedClientHello` state, crypto operations fail,
    /// or validation fails.
    ///
    /// # Panics
    ///
    /// May panic if system time is before `UNIX_EPOCH` (used for server random fallback).
    pub async fn generate_server_hello(&mut self) -> Result<ServerHello> {
        if self.state != HandshakeState::ReceivedClientHello {
            return Err(TlsError::ProtocolError(
                "Cannot generate ServerHello in current state".to_string(),
            ));
        }

        let client_hello = self
            .client_hello
            .as_ref()
            .ok_or_else(|| TlsError::InternalError("ClientHello not stored".to_string()))?;

        // Server random (32 bytes): OS CSPRNG — no security-provider delegation (RFC 8446)
        let mut server_random = [0u8; 32];
        getrandom::fill(&mut server_random)
            .map_err(|e| TlsError::CryptoError(format!("RNG failed: {e}")))?;

        let crypto = self
            .crypto_client
            .as_ref()
            .ok_or_else(|| TlsError::InternalError("Crypto client not set".to_string()))?;

        // Select cipher suite (for now, just take first supported)
        let cipher_suite = client_hello
            .cipher_suites
            .first()
            .copied()
            .ok_or_else(|| TlsError::HandshakeFailure("No cipher suites".to_string()))?;

        // Echo session ID
        let session_id_echo = client_hello.legacy_session_id.clone();

        // Generate X25519 ephemeral keypair for key exchange
        let (server_public_key, server_secret_key) = crypto.x25519_generate_ephemeral().await?;

        // Store secret key for later shared secret derivation
        self.key_schedule.set_server_secret_key(server_secret_key);

        // Create ServerHello extensions with REAL key share
        let extensions = vec![
            crate::messages::Extension::SupportedVersions(vec![0x0304]), // TLS 1.3
            crate::messages::Extension::KeyShare(server_public_key),     // Real X25519 public key
        ];

        let server_hello =
            ServerHello::new(server_random, session_id_echo, cipher_suite, extensions);

        // Validate
        server_hello.validate()?;

        // Store for transcript
        self.server_hello = Some(server_hello.clone());

        // Transition state
        self.state = HandshakeState::SentServerHello;

        Ok(server_hello)
    }

    /// Complete handshake (after Finished messages exchanged)
    ///
    /// # Errors
    ///
    /// Returns an error if not in `SentServerHello` state.
    pub fn complete_handshake(&mut self) -> Result<()> {
        if self.state != HandshakeState::SentServerHello {
            return Err(TlsError::ProtocolError(
                "Cannot complete handshake in current state".to_string(),
            ));
        }

        // Transition to connected
        self.state = HandshakeState::Connected;

        Ok(())
    }

    /// Get key schedule (for key derivation)
    #[must_use]
    pub const fn key_schedule(&self) -> &KeySchedule {
        &self.key_schedule
    }

    /// Get mutable key schedule
    pub const fn key_schedule_mut(&mut self) -> &mut KeySchedule {
        &mut self.key_schedule
    }
}

impl Default for HandshakeStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;

