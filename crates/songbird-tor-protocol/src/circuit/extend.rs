// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Circuit extension - EXTEND2/EXTENDED2 protocol
//!
//! **Phase 2B**: Circuit building

use crate::circuit::{Circuit, CircuitHop};
use crate::crypto::BeardogCryptoClient;
use crate::directory::RelayInfo;
use crate::error::{Error, Result};
use crate::protocol::{RelayCell, RelayCommand};
use std::net::IpAddr;

/// Circuit extension handler
pub struct CircuitExtender {
    beardog: BeardogCryptoClient,
}

impl CircuitExtender {
    /// Create new circuit extender
    #[must_use]
    pub const fn new(beardog: BeardogCryptoClient) -> Self {
        Self {
            beardog,
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
    /// Returns error if relay has no `ntor_key` or `BearDog` crypto fails.
    pub fn create_extend2(
        &self,
        _circuit: &Circuit,
        next_relay: &RelayInfo,
    ) -> Result<(RelayCell, super::create::HandshakeState)> {
        // Require ntor key for handshake
        let relay_ntor_key = next_relay.ntor_key.ok_or_else(|| {
            Error::Protocol(format!("Relay {} has no ntor_key", next_relay.nickname))
        })?;

        // 1. Generate ephemeral X25519 keypair via BearDog
        let client_ephemeral = self.beardog.x25519_generate_ephemeral()?;

        // 2. Construct EXTEND2 relay cell payload
        let mut payload = Vec::new();

        // Link specifiers (2 specifiers: IPv4 address + RSA identity)
        payload.push(2); // Number of link specifiers

        // Link specifier type 0: IPv4 address (6 bytes: IP + port)
        payload.push(0); // Type: IPv4
        payload.push(6); // Length: 6 bytes
        if let IpAddr::V4(ipv4) = next_relay.address {
            payload.extend_from_slice(&ipv4.octets());
        } else {
            return Err(Error::Protocol("IPv6 not yet supported for EXTEND2".to_string()));
        }
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
    pub fn process_extended2(
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
        let ntor = super::NtorHandshake::new(self.beardog.clone());
        let key_material = ntor.complete_handshake(state, handshake_response)?;

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
    use super::*;

    #[test]
    fn test_circuit_extender_creation() {
        let beardog = BeardogCryptoClient::from_env().expect("Failed to create BearDog client");
        let _extender = CircuitExtender::new(beardog);

        // Test passes if it creates successfully
    }

    #[test]
    fn test_relay_command_extend() {
        use crate::protocol::RelayCommand;
        assert_eq!(RelayCommand::Extend as u8, 6);
        assert_eq!(RelayCommand::Extended as u8, 7);
    }
}
