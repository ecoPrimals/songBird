// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Circuit extension - EXTEND2/EXTENDED2 protocol
//!
//! **Phase 2B**: Circuit building

use crate::circuit::{Circuit, CircuitHop};
use crate::crypto::TorProtocolCrypto;
use crate::directory::RelayInfo;
use crate::error::{Error, Result};
use crate::protocol::{RelayCell, RelayCommand};
use songbird_crypto_provider::CryptoProvider;
use std::net::IpAddr;

/// Circuit extension handler
pub struct CircuitExtender {
    security_provider: CryptoProvider,
}

impl CircuitExtender {
    /// Create new circuit extender
    #[must_use]
    pub const fn new(security_provider: CryptoProvider) -> Self {
        Self {
            security_provider,
        }
    }

    /// Extend circuit by one hop
    ///
    /// # Arguments
    /// * `circuit` - Current circuit to extend
    /// * `next_relay` - Relay to add as next hop
    ///
    /// # Returns
    /// * EXTEND2 relay cell to send
    /// * Handshake state for processing EXTENDED2 response
    ///
    /// # Errors
    /// Returns error if relay has no `ntor_key` or `security provider` crypto fails.
    pub async fn create_extend2(
        &self,
        _circuit: &Circuit,
        next_relay: &RelayInfo,
    ) -> Result<(RelayCell, super::create::HandshakeState)> {
        // Require ntor key for handshake
        let relay_ntor_key = next_relay.ntor_key.ok_or_else(|| {
            Error::Protocol(format!("Relay {} has no ntor_key", next_relay.nickname))
        })?;

        // Reject IPv6 before any security-provider I/O (handshake is IPv4 link specifier only).
        if !matches!(next_relay.address, IpAddr::V4(_)) {
            return Err(Error::Protocol("IPv6 not yet supported for EXTEND2".to_string()));
        }

        // 1. Generate ephemeral X25519 keypair via security provider
        let client_ephemeral = self.security_provider.x25519_generate_ephemeral().await?;

        // 2. Construct EXTEND2 relay cell payload
        let mut payload = Vec::new();

        // Link specifiers (2 specifiers: IPv4 address + RSA identity)
        payload.push(2); // Number of link specifiers

        // Link specifier type 0: IPv4 address (6 bytes: IP + port)
        payload.push(0); // Type: IPv4
        payload.push(6); // Length: 6 bytes
        let IpAddr::V4(ipv4) = next_relay.address else {
            unreachable!("IPv6 rejected above");
        };
        payload.extend_from_slice(&ipv4.octets());
        payload.extend_from_slice(&next_relay.or_port.to_be_bytes());

        // Link specifier type 2: Legacy RSA identity (20-byte fingerprint)
        payload.push(2); // Type: RSA identity
        payload.push(20); // Length: 20 bytes
        payload.extend_from_slice(&next_relay.fingerprint);

        // Handshake type (ntor = 0x0002)
        payload.extend_from_slice(&[0x00, 0x02]); // Type: ntor

        // Handshake data length: 84 bytes (20 + 32 + 32)
        payload.extend_from_slice(&84u16.to_be_bytes()); // Length: 84 bytes

        // Handshake data (same as CREATE2 ntor format)
        // Format: ID (20 bytes) || B (32 bytes) || X (32 bytes)
        payload.extend_from_slice(&next_relay.fingerprint); // 20 bytes - node ID
        payload.extend_from_slice(&relay_ntor_key); // 32 bytes - relay ntor key
        payload.extend_from_slice(&client_ephemeral.public_key); // 32 bytes - client ephemeral

        // 3. Save state for EXTENDED2 processing
        let state = super::create::HandshakeState {
            client_ephemeral_secret: client_ephemeral.secret_key,
            client_ephemeral_public: client_ephemeral.public_key,
            node_id: next_relay.fingerprint,
            relay_ntor_key,
        };

        // 4. Create RELAY_EARLY cell (EXTEND2)
        let relay_cell = RelayCell {
            command: RelayCommand::Extend,
            recognized: 0,
            stream_id: 0,
            digest: [0u8; 4], // Populated by OnionCrypto before encryption
            length: u16::try_from(payload.len())
                .map_err(|_| Error::Protocol("EXTEND2 payload too long".to_string()))?,
            data: payload,
        };

        Ok((relay_cell, state))
    }

    /// Process EXTENDED2 response
    ///
    /// # Arguments
    /// * `circuit` - Current circuit
    /// * `state` - Handshake state from `create_extend2()`
    /// * `response` - EXTENDED2 relay cell
    ///
    /// # Returns
    /// * New `CircuitHop` to add to circuit
    ///
    /// # Errors
    /// Returns error if response is invalid or handshake fails.
    pub async fn process_extended2(
        &self,
        _circuit: &Circuit,
        state: &super::create::HandshakeState,
        response: &RelayCell,
        next_relay: RelayInfo,
    ) -> Result<CircuitHop> {
        // Validate response
        if response.command != RelayCommand::Extended {
            return Err(Error::Protocol(format!("Expected EXTENDED response, got {response:?}")));
        }

        // Extract handshake response (skip 2-byte length prefix)
        if response.data.len() < 66 {
            return Err(Error::Protocol(format!(
                "EXTENDED2 response too short: {} bytes",
                response.data.len()
            )));
        }

        let handshake_response = &response.data[2..66]; // 64 bytes

        // Complete handshake using ntor
        let ntor = super::NtorHandshake::new(self.security_provider.clone());
        let key_material = ntor.complete_handshake(state, handshake_response).await?;

        // Create circuit hop
        Ok(CircuitHop::new(
            next_relay,
            key_material.forward_digest,
            key_material.backward_digest,
            key_material.forward_key,
            key_material.backward_key,
        ))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::circuit::{Circuit, CircuitPurpose, HandshakeState};
    use crate::directory::{RelayFlags, RelayInfo};
    use crate::error::Error;
    use crate::protocol::{RelayCell, RelayCommand};
    use std::net::{IpAddr, Ipv6Addr};

    fn ipv4_relay(ntor: Option<[u8; 32]>) -> RelayInfo {
        RelayInfo {
            nickname: "hop".to_string(),
            fingerprint: [0x5Au8; 20],
            address: IpAddr::from([192, 0, 2, 1]),
            or_port: 443,
            dir_port: None,
            flags: RelayFlags::empty(),
            bandwidth: 1,
            ntor_key: ntor,
            version: None,
        }
    }

    #[test]
    fn circuit_extender_new_does_not_panic() {
        let ext = CircuitExtender::new(CryptoProvider::new(
            "/tmp/songbird-tor-protocol-circuit-extender.sock".to_string(),
        ));
        assert_eq!(std::mem::size_of_val(&ext), std::mem::size_of::<CircuitExtender>());
    }

    #[test]
    fn relay_command_extend_wire_values() {
        assert_eq!(RelayCommand::Extend as u8, 6);
        assert_eq!(RelayCommand::Extended as u8, 7);
    }

    #[tokio::test]
    async fn create_extend2_errors_without_ntor_key_before_any_socket_io() {
        let ext = CircuitExtender::new(CryptoProvider::new(
            "/tmp/songbird-tor-protocol-circuit-extender.sock".to_string(),
        ));
        let circuit = Circuit::new(0x8000_0001, CircuitPurpose::General);
        let relay = ipv4_relay(None);
        let err = ext.create_extend2(&circuit, &relay).await.expect_err("missing ntor key");
        assert!(
            matches!(err, Error::Protocol(ref s) if s.contains("ntor_key")),
            "unexpected err: {err:?}"
        );
    }

    #[tokio::test]
    async fn create_extend2_rejects_ipv6_even_with_ntor_key() {
        let ext = CircuitExtender::new(CryptoProvider::new(
            "/tmp/songbird-tor-protocol-circuit-extender.sock".to_string(),
        ));
        let circuit = Circuit::new(1, CircuitPurpose::General);
        let relay = RelayInfo {
            address: IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1)),
            ntor_key: Some([0x11u8; 32]),
            ..ipv4_relay(None)
        };
        assert!(relay.address.is_ipv6());
        let err = ext.create_extend2(&circuit, &relay).await.expect_err("ipv6");
        assert!(
            matches!(err, Error::Protocol(ref s) if s.contains("IPv6")),
            "unexpected err: {err:?}"
        );
    }

    #[tokio::test]
    async fn process_extended2_rejects_non_extended_relay_command() {
        let ext = CircuitExtender::new(CryptoProvider::new(
            "/tmp/songbird-tor-protocol-circuit-extender.sock".to_string(),
        ));
        let circuit = Circuit::new(1, CircuitPurpose::General);
        let state = HandshakeState {
            client_ephemeral_secret: [1u8; 32],
            client_ephemeral_public: [2u8; 32],
            node_id: [3u8; 20],
            relay_ntor_key: [4u8; 32],
        };
        let response = RelayCell {
            command: RelayCommand::Data,
            recognized: 0,
            stream_id: 1,
            digest: [0u8; 4],
            length: 0,
            data: vec![],
        };
        let err = ext
            .process_extended2(&circuit, &state, &response, ipv4_relay(Some([0u8; 32])))
            .await
            .expect_err("wrong command");
        assert!(matches!(err, Error::Protocol(_)));
    }

    #[tokio::test]
    async fn process_extended2_rejects_short_handshake_blob() {
        let ext = CircuitExtender::new(CryptoProvider::new(
            "/tmp/songbird-tor-protocol-circuit-extender.sock".to_string(),
        ));
        let circuit = Circuit::new(1, CircuitPurpose::General);
        let state = HandshakeState {
            client_ephemeral_secret: [1u8; 32],
            client_ephemeral_public: [2u8; 32],
            node_id: [3u8; 20],
            relay_ntor_key: [4u8; 32],
        };
        let response = RelayCell {
            command: RelayCommand::Extended,
            recognized: 0,
            stream_id: 0,
            digest: [0u8; 4],
            length: 10,
            data: vec![0u8; 10],
        };
        let err = ext
            .process_extended2(&circuit, &state, &response, ipv4_relay(Some([0u8; 32])))
            .await
            .expect_err("short extended");
        assert!(
            matches!(err, Error::Protocol(ref s) if s.contains("too short")),
            "unexpected err: {err:?}"
        );
    }
}
