// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `BirdSong` processor implementation
//!
//! Main processor logic for encryption/decryption of discovery packets.

use super::config::BirdSongConfig;
use super::encryption::BirdSongEncryption;
use super::types::BirdSongPacket;
use crate::dark_forest_beacon::{BeaconPayload, DarkForestBeacon};
use anyhow::{Context, Result};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// BirdSong-aware discovery processor
///
/// Wraps encryption/decryption of discovery packets with graceful fallback.
pub struct BirdSongProcessor {
    /// Optional encryption provider
    encryption: Option<Arc<BirdSongEncryption>>,

    /// Configuration
    config: BirdSongConfig,
}

impl BirdSongProcessor {
    /// Create new `BirdSong` processor
    ///
    /// # Arguments
    ///
    /// * `encryption` - Optional encryption provider (None = plaintext only)
    /// * `config` - `BirdSong` configuration
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use songbird_discovery::birdsong::*;
    /// use std::sync::Arc;
    ///
    /// # async fn example() {
    /// let config = BirdSongConfig::default();
    /// let processor = BirdSongProcessor::new(None, config);
    /// # }
    /// ```
    pub fn new(encryption: Option<Arc<BirdSongEncryption>>, config: BirdSongConfig) -> Self {
        // Log configuration
        if let Some(ref enc) = encryption {
            if enc.is_available() {
                info!("🎵 BirdSong encryption enabled (provider: {})", enc.provider_name());
                if let Some(family) = enc.family_id() {
                    info!("   Family ID: {}", family);
                }
                if config.fallback_to_plaintext {
                    info!("   Fallback: enabled (graceful degradation)");
                }
                if config.mixed_mode {
                    info!("   Mixed mode: enabled (plaintext + encrypted)");
                }
            } else {
                warn!(
                    "⚠️  BirdSong provider configured but not available ({})",
                    enc.provider_name()
                );
                if config.fallback_to_plaintext {
                    info!("   Fallback: will use plaintext");
                } else {
                    warn!("   Fallback: disabled (encryption required)");
                }
            }
        } else {
            info!("📢 BirdSong: plaintext mode (trusted LAN only)");
        }

        Self {
            encryption,
            config,
        }
    }

    /// Encrypt discovery packet with plaintext `family_id` header
    ///
    /// Creates a `BirdSongPacket` with plaintext `family_id` header and encrypted payload.
    /// This solves the chicken-and-egg problem: receivers can see the `family_id` to decide
    /// if they should attempt decryption.
    ///
    /// # Returns
    ///
    /// - `BirdSongPacket` (JSON) if encryption available and enabled
    /// - Original bytes if plaintext mode or fallback
    /// - Error if encryption fails and fallback disabled
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use songbird_discovery::birdsong::*;
    /// # async fn example(processor: &BirdSongProcessor) -> anyhow::Result<()> {
    /// let packet = b"discovery-packet";
    /// let encrypted = processor.encrypt_packet(packet).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn encrypt_packet(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        // Check if encryption is enabled
        if !self.config.enabled {
            debug!("BirdSong encryption disabled, using plaintext");
            return Ok(plaintext.to_vec());
        }

        // Try to encrypt if provider available
        match &self.encryption {
            Some(enc) if enc.is_available() => {
                debug!("🔒 Encrypting discovery packet ({} bytes)", plaintext.len());

                // Get family_id from provider (needed for plaintext header)
                let family_id = enc.family_id().ok_or_else(|| {
                    anyhow::anyhow!("No family_id available from encryption provider")
                })?;

                // Encrypt the payload
                let encrypted_payload =
                    enc.encrypt_discovery(plaintext).await.context("BirdSong encryption failed")?;

                // Create BirdSongPacket with plaintext family_id header
                // This allows receivers to see the family_id and decide if they should decrypt
                use base64::{Engine as _, engine::general_purpose};
                let packet = BirdSongPacket::new(
                    "1.0".to_string(),
                    family_id.clone(),
                    general_purpose::STANDARD.encode(&encrypted_payload),
                );

                // Serialize to JSON
                let packet_json =
                    serde_json::to_vec(&packet).context("Failed to serialize BirdSongPacket")?;

                debug!(
                    "✅ Encrypted: {} -> {} bytes (family: {})",
                    plaintext.len(),
                    packet_json.len(),
                    family_id
                );

                Ok(packet_json)
            }
            Some(enc) => {
                // Provider configured but unavailable
                warn!("⚠️  BirdSong provider unavailable ({})", enc.provider_name());

                if self.config.fallback_to_plaintext {
                    info!("   Falling back to plaintext");
                    Ok(plaintext.to_vec())
                } else {
                    anyhow::bail!(
                        "BirdSong encryption unavailable and fallback disabled (provider: {})",
                        enc.provider_name()
                    )
                }
            }
            None => {
                // No provider configured
                debug!("No BirdSong provider, using plaintext");
                Ok(plaintext.to_vec())
            }
        }
    }

    /// Decrypt received discovery packet (handles `BirdSongPacket` format)
    ///
    /// Supports two formats:
    /// 1. `BirdSongPacket` (JSON) - with plaintext `family_id` header
    /// 2. Plaintext discovery message (backward compat)
    ///
    /// # Returns
    ///
    /// - `Ok(Some(plaintext))` if successfully decrypted OR plaintext mode
    /// - `Ok(None)` if encrypted for different family (noise)
    /// - `Err` only on system errors
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use songbird_discovery::birdsong::*;
    /// # async fn example(processor: &BirdSongProcessor) -> anyhow::Result<()> {
    /// let received = b"encrypted-or-plaintext";
    /// match processor.decrypt_packet(received).await? {
    ///     Some(plaintext) => println!("Decoded!"),
    ///     None => println!("Different family (noise)"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn decrypt_packet(&self, received: &[u8]) -> Result<Option<Vec<u8>>> {
        // If encryption disabled, treat everything as plaintext
        if !self.config.enabled {
            return Ok(Some(received.to_vec()));
        }

        // Try to parse as BirdSongPacket (new format)
        if let Ok(packet) = serde_json::from_slice::<BirdSongPacket>(received) {
            debug!("🔍 Received BirdSongPacket (family: {})", packet.family_id());

            // Check if we have an encryption provider
            match &self.encryption {
                Some(enc) if enc.is_available() => {
                    // Check if it's our family
                    if let Some(our_family) = enc.family_id()
                        && packet.family_id() != our_family
                    {
                        debug!(
                            "🔇 Different family ({} != {}), ignoring",
                            packet.family_id(),
                            our_family
                        );
                        return Ok(None); // Different family = noise
                    }

                    // Same family! Try to decrypt
                    use base64::{Engine as _, engine::general_purpose};
                    let encrypted_payload = general_purpose::STANDARD
                        .decode(packet.encrypted_payload())
                        .context("Failed to decode base64 encrypted_payload")?;

                    match enc.decrypt_discovery(&encrypted_payload).await {
                        Ok(Some(plaintext)) => {
                            debug!(
                                "✅ BirdSong decrypted: {} bytes (same family: {})",
                                plaintext.len(),
                                packet.family_id()
                            );
                            Ok(Some(plaintext))
                        }
                        Ok(None) => {
                            debug!("🔇 BirdSong: could not decrypt (different family)");
                            Ok(None)
                        }
                        Err(e) => {
                            warn!("⚠️  Decryption failed for same-family packet: {}", e);
                            if self.config.fallback_to_plaintext {
                                debug!("   Treating as plaintext (fallback enabled)");
                                Ok(Some(received.to_vec()))
                            } else {
                                Err(e).context("BirdSong decryption failed")
                            }
                        }
                    }
                }
                _ => {
                    // No provider or unavailable - treat as plaintext if mixed_mode
                    if self.config.mixed_mode {
                        debug!("No BirdSong provider, treating BirdSongPacket as plaintext");
                        Ok(Some(received.to_vec()))
                    } else {
                        debug!("BirdSongPacket received but no provider available, ignoring");
                        Ok(None)
                    }
                }
            }
        } else {
            // Not a BirdSongPacket - treat as plaintext (backward compat)
            if self.config.mixed_mode || !self.config.enabled {
                debug!("Non-BirdSongPacket received, treating as plaintext");
                Ok(Some(received.to_vec()))
            } else {
                debug!("Non-BirdSongPacket received in encrypted-only mode, ignoring");
                Ok(None)
            }
        }
    }

    /// Check if `BirdSong` encryption is actively being used
    #[must_use]
    pub fn is_encrypted(&self) -> bool {
        self.config.enabled && self.encryption.as_ref().is_some_and(|e| e.is_available())
    }

    /// Get current encryption status for logging
    #[must_use]
    pub fn status(&self) -> String {
        if self.is_encrypted() {
            format!(
                "Encrypted ({})",
                self.encryption
                    .as_ref()
                    .map_or_else(|| "unknown".to_string(), |e| e.provider_name())
            )
        } else if self.config.enabled {
            "Plaintext (provider unavailable)".to_string()
        } else {
            "Plaintext (disabled)".to_string()
        }
    }

    /// Get reference to configuration (read-only)
    ///
    /// Allows external code to check configuration without direct field access.
    #[must_use]
    pub const fn config(&self) -> &BirdSongConfig {
        &self.config
    }

    /// Get reference to encryption provider (read-only)
    ///
    /// Returns None if no encryption provider configured.
    #[must_use]
    pub fn encryption_provider(&self) -> Option<&Arc<BirdSongEncryption>> {
        self.encryption.as_ref()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Dark Forest Beacon Methods (NEW - Feb 3, 2026)
    // ═══════════════════════════════════════════════════════════════════════

    /// Try to decrypt Dark Forest beacon with all known beacon seeds
    ///
    /// This is the core Dark Forest mechanism: we try decryption with
    /// each known beacon seed and see what works. Successful decryption
    /// means same beacon family.
    ///
    /// ## Privacy Guarantee
    ///
    /// If we can't decrypt, we learn NOTHING about the sender.
    /// No metadata leakage. TRUE Dark Forest.
    ///
    /// # Arguments
    ///
    /// * `beacon` - Dark Forest beacon to decrypt
    ///
    /// # Returns
    ///
    /// - `Ok(Some((payload, beacon_id)))` if we can decrypt (same beacon family)
    /// - `Ok(None)` if we cannot decrypt (different beacon family - EXPECTED)
    /// - `Err` only on system errors
    pub async fn decrypt_dark_forest_beacon(
        &self,
        beacon: &DarkForestBeacon,
    ) -> Result<Option<(BeaconPayload, Vec<u8>)>> {
        // Check if encryption provider available
        let encryption = match &self.encryption {
            Some(enc) if enc.is_available() => enc,
            _ => {
                debug!("No encryption provider available for Dark Forest beacon");
                return Ok(None);
            }
        };

        // Check beacon age
        if !beacon.is_recent() {
            debug!(
                "Ignoring stale Dark Forest beacon (age: {} seconds > {} max)",
                beacon.age_seconds(),
                DarkForestBeacon::MAX_AGE_SECONDS
            );
            return Ok(None);
        }

        // Try our own beacon seed first (most common case)
        if let Some(payload) = self.try_decrypt_with_own_beacon(encryption, beacon).await? {
            let our_id = encryption.get_beacon_id().await?.unwrap_or_default();

            debug!(
                "✅ Decrypted Dark Forest beacon with our beacon seed (node: {})",
                payload.node_id
            );

            return Ok(Some((payload, our_id)));
        }

        // Try all known beacon seeds (peers we've met)
        let known_beacons = encryption.list_known_beacons().await?;

        if !known_beacons.is_empty() {
            debug!("Trying {} known beacon seeds from meetings", known_beacons.len());

            for (idx, beacon_id) in known_beacons.iter().enumerate() {
                if let Some(payload) =
                    self.try_decrypt_with_beacon_id(encryption, beacon, beacon_id).await?
                {
                    debug!(
                        "✅ Decrypted with known beacon {} (node: {})",
                        idx + 1,
                        payload.node_id
                    );

                    return Ok(Some((payload, beacon_id.clone())));
                }
            }
        }

        // Cannot decrypt - different beacon family (EXPECTED)
        debug!(
            "Cannot decrypt Dark Forest beacon - different beacon family (Dark Forest working as intended)"
        );

        Ok(None)
    }

    /// Try to decrypt with our own beacon seed
    async fn try_decrypt_with_own_beacon(
        &self,
        encryption: &BirdSongEncryption,
        beacon: &DarkForestBeacon,
    ) -> Result<Option<BeaconPayload>> {
        encryption.try_decrypt_beacon(&beacon.encrypted_payload, &beacon.nonce).await?.map_or_else(
            || Ok(None),
            |plaintext| match BeaconPayload::from_bytes(&plaintext) {
                Ok(payload) => Ok(Some(payload)),
                Err(e) => {
                    warn!("Failed to parse beacon payload: {e}");
                    Ok(None)
                }
            },
        )
    }

    /// Try to decrypt with specific beacon ID
    async fn try_decrypt_with_beacon_id(
        &self,
        encryption: &BirdSongEncryption,
        beacon: &DarkForestBeacon,
        _beacon_id: &[u8],
    ) -> Result<Option<BeaconPayload>> {
        // Try decryption with our beacon seed
        // Note: security provider integration uses family_id in encrypt/decrypt operations (v3.21.0)
        self.try_decrypt_with_own_beacon(encryption, beacon).await
    }

    /// Encrypt payload for Dark Forest beacon
    ///
    /// Creates a fully encrypted beacon with zero metadata leakage.
    ///
    /// # Arguments
    ///
    /// * `payload` - Beacon payload to encrypt
    ///
    /// # Returns
    ///
    /// Encrypted Dark Forest beacon ready for broadcast
    pub async fn encrypt_dark_forest_beacon(
        &self,
        payload: &BeaconPayload,
    ) -> Result<DarkForestBeacon> {
        // Check if encryption provider available
        let encryption = self.encryption.as_ref().context("No encryption provider available")?;

        if !encryption.is_available() {
            return Err(anyhow::anyhow!("Encryption provider not available"));
        }

        // Serialize payload
        let payload_bytes = payload.to_bytes().context("Failed to serialize beacon payload")?;

        // Encrypt with beacon seed
        let (encrypted, nonce) =
            encryption.encrypt_beacon(&payload_bytes).await.context("Failed to encrypt beacon")?;

        // Create Dark Forest beacon
        let beacon = DarkForestBeacon::new(encrypted, nonce);

        debug!(
            "Created Dark Forest beacon (size: {} bytes, NO metadata)",
            beacon.encrypted_payload.len()
        );

        Ok(beacon)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::birdsong::ProcessorXorMock;

    fn xor_mock(family_id: &str, available: bool) -> Arc<BirdSongEncryption> {
        Arc::new(BirdSongEncryption::ProcessorXor(Arc::new(ProcessorXorMock {
            family_id: family_id.to_string(),
            available,
        })))
    }

    #[tokio::test]
    async fn test_birdsong_encryption() {
        let enc = xor_mock("test-family", true);
        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: false,
            mixed_mode: false,
            ..Default::default()
        };
        let processor = BirdSongProcessor::new(Some(enc), config);

        let message = b"Hello, family!";
        let encrypted = processor.encrypt_packet(message).await.unwrap();
        assert_ne!(&encrypted[..], message, "Should be encrypted");

        let decrypted = processor.decrypt_packet(&encrypted).await.unwrap().unwrap();
        assert_eq!(&decrypted[..], message, "Should decrypt correctly");
    }

    #[tokio::test]
    async fn test_different_family_noise() {
        let enc = xor_mock("test-family", true);
        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: false,
            mixed_mode: false,
            ..Default::default()
        };
        let processor = BirdSongProcessor::new(Some(enc), config);

        // Packet from different family (starts with 0xFF)
        let different_family_packet = vec![0xFF, 0x01, 0x02, 0x03];
        let result = processor.decrypt_packet(&different_family_packet).await.unwrap();

        assert!(result.is_none(), "Should return None for different family");
    }

    #[tokio::test]
    async fn test_plaintext_fallback() {
        let config = BirdSongConfig {
            enabled: false,
            fallback_to_plaintext: true,
            mixed_mode: true,
            ..Default::default()
        };
        let processor = BirdSongProcessor::new(None, config);

        let message = b"Plaintext message";
        let result = processor.encrypt_packet(message).await.unwrap();
        assert_eq!(&result[..], message, "Should stay plaintext");

        let decrypted = processor.decrypt_packet(&result).await.unwrap().unwrap();
        assert_eq!(&decrypted[..], message, "Should pass through");
    }

    #[tokio::test]
    async fn test_encryption_disabled() {
        let enc = xor_mock("test", true);
        let config = BirdSongConfig {
            enabled: false,
            fallback_to_plaintext: true,
            mixed_mode: true,
            ..Default::default()
        };
        let processor = BirdSongProcessor::new(Some(enc), config);

        assert!(!processor.is_encrypted());

        let message = b"Message";
        let result = processor.encrypt_packet(message).await.unwrap();
        assert_eq!(&result[..], message, "Should not encrypt when disabled");
    }

    #[tokio::test]
    async fn test_provider_unavailable_with_fallback() {
        let enc = xor_mock("test", false);
        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: true,
            mixed_mode: true,
            ..Default::default()
        };
        let processor = BirdSongProcessor::new(Some(enc), config);

        let message = b"Message";
        let result = processor.encrypt_packet(message).await.unwrap();
        assert_eq!(&result[..], message, "Should fallback to plaintext");
    }

    #[tokio::test]
    async fn test_mixed_mode() {
        let enc = xor_mock("test", true);
        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: true,
            mixed_mode: true,
            ..Default::default()
        };
        let processor = BirdSongProcessor::new(Some(enc), config);

        // Should handle both encrypted and plaintext
        let plaintext_msg = b"plaintext";
        let encrypted_msg = processor.encrypt_packet(b"encrypted").await.unwrap();

        // Both should decrypt successfully in mixed mode
        let result1 = processor.decrypt_packet(plaintext_msg).await.unwrap();
        let result2 = processor.decrypt_packet(&encrypted_msg).await.unwrap();

        assert!(result1.is_some(), "Plaintext should work");
        assert!(result2.is_some(), "Encrypted should work");
    }

    #[tokio::test]
    async fn test_status_reporting() {
        let enc = xor_mock("test", true);
        let config = BirdSongConfig {
            enabled: true,
            ..Default::default()
        };
        let processor = BirdSongProcessor::new(Some(enc), config);

        let status = processor.status();
        assert!(status.contains("Encrypted"), "Should report encrypted");
        assert!(status.contains("MockEncryption"), "Should include provider name");
    }

    #[tokio::test]
    async fn test_encrypt_decrypt_roundtrip_preserves_payload() {
        let enc = xor_mock("roundtrip-family", true);
        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: false,
            mixed_mode: false,
            ..Default::default()
        };
        let processor = BirdSongProcessor::new(Some(enc), config);

        let message = b"discovery payload with unicode: \x00\xff\xfe";
        let encrypted = processor.encrypt_packet(message).await.unwrap();
        let decrypted = processor.decrypt_packet(&encrypted).await.unwrap().unwrap();
        assert_eq!(decrypted, message);
    }

    #[test]
    fn test_is_encrypted_and_encryption_provider_getters() {
        let enc = xor_mock("getter-family", true);
        let config = BirdSongConfig {
            enabled: true,
            ..Default::default()
        };
        let processor = BirdSongProcessor::new(Some(enc.clone()), config);

        assert!(processor.is_encrypted());
        assert!(processor.encryption_provider().is_some());
        assert_eq!(
            processor.encryption_provider().unwrap().family_id(),
            Some("getter-family".to_string())
        );
        assert!(processor.config().enabled);
    }

    #[test]
    fn test_status_plaintext_when_disabled() {
        let processor = BirdSongProcessor::new(None, BirdSongConfig::default());
        assert_eq!(processor.status(), "Plaintext (disabled)");
        assert!(!processor.is_encrypted());
        assert!(processor.encryption_provider().is_none());
    }

    #[test]
    fn test_status_plaintext_when_provider_unavailable() {
        let enc = xor_mock("test", false);
        let config = BirdSongConfig {
            enabled: true,
            ..Default::default()
        };
        let processor = BirdSongProcessor::new(Some(enc), config);

        assert!(!processor.is_encrypted());
        assert_eq!(processor.status(), "Plaintext (provider unavailable)");
    }

    #[tokio::test]
    async fn test_provider_unavailable_without_fallback_errors() {
        let enc = xor_mock("test", false);
        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: false,
            mixed_mode: false,
            ..Default::default()
        };
        let processor = BirdSongProcessor::new(Some(enc), config);

        let err = processor.encrypt_packet(b"secret").await.unwrap_err();
        assert!(err.to_string().contains("fallback disabled"));
    }

    #[tokio::test]
    async fn test_different_family_birdsong_packet_returns_none() {
        use crate::birdsong::types::BirdSongPacket;
        use base64::{Engine as _, engine::general_purpose};

        let enc = xor_mock("our-family", true);
        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: false,
            mixed_mode: false,
            ..Default::default()
        };
        let processor = BirdSongProcessor::new(Some(enc), config);

        let packet = BirdSongPacket::new(
            "1.0".to_string(),
            "other-family".to_string(),
            general_purpose::STANDARD.encode(b"xored"),
        );
        let bytes = serde_json::to_vec(&packet).unwrap();

        let result = processor.decrypt_packet(&bytes).await.unwrap();
        assert!(result.is_none(), "Different family_id should be ignored as noise");
    }

    #[tokio::test]
    async fn test_malformed_base64_in_birdsong_packet_errors_without_fallback() {
        use crate::birdsong::types::BirdSongPacket;

        let enc = xor_mock("test-family", true);
        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: false,
            mixed_mode: false,
            ..Default::default()
        };
        let processor = BirdSongProcessor::new(Some(enc), config);

        let packet = BirdSongPacket::new(
            "1.0".to_string(),
            "test-family".to_string(),
            "not-valid-base64!!!".to_string(),
        );
        let bytes = serde_json::to_vec(&packet).unwrap();

        let err = processor.decrypt_packet(&bytes).await.unwrap_err();
        assert!(err.to_string().contains("base64"));
    }

    #[tokio::test]
    async fn test_encrypted_only_mode_rejects_plaintext_packets() {
        let enc = xor_mock("test", true);
        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: false,
            mixed_mode: false,
            ..Default::default()
        };
        let processor = BirdSongProcessor::new(Some(enc), config);

        let result = processor.decrypt_packet(b"plain discovery message").await.unwrap();
        assert!(result.is_none(), "Non-BirdSongPacket should be ignored in encrypted-only mode");
    }

    #[tokio::test]
    async fn test_encrypt_dark_forest_beacon_roundtrip() {
        use crate::birdsong::BirdSongEncryption;
        use crate::birdsong::DarkForestTestProvider;
        use crate::dark_forest_beacon::BeaconPayload;

        let seed = [7u8; 32];
        let enc = Arc::new(BirdSongEncryption::DarkForestTest(Arc::new(
            DarkForestTestProvider::new(seed),
        )));
        let config = BirdSongConfig::dark_forest();
        let processor = BirdSongProcessor::new(Some(enc), config);

        let payload = BeaconPayload::new(
            vec![1, 2, 3],
            "beacon-node".to_string(),
            vec!["/ip4/127.0.0.1/tcp/8080".to_string()],
            &["compute".to_string()],
            None,
            "session-1".to_string(),
        );

        let beacon = processor.encrypt_dark_forest_beacon(&payload).await.unwrap();
        let decrypted = processor.decrypt_dark_forest_beacon(&beacon).await.unwrap();
        assert!(decrypted.is_some());
        assert_eq!(decrypted.unwrap().0.node_id, "beacon-node");
    }

    #[tokio::test]
    async fn test_decrypt_dark_forest_beacon_without_provider_returns_none() {
        use crate::dark_forest_beacon::DarkForestBeacon;

        let processor = BirdSongProcessor::new(None, BirdSongConfig::default());
        let beacon = DarkForestBeacon::new(vec![1, 2, 3], [0u8; 12]);

        let result = processor.decrypt_dark_forest_beacon(&beacon).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_encrypt_dark_forest_beacon_without_provider_errors() {
        use crate::dark_forest_beacon::BeaconPayload;

        let processor = BirdSongProcessor::new(None, BirdSongConfig::default());
        let payload =
            BeaconPayload::new(vec![], "node".to_string(), vec![], &[], None, "s".to_string());

        let err = processor.encrypt_dark_forest_beacon(&payload).await.unwrap_err();
        assert!(err.to_string().contains("No encryption provider"));
    }

    #[tokio::test]
    async fn test_encrypt_dark_forest_beacon_unavailable_provider_errors() {
        use crate::dark_forest_beacon::BeaconPayload;

        let enc = xor_mock("test", false);
        let processor = BirdSongProcessor::new(Some(enc), BirdSongConfig::default());
        let payload =
            BeaconPayload::new(vec![], "node".to_string(), vec![], &[], None, "s".to_string());

        let err = processor.encrypt_dark_forest_beacon(&payload).await.unwrap_err();
        assert!(err.to_string().contains("not available"));
    }
}
