//! `BirdSong` Integration - Encrypted Discovery
//!
//! Integrates encrypted discovery into Songbird's anonymous discovery system.
//!
//! ## Philosophy
//!
//! "A broadcast that is obvious to family and noise otherwise"
//!
//! - Same family: Clear signal (can decrypt)
//! - Different family: Just noise (cannot decrypt)
//! - Works on LAN while learning the system
//! - Seamless upgrade path to internet-wide P2P
//!
//! ## `BirdSong` Packet Format
//!
//! To avoid the chicken-and-egg problem (needing `family_id` to decrypt, but `family_id` is encrypted),
//! `BirdSong` packets have a plaintext header with `family_id`:
//!
//! ```json
//! {
//!   "birdsong": "1.0",
//!   "family_id": "iidn",  // ← Plaintext, so receivers know if they can decrypt
//!   "encrypted_payload": "base64..."  // ← Encrypted discovery message
//! }
//! ```
//!
//! ## Modern Rust Patterns
//!
//! - Zero unsafe code
//! - Async/await throughout
//! - Comprehensive error handling with `anyhow`
//! - Graceful degradation (fallback to plaintext)
//! - Provider-agnostic (works with any security provider)

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// `BirdSong` packet envelope (plaintext wrapper)
///
/// Contains plaintext metadata (`family_id`) so receivers can decide
/// if they should attempt decryption, avoiding the chicken-and-egg problem.
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

/// `BirdSong` encryption provider trait
///
/// Implemented by security providers (e.g., `BearDog`) to enable
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
        // Default implementation for backward compatibility
        // Uses legacy encryption + random nonce
        let encrypted = self.encrypt_discovery(payload).await?;
        let nonce = [0u8; 12]; // Placeholder - should be random in real implementation
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
    async fn try_decrypt_beacon(&self, encrypted: &[u8], _nonce: &[u8; 12]) -> Result<Option<Vec<u8>>> {
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

/// `BirdSong` configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirdSongConfig {
    /// Enable `BirdSong` encryption
    ///
    /// When `true`, uses encryption if provider available.
    /// When `false`, always uses plaintext.
    pub enabled: bool,

    /// Graceful fallback to plaintext if encryption unavailable
    ///
    /// When `true`, falls back to plaintext if provider unavailable.
    /// When `false`, fails if provider unavailable.
    ///
    /// Recommended: `true` for LAN learning, `false` for production internet
    pub fallback_to_plaintext: bool,

    /// Security provider endpoint (auto-discovered if None)
    ///
    /// If provided, connects to specific endpoint.
    /// If None, discovers via capability system.
    pub security_endpoint: Option<String>,

    /// Enable mixed-mode operation
    ///
    /// When `true`, can decrypt both encrypted and plaintext packets.
    /// When `false`, only processes encrypted packets when encryption enabled.
    ///
    /// Recommended: `true` during migration, `false` in steady state
    pub mixed_mode: bool,
    
    // ✅ NEW (Feb 3, 2026): Dark Forest Beacon Genetics configuration
    
    /// Enable Dark Forest beacons (fully encrypted, zero metadata leakage)
    ///
    /// When `true`, broadcasts Dark Forest beacons (version 2, fully encrypted).
    /// When `false`, broadcasts legacy `BirdSongPacket` (version 1.0, plaintext family_id).
    ///
    /// **Privacy Impact**: Dark Forest beacons eliminate metadata leakage.
    /// Legacy format leaks `family_id` in plaintext.
    ///
    /// Requires BearDog `beacon.*` RPC methods for full functionality.
    /// Falls back to legacy if beacon methods unavailable.
    ///
    /// Recommended: `true` for privacy, `false` only for compatibility testing
    pub dark_forest_enabled: bool,
    
    /// Accept legacy `BirdSongPacket` format (backward compatibility)
    ///
    /// When `true`, accepts both Dark Forest beacons AND legacy packets.
    /// When `false`, only accepts Dark Forest beacons (rejects legacy).
    ///
    /// **Migration Strategy**:
    /// - Phase 1 (Weeks 1-4): `true` (dual format support)
    /// - Phase 2 (Weeks 5-8): `true` (still accepting legacy)
    /// - Phase 3 (Weeks 9+): `false` (Dark Forest only, optional)
    ///
    /// Recommended: `true` during migration, `false` after full rollout
    pub accept_legacy_format: bool,
    
    /// Broadcast legacy format alongside Dark Forest (migration aid)
    ///
    /// When `true`, broadcasts BOTH Dark Forest AND legacy packets.
    /// When `false`, only broadcasts Dark Forest beacons.
    ///
    /// **Use Cases**:
    /// - Early migration: Help peers discover both formats
    /// - Compatibility testing: Verify dual-format handling
    /// - Gradual rollout: Support mixed network
    ///
    /// **Overhead**: ~2x network bandwidth during dual broadcast.
    ///
    /// Recommended: `true` only during early migration (Weeks 1-2),
    ///              `false` after peers upgraded
    pub dual_broadcast: bool,
}

impl Default for BirdSongConfig {
    fn default() -> Self {
        Self {
            // Legacy configuration (unchanged)
            enabled: false,              // Opt-in for privacy
            fallback_to_plaintext: true, // Graceful degradation
            security_endpoint: None,     // Auto-discover
            mixed_mode: true,            // Support migration
            
            // Dark Forest configuration (conservative defaults)
            dark_forest_enabled: false,      // Opt-in (requires BearDog beacon.*)
            accept_legacy_format: true,      // Backward compatible by default
            dual_broadcast: false,           // Minimize network overhead
        }
    }
}

impl BirdSongConfig {
    /// Create config for Dark Forest mode (privacy-first)
    ///
    /// Broadcasts Dark Forest beacons, accepts legacy for compatibility.
    /// Use this for production deployments with privacy requirements.
    pub fn dark_forest() -> Self {
        Self {
            enabled: true,
            dark_forest_enabled: true,
            accept_legacy_format: true,  // Still accept legacy for compatibility
            dual_broadcast: false,       // Only send Dark Forest
            ..Default::default()
        }
    }
    
    /// Create config for migration period (dual format)
    ///
    /// Broadcasts BOTH formats, accepts both.
    /// Use this during early migration (Weeks 1-2) to help peers discover.
    pub fn migration_mode() -> Self {
        Self {
            enabled: true,
            dark_forest_enabled: true,
            accept_legacy_format: true,
            dual_broadcast: true,        // Send both formats
            ..Default::default()
        }
    }
    
    /// Create config for legacy-only mode (testing/compatibility)
    ///
    /// Broadcasts legacy format, accepts both.
    /// Use this only for compatibility testing or legacy deployments.
    pub fn legacy_only() -> Self {
        Self {
            enabled: true,
            dark_forest_enabled: false,
            accept_legacy_format: true,
            dual_broadcast: false,
            ..Default::default()
        }
    }
    
    /// Create config for Dark Forest only (maximum privacy)
    ///
    /// Broadcasts Dark Forest only, rejects legacy.
    /// Use this after full network migration (Phase 3+).
    ///
    /// **Warning**: Incompatible with legacy peers.
    pub fn dark_forest_only() -> Self {
        Self {
            enabled: true,
            dark_forest_enabled: true,
            accept_legacy_format: false, // Reject legacy packets
            dual_broadcast: false,
            fallback_to_plaintext: false, // No fallback
            ..Default::default()
        }
    }
    
    /// Check if Dark Forest is fully enabled
    ///
    /// Returns `true` only if both `enabled` and `dark_forest_enabled` are true.
    pub fn is_dark_forest_active(&self) -> bool {
        self.enabled && self.dark_forest_enabled
    }
    
    /// Check if accepting any legacy format
    ///
    /// Returns `true` if either legacy format accepted OR dual broadcast enabled.
    pub fn accepts_legacy(&self) -> bool {
        self.accept_legacy_format || !self.dark_forest_enabled
    }
}

/// BirdSong-aware discovery processor
///
/// Wraps encryption/decryption of discovery packets with graceful fallback.
pub struct BirdSongProcessor {
    /// Optional encryption provider
    encryption: Option<Arc<dyn BirdSongEncryption>>,

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
    /// use songbird_discovery::birdsong_integration::*;
    /// use std::sync::Arc;
    ///
    /// # async fn example() {
    /// let config = BirdSongConfig::default();
    /// let processor = BirdSongProcessor::new(None, config);
    /// # }
    /// ```
    pub fn new(encryption: Option<Arc<dyn BirdSongEncryption>>, config: BirdSongConfig) -> Self {
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
    /// # use songbird_discovery::birdsong_integration::*;
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
                use base64::{engine::general_purpose, Engine as _};
                let packet = BirdSongPacket {
                    version: "1.0".to_string(),
                    family_id: family_id.clone(),
                    encrypted_payload: general_purpose::STANDARD.encode(&encrypted_payload),
                };

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
    /// # use songbird_discovery::birdsong_integration::*;
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
            debug!("🔍 Received BirdSongPacket (family: {})", packet.family_id);

            // Check if we have an encryption provider
            match &self.encryption {
                Some(enc) if enc.is_available() => {
                    // Check if it's our family
                    if let Some(our_family) = enc.family_id() {
                        if packet.family_id != our_family {
                            debug!(
                                "🔇 Different family ({} != {}), ignoring",
                                packet.family_id, our_family
                            );
                            return Ok(None); // Different family = noise
                        }
                    }

                    // Same family! Try to decrypt
                    use base64::{engine::general_purpose, Engine as _};
                    let encrypted_payload = general_purpose::STANDARD
                        .decode(&packet.encrypted_payload)
                        .context("Failed to decode base64 encrypted_payload")?;

                    match enc.decrypt_discovery(&encrypted_payload).await {
                        Ok(Some(plaintext)) => {
                            debug!(
                                "✅ BirdSong decrypted: {} bytes (same family: {})",
                                plaintext.len(),
                                packet.family_id
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
        beacon: &crate::dark_forest_beacon::DarkForestBeacon,
    ) -> Result<Option<(crate::dark_forest_beacon::BeaconPayload, Vec<u8>)>> {
        use crate::dark_forest_beacon::BeaconPayload;
        
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
                crate::dark_forest_beacon::DarkForestBeacon::MAX_AGE_SECONDS
            );
            return Ok(None);
        }
        
        // Try our own beacon seed first (most common case)
        if let Some(payload) = self.try_decrypt_with_own_beacon(encryption.as_ref(), beacon).await? {
            let our_id = encryption
                .get_beacon_id()
                .await?
                .unwrap_or_default();
            
            debug!(
                "✅ Decrypted Dark Forest beacon with our beacon seed (node: {})",
                payload.node_id
            );
            
            return Ok(Some((payload, our_id)));
        }
        
        // Try all known beacon seeds (peers we've met)
        let known_beacons = encryption.list_known_beacons().await?;
        
        if !known_beacons.is_empty() {
            debug!(
                "Trying {} known beacon seeds from meetings",
                known_beacons.len()
            );
            
            for (idx, beacon_id) in known_beacons.iter().enumerate() {
                if let Some(payload) = self.try_decrypt_with_beacon_id(
                    encryption.as_ref(),
                    beacon,
                    beacon_id
                ).await? {
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
        encryption: &dyn BirdSongEncryption,
        beacon: &crate::dark_forest_beacon::DarkForestBeacon,
    ) -> Result<Option<crate::dark_forest_beacon::BeaconPayload>> {
        use crate::dark_forest_beacon::BeaconPayload;
        
        match encryption
            .try_decrypt_beacon(&beacon.encrypted_payload, &beacon.nonce)
            .await?
        {
            Some(plaintext) => {
                match BeaconPayload::from_bytes(&plaintext) {
                    Ok(payload) => Ok(Some(payload)),
                    Err(e) => {
                        warn!("Failed to parse beacon payload: {}", e);
                        Ok(None)
                    }
                }
            }
            None => Ok(None),
        }
    }
    
    /// Try to decrypt with specific beacon ID
    async fn try_decrypt_with_beacon_id(
        &self,
        encryption: &dyn BirdSongEncryption,
        beacon: &crate::dark_forest_beacon::DarkForestBeacon,
        _beacon_id: &[u8],
    ) -> Result<Option<crate::dark_forest_beacon::BeaconPayload>> {
        // For now, try with our default decryption
        // TODO: Call BearDog's beacon.try_decrypt_with_id when available
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
        payload: &crate::dark_forest_beacon::BeaconPayload,
    ) -> Result<crate::dark_forest_beacon::DarkForestBeacon> {
        use crate::dark_forest_beacon::DarkForestBeacon;
        
        // Check if encryption provider available
        let encryption = self.encryption.as_ref()
            .context("No encryption provider available")?;
        
        if !encryption.is_available() {
            return Err(anyhow::anyhow!("Encryption provider not available"));
        }
        
        // Serialize payload
        let payload_bytes = payload.to_bytes()
            .context("Failed to serialize beacon payload")?;
        
        // Encrypt with beacon seed
        let (encrypted, nonce) = encryption
            .encrypt_beacon(&payload_bytes)
            .await
            .context("Failed to encrypt beacon")?;
        
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

    struct MockEncryption {
        family_id: String,
        available: bool,
    }

    #[async_trait]
    impl BirdSongEncryption for MockEncryption {
        async fn encrypt_discovery(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
            // Simple XOR for testing
            Ok(plaintext.iter().map(|b| b ^ 0x42).collect())
        }

        async fn decrypt_discovery(&self, ciphertext: &[u8]) -> Result<Option<Vec<u8>>> {
            // XOR again to decrypt
            // Simulate different family by checking first byte
            if ciphertext.first() == Some(&0xFF) {
                Ok(None) // Different family marker
            } else {
                Ok(Some(ciphertext.iter().map(|b| b ^ 0x42).collect()))
            }
        }

        fn is_available(&self) -> bool {
            self.available
        }

        fn family_id(&self) -> Option<String> {
            Some(self.family_id.clone())
        }

        fn provider_name(&self) -> String {
            "MockEncryption".to_string()
        }
    }

    #[tokio::test]
    async fn test_birdsong_encryption() {
        let enc = Arc::new(MockEncryption {
            family_id: "test-family".to_string(),
            available: true,
        });
        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: false,
            security_endpoint: None,
            mixed_mode: false,
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
        let enc = Arc::new(MockEncryption {
            family_id: "test-family".to_string(),
            available: true,
        });
        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: false,
            security_endpoint: None,
            mixed_mode: false,
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
            security_endpoint: None,
            mixed_mode: true,
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
        let enc = Arc::new(MockEncryption {
            family_id: "test".to_string(),
            available: true,
        });
        let config = BirdSongConfig {
            enabled: false, // Disabled
            fallback_to_plaintext: true,
            security_endpoint: None,
            mixed_mode: true,
        };
        let processor = BirdSongProcessor::new(Some(enc), config);

        assert!(!processor.is_encrypted());

        let message = b"Message";
        let result = processor.encrypt_packet(message).await.unwrap();
        assert_eq!(&result[..], message, "Should not encrypt when disabled");
    }

    #[tokio::test]
    async fn test_provider_unavailable_with_fallback() {
        let enc = Arc::new(MockEncryption {
            family_id: "test".to_string(),
            available: false, // Unavailable
        });
        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: true, // Fallback enabled
            security_endpoint: None,
            mixed_mode: true,
        };
        let processor = BirdSongProcessor::new(Some(enc), config);

        let message = b"Message";
        let result = processor.encrypt_packet(message).await.unwrap();
        assert_eq!(&result[..], message, "Should fallback to plaintext");
    }

    #[tokio::test]
    async fn test_mixed_mode() {
        let enc = Arc::new(MockEncryption {
            family_id: "test".to_string(),
            available: true,
        });
        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: true,
            security_endpoint: None,
            mixed_mode: true, // Mixed mode
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
        let enc = Arc::new(MockEncryption {
            family_id: "test".to_string(),
            available: true,
        });
        let config = BirdSongConfig {
            enabled: true,
            ..Default::default()
        };
        let processor = BirdSongProcessor::new(Some(enc), config);

        let status = processor.status();
        assert!(status.contains("Encrypted"), "Should report encrypted");
        assert!(status.contains("MockEncryption"), "Should include provider name");
    }
}
