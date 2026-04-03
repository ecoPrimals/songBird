// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Introduction point protocol
//!
//! **Phase 2D**: Onion Service

use crate::protocol::RelayCell;

/// Introduction point
#[derive(Debug, Clone)]
pub struct IntroductionPoint {
    /// Relay identity (Ed25519)
    pub relay_identity: [u8; 32],

    /// Relay onion key (X25519)
    pub onion_key: [u8; 32],

    /// Service-side introduction auth key
    pub service_key: [u8; 32],

    /// Circuit ID to this intro point
    pub circuit_id: u32,
}

/// Auth key type in `ESTABLISH_INTRO` cells
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthKeyType {
    /// Ed25519 key
    Ed25519 = 2,
}

impl IntroductionPoint {
    /// Create `ESTABLISH_INTRO` cell
    ///
    /// Sent by service to introduction point to establish it as an intro point.
    ///
    /// Cell format (Tor proposal 224):
    /// ```text
    /// AUTH_KEY_TYPE    [1 byte]  - 0x02 = Ed25519
    /// AUTH_KEY_LEN     [2 bytes] - 32 for Ed25519
    /// AUTH_KEY         [32 bytes]
    /// N_EXTENSIONS     [1 byte]  - number of extensions
    /// [extensions]     [variable]
    /// HANDSHAKE_AUTH   [32 bytes] - MAC over cell body (placeholder until security provider)
    /// SIG_LEN          [2 bytes]
    /// SIG              [64 bytes] - Ed25519 signature (placeholder until security provider)
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the cell payload length does not fit in `u16`.
    #[must_use]
    pub fn create_establish_intro(&self) -> RelayCell {
        let mut data = Vec::with_capacity(136);

        // AUTH_KEY_TYPE: Ed25519
        data.push(AuthKeyType::Ed25519 as u8);

        // AUTH_KEY_LEN: 32
        data.extend_from_slice(&32u16.to_be_bytes());

        // AUTH_KEY: service authentication key
        data.extend_from_slice(&self.service_key);

        // N_EXTENSIONS: 0 (no extensions)
        data.push(0u8);

        // HANDSHAKE_AUTH: 32-byte MAC (needs security provider HMAC-SHA256)
        // For now, zero-filled — security provider will compute this at runtime
        data.extend_from_slice(&[0u8; 32]);

        // SIG_LEN: 64
        data.extend_from_slice(&64u16.to_be_bytes());

        // SIG: Ed25519 signature (needs security provider)
        // For now, zero-filled — security provider will sign at runtime
        data.extend_from_slice(&[0u8; 64]);

        RelayCell {
            command: crate::protocol::RelayCommand::Introduce1,
            recognized: 0,
            stream_id: 0,
            digest: [0u8; 4],
            length: u16::try_from(data.len()).expect("cell data length fits in u16"),
            data,
        }
    }

    /// Parse INTRODUCE2 cell
    ///
    /// Sent from intro point when a client wants to connect.
    /// Contains rendezvous point info and encrypted data for service.
    ///
    /// Cell format (Tor proposal 224):
    /// ```text
    /// LEGACY_KEY_ID   [20 bytes] - all zeros for v3
    /// AUTH_KEY_TYPE    [1 byte]
    /// AUTH_KEY_LEN     [2 bytes]
    /// AUTH_KEY         [32 bytes]
    /// N_EXTENSIONS     [1 byte]
    /// [extensions]
    /// ENCRYPTED        [variable] - ntor-encrypted handshake + rendezvous info
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if cell is too short or malformed.
    pub fn parse_introduce2(cell: &RelayCell) -> crate::error::Result<IntroductionRequest> {
        let data = &cell.data;

        // Minimum size check: 20 (legacy) + 1 (type) + 2 (len) + 32 (key) + 1 (ext) = 56
        if data.len() < 56 {
            return Err(crate::error::Error::Protocol(format!(
                "INTRODUCE2 cell too short: {} bytes (min 56)",
                cell.data.len()
            )));
        }

        // Skip LEGACY_KEY_ID (20 bytes, all zeros for v3) and AUTH_KEY_TYPE (1 byte)
        let data = &data[21..];

        // AUTH_KEY_LEN
        let auth_key_len = u16::from_be_bytes([data[0], data[1]]) as usize;

        // AUTH_KEY
        let auth_key_end = 2 + auth_key_len;
        if data.len() < auth_key_end + 1 {
            return Err(crate::error::Error::Protocol(
                "INTRODUCE2 cell truncated at auth key".to_string(),
            ));
        }

        // N_EXTENSIONS
        let n_ext = data[auth_key_end] as usize;
        let mut pos = auth_key_end + 1;

        // Skip extensions
        for _ in 0..n_ext {
            if pos + 4 > data.len() {
                return Err(crate::error::Error::Protocol(
                    "INTRODUCE2 cell truncated in extensions".to_string(),
                ));
            }
            let ext_len = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;
            pos += 4 + ext_len;
        }

        // Remaining is the encrypted handshake data
        // When security provider is available, this would be decrypted to reveal:
        // - rendezvous_point identity
        // - rendezvous_cookie
        // - client_public_key (X25519)
        //
        // For now, extract what we can from unencrypted fields
        // In production, security provider decrypts the ENCRYPTED section
        let encrypted = if pos < data.len() {
            &data[pos..]
        } else {
            &[]
        };

        // The encrypted section, once decrypted, contains:
        // RENDEZVOUS_POINT [32 bytes]
        // RENDEZVOUS_COOKIE [20 bytes]
        // CLIENT_PK [32 bytes]
        // If we have enough data (from test or pre-decrypted cells):
        if encrypted.len() >= 84 {
            let mut rendezvous_point = [0u8; 32];
            rendezvous_point.copy_from_slice(&encrypted[0..32]);

            let mut rendezvous_cookie = [0u8; 20];
            rendezvous_cookie.copy_from_slice(&encrypted[32..52]);

            let mut client_public_key = [0u8; 32];
            client_public_key.copy_from_slice(&encrypted[52..84]);

            Ok(IntroductionRequest {
                rendezvous_point,
                rendezvous_cookie,
                client_public_key,
            })
        } else {
            // Encrypted section not yet decrypted — return placeholder
            // security provider will handle decryption in production
            Ok(IntroductionRequest {
                rendezvous_point: [0u8; 32],
                rendezvous_cookie: [0u8; 20],
                client_public_key: [0u8; 32],
            })
        }
    }
}

/// Introduction request from client
#[derive(Debug, Clone)]
pub struct IntroductionRequest {
    /// Rendezvous point relay identity
    pub rendezvous_point: [u8; 32],

    /// Rendezvous cookie (chosen by client)
    pub rendezvous_cookie: [u8; 20],

    /// Client's ephemeral public key (for ntor)
    pub client_public_key: [u8; 32],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_introduction_point_creation() {
        let intro = IntroductionPoint {
            relay_identity: [1u8; 32],
            onion_key: [2u8; 32],
            service_key: [3u8; 32],
            circuit_id: 42,
        };

        assert_eq!(intro.circuit_id, 42);
        assert_eq!(intro.relay_identity[0], 1);
    }

    #[test]
    fn test_establish_intro_cell() {
        let intro = IntroductionPoint {
            relay_identity: [1u8; 32],
            onion_key: [2u8; 32],
            service_key: [3u8; 32],
            circuit_id: 42,
        };

        let cell = intro.create_establish_intro();
        assert_eq!(cell.stream_id, 0);

        // Verify cell structure:
        // AUTH_KEY_TYPE (1) + AUTH_KEY_LEN (2) + AUTH_KEY (32) +
        // N_EXTENSIONS (1) + HANDSHAKE_AUTH (32) + SIG_LEN (2) + SIG (64) = 134
        assert_eq!(cell.data.len(), 134);
        assert_eq!(cell.length, 134);

        // Verify AUTH_KEY_TYPE = Ed25519 (0x02)
        assert_eq!(cell.data[0], AuthKeyType::Ed25519 as u8);

        // Verify AUTH_KEY_LEN = 32
        assert_eq!(u16::from_be_bytes([cell.data[1], cell.data[2]]), 32);

        // Verify AUTH_KEY = service_key
        assert_eq!(&cell.data[3..35], &[3u8; 32]);

        // Verify N_EXTENSIONS = 0
        assert_eq!(cell.data[35], 0);
    }

    #[test]
    fn test_introduction_request() {
        let request = IntroductionRequest {
            rendezvous_point: [1u8; 32],
            rendezvous_cookie: [2u8; 20],
            client_public_key: [3u8; 32],
        };

        assert_eq!(request.rendezvous_cookie[0], 2);
        assert_eq!(request.client_public_key.len(), 32);
    }

    #[test]
    fn test_parse_introduce2_with_plaintext_payload() {
        // Create an INTRODUCE2 cell with unencrypted payload (for testing)
        let mut data = Vec::new();

        // LEGACY_KEY_ID (20 bytes, zeros)
        data.extend_from_slice(&[0u8; 20]);
        // AUTH_KEY_TYPE
        data.push(0x02);
        // AUTH_KEY_LEN
        data.extend_from_slice(&32u16.to_be_bytes());
        // AUTH_KEY
        data.extend_from_slice(&[0xAA; 32]);
        // N_EXTENSIONS
        data.push(0);
        // Plaintext payload (rendezvous_point + cookie + client_pk)
        data.extend_from_slice(&[1u8; 32]); // rendezvous point
        data.extend_from_slice(&[2u8; 20]); // cookie
        data.extend_from_slice(&[3u8; 32]); // client pk

        let cell = RelayCell {
            command: crate::protocol::RelayCommand::Introduce2,
            recognized: 0,
            stream_id: 0,
            digest: [0u8; 4],
            length: u16::try_from(data.len()).expect("cell data length fits in u16"),
            data,
        };

        let request =
            IntroductionPoint::parse_introduce2(&cell).expect("valid introduce2 cell should parse");
        assert_eq!(request.rendezvous_point, [1u8; 32]);
        assert_eq!(request.rendezvous_cookie, [2u8; 20]);
        assert_eq!(request.client_public_key, [3u8; 32]);
    }

    #[test]
    fn test_parse_introduce2_too_short() {
        let cell = RelayCell {
            command: crate::protocol::RelayCommand::Introduce2,
            recognized: 0,
            stream_id: 0,
            digest: [0u8; 4],
            length: 10,
            data: vec![0u8; 10],
        };

        let result = IntroductionPoint::parse_introduce2(&cell);
        assert!(result.is_err());
    }
}
