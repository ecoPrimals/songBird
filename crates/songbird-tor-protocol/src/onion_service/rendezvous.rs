// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Rendezvous protocol
//!
//! **Phase 2D**: Onion Service

use crate::protocol::RelayCell;

/// Rendezvous point for connecting client and service
#[derive(Debug, Clone)]
pub struct RendezvousPoint {
    /// Relay identity
    pub relay_identity: [u8; 32],

    /// Rendezvous cookie (20 bytes, chosen by client)
    pub cookie: [u8; 20],

    /// Circuit ID to rendezvous point
    pub circuit_id: u32,
}

impl RendezvousPoint {
    /// Create RENDEZVOUS1 cell (service -> rendezvous point)
    ///
    /// Sent by service to complete the rendezvous handshake.
    ///
    /// Cell format (Tor spec):
    /// ```text
    /// RENDEZVOUS_COOKIE  [20 bytes] - cookie from INTRODUCE2
    /// HANDSHAKE_INFO     [variable] - ntor handshake response (typically 64 bytes)
    /// ```
    #[must_use]
    pub fn create_rendezvous1(&self, handshake_data: &[u8]) -> RelayCell {
        let mut data = Vec::with_capacity(20 + handshake_data.len());
        data.extend_from_slice(&self.cookie);
        data.extend_from_slice(handshake_data);

        RelayCell {
            command: crate::protocol::RelayCommand::Rendezvous1,
            recognized: 0,
            stream_id: 0,
            digest: [0u8; 4],
            length: u16::try_from(data.len()).expect("data length fits in u16"),
            data,
        }
    }

    /// Parse RENDEZVOUS2 cell (rendezvous point -> client)
    ///
    /// Received by client after service completes the rendezvous.
    /// Contains the service's ntor handshake response.
    ///
    /// Cell format (Tor spec):
    /// ```text
    /// HANDSHAKE_INFO  [variable] - ntor handshake response from service
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if cell command is not RENDEZVOUS2.
    pub fn parse_rendezvous2(cell: &RelayCell) -> crate::error::Result<RendezvousResponse> {
        if cell.command != crate::protocol::RelayCommand::Rendezvous2 {
            return Err(crate::error::Error::Protocol(format!(
                "Expected RENDEZVOUS2 command, got {:?}",
                cell.command
            )));
        }

        Ok(RendezvousResponse {
            handshake_data: cell.data.clone(),
        })
    }

    /// Create `ESTABLISH_RENDEZVOUS` cell (client -> rendezvous point)
    ///
    /// Sent by client to designate a relay as its rendezvous point.
    /// The cookie is used to correlate the rendezvous with the introduction.
    ///
    /// Cell format (Tor spec):
    /// ```text
    /// RENDEZVOUS_COOKIE  [20 bytes] - random cookie chosen by client
    /// ```
    #[must_use]
    pub fn create_establish_rendezvous(cookie: &[u8; 20]) -> RelayCell {
        RelayCell {
            command: crate::protocol::RelayCommand::Rendezvous1,
            recognized: 0,
            stream_id: 0,
            digest: [0u8; 4],
            length: 20,
            data: cookie.to_vec(),
        }
    }

    /// Create INTRODUCE1 cell (client -> introduction point)
    ///
    /// Sent by client to the introduction point to request connection
    /// to the onion service. The intro point forwards this as INTRODUCE2.
    ///
    /// Cell format (Tor proposal 224):
    /// ```text
    /// LEGACY_KEY_ID     [20 bytes] - zeros for v3
    /// AUTH_KEY_TYPE      [1 byte]  - 0x02 = Ed25519
    /// AUTH_KEY_LEN       [2 bytes]
    /// AUTH_KEY           [32 bytes] - service's auth key
    /// N_EXTENSIONS       [1 byte]
    /// ENCRYPTED          [variable] - ntor-encrypted:
    ///   RENDEZVOUS_POINT [32 bytes]
    ///   RENDEZVOUS_COOKIE[20 bytes]
    ///   CLIENT_PK        [32 bytes] - X25519 ephemeral
    /// ```
    #[must_use]
    pub fn create_introduce1(
        rendezvous_point: &[u8; 32],
        cookie: &[u8; 20],
        client_public_key: &[u8; 32],
    ) -> RelayCell {
        let mut data = Vec::with_capacity(140);

        // LEGACY_KEY_ID (20 bytes, zeros for v3)
        data.extend_from_slice(&[0u8; 20]);

        // AUTH_KEY_TYPE: Ed25519
        data.push(0x02);

        // AUTH_KEY_LEN: 32
        data.extend_from_slice(&32u16.to_be_bytes());

        // AUTH_KEY: placeholder (in production, this is the service's intro auth key)
        data.extend_from_slice(&[0u8; 32]);

        // N_EXTENSIONS: 0
        data.push(0u8);

        // ENCRYPTED section (in production, this is ntor-encrypted)
        // For now, include plaintext for testing; BearDog will encrypt in production
        data.extend_from_slice(rendezvous_point);
        data.extend_from_slice(cookie);
        data.extend_from_slice(client_public_key);

        RelayCell {
            command: crate::protocol::RelayCommand::Introduce1,
            recognized: 0,
            stream_id: 0,
            digest: [0u8; 4],
            length: u16::try_from(data.len()).expect("data length fits in u16"),
            data,
        }
    }
}

/// Rendezvous response from service
#[derive(Debug, Clone)]
pub struct RendezvousResponse {
    /// Handshake data from service (ntor)
    pub handshake_data: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rendezvous_point_creation() {
        let rp = RendezvousPoint {
            relay_identity: [1u8; 32],
            cookie: [2u8; 20],
            circuit_id: 42,
        };

        assert_eq!(rp.circuit_id, 42);
        assert_eq!(rp.cookie[0], 2);
    }

    #[test]
    fn test_rendezvous1_cell() {
        let rp = RendezvousPoint {
            relay_identity: [1u8; 32],
            cookie: [2u8; 20],
            circuit_id: 42,
        };

        let handshake = vec![0u8; 64];
        let cell = rp.create_rendezvous1(&handshake);

        assert_eq!(cell.command, crate::protocol::RelayCommand::Rendezvous1);
        assert_eq!(cell.data.len(), 84); // 20 (cookie) + 64 (handshake)
    }

    #[test]
    fn test_establish_rendezvous() {
        let cookie = [5u8; 20];
        let cell = RendezvousPoint::create_establish_rendezvous(&cookie);

        assert_eq!(cell.data.len(), 20);
        assert_eq!(cell.data, cookie);
    }

    #[test]
    fn test_introduce1_cell() {
        let rp = [1u8; 32];
        let cookie = [2u8; 20];
        let client_key = [3u8; 32];

        let cell = RendezvousPoint::create_introduce1(&rp, &cookie, &client_key);

        assert_eq!(cell.command, crate::protocol::RelayCommand::Introduce1);
        // 20 (legacy_key_id) + 1 (auth_type) + 2 (auth_len) + 32 (auth_key) +
        // 1 (n_ext) + 32 (rp) + 20 (cookie) + 32 (client_pk) = 140
        assert_eq!(cell.data.len(), 140);

        // Verify LEGACY_KEY_ID is zeros
        assert_eq!(&cell.data[0..20], &[0u8; 20]);
        // Verify AUTH_KEY_TYPE is Ed25519 (0x02)
        assert_eq!(cell.data[20], 0x02);
        // Verify the rendezvous point is embedded (after header)
        let payload_start = 20 + 1 + 2 + 32 + 1; // = 56
        assert_eq!(&cell.data[payload_start..payload_start + 32], &rp);
        assert_eq!(&cell.data[payload_start + 32..payload_start + 52], &cookie);
        assert_eq!(&cell.data[payload_start + 52..payload_start + 84], &client_key);
    }

    #[test]
    fn test_rendezvous_response() {
        let response = RendezvousResponse {
            handshake_data: vec![0u8; 64],
        };

        assert_eq!(response.handshake_data.len(), 64);
    }
}
