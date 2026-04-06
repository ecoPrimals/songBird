// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Onion address derivation and validation (Tor v3 format)

use crate::error::{OnionError, Result};
use crate::security_crypto::SecurityCryptoClient;

// Import dalek/sha3 types only for standalone/test mode
#[cfg(feature = "standalone")]
use ed25519_dalek::VerifyingKey;
#[cfg(feature = "standalone")]
use sha3::{Digest, Sha3_256};

/// Derive .onion address from an Ed25519 public key using delegated SHA3-256 (Neural API).
pub(crate) async fn derive_onion_address_with_security_provider(
    client: &SecurityCryptoClient,
    pubkey_bytes: &[u8; 32],
) -> Result<String> {
    let mut data = Vec::with_capacity(35);

    // 1. Add public key (32 bytes)
    data.extend_from_slice(pubkey_bytes);

    // 2. Compute checksum: SHA3-256(".onion checksum" || pubkey || 0x03)[0..2]
    let mut checksum_input = Vec::new();
    checksum_input.extend_from_slice(b".onion checksum");
    checksum_input.extend_from_slice(pubkey_bytes);
    checksum_input.push(0x03); // Version 3

    let hash = client.sha3_256(&checksum_input).await?;
    let checksum = &hash[..2];

    // 3. Add checksum (2 bytes)
    data.extend_from_slice(checksum);

    // 4. Add version (1 byte)
    data.push(0x03);

    // 5. Base32 encode (RFC 4648, lowercase, no padding)
    let encoded = base32::encode(
        base32::Alphabet::Rfc4648Lower {
            padding: false,
        },
        &data,
    );

    Ok(format!("{encoded}.onion"))
}

/// Derive .onion address via the security provider (delegated `crypto.sha3_256`).
///
/// # Example
///
/// ```no_run
/// # use songbird_sovereign_onion::{derive_onion_address_via_security_provider, SecurityCryptoClient};
/// # tokio_test::block_on(async {
/// let client = SecurityCryptoClient::from_env();
/// let pubkey_bytes = [0u8; 32]; // Your Ed25519 public key
/// let onion = derive_onion_address_via_security_provider(&client, &pubkey_bytes).await.unwrap();
/// assert!(onion.ends_with(".onion"));
/// # });
/// ```
///
/// # Errors
///
/// Returns error if the security RPC fails or checksum computation fails.
pub async fn derive_onion_address_via_security_provider(
    client: &SecurityCryptoClient,
    pubkey_bytes: &[u8; 32],
) -> Result<String> {
    derive_onion_address_with_security_provider(client, pubkey_bytes).await
}

/// Validate .onion address via the security provider
///
/// # Errors
///
/// Returns error if address format is invalid, checksum fails, or RPC fails.
pub async fn validate_onion_address_via_security_provider(
    client: &SecurityCryptoClient,
    onion: &str,
) -> Result<[u8; 32]> {
    // 1. Remove ".onion" suffix
    let encoded = onion.strip_suffix(".onion").ok_or(OnionError::InvalidFormat)?;

    // 2. Base32 decode
    let data = base32::decode(
        base32::Alphabet::Rfc4648Lower {
            padding: false,
        },
        encoded,
    )
    .ok_or(OnionError::InvalidEncoding)?;

    // 3. Check length (32 + 2 + 1 = 35 bytes)
    if data.len() != 35 {
        return Err(OnionError::InvalidLength(data.len()));
    }

    // 4. Extract components
    let pubkey_bytes = &data[..32];
    let checksum = &data[32..34];
    let version = data[34];

    // 5. Verify version
    if version != 0x03 {
        return Err(OnionError::UnsupportedVersion(version));
    }

    // 6. Verify checksum via security provider
    let mut checksum_input = Vec::new();
    checksum_input.extend_from_slice(b".onion checksum");
    checksum_input.extend_from_slice(pubkey_bytes);
    checksum_input.push(version);

    let hash = client.sha3_256(&checksum_input).await?;
    let expected_checksum = &hash[..2];

    if checksum != expected_checksum {
        return Err(OnionError::ChecksumMismatch);
    }

    // 7. Return public key
    let pubkey_array: [u8; 32] =
        pubkey_bytes.try_into().map_err(|_| OnionError::InvalidPublicKey)?;

    Ok(pubkey_array)
}

/// Standalone derivation (for testing/offline only)
///
/// Derive .onion address from Ed25519 public key (Tor v3 format)
///
/// Format: base32(pubkey || checksum || version).onion
/// - pubkey: 32-byte Ed25519 public key
/// - checksum: First 2 bytes of SHA3-256(".onion checksum" || pubkey || version)
/// - version: 0x03 for v3
///
/// # Example
///
/// ```
/// # #[cfg(feature = "standalone")]
/// # {
/// use ed25519_dalek::SigningKey;
/// use songbird_sovereign_onion::derive_onion_address;
///
/// let signing_key = SigningKey::from_bytes(&[42u8; 32]);
/// let public_key = signing_key.verifying_key();
/// let onion = derive_onion_address(&public_key);
///
/// assert!(onion.ends_with(".onion"));
/// assert_eq!(onion.len(), 62); // 56 chars + ".onion"
/// # }
/// ```
#[cfg(feature = "standalone")]
#[must_use]
pub fn derive_onion_address(pubkey: &VerifyingKey) -> String {
    let mut data = Vec::with_capacity(35);

    // 1. Add public key (32 bytes)
    data.extend_from_slice(pubkey.as_bytes());

    // 2. Compute checksum: SHA3-256(".onion checksum" || pubkey || 0x03)[0..2]
    let mut hasher = Sha3_256::new();
    hasher.update(b".onion checksum");
    hasher.update(pubkey.as_bytes());
    hasher.update([0x03]); // Version 3
    let hash = hasher.finalize();
    let checksum = &hash[..2];

    // 3. Add checksum (2 bytes)
    data.extend_from_slice(checksum);

    // 4. Add version (1 byte)
    data.push(0x03);

    // 5. Base32 encode (RFC 4648, lowercase, no padding)
    let encoded = base32::encode(
        base32::Alphabet::Rfc4648Lower {
            padding: false,
        },
        &data,
    );

    format!("{encoded}.onion")
}

/// Standalone: Parse .onion address to extract Ed25519 public key
///
/// # Example
///
/// ```
/// # #[cfg(feature = "standalone")]
/// # {
/// use songbird_sovereign_onion::parse_onion_address;
///
/// let onion = "vww6ybal4bd7szmgncyruucpgfkqahzddi37ktceo3ah7ngmcopnpyyd.onion";
/// let pubkey = parse_onion_address(onion).unwrap();
/// assert_eq!(pubkey.as_bytes().len(), 32);
/// # }
/// ```
///
/// # Errors
///
/// Returns the same errors as [`validate_onion_address`].
#[cfg(feature = "standalone")]
pub fn parse_onion_address(onion: &str) -> Result<VerifyingKey> {
    validate_onion_address(onion)
}

/// Standalone: Validate .onion address format and checksum
///
/// # Errors
///
/// Returns error if:
/// - Not ending with ".onion"
/// - Invalid base32 encoding
/// - Wrong length (not 35 bytes)
/// - Unsupported version
/// - Invalid public key
/// - Checksum mismatch
#[cfg(feature = "standalone")]
pub fn validate_onion_address(onion: &str) -> Result<VerifyingKey> {
    // 1. Remove ".onion" suffix
    let encoded = onion.strip_suffix(".onion").ok_or(OnionError::InvalidFormat)?;

    // 2. Base32 decode
    let data = base32::decode(
        base32::Alphabet::Rfc4648Lower {
            padding: false,
        },
        encoded,
    )
    .ok_or(OnionError::InvalidEncoding)?;

    // 3. Check length (32 + 2 + 1 = 35 bytes)
    if data.len() != 35 {
        return Err(OnionError::InvalidLength(data.len()));
    }

    // 4. Extract components
    let pubkey_bytes = &data[..32];
    let checksum = &data[32..34];
    let version = data[34];

    // 5. Verify version
    if version != 0x03 {
        return Err(OnionError::UnsupportedVersion(version));
    }

    // 6. Parse public key
    let pubkey_array: [u8; 32] =
        pubkey_bytes.try_into().map_err(|_| OnionError::InvalidPublicKey)?;
    let pubkey =
        VerifyingKey::from_bytes(&pubkey_array).map_err(|_| OnionError::InvalidPublicKey)?;

    // 7. Verify checksum
    let mut hasher = Sha3_256::new();
    hasher.update(b".onion checksum");
    hasher.update(pubkey_bytes);
    hasher.update([version]);
    let hash = hasher.finalize();
    let expected_checksum = &hash[..2];

    if checksum != expected_checksum {
        return Err(OnionError::ChecksumMismatch);
    }

    Ok(pubkey)
}

#[cfg(all(test, feature = "standalone"))]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use ed25519_dalek::SigningKey;

    #[test]
    fn test_derive_onion_address() {
        let mut secret_bytes = [0u8; 32];
        rand::Rng::fill(&mut rand::thread_rng(), &mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let public_key = signing_key.verifying_key();
        let onion = derive_onion_address(&public_key);

        // Check format
        assert!(
            std::path::Path::new(&onion)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("onion"))
        );
        assert_eq!(onion.len(), 62); // 56 chars + ".onion"

        // Check lowercase
        assert_eq!(onion, onion.to_lowercase());
    }

    #[test]
    fn test_validate_onion_address_roundtrip() {
        let mut secret_bytes = [0u8; 32];
        rand::Rng::fill(&mut rand::thread_rng(), &mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let original_pubkey = signing_key.verifying_key();
        let onion = derive_onion_address(&original_pubkey);

        // Parse back
        let parsed_pubkey = validate_onion_address(&onion).unwrap();

        // Should match
        assert_eq!(original_pubkey.as_bytes(), parsed_pubkey.as_bytes());
    }

    #[test]
    fn test_validate_onion_address_invalid_format() {
        let result = validate_onion_address("invalid");
        assert!(matches!(result, Err(OnionError::InvalidFormat)));
    }

    #[test]
    fn test_validate_onion_address_invalid_encoding() {
        let result = validate_onion_address("!!!invalid!!!.onion");
        assert!(matches!(result, Err(OnionError::InvalidEncoding)));
    }

    #[test]
    fn test_validate_onion_address_wrong_length() {
        // Too short
        let result = validate_onion_address("short.onion");
        assert!(matches!(result, Err(OnionError::InvalidLength(_))));
    }

    #[test]
    fn test_validate_onion_address_checksum_mismatch() {
        // Generate valid address
        let mut secret_bytes = [0u8; 32];
        rand::Rng::fill(&mut rand::thread_rng(), &mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let public_key = signing_key.verifying_key();
        let onion = derive_onion_address(&public_key);

        // Decode the address to get raw bytes
        let encoded = onion.strip_suffix(".onion").unwrap();
        let mut data = base32::decode(
            base32::Alphabet::Rfc4648Lower {
                padding: false,
            },
            encoded,
        )
        .unwrap();

        // Corrupt the checksum (bytes 32..34)
        if data.len() >= 34 {
            data[32] ^= 0xFF; // Flip all bits in first checksum byte
        }

        // Re-encode
        let corrupted_encoded = base32::encode(
            base32::Alphabet::Rfc4648Lower {
                padding: false,
            },
            &data,
        );
        let corrupted = format!("{corrupted_encoded}.onion");

        let result = validate_onion_address(&corrupted);
        assert!(matches!(result, Err(OnionError::ChecksumMismatch)));
    }

    #[test]
    fn test_parse_onion_address() {
        let mut secret_bytes = [0u8; 32];
        rand::Rng::fill(&mut rand::thread_rng(), &mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let original_pubkey = signing_key.verifying_key();
        let onion = derive_onion_address(&original_pubkey);

        let parsed_pubkey = parse_onion_address(&onion).unwrap();
        assert_eq!(original_pubkey.as_bytes(), parsed_pubkey.as_bytes());
    }
}

#[cfg(test)]
mod security_provider_path_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::security_crypto::SecurityCryptoClient;

    #[tokio::test]
    async fn validate_via_security_provider_rejects_non_onion_suffix() {
        let client =
            SecurityCryptoClient::from_neural_api_socket("/tmp/songbird-onion-test-invalid.sock");
        let r = validate_onion_address_via_security_provider(&client, "not-onion").await;
        assert!(matches!(r, Err(OnionError::InvalidFormat)));
    }

    #[tokio::test]
    async fn validate_via_security_provider_rejects_bad_base32() {
        let client =
            SecurityCryptoClient::from_neural_api_socket("/tmp/songbird-onion-test-invalid.sock");
        let r = validate_onion_address_via_security_provider(&client, "!!!!.onion").await;
        assert!(matches!(r, Err(OnionError::InvalidEncoding)));
    }

    #[tokio::test]
    async fn validate_via_security_provider_rejects_wrong_decoded_length() {
        let client =
            SecurityCryptoClient::from_neural_api_socket("/tmp/songbird-onion-test-invalid.sock");
        let r = validate_onion_address_via_security_provider(&client, "aa.onion").await;
        assert!(matches!(r, Err(OnionError::InvalidLength(_))));
    }

    #[tokio::test]
    async fn derive_with_security_provider_fails_without_service() {
        let client =
            SecurityCryptoClient::from_neural_api_socket("/tmp/songbird-onion-test-invalid.sock");
        let pk = [7u8; 32];
        let r = derive_onion_address_via_security_provider(&client, &pk).await;
        assert!(r.is_err());
    }

    /// Wrong version is rejected before any RPC (checksum step is skipped).
    #[tokio::test(start_paused = true)]
    async fn validate_via_security_provider_rejects_unsupported_version_without_rpc() {
        let client =
            SecurityCryptoClient::from_neural_api_socket("/tmp/songbird-onion-test-invalid.sock");
        let mut raw = [0u8; 35];
        raw[34] = 0x02;
        let encoded = base32::encode(
            base32::Alphabet::Rfc4648Lower {
                padding: false,
            },
            &raw,
        );
        let onion = format!("{encoded}.onion");
        let r = validate_onion_address_via_security_provider(&client, &onion).await;
        assert!(
            matches!(r, Err(OnionError::UnsupportedVersion(2))),
            "expected UnsupportedVersion(2), got {r:?}"
        );
    }
}
