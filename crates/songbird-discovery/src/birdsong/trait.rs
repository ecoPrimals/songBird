// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `BirdSong` encryption provider trait
//!
//! Defines the interface for security providers to implement encrypted discovery.

use anyhow::Result;
use async_trait::async_trait;

/// 96-bit AEAD nonce for ChaCha20-Poly1305, filled from the OS CSPRNG (`getrandom` workspace dep).
fn random_chacha_nonce() -> Result<[u8; 12]> {
    let mut nonce = [0u8; 12];
    getrandom::fill(&mut nonce)
        .map_err(|e| anyhow::anyhow!("Failed to generate random nonce: {e}"))?;
    Ok(nonce)
}

/// `BirdSong` encryption provider trait
///
/// Implemented by security providers (e.g., `security provider`) to enable
/// encrypted discovery broadcasts.
///
/// ## Provider Responsibilities
///
/// - Encrypt discovery packets using genetic lineage keys (legacy)
/// - Encrypt beacon packets using beacon seeds (Dark Forest)
/// - Decrypt packets from same-family/beacon peers
/// - Return None for packets from different families/beacons
/// - Handle key rotation and lineage changes
/// - Manage beacon genetics (meeting exchange)
///
/// ## Evolution (Feb 3, 2026)
///
/// Extended with Dark Forest beacon methods for zero-metadata-leakage discovery.
/// Legacy methods (`encrypt_discovery`, `decrypt_discovery`) remain for backward compatibility.
#[async_trait]
pub trait BirdSongEncryption: Send + Sync {
    // ═══════════════════════════════════════════════════════════════════════
    // Legacy Methods (Backward Compatibility)
    // ═══════════════════════════════════════════════════════════════════════

    /// Encrypt discovery packet for same-family peers (LEGACY)
    ///
    /// # Arguments
    ///
    /// * `plaintext` - Discovery packet bytes to encrypt
    ///
    /// # Returns
    ///
    /// Encrypted bytes that only same-family peers can decrypt
    ///
    /// # Legacy Note
    ///
    /// This method is used for legacy `BirdSongPacket` format (version 1.0).
    /// For Dark Forest beacons (version 2), use `encrypt_beacon` instead.
    async fn encrypt_discovery(&self, plaintext: &[u8]) -> Result<Vec<u8>>;

    /// Decrypt received discovery packet (LEGACY)
    ///
    /// # Arguments
    ///
    /// * `ciphertext` - Encrypted discovery packet
    ///
    /// # Returns
    ///
    /// - `Ok(Some(plaintext))` if same family (successful decrypt)
    /// - `Ok(None)` if different family (cannot decrypt)
    /// - `Err` only on system errors (not decryption failures)
    ///
    /// # Legacy Note
    ///
    /// This method is used for legacy `BirdSongPacket` format (version 1.0).
    /// For Dark Forest beacons (version 2), use `try_decrypt_beacon` instead.
    async fn decrypt_discovery(&self, ciphertext: &[u8]) -> Result<Option<Vec<u8>>>;

    /// Check if `BirdSong` encryption is available
    ///
    /// Provider may become unavailable if:
    /// - Security service is down
    /// - Keys not yet generated
    /// - Lineage not established
    fn is_available(&self) -> bool;

    /// Get encryption family ID (for logging/debugging) (LEGACY)
    ///
    /// Returns the family ID for lineage-based encryption.
    /// For Dark Forest beacons, use `get_beacon_id` instead.
    fn family_id(&self) -> Option<String>;

    /// Get provider name (for logging)
    fn provider_name(&self) -> String {
        "Unknown".to_string()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Dark Forest Methods (Zero Metadata Leakage)
    // ═══════════════════════════════════════════════════════════════════════

    /// Encrypt payload for Dark Forest beacon (NEW - Feb 3, 2026)
    ///
    /// Uses beacon seed to encrypt, providing zero metadata leakage.
    /// Unlike legacy encryption, beacon encryption reveals nothing about
    /// family membership to passive observers.
    ///
    /// # Arguments
    ///
    /// * `payload` - Beacon payload bytes to encrypt (serialized `BeaconPayload`)
    ///
    /// # Returns
    ///
    /// Tuple of `(encrypted_payload, nonce)` for ChaCha20-Poly1305 AEAD
    ///
    /// # Default Implementation
    ///
    /// Falls back to legacy `encrypt_discovery` with random nonce.
    /// Override this for true Dark Forest support with beacon seeds.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let (encrypted, nonce) = provider.encrypt_beacon(&payload_bytes).await?;
    /// let beacon = DarkForestBeacon::new(encrypted, nonce);
    /// ```
    async fn encrypt_beacon(&self, payload: &[u8]) -> Result<(Vec<u8>, [u8; 12])> {
        // Default: legacy encrypt + random 96-bit nonce (never reuse with the same key).
        let encrypted = self.encrypt_discovery(payload).await?;
        let nonce = random_chacha_nonce()?;
        Ok((encrypted, nonce))
    }

    /// Try to decrypt Dark Forest beacon (NEW - Feb 3, 2026)
    ///
    /// Attempts decryption using our beacon seed.
    /// Returns `Some(payload)` if we share beacon genetics with sender.
    /// Returns `None` if different beacon family (expected, not an error).
    ///
    /// # Arguments
    ///
    /// * `encrypted` - Encrypted payload from Dark Forest beacon
    /// * `nonce` - 12-byte nonce for ChaCha20-Poly1305 AEAD
    ///
    /// # Returns
    ///
    /// - `Ok(Some(payload))` if same beacon family (successful decrypt)
    /// - `Ok(None)` if different beacon family (cannot decrypt - NORMAL)
    /// - `Err` only on system errors (not decryption failures)
    ///
    /// # Privacy Note
    ///
    /// Returning `None` does NOT indicate an error. It means the beacon is
    /// from a different beacon family, which is expected and correct behavior
    /// for Dark Forest privacy.
    ///
    /// # Default Implementation
    ///
    /// Falls back to legacy `decrypt_discovery`.
    /// Override this for true Dark Forest support with beacon seeds.
    async fn try_decrypt_beacon(
        &self,
        encrypted: &[u8],
        _nonce: &[u8; 12],
    ) -> Result<Option<Vec<u8>>> {
        // Default implementation for backward compatibility
        // Ignores nonce and uses legacy decryption
        self.decrypt_discovery(encrypted).await
    }

    /// Get our beacon ID derived from beacon seed (NEW - Feb 3, 2026)
    ///
    /// Returns the public beacon identifier derived from our beacon seed.
    /// This is used to identify which beacon family we belong to.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(beacon_id))` if beacon seed available
    /// - `Ok(None)` if beacon genetics not yet established
    /// - `Err` on system errors
    ///
    /// # Beacon ID Derivation
    ///
    /// Typically: `BLAKE3(beacon_seed || "beacon-id-v1")[..16]`
    ///
    /// # Default Implementation
    ///
    /// Returns `None` (beacon genetics not supported).
    /// Override this to enable Dark Forest beacon functionality.
    async fn get_beacon_id(&self) -> Result<Option<Vec<u8>>> {
        // Default implementation - beacon genetics not supported
        Ok(None)
    }

    /// List known beacon IDs from meetings (NEW - Feb 3, 2026)
    ///
    /// Returns beacon IDs of peers we've "met" and exchanged beacon genetics with.
    /// These are the beacon families whose beacons we can decrypt.
    ///
    /// # Returns
    ///
    /// Vector of known beacon IDs from meetings.
    /// Empty vector if no meetings or beacon genetics not supported.
    ///
    /// # Meeting Exchange
    ///
    /// Beacon genetics are exchanged during "meeting" events:
    /// - Explicit: User-initiated meeting request
    /// - Implicit: Successful trust establishment
    ///
    /// # Default Implementation
    ///
    /// Returns empty vector (no meetings tracked).
    /// Override this to enable multi-beacon decryption.
    async fn list_known_beacons(&self) -> Result<Vec<Vec<u8>>> {
        // Default implementation - no known beacons
        Ok(Vec::new())
    }

    /// Check if Dark Forest beacon support is available (NEW - Feb 3, 2026)
    ///
    /// Returns `true` if provider supports Dark Forest beacons:
    /// - `encrypt_beacon` implemented
    /// - `try_decrypt_beacon` implemented  
    /// - `get_beacon_id` returns Some
    /// - Beacon seed available
    ///
    /// # Default Implementation
    ///
    /// Returns `false` (Dark Forest not supported).
    /// Override this to indicate Dark Forest capability.
    async fn supports_dark_forest(&self) -> bool {
        // Check if beacon ID is available
        self.get_beacon_id().await.ok().flatten().is_some()
    }
}
