//! BirdSong type definitions
//!
//! Core types for BirdSong encrypted discovery packets.

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
    /// Create new BirdSong packet
    pub fn new(version: String, family_id: String, encrypted_payload: String) -> Self {
        Self {
            version,
            family_id,
            encrypted_payload,
        }
    }

    /// Get the protocol version
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Get the family ID
    pub fn family_id(&self) -> &str {
        &self.family_id
    }

    /// Get the encrypted payload
    pub fn encrypted_payload(&self) -> &str {
        &self.encrypted_payload
    }
}
