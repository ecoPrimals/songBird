# Dark Forest Beacon Genetics Evolution Plan
**Version**: 1.0.0  
**Date**: February 3, 2026  
**Status**: INVESTIGATION COMPLETE - READY FOR IMPLEMENTATION

---

## 🎯 Executive Summary

### The Problem

**Current State** (songbird-discovery/src/birdsong_integration.rs:46-56):
```rust
pub struct BirdSongPacket {
    pub version: String,
    pub family_id: String,  // ← PLAINTEXT! Metadata leakage
    pub encrypted_payload: String,
}
```

**Network Observation**:
```json
{
  "birdsong": "1.0",
  "family_id": "nat0",  ← Attackers can see this!
  "encrypted_payload": "..."
}
```

**Attack Vector**: Passive observers can:
- See which families exist
- Track family membership
- Build social graphs
- Target specific families

### The Solution

**Dark Forest Beacons** - Fully encrypted packets where ONLY metadata is:
- Encrypted blob (looks like random noise)
- Nonce (public, required for decryption)
- Timestamp (replay protection)

**Discovery Mechanism**: Try decryption with all known beacon seeds
- Success = same family, extract peer info
- Failure = different family, ignore as noise

**TRUE Dark Forest**: Observers see only random data, no metadata leaks.

---

## 🏗️ Architecture

### Two-Seed Model

```
┌─────────────────────────────────────────────────────────────┐
│                    BEACON SEED (Discovery)                  │
│                                                             │
│  • Purpose: Who can see my beacons?                        │
│  • Model: Social graph of meetings                         │
│  • Storage: BearDog genetics                                │
│  • Usage: Songbird BirdSong broadcasts                      │
│  • Exchange: On "meeting" (explicit or implicit)            │
│                                                             │
│  Evolution: Social visibility graph                        │
│  Not strict inheritance - more like contact list           │
└─────────────────────────────────────────────────────────────┘
                              ↓
                    After beacon decryption
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                   LINEAGE SEED (Permissions)                 │
│                                                             │
│  • Purpose: What can they do after meeting?                │
│  • Model: Cryptographic family trust                        │
│  • Storage: BearDog genetics (unchanged)                    │
│  • Usage: All primals for permission verification           │
│  • Exchange: Unchanged from current implementation         │
│                                                             │
│  Evolution: Unchanged - just decoupled from beacon         │
└─────────────────────────────────────────────────────────────┘
```

### Key Insights

1. **Beacon ≠ Lineage**: Discovery visibility separate from permissions
2. **Social Graph**: Beacon genetics exchanged on "meeting", not inherited
3. **Backward Compatible**: New format coexists with legacy during migration
4. **Progressive Enhancement**: Works with or without BearDog

---

## 📊 Current Implementation Analysis

### ✅ What We Have

| Component | Location | Status |
|-----------|----------|--------|
| `BirdSongPacket` | `songbird-discovery/src/birdsong_integration.rs` | ⚠️ Has plaintext `family_id` |
| `BirdSongEncryption` trait | `songbird-discovery/src/birdsong_integration.rs` | ✅ Good abstraction |
| `BearDogBirdSongProvider` | `songbird-discovery/src/beardog_birdsong_provider.rs` | ✅ Unix socket RPC client |
| Encrypt/decrypt logic | Both files | ✅ ChaCha20-Poly1305 AEAD |
| Mixed mode | `BirdSongConfig` | ✅ Supports migration |

### ❌ What We Need

| Component | Location | Status |
|-----------|----------|--------|
| `DarkForestBeacon` struct | NEW: `songbird-discovery/src/dark_forest_beacon.rs` | ❌ Not yet implemented |
| Multi-beacon decryption | NEW: `BirdSongProcessor` enhancement | ❌ Not yet implemented |
| Beacon seed management | NEW: BearDog side (parallel evolution) | ❌ Not yet implemented |
| Meeting exchange protocol | NEW: Cross-primal | ❌ Not yet implemented |
| Backward compat mode | NEW: `BirdSongConfig` enhancement | ❌ Not yet implemented |

---

## 🚀 Implementation Plan

### Phase 1: Foundation (Songbird Side) - Week 1

#### 1.1 Create Dark Forest Beacon Format

**File**: `crates/songbird-discovery/src/dark_forest_beacon.rs` (NEW)

```rust
//! Dark Forest Beacon - TRUE encrypted discovery with zero metadata leakage
//!
//! Unlike BirdSongPacket which has plaintext family_id, Dark Forest beacons
//! are FULLY encrypted. Observers see only noise.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Dark Forest beacon - completely encrypted discovery packet
/// 
/// **Privacy Guarantee**: Observers see only:
/// - Random-looking encrypted blob
/// - Public nonce (required for decryption)
/// - Timestamp (prevents replay attacks)
/// 
/// **NO metadata leakage** - family_id, capabilities, endpoints all encrypted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DarkForestBeacon {
    /// Encrypted payload (opaque to outsiders, signal to family)
    pub encrypted_payload: Vec<u8>,
    
    /// Nonce for ChaCha20-Poly1305 (public, 12 bytes)
    pub nonce: [u8; 12],
    
    /// Timestamp (UNIX epoch seconds) for replay protection
    pub timestamp: u64,
    
    /// Protocol version (2 = Dark Forest format)
    pub version: u8,
}

/// Payload inside Dark Forest beacon (only visible after decryption)
/// 
/// This is what family members see after successful decryption.
/// Observers without beacon genetics see only encrypted_payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconPayload {
    /// Beacon ID of sender (derived from their beacon seed)
    pub beacon_id: Vec<u8>,
    
    /// Node ID
    pub node_id: String,
    
    /// Network endpoints (multiaddr format)
    pub endpoints: Vec<String>,
    
    /// Capabilities hash (not full list - privacy)
    /// Full capabilities exchanged after trust establishment
    pub capabilities_hash: [u8; 32],
    
    /// Cluster ID if part of cluster
    pub cluster_id: Option<String>,
    
    /// Session ID (rotates periodically)
    pub session_id: String,
    
    /// Timestamp when payload created
    pub created_at: u64,
}

impl DarkForestBeacon {
    /// Create new Dark Forest beacon
    pub fn new(encrypted_payload: Vec<u8>, nonce: [u8; 12]) -> Self {
        Self {
            encrypted_payload,
            nonce,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            version: 2,  // Version 2 = Dark Forest format
        }
    }
    
    /// Check if beacon is recent (within 5 minutes)
    pub fn is_recent(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let age = now.saturating_sub(self.timestamp);
        age < 300  // 5 minutes
    }
    
    /// Serialize to JSON bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
    
    /// Deserialize from JSON bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

impl BeaconPayload {
    /// Hash capabilities for privacy-preserving comparison
    pub fn hash_capabilities(capabilities: &[String]) -> [u8; 32] {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        for cap in capabilities {
            hasher.update(cap.as_bytes());
        }
        let hash = hasher.finalize();
        let mut result = [0u8; 32];
        result.copy_from_slice(hash.as_bytes());
        result
    }
    
    /// Serialize to JSON bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
    
    /// Deserialize from JSON bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dark_forest_beacon_roundtrip() {
        let beacon = DarkForestBeacon::new(
            vec![1, 2, 3, 4],
            [0u8; 12],
        );
        
        let bytes = beacon.to_bytes().unwrap();
        let decoded = DarkForestBeacon::from_bytes(&bytes).unwrap();
        
        assert_eq!(decoded.version, 2);
        assert_eq!(decoded.encrypted_payload, vec![1, 2, 3, 4]);
    }
    
    #[test]
    fn test_beacon_payload_roundtrip() {
        let payload = BeaconPayload {
            beacon_id: vec![1, 2, 3],
            node_id: "test-node".to_string(),
            endpoints: vec!["/ip4/127.0.0.1/tcp/1234".to_string()],
            capabilities_hash: [0u8; 32],
            cluster_id: None,
            session_id: "session-123".to_string(),
            created_at: 1234567890,
        };
        
        let bytes = payload.to_bytes().unwrap();
        let decoded = BeaconPayload::from_bytes(&bytes).unwrap();
        
        assert_eq!(decoded.node_id, "test-node");
    }
    
    #[test]
    fn test_beacon_is_recent() {
        let beacon = DarkForestBeacon::new(vec![], [0u8; 12]);
        assert!(beacon.is_recent());
        
        let old_beacon = DarkForestBeacon {
            encrypted_payload: vec![],
            nonce: [0u8; 12],
            timestamp: 1000000,  // Very old
            version: 2,
        };
        assert!(!old_beacon.is_recent());
    }
}
```

**Deliverables**:
- ✅ `DarkForestBeacon` struct (fully encrypted)
- ✅ `BeaconPayload` struct (decrypted content)
- ✅ Serialization/deserialization
- ✅ Replay protection (timestamp check)
- ✅ Unit tests

---

#### 1.2 Extend BirdSongEncryption Trait

**File**: `crates/songbird-discovery/src/birdsong_integration.rs` (MODIFY)

**Add new methods** to support Dark Forest:

```rust
#[async_trait]
pub trait BirdSongEncryption: Send + Sync {
    // Existing methods (unchanged)
    async fn encrypt_discovery(&self, plaintext: &[u8]) -> Result<Vec<u8>>;
    async fn decrypt_discovery(&self, ciphertext: &[u8]) -> Result<Option<Vec<u8>>>;
    fn is_available(&self) -> bool;
    fn family_id(&self) -> Option<String>;
    fn provider_name(&self) -> String;
    
    // NEW: Dark Forest beacon methods
    
    /// Encrypt payload for Dark Forest beacon
    /// 
    /// Returns (encrypted_payload, nonce) tuple
    async fn encrypt_beacon(&self, payload: &[u8]) -> Result<(Vec<u8>, [u8; 12])> {
        // Default implementation for backward compatibility
        // Uses existing encrypt_discovery and generates nonce
        let encrypted = self.encrypt_discovery(payload).await?;
        let nonce = [0u8; 12]; // Placeholder - should be random
        Ok((encrypted, nonce))
    }
    
    /// Try to decrypt Dark Forest beacon
    /// 
    /// Returns Some(payload) if we can decrypt (same beacon family)
    /// Returns None if we cannot decrypt (different beacon family)
    async fn try_decrypt_beacon(&self, encrypted: &[u8], nonce: &[u8; 12]) -> Result<Option<Vec<u8>>> {
        // Default implementation for backward compatibility
        let _ = nonce; // Unused in legacy mode
        self.decrypt_discovery(encrypted).await
    }
    
    /// Get our beacon ID (derived from beacon seed)
    /// 
    /// Returns None if beacon genetics not available
    async fn get_beacon_id(&self) -> Result<Option<Vec<u8>>> {
        // Default implementation - not supported in legacy mode
        Ok(None)
    }
    
    /// List known beacon IDs (peers we've met)
    /// 
    /// Returns empty vec if beacon genetics not supported
    async fn list_known_beacons(&self) -> Result<Vec<Vec<u8>>> {
        // Default implementation - not supported in legacy mode
        Ok(Vec::new())
    }
}
```

**Deliverables**:
- ✅ New trait methods with default implementations
- ✅ Backward compatible (existing providers still work)
- ✅ Forward compatible (new providers can override)

---

#### 1.3 Implement Multi-Beacon Decryption

**File**: `crates/songbird-discovery/src/birdsong_integration.rs` (MODIFY)

**Add new method** to `BirdSongProcessor`:

```rust
impl BirdSongProcessor {
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
    pub async fn decrypt_dark_forest_beacon(
        &self,
        beacon: &DarkForestBeacon,
    ) -> Result<Option<(BeaconPayload, Vec<u8>)>> {
        // Check beacon age
        if !beacon.is_recent() {
            debug!("Ignoring stale beacon (age > 5 min)");
            return Ok(None);
        }
        
        // Try our own beacon seed first (most common case)
        if let Some(payload) = self.try_decrypt_with_own_beacon(beacon).await? {
            let our_id = self.encryption_provider
                .get_beacon_id()
                .await?
                .unwrap_or_default();
            return Ok(Some((payload, our_id)));
        }
        
        // Try all known beacon seeds (peers we've met)
        let known_beacons = self.encryption_provider
            .list_known_beacons()
            .await?;
        
        for beacon_id in known_beacons {
            if let Some(payload) = self.try_decrypt_with_beacon_id(beacon, &beacon_id).await? {
                return Ok(Some((payload, beacon_id)));
            }
        }
        
        // Cannot decrypt - different beacon family
        // This is EXPECTED and CORRECT behavior for Dark Forest
        debug!("Cannot decrypt Dark Forest beacon - different beacon family (as expected)");
        Ok(None)
    }
    
    /// Try to decrypt with our own beacon seed
    async fn try_decrypt_with_own_beacon(
        &self,
        beacon: &DarkForestBeacon,
    ) -> Result<Option<BeaconPayload>> {
        match self.encryption_provider
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
        beacon: &DarkForestBeacon,
        beacon_id: &[u8],
    ) -> Result<Option<BeaconPayload>> {
        // This would call BearDog's beacon.try_decrypt_with_id RPC
        // For now, we just try with our default decryption
        // Full implementation requires BearDog Phase 1
        let _ = beacon_id;
        self.try_decrypt_with_own_beacon(beacon).await
    }
}
```

**Deliverables**:
- ✅ Multi-beacon decryption logic
- ✅ Fallback chain (own beacon → known beacons)
- ✅ Privacy-preserving failure handling
- ✅ Replay protection checks

---

### Phase 2: Configuration & Migration - Week 2

#### 2.1 Extend BirdSongConfig

**File**: `crates/songbird-discovery/src/birdsong_integration.rs` (MODIFY)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirdSongConfig {
    // Existing fields (unchanged)
    pub enabled: bool,
    pub fallback_to_plaintext: bool,
    pub security_endpoint: Option<String>,
    pub mixed_mode: bool,
    
    // NEW: Dark Forest configuration
    
    /// Enable Dark Forest beacons (fully encrypted)
    /// 
    /// When true, broadcasts Dark Forest beacons (version 2)
    /// When false, broadcasts legacy BirdSongPacket (version 1.0)
    /// 
    /// Requires BearDog beacon.* RPC methods
    pub dark_forest_enabled: bool,
    
    /// Accept legacy BirdSongPacket (backward compatibility)
    /// 
    /// When true, accepts both Dark Forest and legacy formats
    /// When false, only accepts Dark Forest format
    /// 
    /// Recommended: true during migration, false after
    pub accept_legacy_format: bool,
    
    /// Broadcast legacy format alongside Dark Forest
    /// 
    /// When true, broadcasts BOTH formats for migration
    /// When false, only broadcasts Dark Forest
    /// 
    /// Recommended: true during early migration, false after
    pub dual_broadcast: bool,
}

impl Default for BirdSongConfig {
    fn default() -> Self {
        Self {
            // Existing defaults
            enabled: false,
            fallback_to_plaintext: true,
            security_endpoint: None,
            mixed_mode: true,
            
            // Dark Forest defaults (conservative)
            dark_forest_enabled: false,      // Opt-in
            accept_legacy_format: true,      // Backward compatible
            dual_broadcast: false,           // Not needed by default
        }
    }
}

impl BirdSongConfig {
    /// Create config for Dark Forest mode
    pub fn dark_forest() -> Self {
        Self {
            enabled: true,
            dark_forest_enabled: true,
            accept_legacy_format: true,  // Still accept legacy
            dual_broadcast: false,       // Only send Dark Forest
            ..Default::default()
        }
    }
    
    /// Create config for migration period (dual format)
    pub fn migration_mode() -> Self {
        Self {
            enabled: true,
            dark_forest_enabled: true,
            accept_legacy_format: true,
            dual_broadcast: true,        // Send both formats
            ..Default::default()
        }
    }
    
    /// Create config for legacy-only mode
    pub fn legacy_only() -> Self {
        Self {
            enabled: true,
            dark_forest_enabled: false,
            accept_legacy_format: true,
            dual_broadcast: false,
            ..Default::default()
        }
    }
}
```

**Deliverables**:
- ✅ Dark Forest configuration options
- ✅ Migration mode support
- ✅ Backward compatibility flags
- ✅ Sensible defaults

---

#### 2.2 Environment Variables

**File**: `crates/songbird-orchestrator/src/env_config.rs` (MODIFY)

```rust
/// Dark Forest configuration from environment
pub fn dark_forest_enabled() -> bool {
    std::env::var("SONGBIRD_DARK_FOREST")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(false)
}

pub fn accept_legacy_birdsong() -> bool {
    std::env::var("SONGBIRD_ACCEPT_LEGACY_BIRDSONG")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(true)  // Default true for backward compat
}

pub fn dual_broadcast() -> bool {
    std::env::var("SONGBIRD_DUAL_BROADCAST")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(false)
}
```

**Environment Variables**:
```bash
# Enable Dark Forest beacons
SONGBIRD_DARK_FOREST=true

# Accept legacy BirdSongPacket (backward compat)
SONGBIRD_ACCEPT_LEGACY_BIRDSONG=true

# Broadcast both formats (migration)
SONGBIRD_DUAL_BROADCAST=true

# Existing (unchanged)
BIRDSONG_ENABLED=true
BIRDSONG_FALLBACK_PLAINTEXT=true
```

**Deliverables**:
- ✅ Environment variable parsing
- ✅ Sensible defaults
- ✅ Documentation

---

### Phase 3: Broadcasting - Week 3

#### 3.1 Dark Forest Broadcaster

**File**: `crates/songbird-discovery/src/anonymous/broadcaster.rs` (NEW METHOD)

```rust
impl Broadcaster {
    /// Broadcast Dark Forest beacon (fully encrypted)
    pub async fn broadcast_dark_forest(&self) -> Result<()> {
        // Build beacon payload
        let payload = BeaconPayload {
            beacon_id: self.get_our_beacon_id().await?,
            node_id: self.node_id.clone(),
            endpoints: self.get_endpoints(),
            capabilities_hash: self.hash_capabilities(),
            cluster_id: self.cluster_id.clone(),
            session_id: self.session_id.clone(),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        // Serialize payload
        let payload_bytes = payload.to_bytes()
            .context("Failed to serialize beacon payload")?;
        
        // Encrypt with beacon seed
        let (encrypted, nonce) = self.encryption_provider
            .encrypt_beacon(&payload_bytes)
            .await
            .context("Failed to encrypt beacon")?;
        
        // Create Dark Forest beacon
        let beacon = DarkForestBeacon::new(encrypted, nonce);
        
        // Serialize beacon
        let beacon_bytes = beacon.to_bytes()
            .context("Failed to serialize Dark Forest beacon")?;
        
        // Broadcast via UDP multicast
        self.multicast_send(&beacon_bytes).await?;
        
        info!(
            "Broadcasted Dark Forest beacon (size: {} bytes, NO metadata leakage)",
            beacon_bytes.len()
        );
        
        Ok(())
    }
    
    /// Broadcast legacy BirdSongPacket (for migration)
    pub async fn broadcast_legacy(&self) -> Result<()> {
        // Existing implementation (unchanged)
        // ...
    }
    
    /// Broadcast based on configuration
    pub async fn broadcast(&self) -> Result<()> {
        if self.config.dark_forest_enabled {
            // Dark Forest mode
            self.broadcast_dark_forest().await?;
            
            if self.config.dual_broadcast {
                // Also broadcast legacy for migration
                self.broadcast_legacy().await?;
            }
        } else {
            // Legacy mode only
            self.broadcast_legacy().await?;
        }
        
        Ok(())
    }
}
```

**Deliverables**:
- ✅ Dark Forest broadcasting
- ✅ Dual broadcast for migration
- ✅ Legacy broadcast preserved
- ✅ Configuration-driven behavior

---

### Phase 4: Reception & Processing - Week 3

#### 4.1 Multi-Format Reception

**File**: `crates/songbird-discovery/src/anonymous/listener.rs` (MODIFY)

```rust
impl Listener {
    /// Process received packet (auto-detects format)
    pub async fn process_received_packet(&self, bytes: &[u8]) -> Result<()> {
        // Try Dark Forest format first
        if let Ok(beacon) = DarkForestBeacon::from_bytes(bytes) {
            if beacon.version == 2 {
                return self.process_dark_forest_beacon(beacon).await;
            }
        }
        
        // Try legacy BirdSongPacket format
        if self.config.accept_legacy_format {
            if let Ok(packet) = serde_json::from_slice::<BirdSongPacket>(bytes) {
                if packet.version == "1.0" {
                    warn!("Received legacy BirdSongPacket (plaintext family_id) - consider upgrading sender");
                    return self.process_legacy_packet(packet).await;
                }
            }
        }
        
        // Unknown format
        debug!("Received unknown packet format - ignoring");
        Ok(())
    }
    
    /// Process Dark Forest beacon
    async fn process_dark_forest_beacon(&self, beacon: DarkForestBeacon) -> Result<()> {
        // Try to decrypt
        match self.processor.decrypt_dark_forest_beacon(&beacon).await? {
            Some((payload, beacon_id)) => {
                info!(
                    "Decrypted Dark Forest beacon from {} (beacon_id: {})",
                    payload.node_id,
                    hex::encode(&beacon_id)
                );
                
                // Process discovered peer
                self.handle_discovered_peer(payload, beacon_id).await?;
            }
            None => {
                // Cannot decrypt - different beacon family
                // This is EXPECTED and NORMAL
                debug!("Ignoring beacon from different beacon family (Dark Forest working as intended)");
            }
        }
        
        Ok(())
    }
    
    /// Process legacy BirdSongPacket
    async fn process_legacy_packet(&self, packet: BirdSongPacket) -> Result<()> {
        warn!(
            "Processing legacy BirdSongPacket with PLAINTEXT family_id: {} (metadata leakage!)",
            packet.family_id
        );
        
        // Existing legacy processing
        // ...
        
        Ok(())
    }
}
```

**Deliverables**:
- ✅ Auto-detection of packet format
- ✅ Dark Forest beacon processing
- ✅ Legacy packet processing (backward compat)
- ✅ Deprecation warnings

---

## 📋 Testing Strategy

### Unit Tests

**File**: `crates/songbird-discovery/src/dark_forest_beacon.rs`

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_beacon_serialization() { ... }
    
    #[test]
    fn test_payload_serialization() { ... }
    
    #[test]
    fn test_replay_protection() { ... }
    
    #[test]
    fn test_capabilities_hashing() { ... }
}
```

**File**: `crates/songbird-discovery/src/birdsong_integration.rs`

```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_dark_forest_encryption_roundtrip() { ... }
    
    #[tokio::test]
    async fn test_multi_beacon_decryption() { ... }
    
    #[tokio::test]
    async fn test_different_beacon_cannot_decrypt() { ... }
    
    #[tokio::test]
    async fn test_legacy_fallback() { ... }
}
```

### Integration Tests

**File**: `crates/songbird-discovery/tests/dark_forest_integration_tests.rs` (NEW)

```rust
#[tokio::test]
async fn test_dark_forest_discovery_e2e() {
    // 1. Start two Songbird nodes with same beacon seed
    // 2. Node A broadcasts Dark Forest beacon
    // 3. Node B receives and decrypts successfully
    // 4. Start third node with different beacon seed
    // 5. Node C cannot decrypt (sees only noise)
}

#[tokio::test]
async fn test_migration_dual_broadcast() {
    // Test dual broadcast mode during migration
}

#[tokio::test]
async fn test_legacy_backward_compatibility() {
    // Ensure legacy nodes can still communicate
}
```

---

## 🔄 Migration Path

### Phase 1: Dual Format (Weeks 1-4)

```
┌─────────────────────────────────────────────────────┐
│ MIGRATION PHASE 1: Dual Format Support             │
├─────────────────────────────────────────────────────┤
│                                                     │
│ Configuration:                                      │
│   SONGBIRD_DARK_FOREST=true                        │
│   SONGBIRD_ACCEPT_LEGACY_BIRDSONG=true             │
│   SONGBIRD_DUAL_BROADCAST=true                     │
│                                                     │
│ Behavior:                                           │
│   • Broadcasts: Dark Forest + Legacy               │
│   • Accepts: Dark Forest + Legacy                  │
│   • Logs warnings for legacy packets               │
│                                                     │
│ Goal: Ensure backward compatibility                │
└─────────────────────────────────────────────────────┘
```

### Phase 2: Dark Forest Preferred (Weeks 5-8)

```
┌─────────────────────────────────────────────────────┐
│ MIGRATION PHASE 2: Dark Forest Preferred           │
├─────────────────────────────────────────────────────┤
│                                                     │
│ Configuration:                                      │
│   SONGBIRD_DARK_FOREST=true                        │
│   SONGBIRD_ACCEPT_LEGACY_BIRDSONG=true             │
│   SONGBIRD_DUAL_BROADCAST=false  ← Changed         │
│                                                     │
│ Behavior:                                           │
│   • Broadcasts: Dark Forest only                   │
│   • Accepts: Dark Forest + Legacy                  │
│   • Logs deprecation warnings for legacy           │
│                                                     │
│ Goal: Reduce network overhead, still compatible    │
└─────────────────────────────────────────────────────┘
```

### Phase 3: Dark Forest Only (Weeks 9+)

```
┌─────────────────────────────────────────────────────┐
│ MIGRATION PHASE 3: Dark Forest Only (Optional)     │
├─────────────────────────────────────────────────────┤
│                                                     │
│ Configuration:                                      │
│   SONGBIRD_DARK_FOREST=true                        │
│   SONGBIRD_ACCEPT_LEGACY_BIRDSONG=false ← Changed  │
│   SONGBIRD_DUAL_BROADCAST=false                    │
│                                                     │
│ Behavior:                                           │
│   • Broadcasts: Dark Forest only                   │
│   • Accepts: Dark Forest only                      │
│   • Rejects legacy packets with error              │
│                                                     │
│ Goal: TRUE Dark Forest, no backward compat         │
└─────────────────────────────────────────────────────┘
```

---

## 🎯 Success Criteria

### Songbird Complete When:

- [ ] `DarkForestBeacon` struct implemented
- [ ] `BeaconPayload` struct implemented
- [ ] Multi-beacon decryption working
- [ ] Broadcasting Dark Forest beacons
- [ ] Auto-detection of packet format
- [ ] Configuration options working
- [ ] Environment variables parsed
- [ ] Unit tests pass (>90% coverage)
- [ ] Integration tests pass
- [ ] Backward compatibility verified
- [ ] Builds on x86_64 and aarch64
- [ ] Documentation complete

### Privacy Guarantees Verified:

- [ ] Network capture shows only encrypted blobs
- [ ] No plaintext metadata in packets
- [ ] Different beacon families cannot decrypt
- [ ] Same beacon family can discover each other
- [ ] Replay attacks prevented
- [ ] Session rotation working

---

## 📊 Current Status

### Investigation Phase: ✅ COMPLETE

- [x] Analyzed current BirdSongPacket implementation
- [x] Identified metadata leakage (plaintext family_id)
- [x] Reviewed BearDogBirdSongProvider architecture
- [x] Designed Dark Forest beacon format
- [x] Planned migration strategy
- [x] Created comprehensive implementation plan

### Next Actions:

1. **Immediate** (This session):
   - Create `dark_forest_beacon.rs` with basic structures
   - Add tests for beacon serialization
   - Update `birdsong_integration.rs` trait

2. **Week 1** (Phase 1):
   - Implement Dark Forest beacon format
   - Add multi-beacon decryption logic
   - Create unit tests

3. **Week 2** (Phase 2):
   - Add configuration options
   - Implement environment variable parsing
   - Add backward compatibility

4. **Week 3** (Phase 3-4):
   - Implement broadcasting
   - Implement reception
   - Create integration tests

---

## 🔗 Dependencies

### External (BearDog Side - Parallel Evolution):

**Required from BearDog** (blocking Dark Forest full functionality):
- [ ] `beacon.encrypt` RPC method
- [ ] `beacon.try_decrypt` RPC method
- [ ] `beacon.get_id` RPC method
- [ ] `beacon.list_known` RPC method
- [ ] Beacon seed management

**Songbird can proceed independently** with:
- ✅ Dark Forest packet format (no BearDog needed)
- ✅ Serialization/deserialization (no BearDog needed)
- ✅ Configuration system (no BearDog needed)
- ✅ Backward compatibility (existing code works)

**Full functionality requires** BearDog Phase 1 complete

### Internal (Songbird):

- ✅ `songbird-discovery` crate exists
- ✅ `BirdSongEncryption` trait exists
- ✅ `BearDogBirdSongProvider` exists
- ✅ Unix socket RPC client exists
- ✅ Multicast broadcasting exists
- ✅ Discovery processing exists

**No blocking dependencies** - can start immediately

---

## 📝 Notes

### Design Decisions:

1. **Two-Seed Model**: Beacon (discovery) separate from Lineage (permissions)
2. **Backward Compatible**: Coexist with legacy during migration
3. **Progressive Enhancement**: Works with or without BearDog
4. **Privacy First**: No metadata in packets by default

### Trade-offs:

1. **Performance**: Multi-beacon decryption slower than plaintext family_id check
   - **Mitigation**: Cache beacon IDs, try own beacon first
   
2. **Complexity**: More code than legacy single-format
   - **Mitigation**: Clear abstractions, comprehensive tests
   
3. **Migration**: Need dual-format period
   - **Mitigation**: Phased rollout, clear documentation

### Future Enhancements:

1. **Meeting Exchange Protocol**: Explicit beacon genetics exchange
2. **Beacon Rotation**: Periodic beacon seed rotation for forward secrecy
3. **Selective Visibility**: Different beacon seeds for different peer groups
4. **Bandwidth Optimization**: Bloom filters for beacon ID matching

---

## 🚀 Ready to Proceed

**Status**: Investigation complete, plan approved, ready for implementation

**Estimated Timeline**: 3-4 weeks for full Songbird implementation

**Risk Level**: Low (backward compatible, incremental rollout)

**Impact**: High (TRUE Dark Forest, no metadata leakage)

---

*"Beacon genetics is who you've met. Lineage is what they can do. Dark Forest is privacy by default."*
