//! Protocol constants and helpers for TLS 1.3
//!
//! RFC 8446 constants and utility functions used throughout the handshake.

/// TLS 1.3 record content types
pub const CONTENT_TYPE_HANDSHAKE: u8 = 0x16;
pub const CONTENT_TYPE_APPLICATION_DATA: u8 = 0x17;
pub const CONTENT_TYPE_ALERT: u8 = 0x15;
pub const CONTENT_TYPE_CHANGE_CIPHER_SPEC: u8 = 0x14;

/// TLS 1.3 handshake message types
pub const HANDSHAKE_TYPE_CLIENT_HELLO: u8 = 0x01;
pub const HANDSHAKE_TYPE_SERVER_HELLO: u8 = 0x02;
pub const HANDSHAKE_TYPE_ENCRYPTED_EXTENSIONS: u8 = 0x08;
pub const HANDSHAKE_TYPE_CERTIFICATE: u8 = 0x0B;
pub const HANDSHAKE_TYPE_CERTIFICATE_VERIFY: u8 = 0x0F;
pub const HANDSHAKE_TYPE_FINISHED: u8 = 0x14;

/// TLS version constants
pub const TLS_1_2_VERSION: [u8; 2] = [0x03, 0x03];
pub const TLS_1_3_VERSION: [u8; 2] = [0x03, 0x04];

/// TLS 1.3 cipher suites
pub const TLS_AES_128_GCM_SHA256: u16 = 0x1301;
pub const TLS_AES_256_GCM_SHA384: u16 = 0x1302;
pub const TLS_CHACHA20_POLY1305_SHA256: u16 = 0x1303;

/// TLS extension types
pub const EXTENSION_SERVER_NAME: u16 = 0x0000;
pub const EXTENSION_SUPPORTED_GROUPS: u16 = 0x000A;
pub const EXTENSION_SIGNATURE_ALGORITHMS: u16 = 0x000D;
pub const EXTENSION_ALPN: u16 = 0x0010;
pub const EXTENSION_SUPPORTED_VERSIONS: u16 = 0x002B;
pub const EXTENSION_KEY_SHARE: u16 = 0x0033;
pub const EXTENSION_PSK_KEY_EXCHANGE_MODES: u16 = 0x002D;

/// Named group (curve) identifiers
pub const NAMED_GROUP_X25519: u16 = 0x001D;
pub const NAMED_GROUP_SECP256R1: u16 = 0x0017;
pub const NAMED_GROUP_SECP384R1: u16 = 0x0018;

/// Signature algorithm identifiers
pub const SIG_ALG_RSA_PSS_RSAE_SHA256: u16 = 0x0804;
pub const SIG_ALG_ECDSA_SECP256R1_SHA256: u16 = 0x0403;
pub const SIG_ALG_RSA_PKCS1_SHA256: u16 = 0x0401;

/// ALPN protocol identifiers
pub const ALPN_HTTP_1_1: &[u8] = b"http/1.1";
pub const ALPN_HTTP_2: &[u8] = b"h2";

/// TLS record layer constants
pub const TLS_RECORD_HEADER_SIZE: usize = 5;
pub const TLS_HANDSHAKE_HEADER_SIZE: usize = 4;
pub const MAX_TLS_RECORD_SIZE: usize = 16384; // 16 KB
pub const POLY1305_TAG_SIZE: usize = 16;

/// Get handshake message type name for logging
#[must_use]
pub const fn handshake_type_name(msg_type: u8) -> &'static str {
    match msg_type {
        HANDSHAKE_TYPE_CLIENT_HELLO => "ClientHello",
        HANDSHAKE_TYPE_SERVER_HELLO => "ServerHello",
        HANDSHAKE_TYPE_ENCRYPTED_EXTENSIONS => "EncryptedExtensions",
        HANDSHAKE_TYPE_CERTIFICATE => "Certificate",
        HANDSHAKE_TYPE_CERTIFICATE_VERIFY => "CertificateVerify",
        HANDSHAKE_TYPE_FINISHED => "Finished",
        _ => "Unknown",
    }
}

/// Check if a message type indicates handshake message
#[must_use]
pub const fn is_handshake_message(content_type: u8) -> bool {
    content_type == CONTENT_TYPE_HANDSHAKE
}

/// Check if a message type is application data
#[must_use]
pub const fn is_application_data(content_type: u8) -> bool {
    content_type == CONTENT_TYPE_APPLICATION_DATA
}

/// Construct AAD (Additional Authenticated Data) for AEAD encryption
///
/// TLS 1.3 AAD format:
/// - `ContentType` (1 byte)
/// - `ProtocolVersion` (2 bytes) - always 0x0303 for TLS 1.3
/// - Length (2 bytes)
#[must_use]
pub const fn construct_aad(content_type: u8, length: u16) -> [u8; 5] {
    [
        content_type,
        TLS_1_2_VERSION[0],
        TLS_1_2_VERSION[1],
        (length >> 8) as u8,
        (length & 0xFF) as u8,
    ]
}

/// Construct nonce for AEAD encryption from IV and sequence number
///
/// TLS 1.3 nonce construction:
/// - XOR IV with padded sequence number (RFC 8446 Section 5.3)
#[must_use]
pub fn construct_nonce(iv: &[u8], sequence_number: u64) -> Vec<u8> {
    let mut nonce = iv.to_vec();
    let seq_bytes = sequence_number.to_be_bytes();

    // XOR the last 8 bytes of the nonce with the sequence number
    for (i, &byte) in seq_bytes.iter().enumerate() {
        let nonce_idx = nonce.len() - 8 + i;
        nonce[nonce_idx] ^= byte;
    }

    nonce
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handshake_type_names() {
        assert_eq!(handshake_type_name(0x01), "ClientHello");
        assert_eq!(handshake_type_name(0x02), "ServerHello");
        assert_eq!(handshake_type_name(0x14), "Finished");
        assert_eq!(handshake_type_name(0xFF), "Unknown");
    }

    #[test]
    fn test_message_type_checks() {
        assert!(is_handshake_message(0x16));
        assert!(is_application_data(0x17));
        assert!(!is_handshake_message(0x17));
        assert!(!is_application_data(0x16));
    }

    #[test]
    fn test_aad_construction() {
        let aad = construct_aad(0x17, 100);
        assert_eq!(aad.len(), 5);
        assert_eq!(aad[0], 0x17); // ContentType
        assert_eq!(aad[1], 0x03); // Version major
        assert_eq!(aad[2], 0x03); // Version minor
        assert_eq!(aad[3], 0x00); // Length high byte
        assert_eq!(aad[4], 0x64); // Length low byte (100)
    }

    #[test]
    fn test_nonce_construction() {
        let iv = vec![0x00; 12];
        let nonce0 = construct_nonce(&iv, 0);
        let nonce1 = construct_nonce(&iv, 1);

        // Different sequence numbers should produce different nonces
        assert_ne!(nonce0, nonce1);

        // Last byte should differ by 1
        assert_eq!(nonce1[11], nonce0[11] ^ 1);
    }

    #[test]
    fn test_nonce_sequence_wrapping() {
        let iv = vec![0x00; 12];
        let nonce_max = construct_nonce(&iv, u64::MAX);
        let nonce_zero = construct_nonce(&iv, 0);

        // Should produce different nonces
        assert_ne!(nonce_max, nonce_zero);
    }
}
