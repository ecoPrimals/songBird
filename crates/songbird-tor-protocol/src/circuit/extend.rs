//! Circuit extension - EXTEND2/EXTENDED2 protocol
//!
//! **Phase 2B**: Circuit building

use crate::crypto::BeardogCryptoClient;
use crate::protocol::{RelayCell, RelayCommand};
use crate::error::{Error, Result};
use crate::circuit::{Circuit, CircuitHop};
use crate::directory::RelayInfo;
use std::net::IpAddr;

/// Circuit extension handler
pub struct CircuitExtender {
    beardog: BeardogCryptoClient,
}

impl CircuitExtender {
    /// Create new circuit extender
    pub fn new(beardog: BeardogCryptoClient) -> Self {
        Self { beardog }
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
    pub fn create_extend2(
        &self,
        _circuit: &Circuit,
        next_relay: &RelayInfo,
    ) -> Result<(RelayCell, super::create::HandshakeState)> {
        // 1. Generate ephemeral X25519 keypair via BearDog
        let client_ephemeral = self.beardog.x25519_generate_ephemeral()?;

        // 2. Construct EXTEND2 relay cell payload
        let mut payload = Vec::new();
        
        // Link specifiers (simplified)
        payload.push(2); // Number of link specifiers
        
        // IPv4 address specifier (type 0)
        payload.push(0); // Type: IPv4
        payload.push(6); // Length: 6 bytes
        if let IpAddr::V4(ipv4) = next_relay.address {
            payload.extend_from_slice(&ipv4.octets());
        } else {
            return Err(Error::Protocol("IPv6 not yet supported for EXTEND2".to_string()));
        }
        payload.extend_from_slice(&next_relay.or_port.to_be_bytes());
        
        // Identity fingerprint specifier (type 2)
        payload.push(2); // Type: Ed25519 ID
        payload.push(32); // Length: 32 bytes
        // TODO: Get Ed25519 identity from relay descriptor
        // For now, use SHA1 fingerprint padded/truncated to 32 bytes
        let mut ed25519_id = [0u8; 32];
        ed25519_id[..20.min(next_relay.fingerprint.len())]
            .copy_from_slice(&next_relay.fingerprint[..20.min(next_relay.fingerprint.len())]);
        payload.extend_from_slice(&ed25519_id);
        
        // Handshake type (ntor)
        payload.extend_from_slice(&[0x00, 0x02]); // Type: ntor (0x0002)
        
        // Handshake data length
        payload.extend_from_slice(&[0x00, 0x54]); // Length: 84 bytes
        
        // Handshake data (same as CREATE2)
        payload.extend_from_slice(&ed25519_id);               // 32 bytes - relay identity
        payload.extend_from_slice(&ed25519_id);               // 32 bytes - relay ntor key (TODO: from descriptor)
        payload.extend_from_slice(&client_ephemeral.public_key); // 32 bytes

        // 3. Save state for EXTENDED2 processing
        let state = super::create::HandshakeState {
            client_ephemeral_secret: client_ephemeral.secret_key,
            client_ephemeral_public: client_ephemeral.public_key,
            relay_identity: ed25519_id,
            relay_ntor_key: ed25519_id, // TODO: Get from descriptor
        };

        // 4. Create RELAY_EARLY cell (EXTEND2)
        let relay_cell = RelayCell {
            command: RelayCommand::Extend,
            recognized: 0,
            stream_id: 0,
            digest: [0u8; 4], // TODO: Calculate digest
            length: payload.len() as u16,
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
    /// * New CircuitHop to add to circuit
    pub fn process_extended2(
        &self,
        _circuit: &Circuit,
        state: super::create::HandshakeState,
        response: &RelayCell,
        next_relay: RelayInfo,
    ) -> Result<CircuitHop> {
        // Validate response
        if response.command != RelayCommand::Extended {
            return Err(Error::Protocol(format!(
                "Expected EXTENDED response, got {:?}",
                response.command
            )));
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
    use crate::directory::RelayFlags;

    #[test]
    fn test_circuit_extender_creation() {
        let beardog = BeardogCryptoClient::from_env()
            .expect("Failed to create BearDog client");
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
