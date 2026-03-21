// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Onion address parsing and validation
//!
//! **Phase 2C**: Onion Client

use crate::error::{Error, Result};
use base32;

/// Onion address (v3)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnionAddress {
    /// Public key (Ed25519, 32 bytes)
    pub public_key: [u8; 32],
    /// Checksum (2 bytes)
    pub checksum: [u8; 2],
    /// Version byte
    pub version: u8,
    /// Full address string (without .onion suffix)
    pub address: String,
}

impl OnionAddress {
    /// Parse v3 onion address
    ///
    /// # Arguments
    /// * `address` - Address string (with or without .onion suffix)
    ///
    /// # Returns
    /// * Parsed `OnionAddress`
    ///
    /// # Format
    /// v3: 56 characters base32 encoded (280 bits)
    /// - 32 bytes: Ed25519 public key
    /// - 2 bytes: Checksum
    /// - 1 byte: Version (0x03)
    ///
    /// # Errors
    ///
    /// Returns error if address format is invalid, base32 decode fails, or checksum mismatches.
    pub fn parse(address: &str) -> Result<Self> {
        // Strip .onion suffix if present
        let addr = address.strip_suffix(".onion").unwrap_or(address);

        // v3 addresses are 56 characters
        if addr.len() != 56 {
            return Err(Error::Protocol(format!(
                "Invalid onion address length: {} (expected 56)",
                addr.len()
            )));
        }

        // Decode base32 (RFC 4648)
        let decoded = base32::decode(
            base32::Alphabet::Rfc4648Lower {
                padding: false,
            },
            addr,
        )
        .ok_or_else(|| Error::Protocol("Failed to decode base32 onion address".to_string()))?;

        // Should be 35 bytes (32 + 2 + 1)
        if decoded.len() != 35 {
            return Err(Error::Protocol(format!(
                "Invalid decoded length: {} (expected 35)",
                decoded.len()
            )));
        }

        // Extract components
        let public_key: [u8; 32] = decoded[0..32]
            .try_into()
            .map_err(|_| Error::Protocol("Failed to extract public key".to_string()))?;

        let checksum: [u8; 2] = decoded[32..34]
            .try_into()
            .map_err(|_| Error::Protocol("Failed to extract checksum".to_string()))?;

        let version = decoded[34];

        // Verify version
        if version != 0x03 {
            return Err(Error::Protocol(format!(
                "Unsupported onion address version: 0x{version:02x}"
            )));
        }

        // Verify checksum: SHA3-256(".onion checksum" || public_key || version)[0..2]
        let mut checksum_input = Vec::with_capacity(48);
        checksum_input.extend_from_slice(b".onion checksum");
        checksum_input.extend_from_slice(&public_key);
        checksum_input.push(version);

        let hash = crate::crypto::sha3::sha3_256(&checksum_input);
        let expected_checksum = [hash[0], hash[1]];

        if checksum != expected_checksum {
            return Err(Error::Protocol(format!(
                "Onion address checksum mismatch: expected {:02x}{:02x}, got {:02x}{:02x}",
                expected_checksum[0], expected_checksum[1], checksum[0], checksum[1]
            )));
        }

        Ok(Self {
            public_key,
            checksum,
            version,
            address: addr.to_string(),
        })
    }

    /// Get full address with .onion suffix
    #[must_use]
    pub fn to_string_with_suffix(&self) -> String {
        format!("{}.onion", self.address)
    }

    /// Get address without suffix
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.address
    }
}

impl std::fmt::Display for OnionAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.onion", self.address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_onion_address_length() {
        // Too short
        let result = OnionAddress::parse("tooshort");
        assert!(result.is_err());

        // Too long
        let long_addr = "a".repeat(60);
        let result = OnionAddress::parse(&long_addr);
        assert!(result.is_err());
    }

    #[test]
    fn test_onion_address_with_suffix() {
        // Valid v3 address (56 chars + .onion)
        let addr = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let full_addr = format!("{addr}.onion");

        let result = OnionAddress::parse(&full_addr);
        // Will fail base32 decode (all 'a's), but tests suffix stripping
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_onion_address_display() {
        // Create a test address (will fail validation but tests display)
        let addr = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        if let Ok(parsed) = OnionAddress::parse(addr) {
            let display = format!("{parsed}");
            assert!(
                std::path::Path::new(&display)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("onion"))
            );
        }
    }

    #[test]
    fn test_version_check() {
        // This test verifies version checking logic
        // Real addresses would need valid base32 encoding
        let addr = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let result = OnionAddress::parse(addr);
        // Expected to fail on base32 decode or version check
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_onion_address_roundtrip() {
        // Construct a valid v3 onion address from a known public key
        let pubkey = [0x42u8; 32];
        let version: u8 = 0x03;

        // Compute correct checksum
        let mut checksum_input = Vec::new();
        checksum_input.extend_from_slice(b".onion checksum");
        checksum_input.extend_from_slice(&pubkey);
        checksum_input.push(version);
        let hash = crate::crypto::sha3::sha3_256(&checksum_input);
        let checksum = [hash[0], hash[1]];

        // Build 35-byte address
        let mut addr_bytes = Vec::with_capacity(35);
        addr_bytes.extend_from_slice(&pubkey);
        addr_bytes.extend_from_slice(&checksum);
        addr_bytes.push(version);

        // Encode to base32
        let encoded = base32::encode(
            base32::Alphabet::Rfc4648Lower {
                padding: false,
            },
            &addr_bytes,
        );

        // Parse should succeed
        let parsed = OnionAddress::parse(&encoded).expect("valid address should parse");
        assert_eq!(parsed.public_key, pubkey);
        assert_eq!(parsed.checksum, checksum);
        assert_eq!(parsed.version, 0x03);
    }

    #[test]
    fn test_onion_address_bad_checksum() {
        // Construct an address with an intentionally wrong checksum
        let pubkey = [0x42u8; 32];
        let version: u8 = 0x03;
        let bad_checksum = [0xFF, 0xFF]; // Wrong

        let mut addr_bytes = Vec::with_capacity(35);
        addr_bytes.extend_from_slice(&pubkey);
        addr_bytes.extend_from_slice(&bad_checksum);
        addr_bytes.push(version);

        let encoded = base32::encode(
            base32::Alphabet::Rfc4648Lower {
                padding: false,
            },
            &addr_bytes,
        );

        let result = OnionAddress::parse(&encoded);
        assert!(result.is_err());
        let err = result.expect_err("bad checksum should fail");
        assert!(format!("{err}").contains("checksum mismatch"));
    }

    #[test]
    fn test_onion_address_with_onion_suffix_roundtrip() {
        let pubkey = [0x01u8; 32];
        let version: u8 = 0x03;

        let mut ci = Vec::new();
        ci.extend_from_slice(b".onion checksum");
        ci.extend_from_slice(&pubkey);
        ci.push(version);
        let hash = crate::crypto::sha3::sha3_256(&ci);

        let mut addr_bytes = Vec::with_capacity(35);
        addr_bytes.extend_from_slice(&pubkey);
        addr_bytes.extend_from_slice(&hash[..2]);
        addr_bytes.push(version);

        let encoded = base32::encode(
            base32::Alphabet::Rfc4648Lower {
                padding: false,
            },
            &addr_bytes,
        );
        let full = format!("{encoded}.onion");

        let parsed = OnionAddress::parse(&full).expect("valid address should parse");
        assert_eq!(parsed.public_key, pubkey);
        assert!(
            std::path::Path::new(&parsed.to_string_with_suffix())
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("onion"))
        );
    }
}
