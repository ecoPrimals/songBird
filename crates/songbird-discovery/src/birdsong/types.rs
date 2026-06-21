// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `BirdSong` type definitions
//!
//! Core types for `BirdSong` encrypted discovery packets.

use serde::{Deserialize, Serialize};

/// `BirdSong` packet envelope (plaintext wrapper)
///
/// Contains plaintext metadata (`family_id`) so receivers can decide
/// if they should attempt decryption, avoiding the chicken-and-egg problem.
///
/// ## Packet Format
///
/// To avoid the chicken-and-egg problem (needing `family_id` to decrypt, but `family_id` is encrypted),
/// `BirdSong` packets have a plaintext header with `family_id`:
///
/// ```json
/// {
///   "birdsong": "1.0",
///   "family_id": "iidn",  // ← Plaintext, so receivers know if they can decrypt
///   "encrypted_payload": "base64..."  // ← Encrypted discovery message
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirdSongPacket {
    /// `BirdSong` protocol version
    #[serde(rename = "birdsong")]
    pub version: String,

    /// Family ID (plaintext) - allows receivers to decide if they can decrypt
    pub family_id: String,

    /// Encrypted payload (base64)
    pub encrypted_payload: String,
}

impl BirdSongPacket {
    /// Create new `BirdSong` packet
    #[must_use]
    pub const fn new(version: String, family_id: String, encrypted_payload: String) -> Self {
        Self {
            version,
            family_id,
            encrypted_payload,
        }
    }

    /// Get the protocol version
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Get the family ID
    #[must_use]
    pub fn family_id(&self) -> &str {
        &self.family_id
    }

    /// Get the encrypted payload
    #[must_use]
    pub fn encrypted_payload(&self) -> &str {
        &self.encrypted_payload
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn new_and_accessors() {
        let p = BirdSongPacket::new(
            String::from("1.0"),
            String::from("fam"),
            String::from("cGF5bG9hZA=="),
        );
        assert_eq!(p.version(), "1.0");
        assert_eq!(p.family_id(), "fam");
        assert_eq!(p.encrypted_payload(), "cGF5bG9hZA==");
    }

    #[test]
    fn serde_uses_birdsong_field_name() {
        let p = BirdSongPacket::new(String::from("1.0"), String::from("fam"), String::from("x"));
        let v: serde_json::Value = serde_json::to_value(&p).unwrap();
        assert_eq!(v.get("birdsong").and_then(|x| x.as_str()), Some("1.0"));
        assert_eq!(v.get("family_id").and_then(|x| x.as_str()), Some("fam"));
    }

    #[test]
    fn serde_roundtrip() {
        let p =
            BirdSongPacket::new(String::from("1.0"), String::from("iidn"), String::from("payload"));
        let json = serde_json::to_string(&p).unwrap();
        let back: BirdSongPacket = serde_json::from_str(&json).unwrap();
        assert_eq!(p.version, back.version);
        assert_eq!(p.family_id, back.family_id);
        assert_eq!(p.encrypted_payload, back.encrypted_payload);
    }

    #[test]
    fn debug_clone_cover() {
        let p = BirdSongPacket::new("1.0".into(), "f".into(), "e".into());
        let _ = format!("{p:?}");
        let q = p.clone();
        assert_eq!(q.family_id, p.family_id);
    }
}
