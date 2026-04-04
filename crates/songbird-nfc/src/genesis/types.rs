// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Genesis credentials (encrypted)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisCredentials {
    /// Primal identity (Ed25519 public key)
    pub identity: Vec<u8>,

    /// Family seed (encrypted, shared secret)
    pub family_seed: Vec<u8>,

    /// Lineage path (encrypted)
    pub lineage: Vec<String>,

    /// Beacon endpoints (encrypted)
    pub beacons: Vec<String>,

    /// Timestamp (Unix milliseconds)
    pub timestamp: i64,
}

/// Decode hex or base64 encoded bytes
#[expect(
    clippy::unnecessary_wraps,
    reason = "Result kept for uniform error propagation at call sites"
)]
pub(super) fn decode_hex_or_b64(s: &str) -> Result<Vec<u8>> {
    if let Ok(bytes) = hex::decode(s) {
        return Ok(bytes);
    }
    Ok(s.as_bytes().to_vec())
}

pub(super) mod hex {
    pub(in crate::genesis) fn encode(data: &[u8]) -> String {
        use std::fmt::Write;
        let mut s = String::with_capacity(data.len() * 2);
        for b in data {
            let _ = write!(s, "{b:02x}");
        }
        s
    }

    pub(in crate::genesis) fn decode(s: &str) -> std::result::Result<Vec<u8>, String> {
        if !s.len().is_multiple_of(2) {
            return Err("odd hex length".to_string());
        }
        (0..s.len())
            .step_by(2)
            .map(|i| {
                u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| format!("invalid hex at {i}: {e}"))
            })
            .collect()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn test_hex_encode() {
        assert_eq!(hex::encode(&[0x00, 0xff, 0xab]), "00ffab");
        assert_eq!(hex::encode(&[]), "");
        assert_eq!(hex::encode(&[0x0d, 0xa4]), "0da4");
    }

    #[test]
    fn test_hex_decode() {
        assert_eq!(hex::decode("00ffab").unwrap(), vec![0x00, 0xff, 0xab]);
        assert_eq!(hex::decode("").unwrap(), Vec::<u8>::new());
        assert_eq!(hex::decode("0da4").unwrap(), vec![0x0d, 0xa4]);
    }

    #[test]
    fn test_hex_decode_odd_length() {
        assert!(hex::decode("abc").is_err());
    }

    #[test]
    fn test_hex_decode_invalid() {
        assert!(hex::decode("zzzz").is_err());
    }

    #[test]
    fn test_hex_roundtrip() {
        let data = vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
        assert_eq!(hex::decode(&hex::encode(&data)).unwrap(), data);
    }

    #[test]
    fn test_decode_hex_or_b64_hex() {
        let result = decode_hex_or_b64("48656c6c6f").unwrap();
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_decode_hex_or_b64_fallback() {
        let result = decode_hex_or_b64("Hello!").unwrap();
        assert_eq!(result, b"Hello!");
    }

    #[test]
    fn test_genesis_credentials_serialization() {
        let creds = GenesisCredentials {
            identity: vec![1, 2, 3],
            family_seed: vec![4, 5, 6],
            lineage: vec!["root".to_string(), "child".to_string()],
            beacons: vec!["beacon1.onion".to_string()],
            timestamp: 1707350400000,
        };
        let json = serde_json::to_vec(&creds).unwrap();
        let decoded: GenesisCredentials = serde_json::from_slice(&json).unwrap();
        assert_eq!(decoded.identity, creds.identity);
        assert_eq!(decoded.family_seed, creds.family_seed);
        assert_eq!(decoded.lineage, creds.lineage);
        assert_eq!(decoded.beacons, creds.beacons);
        assert_eq!(decoded.timestamp, creds.timestamp);
    }

    #[test]
    fn decode_hex_or_b64_prefers_valid_hex_over_ascii_fallback() {
        let result = decode_hex_or_b64("deadbeef").unwrap();
        assert_eq!(result, vec![0xde, 0xad, 0xbe, 0xef]);
    }

    #[test]
    fn decode_hex_or_b64_invalid_hex_falls_back_to_raw_bytes() {
        let result = decode_hex_or_b64("not-hex!").unwrap();
        assert_eq!(result, b"not-hex!".as_slice());
    }

    #[test]
    fn hex_decode_rejects_non_hex_digit() {
        assert!(hex::decode("0g").is_err());
    }

    #[test]
    fn genesis_credentials_empty_vectors_roundtrip() {
        let creds = GenesisCredentials {
            identity: vec![],
            family_seed: vec![],
            lineage: vec![],
            beacons: vec![],
            timestamp: 0,
        };
        let json = serde_json::to_vec(&creds).unwrap();
        let back: GenesisCredentials = serde_json::from_slice(&json).unwrap();
        assert_eq!(back.timestamp, 0);
        assert!(back.lineage.is_empty());
    }
}
