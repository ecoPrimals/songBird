//! `BirdSong` Encryption Trait
//!
//! `BearDog` provides encryption/decryption for birdSong broadcasts.

use super::genesis::GenesisWitnessProof;
use super::lineage::LineageProof;
use serde::{Deserialize, Serialize};

/// `BirdSong` encryption provider
///
/// `BearDog` implements this to encrypt/decrypt birdSong messages.
#[async_trait::async_trait]
pub trait BirdSongCrypto: Send + Sync {
    /// Encrypt payload for a specific lineage
    ///
    /// Creates an encrypted birdSong that only the specified lineage can decrypt.
    async fn encrypt_for_lineage(
        &self,
        payload: &[u8],
        lineage_hint: LineageHint,
    ) -> anyhow::Result<EncryptedBirdSong>;

    /// Decrypt birdSong (if we're in the lineage)
    ///
    /// Returns the decrypted payload if we have the key, None otherwise.
    async fn decrypt_birdsong(
        &self,
        encrypted: &EncryptedBirdSong,
    ) -> anyhow::Result<Option<Vec<u8>>>;

    /// Request decryption key for a lineage
    ///
    /// `BearDog` verifies the lineage proof, then provides the key if authorized.
    async fn request_key(
        &self,
        lineage_hint: &LineageHint,
        proof: LineageProof,
    ) -> anyhow::Result<BroadcastKey>;

    /// Batch key request (for efficiency)
    ///
    /// Request multiple keys at once to amortize overhead.
    async fn request_keys_batch(
        &self,
        requests: Vec<(LineageHint, LineageProof)>,
    ) -> anyhow::Result<Vec<BroadcastKey>>;
}

/// Encrypted birdSong message
///
/// This is what gets broadcast over UDP.
/// Only family (with lineage proof) can decrypt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedBirdSong {
    /// Protocol version (for future evolution)
    pub version: u8,

    /// Encrypted payload
    pub ciphertext: Vec<u8>,

    /// Hint about which lineage can decrypt
    /// NOT a full lineage proof, just a hint for key selection
    pub lineage_hint: LineageHint,

    /// Timestamp (prevents replay attacks)
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Signature (proves authenticity, NOT for decryption)
    pub signature: Vec<u8>,

    /// Optional genesis witness for new nodes
    /// Present when a new node is broadcasting its genesis certification
    /// during initial discovery ("bird in a dark forest")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genesis_witness: Option<GenesisWitnessProof>,
}

/// Hint about which lineage can decrypt
///
/// This is NOT a proof, just a hint to help select the right key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LineageHint {
    /// My direct descendants only (children)
    DirectDescendants,

    /// All descendants (any generation)
    AllDescendants,

    /// Specific lineage root
    LineageRoot(String),

    /// Broadcast to all (but only family can decrypt)
    Universal,
}

/// Broadcast key for encrypting/decrypting birdSong
///
/// Derived from lineage, distributed by `BearDog`.
/// **Phase 3**: Used for `BirdSong` encrypted broadcasts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastKey {
    /// Key identifier (for caching)
    pub key_id: String,

    /// The actual key material (kept private)
    /// **Phase 3**: Used by `encrypt_broadcast()` and `decrypt_broadcast()` methods
    #[allow(dead_code)] // Phase 3: BirdSong encryption implementation pending
    #[serde(skip)]
    pub(crate) key_data: Vec<u8>,

    /// When this key becomes valid
    pub valid_from: chrono::DateTime<chrono::Utc>,

    /// When this key expires (for rotation)
    pub valid_until: chrono::DateTime<chrono::Utc>,
}

impl BroadcastKey {
    /// Encrypt broadcast data using this key (Phase 3 placeholder)
    ///
    /// **Status**: Phase 3 - Mock implementation for testing
    /// Once `BearDog` integration is complete, this will use real genetic cryptography.
    #[allow(dead_code)] // Phase 3: Called by BirdSong protocol
    #[must_use]
    pub fn encrypt_broadcast(&self, data: &[u8]) -> Vec<u8> {
        // Mock XOR encryption for testing
        // Phase 3: Replace with real BirdSong encryption
        self.key_data.iter().cycle().zip(data.iter()).map(|(k, d)| k ^ d).collect()
    }

    /// Check if this key is currently valid
    #[must_use]
    pub fn is_valid(&self) -> bool {
        let now = chrono::Utc::now();
        now >= self.valid_from && now <= self.valid_until
    }

    /// Check if this key is expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.valid_until
    }

    /// Time until expiration
    #[must_use]
    pub fn time_until_expiry(&self) -> Option<chrono::Duration> {
        let now = chrono::Utc::now();
        if now > self.valid_until {
            None
        } else {
            Some(self.valid_until - now)
        }
    }
}

impl EncryptedBirdSong {
    /// Check if this birdSong is recent (not a replay)
    ///
    /// Rejects messages older than 60 seconds.
    #[must_use]
    pub fn is_recent(&self) -> bool {
        let age = chrono::Utc::now() - self.timestamp;
        age.num_seconds() < 60
    }

    /// Check if timestamp is valid (not in the future)
    #[must_use]
    pub fn has_valid_timestamp(&self) -> bool {
        let now = chrono::Utc::now();
        self.timestamp <= now
    }
}
