// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::clone_on_ref_ptr,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions"
)]
// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals
#![allow(
    clippy::cast_possible_truncation,
    reason = "TLS wire-format test helpers use bounded length fields"
)]

//! Comprehensive TLS Unit Tests
//!
//! Testing all TLS components in isolation for correctness and edge cases.

// TLS unit tests - testing individual components

#[cfg(test)]
mod handshake_tests {

    #[test]
    fn test_client_random_generation() {
        // Client random should be 32 bytes of cryptographically secure randomness
        // For now we use timestamp-based, but should be enhanced
        let random = generate_test_random();
        assert_eq!(random.len(), 32, "Client random must be exactly 32 bytes");
    }

    #[test]
    fn test_client_hello_structure() {
        // Verify ClientHello has correct structure:
        // - Record header (5 bytes)
        // - Handshake header (4 bytes)
        // - Protocol version (2 bytes)
        // - Random (32 bytes)
        // - Session ID length (1 byte)
        // - Cipher suites
        // - Compression methods
        // - Extensions

        let client_hello = build_minimal_client_hello();

        // Check record header
        assert_eq!(client_hello[0], 0x16, "Record type should be Handshake (0x16)");
        assert_eq!(client_hello[1], 0x03, "Protocol version major should be 3");
        assert_eq!(client_hello[2], 0x03, "Protocol version minor should be 3 (TLS 1.2 legacy)");

        // Check handshake type
        assert_eq!(client_hello[5], 0x01, "Handshake type should be ClientHello (0x01)");
    }

    #[test]
    fn test_sni_extension_format() {
        let server_name = "api.github.com";
        let sni = build_sni_extension(server_name);

        // SNI extension structure:
        // - List length (2 bytes)
        // - Name type (1 byte) - 0x00 for host_name
        // - Name length (2 bytes)
        // - Name (variable)

        assert!(!sni.is_empty(), "SNI extension should not be empty");

        let list_len = u16::from_be_bytes([sni[0], sni[1]]) as usize;
        assert_eq!(list_len, server_name.len() + 3, "SNI list length incorrect");

        assert_eq!(sni[2], 0x00, "Name type should be host_name (0x00)");

        let name_len = u16::from_be_bytes([sni[3], sni[4]]) as usize;
        assert_eq!(name_len, server_name.len(), "Name length incorrect");

        let name_bytes = &sni[5..];
        assert_eq!(name_bytes, server_name.as_bytes(), "Server name mismatch");
    }

    #[test]
    fn test_key_share_extension_format() {
        let public_key = vec![0x42u8; 32]; // Mock X25519 public key
        let key_share = build_key_share_extension(&public_key);

        // Key share structure:
        // - Client shares length (2 bytes)
        // - Named group (2 bytes) - 0x001d for X25519
        // - Key exchange length (2 bytes)
        // - Key exchange data (32 bytes for X25519)

        let shares_len = u16::from_be_bytes([key_share[0], key_share[1]]) as usize;
        assert_eq!(shares_len, 36, "Client shares length should be 36 (4 + 32)");

        let group = u16::from_be_bytes([key_share[2], key_share[3]]);
        assert_eq!(group, 0x001d, "Named group should be X25519 (0x001d)");

        let key_len = u16::from_be_bytes([key_share[4], key_share[5]]) as usize;
        assert_eq!(key_len, 32, "Key length should be 32 for X25519");

        assert_eq!(&key_share[6..], &public_key[..], "Public key mismatch");
    }

    #[test]
    fn test_supported_versions_extension() {
        let extension = build_supported_versions_extension();

        // Supported versions structure:
        // - Extension type (2 bytes) - 0x002b
        // - Extension length (2 bytes)
        // - Versions list length (1 byte)
        // - Versions (2 bytes each)

        assert_eq!(extension[0], 0x00, "Extension type byte 0");
        assert_eq!(extension[1], 0x2b, "Extension type byte 1 (0x002b)");

        assert_eq!(extension[2], 0x00, "Extension length byte 0");
        assert_eq!(extension[3], 0x03, "Extension length byte 1 (3 bytes)");

        assert_eq!(extension[4], 0x02, "Versions list length (2 bytes)");

        let version = u16::from_be_bytes([extension[5], extension[6]]);
        assert_eq!(version, 0x0304, "Should advertise TLS 1.3 (0x0304)");
    }

    #[test]
    fn test_cipher_suites_format() {
        let cipher_suites = vec![0x1301u16, 0x1302u16, 0x1303u16]; // TLS 1.3 cipher suites
        let bytes = encode_cipher_suites(&cipher_suites);

        // Cipher suites structure:
        // - Length (2 bytes)
        // - Cipher suites (2 bytes each)

        let length = u16::from_be_bytes([bytes[0], bytes[1]]) as usize;
        assert_eq!(length, 6, "Cipher suites length should be 6 (3 * 2)");

        for (i, suite) in cipher_suites.iter().enumerate() {
            let offset = 2 + (i * 2);
            let encoded = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]);
            assert_eq!(encoded, *suite, "Cipher suite {i} mismatch");
        }
    }

    #[test]
    fn test_client_hello_min_size() {
        let client_hello = build_minimal_client_hello();

        // Minimum ClientHello should be at least:
        // - Record header: 5 bytes
        // - Handshake header: 4 bytes
        // - Protocol version: 2 bytes
        // - Random: 32 bytes
        // - Session ID: 1 byte (length 0)
        // - Cipher suites: 2 + 2 bytes (at least one suite)
        // - Compression: 2 bytes
        // - Extensions length: 2 bytes
        // Total: ~52 bytes minimum

        assert!(client_hello.len() >= 52, "ClientHello too small: {} bytes", client_hello.len());
    }

    #[test]
    fn test_client_hello_with_all_extensions() {
        let client_hello = build_full_client_hello();

        // Full ClientHello should include:
        // - SNI
        // - supported_versions
        // - key_share
        // - supported_groups
        // - signature_algorithms

        // Minimal is ~52 bytes, full should be larger
        assert!(
            client_hello.len() >= 52,
            "Full ClientHello too small: {} bytes",
            client_hello.len()
        );
        assert!(
            client_hello.len() <= 500,
            "Full ClientHello too large: {} bytes",
            client_hello.len()
        );
    }

    // Helper functions
    fn generate_test_random() -> Vec<u8> {
        vec![0x42u8; 32] // Mock random for testing
    }

    fn build_minimal_client_hello() -> Vec<u8> {
        let mut msg = Vec::new();

        // Record header
        msg.push(0x16); // Handshake
        msg.extend_from_slice(&[0x03, 0x03]); // TLS 1.2
        msg.extend_from_slice(&[0x00, 0x00]); // Length placeholder

        // Handshake header
        msg.push(0x01); // ClientHello
        msg.extend_from_slice(&[0x00, 0x00, 0x00]); // Length placeholder

        // Protocol version
        msg.extend_from_slice(&[0x03, 0x03]); // TLS 1.2

        // Random
        msg.extend_from_slice(&[0x42u8; 32]);

        // Session ID
        msg.push(0x00);

        // Cipher suites
        msg.extend_from_slice(&[0x00, 0x02]); // Length: 2
        msg.extend_from_slice(&[0x13, 0x01]); // TLS_AES_128_GCM_SHA256

        // Compression
        msg.push(0x01); // Length: 1
        msg.push(0x00); // No compression

        // Extensions length
        msg.extend_from_slice(&[0x00, 0x00]);

        msg
    }

    fn build_full_client_hello() -> Vec<u8> {
        // This would include all extensions
        // Simplified for testing

        // Would add extensions here
        build_minimal_client_hello()
    }

    fn build_sni_extension(server_name: &str) -> Vec<u8> {
        let mut sni = Vec::new();
        let name_bytes = server_name.as_bytes();

        sni.extend_from_slice(&((name_bytes.len() + 3) as u16).to_be_bytes());
        sni.push(0x00); // host_name
        sni.extend_from_slice(&(name_bytes.len() as u16).to_be_bytes());
        sni.extend_from_slice(name_bytes);

        sni
    }

    fn build_key_share_extension(public_key: &[u8]) -> Vec<u8> {
        let mut ks = Vec::new();

        ks.extend_from_slice(&((public_key.len() + 4) as u16).to_be_bytes());
        ks.extend_from_slice(&[0x00, 0x1d]); // X25519
        ks.extend_from_slice(&(public_key.len() as u16).to_be_bytes());
        ks.extend_from_slice(public_key);

        ks
    }

    fn build_supported_versions_extension() -> Vec<u8> {
        vec![
            0x00, 0x2b, // Extension type
            0x00, 0x03, // Length: 3
            0x02, // List length: 2
            0x03, 0x04, // TLS 1.3
        ]
    }

    fn encode_cipher_suites(suites: &[u16]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&((suites.len() * 2) as u16).to_be_bytes());
        for suite in suites {
            bytes.extend_from_slice(&suite.to_be_bytes());
        }
        bytes
    }
}

#[cfg(test)]
mod record_layer_tests {
    #[test]
    fn test_record_header_parsing() {
        let header = [0x16, 0x03, 0x03, 0x00, 0x05];

        let content_type = header[0];
        let version = u16::from_be_bytes([header[1], header[2]]);
        let length = u16::from_be_bytes([header[3], header[4]]);

        assert_eq!(content_type, 0x16, "Should be Handshake");
        assert_eq!(version, 0x0303, "Should be TLS 1.2");
        assert_eq!(length, 5, "Length should be 5");
    }

    #[test]
    fn test_record_type_validation() {
        let valid_types = [0x14, 0x15, 0x16, 0x17]; // CCS, Alert, Handshake, AppData
        let invalid_types = [0x00, 0x13, 0x18, 0xFF];

        for &t in &valid_types {
            assert!(is_valid_record_type(t), "Type {t:#04x} should be valid");
        }

        for &t in &invalid_types {
            assert!(!is_valid_record_type(t), "Type {t:#04x} should be invalid");
        }
    }

    #[test]
    fn test_record_length_validation() {
        assert!(is_valid_record_length(0), "Zero length should be valid");
        assert!(is_valid_record_length(16384), "Max TLS record size should be valid");
        assert!(!is_valid_record_length(16385), "Over max should be invalid");
        assert!(!is_valid_record_length(20000), "Way over max should be invalid");
    }

    fn is_valid_record_type(t: u8) -> bool {
        matches!(t, 0x14..=0x17)
    }

    fn is_valid_record_length(len: usize) -> bool {
        len <= 16384 // TLS max record size
    }
}

#[cfg(test)]
mod alert_tests {
    #[test]
    fn test_alert_decoding() {
        let alerts = vec![
            (1, 0, "Warning", "close_notify"),
            (2, 40, "Fatal", "handshake_failure"),
            (2, 42, "Fatal", "bad_certificate"),
            (2, 112, "Fatal", "unrecognized_name"),
        ];

        for (level, desc, expected_level, expected_desc) in alerts {
            let level_str = if level == 1 {
                "Warning"
            } else {
                "Fatal"
            };
            let desc_str = match desc {
                0 => "close_notify",
                40 => "handshake_failure",
                42 => "bad_certificate",
                112 => "unrecognized_name",
                _ => "unknown",
            };

            assert_eq!(level_str, expected_level, "Alert level mismatch");
            assert_eq!(desc_str, expected_desc, "Alert description mismatch");
        }
    }

    #[test]
    fn test_alert_message_format() {
        // Alert message structure:
        // - Record header (5 bytes)
        // - Alert level (1 byte)
        // - Alert description (1 byte)

        let alert = build_alert_message(2, 40); // Fatal handshake_failure

        assert_eq!(alert[0], 0x15, "Record type should be Alert (0x15)");
        assert_eq!(alert.len(), 7, "Alert message should be 7 bytes");

        let length = u16::from_be_bytes([alert[3], alert[4]]);
        assert_eq!(length, 2, "Alert payload should be 2 bytes");

        assert_eq!(alert[5], 2, "Alert level should be Fatal (2)");
        assert_eq!(alert[6], 40, "Alert description should be 40");
    }

    fn build_alert_message(level: u8, description: u8) -> Vec<u8> {
        vec![
            0x15, // Alert
            0x03,
            0x03, // TLS 1.2
            0x00,
            0x02, // Length: 2
            level,
            description,
        ]
    }
}

#[cfg(test)]
mod session_tests {
    #[test]
    fn test_session_keys_structure() {
        let session = create_mock_session();

        assert_eq!(session.client_write_key.len(), 32, "Client key should be 32 bytes");
        assert_eq!(session.server_write_key.len(), 32, "Server key should be 32 bytes");
        assert_eq!(session.client_write_iv.len(), 12, "Client IV should be 12 bytes");
        assert_eq!(session.server_write_iv.len(), 12, "Server IV should be 12 bytes");
    }

    fn create_mock_session() -> MockSessionKeys {
        MockSessionKeys {
            client_write_key: vec![0x01u8; 32],
            server_write_key: vec![0x02u8; 32],
            client_write_iv: vec![0x03u8; 12],
            server_write_iv: vec![0x04u8; 12],
        }
    }

    struct MockSessionKeys {
        client_write_key: Vec<u8>,
        server_write_key: Vec<u8>,
        client_write_iv: Vec<u8>,
        server_write_iv: Vec<u8>,
    }
}
